# Phase 5.3 Week 2 Day 3 - 错误系统改进

**日期**: 2025-10-15
**阶段**: Phase 5.3 Week 2 - UX 改进
**任务**: 统一错误系统与用户友好的错误消息
**状态**: ✅ 完成

---

## 执行摘要

成功设计并实现了统一的错误系统，包含30+错误代码、建议性修复方案和用户友好的错误格式。将新系统应用到 shell_executor 和 config 模块，并测试了实际效果。错误消息现在包含错误代码、详细描述、修复建议、示例命令和文档链接，大幅提升了用户体验。

### 关键成果

- ✅ **错误代码系统**: 定义30+错误代码（E001-E999）
- ✅ **统一错误类型**: 创建 RealError 类型与 FixSuggestion 系统
- ✅ **Shell 模块应用**: 完整迁移 shell_executor 到新系统
- ✅ **Config 模块应用**: 迁移配置加载错误处理
- ✅ **用户友好格式**: 彩色输出、编号建议、文档链接

---

## 设计与实现

### 1. 错误代码系统

#### 错误代码分类

```rust
pub enum ErrorCode {
    // 配置错误 (E001-E099)
    ConfigNotFound,          // E001 - 配置文件不存在
    ConfigParseError,        // E002 - 配置文件解析失败
    ConfigValidationError,   // E003 - 配置验证失败
    EnvFileNotFound,         // E004 - 环境变量文件不存在
    EnvVarMissing,          // E005 - 缺少必需的环境变量
    ApiKeyInvalid,          // E006 - API Key 格式无效
    ApiKeyEmpty,            // E007 - API Key 为空

    // LLM 错误 (E100-E199)
    LlmNotConfigured,       // E100 - LLM 未配置
    LlmConnectionError,     // E101 - 无法连接到 LLM 服务
    LlmAuthError,           // E102 - LLM 认证失败
    LlmRateLimitError,      // E103 - 超过 API 调用限额
    LlmTimeoutError,        // E104 - LLM 请求超时
    LlmResponseError,       // E105 - LLM 响应格式错误
    LlmModelNotFound,       // E106 - 模型不存在

    // 工具错误 (E200-E299)
    ToolNotFound,           // E200 - 工具不存在
    ToolParameterError,     // E201 - 工具参数错误
    ToolExecutionError,     // E202 - 工具执行失败
    ToolTimeoutError,       // E203 - 工具执行超时
    ToolPermissionDenied,   // E204 - 工具权限不足

    // Shell 错误 (E300-E399)
    ShellDisabled,          // E300 - Shell 功能未启用
    ShellCommandEmpty,      // E301 - Shell 命令为空
    ShellDangerousCommand,  // E302 - 检测到危险命令
    ShellTimeoutError,      // E303 - Shell 命令超时
    ShellExecutionError,    // E304 - Shell 命令执行失败

    // 网络错误 (E600-E699)
    NetworkError,           // E600 - 网络错误
    HttpError,              // E601 - HTTP 请求失败
    DnsError,               // E602 - DNS 解析失败
    SslError,               // E603 - SSL/TLS 错误

    // 文件系统错误 (E700-E799)
    FileNotFound,           // E700 - 文件不存在
    FileReadError,          // E701 - 文件读取失败
    FileWriteError,         // E702 - 文件写入失败
    DirectoryNotFound,      // E703 - 目录不存在
}
```

**设计原则**:
- 分类明确：每类错误100个代码空间
- 编号一致：前缀表示类别（E0=配置，E1=LLM，E3=Shell）
- 可扩展：预留空间用于未来新错误

### 2. 建议性修复方案

#### FixSuggestion 结构

```rust
#[derive(Debug, Clone)]
pub struct FixSuggestion {
    pub description: String,
    pub command: Option<String>,
    pub doc_link: Option<String>,
}

impl FixSuggestion {
    pub fn new(description: impl Into<String>) -> Self { ... }

    pub fn with_command(mut self, command: impl Into<String>) -> Self { ... }

    pub fn with_doc(mut self, link: impl Into<String>) -> Self { ... }
}
```

**Builder 模式**：链式调用，可选添加命令或文档链接

**使用示例**:
```rust
FixSuggestion::new("运行配置向导创建配置文件")
    .with_command("realconsole wizard")

FixSuggestion::new("参考示例配置文件")
    .with_doc("https://docs.realconsole.com/config")
```

### 3. RealError 统一错误类型

