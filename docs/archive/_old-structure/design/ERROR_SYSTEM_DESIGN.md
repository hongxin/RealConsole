# RealConsole 错误系统设计

**日期**: 2025-10-15
**阶段**: Phase 5.3 Week 2 Day 3
**状态**: 设计中

---

## 概述

设计统一的错误处理系统，提供友好的错误消息、明确的错误代码、和建议性的修复方案，提升用户体验和问题排查效率。

### 设计目标

1. **一致性**: 所有模块使用统一的错误格式
2. **可操作性**: 提供明确的修复建议
3. **可追踪性**: 错误代码便于搜索和文档化
4. **用户友好**: 清晰的错误消息，避免技术术语
5. **国际化准备**: 错误代码与消息分离

---

## 错误代码系统

### 编码规则

```
EXXX - 错误代码格式

E = Error
XXX = 三位数字

分类:
E001-E099: 配置错误
E100-E199: LLM 错误
E200-E299: 工具执行错误
E300-E399: Shell 执行错误
E400-E499: 内存/日志错误
E500-E599: Intent DSL 错误
E600-E699: 网络错误
E700-E799: 文件系统错误
E800-E899: 权限错误
E900-E999: 内部错误
```

### 错误代码清单

#### 配置错误 (E001-E099)

| 代码 | 名称 | 描述 | 建议 |
|------|------|------|------|
| E001 | ConfigNotFound | 配置文件不存在 | 运行 `realconsole wizard` 创建配置 |
| E002 | ConfigParseError | 配置文件格式错误 | 检查 YAML 语法，参考 config/minimal.yaml |
| E003 | ConfigValidationError | 配置验证失败 | 查看具体字段错误信息 |
| E004 | EnvFileNotFound | .env 文件不存在 | 创建 .env 文件或运行 wizard |
| E005 | EnvVarMissing | 必需的环境变量缺失 | 在 .env 中设置变量 |
| E006 | ApiKeyInvalid | API Key 格式无效 | 检查 API Key 格式（应以 sk- 开头） |
| E007 | ApiKeyEmpty | API Key 为空 | 在 .env 中设置 DEEPSEEK_API_KEY |

#### LLM 错误 (E100-E199)

| 代码 | 名称 | 描述 | 建议 |
|------|------|------|------|
| E100 | LlmNotConfigured | LLM 未配置 | 在配置文件中设置 llm.primary |
| E101 | LlmConnectionError | 无法连接到 LLM 服务 | 检查网络和 endpoint 配置 |
| E102 | LlmAuthError | LLM 认证失败 | 检查 API Key 是否正确 |
| E103 | LlmRateLimitError | 超过 API 调用限额 | 稍后重试或升级 API 套餐 |
| E104 | LlmTimeoutError | LLM 请求超时 | 检查网络或增加超时时间 |
| E105 | LlmResponseError | LLM 响应格式错误 | 联系技术支持 |
| E106 | LlmModelNotFound | 模型不存在 | 检查模型名称或更新配置 |

#### 工具执行错误 (E200-E299)

| 代码 | 名称 | 描述 | 建议 |
|------|------|------|------|
| E200 | ToolNotFound | 工具不存在 | 使用 /tools list 查看可用工具 |
| E201 | ToolParameterError | 工具参数错误 | 使用 /tools info <name> 查看参数 |
| E202 | ToolExecutionError | 工具执行失败 | 查看错误详情 |
| E203 | ToolTimeoutError | 工具执行超时 | 增加超时时间或优化操作 |
| E204 | ToolPermissionDenied | 工具权限不足 | 检查文件/目录权限 |

#### Shell 执行错误 (E300-E399)

| 代码 | 名称 | 描述 | 建议 |
|------|------|------|------|
| E300 | ShellDisabled | Shell 功能未启用 | 在配置中设置 features.shell_enabled: true |
| E301 | ShellCommandEmpty | Shell 命令为空 | 输入有效的 shell 命令 |
| E302 | ShellDangerousCommand | 检测到危险命令 | 此命令已被阻止，详见安全策略 |
| E303 | ShellTimeoutError | Shell 命令超时 | 命令执行时间过长（超过30秒） |
| E304 | ShellExecutionError | Shell 命令执行失败 | 查看命令输出和退出码 |

#### 网络错误 (E600-E699)

