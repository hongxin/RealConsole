# JSON 解析错误调试改进

**日期**: 2025-11-02
**版本**: v1.22.2-dev
**问题**: 任务分解失败时缺乏详细的错误信息

## 问题描述

### 原始错误

用户执行以下命令时遇到错误：

```bash
/plan 帮我分别看看人民日报、新华网、浙江日报网站的今天新闻
```

错误信息：
```
[ERROR] 任务分解失败: LLM 错误: Parse error: Failed to parse JSON response: error decoding response body
```

### 问题分析

**错误链路**：
1. 用户输入 `/plan` 命令
2. `TaskDecomposer` 调用 LLM 进行任务分解
3. `DeepseekClient` 发送 HTTP 请求
4. `HttpClientBase::handle_response()` 解析响应
5. ❌ JSON 解析失败，但无法看到响应体内容

**根本原因**：
- HTTP 状态码是 200（成功）
- 但响应体不是有效的 JSON 格式
- 原始代码直接使用 `resp.json().await`，失败时无法查看响应内容

## 解决方案

### 方案 1：增加响应体调试输出（已实施）

#### 修改文件
`src/llm/http_base.rs` - `handle_response()` 方法

#### 关键改进

**修改前**：
```rust
// 解析 JSON
resp.json()
    .await
    .map_err(|e| LlmError::Parse(format!("Failed to parse JSON response: {}", e)))
```

**修改后**：
```rust
// 先获取文本响应
let text = resp
    .text()
    .await
    .map_err(|e| LlmError::Network(format!("Failed to read response body: {}", e)))?;

// 尝试解析 JSON
serde_json::from_str(&text).map_err(|e| {
    // 调试输出：打印完整响应体到 stderr
    eprintln!("\n{}", "=".repeat(80));
    eprintln!("[DEBUG] JSON 解析失败");
    eprintln!("[DEBUG] 解析错误: {}", e);
    eprintln!("[DEBUG] 响应体长度: {} 字节", text.len());
    eprintln!("{}", "-".repeat(80));
    eprintln!("[DEBUG] 完整响应体:");
    eprintln!("{}", text);
    eprintln!("{}\n", "=".repeat(80));

    // 返回错误，包含截断的响应体预览
    let preview = crate::utils::string::truncate_safe(&text, 200);
    LlmError::Parse(format!(
        "Failed to parse JSON response: {}. Response preview: {}",
        e, preview
    ))
})
```

#### 改进要点

1. **两阶段处理**
   - 第一阶段：获取文本响应（`resp.text().await`）
   - 第二阶段：解析 JSON（`serde_json::from_str`）

2. **详细调试输出**（打印到 stderr）
   - 解析错误信息
   - 响应体长度
   - 完整响应体内容

3. **用户友好的错误信息**
   - 包含响应体预览（截断到 200 字节）
   - 使用 `truncate_safe()` 避免 UTF-8 截断错误

## 使用方法

### 复现原始错误

```bash
# 重新编译
cargo build --release

# 运行 RealConsole
./target/release/realconsole

# 执行可能失败的命令
/plan 帮我分别看看人民日报、新华网、浙江日报网站的今天新闻
```

### 新的输出示例

**终端输出（stderr）**：
```
================================================================================
[DEBUG] JSON 解析失败
[DEBUG] 解析错误: expected value at line 1 column 1
[DEBUG] 响应体长度: 245 字节
--------------------------------------------------------------------------------
[DEBUG] 完整响应体:
I cannot help you access external websites or fetch real-time news content...
================================================================================
```

**用户看到的错误（stdout）**：
```
[ERROR] 任务分解失败: LLM 错误: Parse error: Failed to parse JSON response: expected value at line 1 column 1. Response preview: I cannot help you access external websites or fetch real-time news content...
```

## 后续优化建议

### 方案 2：优化任务分解提示词（中期）

在 `src/task/decomposer.rs` 的提示词中：
- 明确说明 RealConsole 没有网络工具
- 要求 LLM 不要生成需要访问外部网站的任务
- 强调只输出纯 JSON，不要有任何解释

### 方案 3：实现容错的 JSON 提取（长期）

参考 `TaskDecomposer::extract_json()` 的逻辑，实现通用的 JSON 提取功能：
- 自动识别 ```json ... ``` 代码块
- 提取混合文本中的 JSON 部分
- 提高对非标准响应的容错能力

## 测试结果

✅ **编译验证**
```bash
cargo check        # 通过
cargo build --release  # 通过
```

✅ **代码质量**
- 无新增 clippy 警告
- 遵循 Rust 最佳实践
- 使用已有的 `truncate_safe()` 工具函数

## 影响范围

### 受益场景
1. 任务分解（`/plan`）失败时
2. LLM 工具调用（Function Calling）失败时
3. 所有使用 `HttpClientBase::handle_response()` 的地方

### 不受影响
- 正常的 JSON 响应解析（无性能损失）
- HTTP 错误处理（4xx/5xx）
- 其他模块功能

## 总结

这次改进是一个**低风险、高价值**的调试增强：

- ✅ 实施简单（20行代码）
- ✅ 不改变原有逻辑
- ✅ 提供详细的调试信息
- ✅ 帮助快速定位 LLM 响应问题
- ✅ 为后续优化提供数据支持

通过这次改进，当 LLM 返回非 JSON 响应时，开发者可以立即看到完整的响应内容，快速判断问题根源（是 LLM 理解错误、API 限流、还是其他原因），从而更快地解决问题。