#### 结构定义

```rust
#[derive(Error, Debug)]
pub struct RealError {
    pub code: ErrorCode,
    pub message: String,
    pub suggestions: Vec<FixSuggestion>,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl RealError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self { ... }

    pub fn with_suggestion(mut self, suggestion: FixSuggestion) -> Self { ... }

    pub fn with_suggestions(mut self, suggestions: Vec<FixSuggestion>) -> Self { ... }

    pub fn with_source(
        mut self,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self { ... }

    pub fn format_user_friendly(&self) -> String { ... }
}
```

**核心功能**:
- 使用 `thiserror::Error` 自动实现 Error trait
- Builder 模式添加建议和源错误
- `format_user_friendly()` 生成彩色格式化输出

#### 格式化输出示例

```
✗ [E302] 检测到危险命令

命令包含危险操作，已被安全策略阻止

建议修复方案:

  1. 此命令可能造成系统损坏，建议使用更安全的替代方案
  2. 查看允许的命令列表和安全策略
     文档: https://docs.realconsole.com/shell-safety
```

**格式特点**:
- ✗ 符号 + 黄色错误代码
- 红色粗体错误名称
- 详细错误消息
- 绿色编号建议列表
- 灰色命令和文档链接
- 可选的源错误详情

### 4. Shell 模块迁移

#### 修改前（使用 String 错误）

```rust
fn is_safe_command(command: &str) -> Result<(), String> {
    if command.trim().is_empty() {
        return Err("Shell 命令不能为空".to_string());
    }
    // ...
}

pub async fn execute_shell(command: &str) -> Result<String, String> {
    // ...
}
```

#### 修改后（使用 RealError）

```rust
fn is_safe_command(command: &str) -> Result<(), RealError> {
    if command.trim().is_empty() {
        return Err(RealError::new(
            ErrorCode::ShellCommandEmpty,
            "Shell 命令不能为空",
        )
        .with_suggestion(FixSuggestion::new("输入有效的 shell 命令")));
    }

    // 检查危险模式
    if re.is_match(command) {
        return Err(RealError::new(
            ErrorCode::ShellDangerousCommand,
            "命令包含危险操作，已被安全策略阻止",
        )
        .with_suggestion(
            FixSuggestion::new("此命令可能造成系统损坏，建议使用更安全的替代方案"),
        )
        .with_suggestion(
            FixSuggestion::new("查看允许的命令列表和安全策略")
                .with_doc("https://docs.realconsole.com/shell-safety"),
        ));
    }

    Ok(())
}

pub async fn execute_shell(command: &str) -> Result<String, RealError> {
    // 超时处理
    Err(_) => {
        return Err(RealError::new(
            ErrorCode::ShellTimeoutError,
            format!("命令执行超时（超过 {} 秒）", COMMAND_TIMEOUT),
        )
        .with_suggestion(
            FixSuggestion::new("命令执行时间过长，请检查命令或增加超时时间"),
        )
        .with_suggestion(
            FixSuggestion::new("在配置文件中调整 features.shell_timeout")
                .with_command("vi realconsole.yaml"),
        ));
    }

    // 执行失败处理
    if !output.status.success() && result_text.is_empty() {
        return Err(RealError::new(
            ErrorCode::ShellExecutionError,
            format!("命令执行失败（退出码: {}）", output.status.code().unwrap_or(-1)),
        )
        .with_suggestion(FixSuggestion::new("检查命令语法和参数是否正确"))
        .with_suggestion(
            FixSuggestion::new("查看命令的帮助信息").with_command("man <command>"),
        ));
    }
}
```

**改进点**:
- 精确的错误代码（E301, E302, E303, E304）
- 多条建议（安全替代方案 + 文档链接）
- 可执行命令（vi realconsole.yaml, man <command>）
- 保留源错误信息（通过 map_err）

### 5. Config 模块迁移

#### 修改前（使用 anyhow）

```rust
use anyhow::{Context, Result};

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("无法读取配置文件: {}", path.display()))?;

        let config: Config = serde_yaml::from_str(&expanded)
            .with_context(|| format!("配置文件解析失败: {}", path.display()))?;

        Ok(config)
    }
}
```

#### 修改后（使用 RealError）