| 代码 | 名称 | 描述 | 建议 |
|------|------|------|------|
| E600 | NetworkError | 网络错误 | 检查网络连接 |
| E601 | HttpError | HTTP 请求失败 | 检查服务状态和网络 |
| E602 | DnsError | DNS 解析失败 | 检查域名配置 |
| E603 | SslError | SSL/TLS 错误 | 检查证书或使用 HTTP |

#### 文件系统错误 (E700-E799)

| 代码 | 名称 | 描述 | 建议 |
|------|------|------|------|
| E700 | FileNotFound | 文件不存在 | 检查文件路径 |
| E701 | FileReadError | 文件读取失败 | 检查文件权限 |
| E702 | FileWriteError | 文件写入失败 | 检查目录权限和磁盘空间 |
| E703 | DirectoryNotFound | 目录不存在 | 创建目录或检查路径 |

---

## 错误类型设计

### 核心错误类型

```rust
use thiserror::Error;
use std::fmt;

/// RealConsole 错误代码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // 配置错误 (E001-E099)
    ConfigNotFound,
    ConfigParseError,
    ConfigValidationError,
    EnvFileNotFound,
    EnvVarMissing,
    ApiKeyInvalid,
    ApiKeyEmpty,

    // LLM 错误 (E100-E199)
    LlmNotConfigured,
    LlmConnectionError,
    LlmAuthError,
    LlmRateLimitError,
    LlmTimeoutError,
    LlmResponseError,
    LlmModelNotFound,

    // 工具错误 (E200-E299)
    ToolNotFound,
    ToolParameterError,
    ToolExecutionError,
    ToolTimeoutError,
    ToolPermissionDenied,

    // Shell 错误 (E300-E399)
    ShellDisabled,
    ShellCommandEmpty,
    ShellDangerousCommand,
    ShellTimeoutError,
    ShellExecutionError,

    // 网络错误 (E600-E699)
    NetworkError,
    HttpError,
    DnsError,
    SslError,

    // 文件系统错误 (E700-E799)
    FileNotFound,
    FileReadError,
    FileWriteError,
    DirectoryNotFound,
}

impl ErrorCode {
    /// 获取错误代码编号
    pub fn code(&self) -> &'static str {
        match self {
            // 配置错误
            Self::ConfigNotFound => "E001",
            Self::ConfigParseError => "E002",
            Self::ConfigValidationError => "E003",
            Self::EnvFileNotFound => "E004",
            Self::EnvVarMissing => "E005",
            Self::ApiKeyInvalid => "E006",
            Self::ApiKeyEmpty => "E007",

            // LLM 错误
            Self::LlmNotConfigured => "E100",
            Self::LlmConnectionError => "E101",
            Self::LlmAuthError => "E102",
            Self::LlmRateLimitError => "E103",
            Self::LlmTimeoutError => "E104",
            Self::LlmResponseError => "E105",
            Self::LlmModelNotFound => "E106",

            // 工具错误
            Self::ToolNotFound => "E200",
            Self::ToolParameterError => "E201",
            Self::ToolExecutionError => "E202",
            Self::ToolTimeoutError => "E203",
            Self::ToolPermissionDenied => "E204",

            // Shell 错误
            Self::ShellDisabled => "E300",
            Self::ShellCommandEmpty => "E301",
            Self::ShellDangerousCommand => "E302",
            Self::ShellTimeoutError => "E303",
            Self::ShellExecutionError => "E304",

            // 网络错误
            Self::NetworkError => "E600",
            Self::HttpError => "E601",
            Self::DnsError => "E602",
            Self::SslError => "E603",

            // 文件系统错误
            Self::FileNotFound => "E700",
            Self::FileReadError => "E701",
            Self::FileWriteError => "E702",
            Self::DirectoryNotFound => "E703",
        }
    }

    /// 获取友好的错误名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::ConfigNotFound => "配置文件不存在",
            Self::ConfigParseError => "配置文件解析失败",
            Self::ConfigValidationError => "配置验证失败",
            Self::EnvFileNotFound => "环境变量文件不存在",
            Self::EnvVarMissing => "缺少必需的环境变量",
            Self::ApiKeyInvalid => "API Key 格式无效",
            Self::ApiKeyEmpty => "API Key 为空",

            Self::LlmNotConfigured => "LLM 未配置",
            Self::LlmConnectionError => "无法连接到 LLM 服务",
            Self::LlmAuthError => "LLM 认证失败",
            Self::LlmRateLimitError => "超过 API 调用限额",
            Self::LlmTimeoutError => "LLM 请求超时",
            Self::LlmResponseError => "LLM 响应格式错误",
            Self::LlmModelNotFound => "模型不存在",

            Self::ToolNotFound => "工具不存在",
            Self::ToolParameterError => "工具参数错误",
            Self::ToolExecutionError => "工具执行失败",
            Self::ToolTimeoutError => "工具执行超时",
            Self::ToolPermissionDenied => "工具权限不足",

            Self::ShellDisabled => "Shell 功能未启用",
            Self::ShellCommandEmpty => "Shell 命令为空",
            Self::ShellDangerousCommand => "检测到危险命令",
            Self::ShellTimeoutError => "Shell 命令超时",
            Self::ShellExecutionError => "Shell 命令执行失败",

            Self::NetworkError => "网络错误",
            Self::HttpError => "HTTP 请求失败",
            Self::DnsError => "DNS 解析失败",
            Self::SslError => "SSL/TLS 错误",

            Self::FileNotFound => "文件不存在",
            Self::FileReadError => "文件读取失败",
            Self::FileWriteError => "文件写入失败",
            Self::DirectoryNotFound => "目录不存在",
        }
    }
}

/// 修复建议
#[derive(Debug, Clone)]
pub struct FixSuggestion {
    /// 建议描述
    pub description: String,
    /// 示例命令（可选）
    pub command: Option<String>,
    /// 文档链接（可选）
    pub doc_link: Option<String>,
}

impl FixSuggestion {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            command: None,
            doc_link: None,
        }
    }

    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn with_doc(mut self, link: impl Into<String>) -> Self {
        self.doc_link = Some(link.into());
        self
    }
}

/// RealConsole 统一错误类型
#[derive(Error, Debug)]
pub struct RealError {
    /// 错误代码
    pub code: ErrorCode,
    /// 错误消息
    pub message: String,
    /// 修复建议
    pub suggestions: Vec<FixSuggestion>,
    /// 底层错误（可选）
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl RealError {
    /// 创建新的错误
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            suggestions: Vec::new(),
            source: None,
        }
    }

    /// 添加修复建议
    pub fn with_suggestion(mut self, suggestion: FixSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// 添加多个建议
    pub fn with_suggestions(mut self, suggestions: Vec<FixSuggestion>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    /// 添加底层错误
    pub fn with_source(mut self, source: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// 格式化为用户友好的错误消息
    pub fn format_user_friendly(&self) -> String {
        let mut output = String::new();

        // 错误标题
        output.push_str(&format!("\n{} [{}] {}\n",
            "✗".red(),
            self.code.code().yellow(),
            self.code.name().red().bold()
        ));

        // 错误详情
        output.push_str(&format!("\n{}\n", self.message));

        // 修复建议
        if !self.suggestions.is_empty() {
            output.push_str(&format!("\n{}\n", "建议修复方案:".green().bold()));
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                output.push_str(&format!("\n  {}. {}", i + 1, suggestion.description));

                if let Some(cmd) = &suggestion.command {
                    output.push_str(&format!("\n     {}: {}", "命令".dimmed(), cmd.cyan()));
                }

                if let Some(link) = &suggestion.doc_link {
                    output.push_str(&format!("\n     {}: {}", "文档".dimmed(), link.blue().underline()));
                }
            }
            output.push('\n');
        }

        // 底层错误
        if let Some(source) = &self.source {
            output.push_str(&format!("\n{} {}\n", "详细信息:".dimmed(), source.to_string().dimmed()));
        }

        output
    }
}

impl fmt::Display for RealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.code.code(), self.code.name(), self.message)
    }
}
```

