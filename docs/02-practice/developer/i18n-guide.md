# RealConsole 国际化（i18n）指南

**版本**: v1.1.0
**最后更新**: 2025-10-17

## 概述

RealConsole 的国际化系统遵循"一分为三"设计哲学，提供灵活、可扩展且容错的多语言支持。

### 设计哲学

- **明确态**：当前支持的已知语言（zh-CN 简体中文、en-US 美式英语）
- **演化态**：可扩展的架构，便于添加新语言
- **容错态**：多层回退机制，确保用户总能看到友好的界面

## 架构设计

### 核心组件

1. **i18n 模块** (`src/i18n.rs`)
   - `Language` enum：支持的语言枚举
   - `I18n` struct：国际化管理器
   - 全局函数：`t()`, `t_with_args()`, `set_language()`

2. **翻译资源** (`locales/`)
   - `zh-CN.yaml`：简体中文翻译
   - `en-US.yaml`：美式英语翻译

3. **配置支持**
   - CLI 参数：`--lang`
   - 配置文件：`display.language`
   - 环境变量：`REALCONSOLE_LANG`

### 语言选择优先级

```
命令行参数 (--lang)
    ↓ (未指定)
配置文件 (display.language)
    ↓ (未指定)
环境变量 (REALCONSOLE_LANG)
    ↓ (未指定)
系统语言 (LANG环境变量)
    ↓ (未指定/不支持)
默认中文 (zh-CN)
```

## 使用方法

### 1. 用户使用

#### 命令行指定语言

```bash
# 使用中文（默认）
realconsole

# 使用英文
realconsole --lang en-US
realconsole --lang en

# 也支持中文别名
realconsole --lang zh-CN
realconsole --lang 中文
```

#### 配置文件指定

在 `realconsole.yaml` 中添加：

```yaml
display:
  mode: minimal
  language: en-US  # 或 zh-CN
```

#### 环境变量指定

```bash
export REALCONSOLE_LANG=en-US
realconsole
```

### 2. 开发者使用

#### 添加新的翻译字符串

1. **在代码中使用 i18n 函数**：

```rust
use crate::i18n;

// 简单翻译
let msg = i18n::t("welcome.hint");
println!("{}", msg);

// 带参数的翻译
let msg = i18n::t_with_args("llm.init_failed", &[("type", "Primary")]);
eprintln!("{}", msg);
```

2. **在翻译文件中添加对应的键值**：

`locales/zh-CN.yaml`:
```yaml
welcome.hint: "直接输入问题或"
llm.init_failed: "⚠ {type} LLM 初始化失败:"
```

`locales/en-US.yaml`:
```yaml
welcome.hint: "Enter your question or"
llm.init_failed: "⚠ {type} LLM initialization failed:"
```

#### 翻译键命名规范

使用点分层级结构，遵循以下规范：

- **顶层分类**：按功能模块分组
  - `welcome.*` - 欢迎信息
  - `command.*` - 命令相关
  - `config.*` - 配置相关
  - `error.*` - 错误信息
  - `llm.*` - LLM 相关

- **命名风格**：小写字母，单词间用下划线
  - 正确：`first_run.welcome`, `config.not_found`
  - 错误：`FirstRun.Welcome`, `config.notFound`

- **参数占位符**：使用花括号
  - `"欢迎 {user_name}"`
  - `"{count} 个文件"`

#### 添加新语言支持

1. **在 `Language` enum 中添加新语言**：

```rust
pub enum Language {
    ZhCn,
    EnUs,
    JaJp,  // 新增日语
}
```

2. **实现相关方法**：

```rust
impl Language {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "zh-cn" | "zh" | "chinese" | "中文" => Some(Language::ZhCn),
            "en-us" | "en" | "english" => Some(Language::EnUs),
            "ja-jp" | "ja" | "japanese" | "日本語" => Some(Language::JaJp),
            _ => None,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::ZhCn => "zh-CN",
            Language::EnUs => "en-US",
            Language::JaJp => "ja-JP",
        }
    }
}
```

3. **创建翻译文件** `locales/ja-JP.yaml`

4. **添加内置翻译**（用于回退）：

```rust
fn builtin_translations_ja_jp() -> HashMap<String, String> {
    // ... 实现日语翻译
}
```

## 最佳实践

### 1. 翻译完整性

- ✅ 同时更新所有语言文件
- ✅ 保持键名一致
- ✅ 测试所有语言的显示效果

### 2. 参数化字符串

优先使用参数化而非字符串拼接：

```rust
// ✅ 推荐
i18n::t_with_args("message.count", &[("count", &count.to_string())])

// ❌ 不推荐
format!("{} {}", i18n::t("message.prefix"), count)
```

### 3. 上下文清晰

确保翻译键名能清楚表达含义：

```rust
// ✅ 清晰
i18n::t("config.wizard_failed")

// ❌ 模糊
i18n::t("error.1")
```

### 4. 回退机制

始终提供内置翻译作为回退：

```rust
// i18n 模块会自动处理：
// 1. 尝试从文件加载
// 2. 失败则使用内置翻译
// 3. 仍然失败则返回键名本身
```

## 测试

### 单元测试

```bash
# 运行 i18n 模块测试
cargo test --lib i18n

# 测试翻译功能
cargo test test_i18n_translation
```

### 手动测试

```bash
# 测试不同语言
./target/debug/realconsole --lang zh-CN
./target/debug/realconsole --lang en-US

# 测试回退机制
# 删除语言文件，应该使用内置翻译
mv locales/zh-CN.yaml locales/zh-CN.yaml.bak
./target/debug/realconsole
mv locales/zh-CN.yaml.bak locales/zh-CN.yaml
```

## 常见问题

### Q: 如何判断当前使用的语言？

A: 使用 `i18n::current_language()` 获取：

```rust
let lang = i18n::current_language();
println!("当前语言: {}", lang.code());
```

### Q: 翻译文件不生效怎么办？

A: 检查以下几点：
1. 文件路径是否正确（`locales/zh-CN.yaml`）
2. YAML 格式是否正确
3. 键名是否匹配
4. 是否重启了程序（翻译在启动时加载）

### Q: 如何处理长文本翻译？

A: 对于长文本，可以使用 YAML 的多行语法：

```yaml
help.full_text: |
  这是一段很长的帮助文本。
  可以跨越多行。
  保持良好的可读性。
```

### Q: 如何处理复数形式？

A: 当前版本暂不支持自动复数处理，建议：

```yaml
# 方案1：使用参数化
message.items: "{count} 个项目"

# 方案2：提供多个键
message.one_item: "1 个项目"
message.many_items: "{count} 个项目"
```

## 贡献指南

### 添加翻译

1. Fork 项目
2. 在 `locales/` 目录添加或修改翻译
3. 确保所有语言文件同步更新
4. 提交 PR，说明翻译的改进点

### 翻译规范

- **准确性**：准确传达原意
- **简洁性**：符合目标语言习惯
- **一致性**：术语翻译保持统一
- **本地化**：考虑文化差异

## 参考资料

- **i18n 模块源码**: `src/i18n.rs`
- **翻译资源**: `locales/`
- **使用示例**: `src/main.rs`, `src/repl.rs`
- **配置文档**: `docs/02-practice/user/user-guide.md`

---

如有问题或建议，请在 GitHub 提 Issue 或 PR。