```rust
use crate::error::{ErrorCode, FixSuggestion, RealError};

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, RealError> {
        let content = fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                RealError::new(
                    ErrorCode::ConfigNotFound,
                    format!("配置文件不存在: {}", path.display()),
                )
                .with_suggestion(
                    FixSuggestion::new("运行配置向导创建配置文件")
                        .with_command("realconsole wizard"),
                )
                .with_suggestion(
                    FixSuggestion::new("参考示例配置手动创建")
                        .with_command("cp config/minimal.yaml realconsole.yaml"),
                )
            } else {
                RealError::new(
                    ErrorCode::FileReadError,
                    format!("无法读取配置文件: {}", path.display()),
                )
                .with_suggestion(FixSuggestion::new("检查文件权限和路径是否正确"))
                .with_source(e)
            }
        })?;

        let config: Config = serde_yaml::from_str(&expanded).map_err(|e| {
            RealError::new(
                ErrorCode::ConfigParseError,
                format!("配置文件解析失败: {}", path.display()),
            )
            .with_suggestion(FixSuggestion::new("检查 YAML 语法是否正确"))
            .with_suggestion(
                FixSuggestion::new("参考示例配置文件")
                    .with_doc("https://docs.realconsole.com/config"),
            )
            .with_source(e)
        })?;

        Ok(config)
    }
}
```

**改进点**:
- 区分 NotFound 和 ReadError（不同错误代码）
- 向导命令建议（realconsole wizard）
- YAML 语法检查建议
- 保留 serde_yaml 源错误

### 6. Agent 错误显示更新

#### 修改前

```rust
Err(e) => {
    format!("{} {}", "Shell 执行失败:".red(), e)
}
```

#### 修改后

```rust
Err(e) => {
    // 使用用户友好的错误格式
    e.format_user_friendly()
}
```

**更新位置**:
- `handle_shell()` - Shell 命令错误（第230行）
- `execute_intent()` - Intent 执行错误（第519行）

### 7. Main 错误显示更新

#### 修改前

```rust
Err(e) => {
    eprintln!("{} {}", "配置加载失败:".red(), e);
    eprintln!("{}", "使用默认配置".yellow());
    config::Config::default()
}
```

#### 修改后

```rust
Err(e) => {
    // 使用用户友好的错误格式显示详细信息
    eprintln!("{}", e.format_user_friendly());
    eprintln!("\n{}", "使用默认配置继续运行...".yellow());
    config::Config::default()
}
```

---

## 测试验证

### 1. 配置解析错误测试

**测试场景**: 无效的 YAML 语法

```bash
$ ./target/release/realconsole --config invalid.yaml --once "help"
```

**输出**:
```
✗ [E002] 配置文件解析失败

配置文件解析失败: invalid.yaml

建议修复方案:

  1. 检查 YAML 语法是否正确
  2. 参考示例配置文件
     文档: https://docs.realconsole.com/config

详细信息: did not find expected key at line 5 column 16, while parsing a block mapping at line 2 column 1


使用默认配置继续运行...
```

**验证点**:
- ✅ 显示错误代码 E002
- ✅ 显示错误标题和消息
- ✅ 显示编号建议列表
- ✅ 显示文档链接
- ✅ 显示 serde_yaml 源错误详情
- ✅ 程序优雅降级到默认配置

### 2. 危险命令阻止测试

**测试场景**: 执行危险的 rm 命令

```bash
$ ./target/release/realconsole --config config/minimal.yaml --once "!rm -rf /"
```

**输出**:
```
✗ [E302] 检测到危险命令

命令包含危险操作，已被安全策略阻止

建议修复方案:

  1. 此命令可能造成系统损坏，建议使用更安全的替代方案
  2. 查看允许的命令列表和安全策略
     文档: https://docs.realconsole.com/shell-safety
```

**验证点**:
- ✅ 显示错误代码 E302
- ✅ 清晰的危险警告
- ✅ 安全替代方案建议
- ✅ 文档链接指向安全策略

### 3. Shell 超时错误测试

**测试场景**: 执行超时命令（sleep 35秒，超过30秒限制）

```bash
$ ./target/release/realconsole --config config/minimal.yaml --once "!sleep 35"
```

**输出**:
```
✗ [E303] Shell 命令超时

命令执行超时（超过 30 秒）

建议修复方案:

  1. 命令执行时间过长，请检查命令或增加超时时间
  2. 在配置文件中调整 features.shell_timeout
     命令: vi realconsole.yaml
```

**验证点**:
- ✅ 显示错误代码 E303
- ✅ 超时时间信息
- ✅ 配置调整建议
- ✅ 可执行命令（vi realconsole.yaml）