---

## 使用示例

### 示例 1: 配置文件不存在

```rust
use error::{RealError, ErrorCode, FixSuggestion};

fn load_config(path: &str) -> Result<Config, RealError> {
    if !Path::new(path).exists() {
        return Err(
            RealError::new(
                ErrorCode::ConfigNotFound,
                format!("配置文件 '{}' 不存在", path)
            )
            .with_suggestion(
                FixSuggestion::new("运行配置向导创建配置文件")
                    .with_command("realconsole wizard")
            )
            .with_suggestion(
                FixSuggestion::new("复制示例配置并手动修改")
                    .with_command("cp config/minimal.yaml realconsole.yaml")
                    .with_doc("https://docs.realconsole.com/config")
            )
        );
    }

    // ...
}
```

**输出**:
```
✗ [E001] 配置文件不存在

配置文件 'realconsole.yaml' 不存在

建议修复方案:

  1. 运行配置向导创建配置文件
     命令: realconsole wizard

  2. 复制示例配置并手动修改
     命令: cp config/minimal.yaml realconsole.yaml
     文档: https://docs.realconsole.com/config
```

### 示例 2: LLM 认证失败

```rust
fn connect_to_llm(api_key: &str) -> Result<Client, RealError> {
    match validate_api_key(api_key) {
        Err(e) if e.status() == 401 => {
            Err(
                RealError::new(
                    ErrorCode::LlmAuthError,
                    "Deepseek API Key 认证失败"
                )
                .with_suggestion(
                    FixSuggestion::new("检查 .env 文件中的 DEEPSEEK_API_KEY 是否正确")
                        .with_command("cat .env | grep DEEPSEEK_API_KEY")
                )
                .with_suggestion(
                    FixSuggestion::new("从官网获取新的 API Key")
                        .with_doc("https://platform.deepseek.com")
                )
                .with_source(e)
            )
        }
        // ...
    }
}
```

