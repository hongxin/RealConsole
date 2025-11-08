//! 语音播报内容过滤器
//!
//! 将 LLM 响应文本转换为适合语音播报的内容

use regex::Regex;

/// 内容过滤配置
#[derive(Debug, Clone)]
pub struct FilterConfig {
    /// 是否过滤代码块
    pub filter_code_blocks: bool,
    /// 最大长度（字符数）
    pub max_length: usize,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            filter_code_blocks: true,
            max_length: 200,
        }
    }
}

/// 过滤并准备语音播报内容
///
/// # 处理逻辑
/// 1. 移除 Markdown 代码块（```...```）
/// 2. 移除行内代码标记（`...`）
/// 3. 移除 Markdown 格式标记（**、##、-）
/// 4. 截断到指定长度
/// 5. 清理多余空白
///
/// # 参数
/// - `text`: 原始文本
/// - `config`: 过滤配置
///
/// # 返回
/// 处理后的文本，如果为空或不适合播报则返回 None
pub fn filter_for_voice(text: &str, config: &FilterConfig) -> Option<String> {
    let mut result = text.to_string();

    // 1. 移除代码块（```...```）
    if config.filter_code_blocks {
        // 匹配 ```任意内容``` 或 ~~~任意内容~~~
        let code_block_re = Regex::new(r"```[\s\S]*?```|~~~[\s\S]*?~~~").ok()?;
        result = code_block_re.replace_all(&result, "[代码块]").to_string();
    }

    // 2. 移除行内代码标记 `...`
    let inline_code_re = Regex::new(r"`([^`]+)`").ok()?;
    result = inline_code_re.replace_all(&result, "$1").to_string();

    // 3. 移除 Markdown 格式标记
    // - 移除粗体/斜体标记 **text** 或 *text*
    result = result.replace("**", "");

    // - 移除标题标记 ## text（多行模式）
    let heading_re = Regex::new(r"(?m)^#{1,6}\s+").ok()?;
    result = heading_re.replace_all(&result, "").to_string();

    // - 移除列表标记 - item 或 * item（多行模式）
    let list_re = Regex::new(r"(?m)^[\s]*[-*]\s+").ok()?;
    result = list_re.replace_all(&result, "").to_string();

    // - 最后处理单独的星号（斜体标记）
    result = result.replace("*", "");

    // 4. 移除链接标记 [text](url)
    let link_re = Regex::new(r"\[([^\]]+)\]\([^\)]+\)").ok()?;
    result = link_re.replace_all(&result, "$1").to_string();

    // 5. 清理多余的空白
    // - 移除多个连续空格
    let whitespace_re = Regex::new(r"\s+").ok()?;
    result = whitespace_re.replace_all(&result, " ").to_string();

    // - 移除首尾空白
    result = result.trim().to_string();

    // 6. 检查是否为空
    if result.is_empty() {
        return None;
    }

    // 7. 截断到指定长度
    if result.chars().count() > config.max_length {
        let truncated: String = result.chars().take(config.max_length).collect();
        result = format!("{}... 后续内容已省略", truncated.trim());
    }

    // 8. 过滤不适合播报的内容
    // - 如果全是标点符号或特殊字符，跳过
    if result.chars().all(|c| !c.is_alphanumeric() && !c.is_whitespace()) {
        return None;
    }

    // - 如果太短（少于 3 个字符），跳过
    if result.chars().count() < 3 {
        return None;
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_code_blocks() {
        let config = FilterConfig::default();
        let text = "这是一段文本\n```rust\nfn main() {}\n```\n继续文本";
        let result = filter_for_voice(text, &config);
        assert!(result.is_some());
        let filtered = result.unwrap();
        assert!(filtered.contains("这是一段文本"));
        assert!(filtered.contains("[代码块]"));
        assert!(!filtered.contains("fn main"));
    }

    #[test]
    fn test_filter_inline_code() {
        let config = FilterConfig::default();
        let text = "使用 `cargo build` 来编译项目";
        let result = filter_for_voice(text, &config);
        assert_eq!(result.unwrap(), "使用 cargo build 来编译项目");
    }

    #[test]
    fn test_filter_markdown() {
        let config = FilterConfig::default();
        let text = "## 标题\n这是**粗体**和*斜体*文本\n- 列表项1\n- 列表项2";
        let result = filter_for_voice(text, &config);
        assert!(result.is_some());
        let filtered = result.unwrap();
        assert!(!filtered.contains("##"));
        assert!(!filtered.contains("**"));
        assert!(!filtered.contains("*"));
        assert!(!filtered.contains("-"));
    }

    #[test]
    fn test_truncate_long_text() {
        let config = FilterConfig {
            max_length: 20,
            ..Default::default()
        };
        let text = "这是一段很长很长很长很长很长很长很长的文本内容";
        let result = filter_for_voice(text, &config);
        assert!(result.is_some());
        let filtered = result.unwrap();
        assert!(filtered.contains("... 后续内容已省略"));
        assert!(filtered.chars().count() < 50);
    }

    #[test]
    fn test_filter_empty_result() {
        let config = FilterConfig::default();

        // 只有代码块，过滤后基本为空（只剩 "[代码块]"）
        let text = "```rust\nfn main() {}\n```";
        let result = filter_for_voice(text, &config);
        // "[代码块]" 是 5 个字符，会被保留
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "[代码块]");

        // 只有空白
        let text = "   \n\n   ";
        let result = filter_for_voice(text, &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_filter_short_text() {
        let config = FilterConfig::default();
        let text = "ok";
        let result = filter_for_voice(text, &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_filter_link() {
        let config = FilterConfig::default();
        let text = "查看 [文档](https://example.com) 了解更多";
        let result = filter_for_voice(text, &config);
        assert_eq!(result.unwrap(), "查看 文档 了解更多");
    }

    #[test]
    fn test_real_world_example() {
        let config = FilterConfig::default();
        let text = r#"好的，让我帮你分析这个函数：

```rust
fn calculate(x: i32, y: i32) -> i32 {
    x + y
}
```

这个函数的作用是**计算两个整数的和**。它接受两个 `i32` 类型的参数，返回它们的和。

主要特点：
- 简单直接
- 类型安全
- 性能高效"#;

        let result = filter_for_voice(text, &config);
        assert!(result.is_some());
        let filtered = result.unwrap();

        // 应该包含主要文本
        assert!(filtered.contains("这个函数的作用是计算两个整数的和"));

        // 不应该包含代码
        assert!(!filtered.contains("fn calculate"));

        // 不应该包含 Markdown 标记
        assert!(!filtered.contains("**"));
        assert!(!filtered.contains("`i32`"));
    }
}