### 4. 单元测试更新

**Shell 模块测试** (10个测试全部通过):
```bash
$ cargo test --lib shell_executor::tests
running 10 tests
test shell_executor::tests::test_execute_shell_empty ... ok
test shell_executor::tests::test_execute_shell_dangerous ... ok
test shell_executor::tests::test_execute_shell_timeout ... ok
test shell_executor::tests::test_is_safe_command ... ok
test shell_executor::tests::test_is_safe_command_additional_patterns ... ok
... (其他测试)

test result: ok. 10 passed; 0 failed
```

**Error 模块测试** (7个新增测试):
```rust
#[test]
fn test_error_code() { ... }           // 错误代码映射

#[test]
fn test_fix_suggestion() { ... }       // Builder 模式

#[test]
fn test_real_error_creation() { ... }  // 错误创建

#[test]
fn test_real_error_with_suggestions() { ... }  // 建议添加

#[test]
fn test_error_display() { ... }        // Display trait

#[test]
fn test_format_user_friendly() { ... } // 格式化输出

#[test]
fn test_error_code_uniqueness() { ... } // 代码唯一性
```

---

## 代码变更统计

### 新增文件

| 文件 | 行数 | 说明 |
|------|------|------|
| `src/error.rs` | 392 | 错误系统核心实现 |
| `docs/design/ERROR_SYSTEM_DESIGN.md` | 450+ | 错误系统设计文档 |
| `docs/progress/WEEK2_DAY3_SUMMARY.md` | 本文档 | Day 3 工作总结 |

### 修改文件

| 文件 | 变更 | 说明 |
|------|------|------|
| `src/lib.rs` | +2行 | 导出 error 模块 |
| `src/main.rs` | +1行, ~3行 | 添加 error 模块，更新错误显示 |
| `src/shell_executor.rs` | ~60行 | 迁移到 RealError |
| `src/config.rs` | ~40行 | 迁移到 RealError |
| `src/agent.rs` | ~6行 | 更新错误显示 |

**总计**: ~952行新增/修改代码

### 编译状态

| 指标 | 状态 |
|------|------|
| 编译成功 | ✅ |
| Clippy 错误 | 0 |
| Clippy 警告 | 20（dead code，预期） |
| 测试通过 | 271/271 (新增7个测试) |

---

## 技术亮点

### 1. Builder 模式设计

```rust
let error = RealError::new(ErrorCode::ShellTimeoutError, "超时")
    .with_suggestion(FixSuggestion::new("增加超时时间"))
    .with_suggestion(
        FixSuggestion::new("修改配置").with_command("vi config.yaml")
    );
```

**优点**:
- 链式调用，代码清晰
- 可选参数灵活组合
- 类型安全（编译时检查）

### 2. 错误源保留

```rust
.map_err(|e| {
    RealError::new(...)
        .with_source(e)  // 保留 serde_yaml 原始错误
})
```

**优点**:
- 不丢失底层错误信息
- 方便调试和问题定位
- 符合 Rust 错误处理最佳实践

### 3. 彩色输出优化

```rust
pub fn format_user_friendly(&self) -> String {
    output.push_str(&format!(
        "\n{} [{}] {}\n",
        "✗",
        self.code.code().yellow(),
        self.code.name().red().bold()
    ));
    // 绿色建议、灰色命令、蓝色链接
}
```

**优点**:
- 视觉层次分明
- 重要信息突出
- 符合 CLI 用户习惯

### 4. 错误代码分类

```
E001-E099: 配置相关（Config, Env, API Key）
E100-E199: LLM 相关（连接、认证、超时）
E200-E299: 工具相关（Tool Calling）
E300-E399: Shell 相关（安全、超时、执行）
E600-E699: 网络相关（HTTP, DNS, SSL）
E700-E799: 文件系统相关（读写、权限）
```

**优点**:
- 分类清晰，易于管理
- 编号有规律，易于记忆
- 预留空间，便于扩展

### 5. 与 thiserror 集成

```rust
#[derive(Error, Debug)]
pub struct RealError {
    ...
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for RealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.code.code(), self.code.name(), self.message)
    }
}
```

**优点**:
- 自动实现 Error trait
- 兼容 `?` 操作符
- 支持错误链（source）
- 同时支持简单显示和友好格式

---

## 用户体验改进

### 改进前