### 示例 3: Shell 危险命令

```rust
fn execute_shell(command: &str) -> Result<String, RealError> {
    if is_dangerous(command) {
        return Err(
            RealError::new(
                ErrorCode::ShellDangerousCommand,
                format!("命令 '{}' 包含危险操作", command)
            )
            .with_suggestion(
                FixSuggestion::new("此命令已被安全策略阻止")
            )
            .with_suggestion(
                FixSuggestion::new("查看允许的命令列表")
                    .with_doc("https://docs.realconsole.com/shell-safety")
            )
        );
    }

    // ...
}
```

---

## 实现计划

### Phase 1: 基础错误系统 (Day 3 上午)
- [x] 设计错误代码系统
- [ ] 创建 `src/error.rs` 模块
- [ ] 实现 `ErrorCode`、`FixSuggestion`、`RealError`
- [ ] 添加单元测试

### Phase 2: 核心模块集成 (Day 3 下午)
- [ ] 在 `config.rs` 中应用错误系统
- [ ] 在 `shell_executor.rs` 中应用
- [ ] 在 `wizard/` 中应用
- [ ] 更新错误消息

### Phase 3: 完善与优化 (Week 2 后续)
- [ ] 添加更多错误代码
- [ ] 完善修复建议
- [ ] 创建错误文档（每个代码一个页面）
- [ ] 国际化支持（i18n）

---

## 错误消息规范

### 格式规范

```
✗ [EXXX] 错误名称

错误详细描述（1-2句话，说明发生了什么）

建议修复方案:

  1. 具体的修复步骤
     命令: 示例命令（如果有）
     文档: 文档链接（如果有）

  2. 替代方案（可选）

详细信息: 底层错误信息（如果有）
```

### 编写原则

1. **清晰简洁**: 避免技术术语，使用用户能理解的语言
2. **可操作**: 提供具体的修复步骤，不只是描述问题
3. **友好态度**: 使用"建议"而非"必须"，避免责怪用户
4. **上下文**: 提供足够的上下文帮助定位问题
5. **一致性**: 所有错误使用相同的格式

### 示例对比

**❌ 差的错误消息**:
```
Error: File not found
```

**✅ 好的错误消息**:
```
✗ [E700] 文件不存在

文件 '/path/to/file.txt' 不存在

建议修复方案:

  1. 检查文件路径是否正确
     命令: ls -la /path/to/

  2. 如果文件被移动或删除，请重新创建
```

---

## 国际化准备

### 错误代码与消息分离

```rust
// 错误代码是常量
ErrorCode::ConfigNotFound => "E001"

// 消息可以从 i18n 资源加载
fn get_message(code: ErrorCode, locale: &str) -> String {
    match locale {
        "zh_CN" => load_chinese_message(code),
        "en_US" => load_english_message(code),
        _ => load_default_message(code),
    }
}
```

### 未来扩展

1. 支持多语言错误消息
2. 用户可选择语言（`LANG` 环境变量）
3. 错误消息模板化（支持参数替换）

---

**文档版本**: v1.0
**编写日期**: 2025-10-15
**作者**: RealConsole Team
**状态**: 设计完成，待实现
