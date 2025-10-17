# LLM 模块测试覆盖率提升方案

## 📊 当前状况

### 覆盖率现状
- **llm/deepseek.rs**: 17.61% (需要 Deepseek API key)
- **llm/ollama.rs**: 18.01% (需要 Ollama 服务)
- **llm/mod.rs**: 82.92% (基础模块，已有良好覆盖)

### 问题分析
- ✅ 有基础单元测试（创建客户端等）
- ❌ 缺少 HTTP 请求的 mock 测试
- ❌ 未测试错误处理路径
- ❌ 未测试重试逻辑
- ❌ 未测试流式输出

## 🎯 改进策略

### 策略 1: 使用 Mock HTTP Server ✅ 推荐

**工具**: `mockito` crate

**优点**:
- ✅ 真实的 HTTP 测试
- ✅ 不需要真实 API
- ✅ 可测试各种响应场景
- ✅ 易于维护

**实施内容**:

1. **添加依赖**
   ```toml
   [dev-dependencies]
   mockito = "1.6"
   ```

2. **创建 mock 测试**
   ```rust
   #[tokio::test]
   async fn test_deepseek_chat_success() {
       let mut server = mockito::Server::new_async().await;
       
       // Mock 成功响应
       let mock = server.mock("POST", "/chat/completions")
           .with_status(200)
           .with_header("content-type", "application/json")
           .with_body(r#"{
               "choices": [{
                   "message": {
                       "content": "Hello!"
                   }
               }]
           }"#)
           .create_async()
           .await;
       
       let client = DeepseekClient::new("test-key", "test-model", server.url()).unwrap();
       let result = client.chat(vec![Message::user("Hi")]).await;
       
       assert!(result.is_ok());
       mock.assert_async().await;
   }
   ```

3. **测试覆盖场景**
   - ✅ 成功响应
   - ✅ HTTP 错误（40x, 50x）
   - ✅ 超时
   - ✅ 格式错误
   - ✅ 重试逻辑
   - ✅ 工具调用响应

**预计工作量**: 60 分钟
- 添加依赖和基础设施: 15 分钟
- Deepseek mock 测试: 20 分钟
- Ollama mock 测试: 20 分钟
- 验证覆盖率: 5 分钟

---

### 策略 2: 使用 Trait Mock

**工具**: 手写 mock trait

**优点**:
- ✅ 完全控制
- ✅ 不依赖外部库
- ✅ 轻量级

**缺点**:
- ❌ 需要重构现有代码（添加 trait 抽象）
- ❌ 工作量较大
- ❌ 不测试真实 HTTP

**预计工作量**: 120+ 分钟

---

### 策略 3: 录制/回放模式

**工具**: `vcr` 或类似工具

**优点**:
- ✅ 真实 API 响应
- ✅ 一次录制，多次使用

**缺点**:
- ❌ 需要初次运行时有 API key
- ❌ 响应固定，难以测试边界情况
- ❌ Rust 生态支持不成熟

**预计工作量**: 90 分钟

---

## 📝 推荐方案：策略 1 (mockito)

### 实施计划

#### Phase 1: 基础设施 (15分钟)
1. 添加 `mockito` 依赖
2. 创建 mock 测试辅助函数
3. 设置测试框架

#### Phase 2: Deepseek 测试 (20分钟)
测试场景：
- [x] 成功的 chat 请求
- [x] 成功的 chat_with_tools 请求
- [x] HTTP 400 错误
- [x] HTTP 500 错误
- [x] 超时错误
- [x] JSON 解析错误
- [x] 重试成功场景

#### Phase 3: Ollama 测试 (20分钟)
测试场景：
- [x] 成功的 chat 请求
- [x] 思考标签过滤
- [x] HTTP 错误处理
- [x] 连接错误
- [x] 诊断功能

#### Phase 4: 验证 (5分钟)
- 运行覆盖率报告
- 确认覆盖率提升
- 更新文档

### 预期成果

| 模块 | 当前 | 目标 | 提升 |
|------|------|------|------|
| deepseek.rs | 17.61% | **70%+** | +300% |
| ollama.rs | 18.01% | **70%+** | +290% |
| 总体 LLM | ~18% | **70%+** | +290% |

---

## 🔧 实施代码示例

### 测试结构
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    /// 创建 mock server 和 client
    async fn create_mock_client() -> (mockito::Server, DeepseekClient) {
        let server = mockito::Server::new_async().await;
        let client = DeepseekClient::new(
            "test-key",
            "test-model", 
            server.url()
        ).unwrap();
        (server, client)
    }

    #[tokio::test]
    async fn test_chat_success() {
        // 测试成功响应
    }

    #[tokio::test]
    async fn test_chat_http_error() {
        // 测试 HTTP 错误
    }

    #[tokio::test]
    async fn test_chat_retry_success() {
        // 测试重试逻辑
    }

    #[tokio::test]
    async fn test_tools_call() {
        // 测试工具调用
    }
}
```

### Mock 响应示例
```rust
// 成功响应
let _mock = server.mock("POST", "/chat/completions")
    .with_status(200)
    .with_body(r#"{
        "choices": [{
            "message": {"content": "Hello!"}
        }]
    }"#)
    .create_async()
    .await;

// 错误响应
let _mock = server.mock("POST", "/chat/completions")
    .with_status(500)
    .with_body(r#"{"error": "Internal Server Error"}"#)
    .create_async()
    .await;

// 工具调用响应
let _mock = server.mock("POST", "/chat/completions")
    .with_status(200)
    .with_body(r#"{
        "choices": [{
            "message": {
                "tool_calls": [{
                    "id": "call_1",
                    "function": {
                        "name": "calculator",
                        "arguments": "{\"expression\": \"2+2\"}"
                    }
                }]
            }
        }]
    }"#)
    .create_async()
    .await;
```

---

## ✅ 验证标准

1. **覆盖率达标**
   - [ ] Deepseek.rs ≥ 70%
   - [ ] Ollama.rs ≥ 70%
   - [ ] 所有测试通过

2. **测试质量**
   - [ ] 测试成功路径
   - [ ] 测试错误处理
   - [ ] 测试重试逻辑
   - [ ] 测试边界情况

3. **文档更新**
   - [ ] 更新测试说明
   - [ ] 更新覆盖率报告

---

## 📊 投入产出分析

| 方案 | 工作量 | 覆盖率提升 | 维护成本 | 推荐度 |
|------|--------|------------|----------|--------|
| Mock HTTP | 60分钟 | 50%+ | 低 | ⭐⭐⭐⭐⭐ |
| Trait Mock | 120分钟 | 60%+ | 中 | ⭐⭐⭐ |
| 录制回放 | 90分钟 | 40%+ | 中 | ⭐⭐ |

---

## 🚀 立即开始

推荐立即执行**策略 1: Mock HTTP Server**

**理由**:
1. 工作量适中（60分钟）
2. 效果显著（覆盖率提升 3倍）
3. 易于维护
4. 符合最佳实践

**开始命令**:
```bash
# 添加依赖
echo 'mockito = "1.6"' >> Cargo.toml

# 开始实施
# 1. Deepseek mock 测试
# 2. Ollama mock 测试
# 3. 运行覆盖率验证
```

---

**生成时间**: 2025-10-15
**推荐方案**: 策略 1 - Mock HTTP Server
**预计时间**: 60 分钟
**预期收益**: 覆盖率从 18% → 70%+