```
Shell 执行失败: [E302] 检测到危险命令: 命令包含危险操作，已被安全策略阻止
```

### 改进后

```
✗ [E302] 检测到危险命令

命令包含危险操作，已被安全策略阻止

建议修复方案:

  1. 此命令可能造成系统损坏，建议使用更安全的替代方案
  2. 查看允许的命令列表和安全策略
     文档: https://docs.realconsole.com/shell-safety
```

**提升对比**:
- ✅ 错误代码突出显示（黄色）
- ✅ 错误名称清晰（红色粗体）
- ✅ 多条建议（编号列表）
- ✅ 可执行命令（示例）
- ✅ 文档链接（帮助资源）
- ✅ 视觉层次分明

---

## 下一步计划

### Week 2 剩余任务（Day 4）

1. **进度指示器优化** (上午)
   - LLM 流式输出进度条
   - 长时间操作 spinner
   - 取消操作支持（Ctrl+C）

2. **帮助系统增强** (下午)
   - 上下文敏感帮助
   - 示例命令库
   - 快速参考卡片

3. **Week 2 总结** (傍晚)
   - 编写 Week 2 完整总结
   - 更新 CHANGELOG.md
   - 准备 Week 3 计划

### 未来改进空间

1. **国际化支持**: 当前硬编码中文，未来支持多语言
2. **错误恢复**: 部分错误可支持自动重试
3. **错误统计**: 记录常见错误，辅助改进
4. **错误模板**: 为第三方工具提供错误创建模板

---

## 经验总结

### 成功经验

1. **设计先行**: 先设计文档，再实现代码
2. **Builder 模式**: 灵活组合，代码清晰
3. **渐进式迁移**: 先 shell，再 config，再 agent
4. **实际测试**: 真实场景验证用户体验
5. **保留源错误**: 不丢失调试信息

### 遇到的问题

1. **模块导入**: 忘记在 main.rs 添加 error 模块
   - 解决：编译错误提示后快速修复

2. **错误源类型**: `with_source` 需要 boxed error
   - 解决：使用 `Into<Box<dyn Error>>` trait bound

3. **彩色输出测试**: 终端颜色在 CI 中可能失效
   - 待解决：考虑环境变量控制彩色输出

---

## 附录

### A. 错误代码速查表

```
配置相关:
  E001 - ConfigNotFound        配置文件不存在
  E002 - ConfigParseError      配置文件解析失败
  E003 - ConfigValidationError 配置验证失败
  E006 - ApiKeyInvalid         API Key 格式无效
  E007 - ApiKeyEmpty           API Key 为空

LLM 相关:
  E100 - LlmNotConfigured      LLM 未配置
  E101 - LlmConnectionError    无法连接到 LLM 服务
  E102 - LlmAuthError          LLM 认证失败
  E103 - LlmRateLimitError     超过 API 调用限额
  E104 - LlmTimeoutError       LLM 请求超时

Shell 相关:
  E300 - ShellDisabled         Shell 功能未启用
  E301 - ShellCommandEmpty     Shell 命令为空
  E302 - ShellDangerousCommand 检测到危险命令
  E303 - ShellTimeoutError     Shell 命令超时
  E304 - ShellExecutionError   Shell 命令执行失败

文件系统:
  E700 - FileNotFound          文件不存在
  E701 - FileReadError         文件读取失败
  E702 - FileWriteError        文件写入失败
```

### B. 错误创建模板

```rust
// 简单错误
RealError::new(
    ErrorCode::ToolNotFound,
    "工具 'calculator' 不存在",
)

// 带建议
RealError::new(
    ErrorCode::ConfigNotFound,
    "配置文件不存在",
)
.with_suggestion(
    FixSuggestion::new("运行向导").with_command("realconsole wizard")
)

// 带源错误
RealError::new(
    ErrorCode::FileReadError,
    format!("无法读取文件: {}", path),
)
.with_suggestion(FixSuggestion::new("检查文件权限"))
.with_source(io_error)
```

### C. 测试检查清单

- [ ] 错误代码正确显示
- [ ] 错误消息清晰易懂
- [ ] 建议有实际帮助
- [ ] 命令可直接执行
- [ ] 文档链接有效
- [ ] 彩色输出正常
- [ ] 源错误保留
- [ ] 单元测试覆盖

---

**文档版本**: v1.0
**编写日期**: 2025-10-15
**作者**: RealConsole Team
**状态**: ✅ Day 3 完成
