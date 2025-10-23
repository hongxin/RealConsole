//! 字符串工具函数
//!
//! 提供安全的字符串操作工具，特别是处理 UTF-8 多字节字符时的安全截断。

/// 安全截断字符串到指定字节长度
///
/// 该函数会自动调整截断位置，确保不会切到 UTF-8 多字节字符的中间。
///
/// # 参数
///
/// - `s`: 要截断的字符串
/// - `max_bytes`: 最大字节长度
///
/// # 返回
///
/// 截断后的字符串（如果需要截断，会添加 "..." 后缀）
///
/// # 示例
///
/// ```
/// use realconsole::utils::string::truncate_safe;
///
/// // 英文字符串
/// assert_eq!(truncate_safe("Hello, World!", 5), "Hello...");
///
/// // 中文字符串（每个中文字符占 3 字节）
/// assert_eq!(truncate_safe("你好世界", 6), "你好...");
///
/// // 混合字符串
/// assert_eq!(truncate_safe("Hello你好", 8), "Hello你...");
///
/// // 不需要截断
/// assert_eq!(truncate_safe("Short", 10), "Short");
/// ```
pub fn truncate_safe(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    // 找到安全的截断位置（UTF-8 字符边界）
    let mut cutoff = max_bytes.min(s.len());
    while cutoff > 0 && !s.is_char_boundary(cutoff) {
        cutoff -= 1;
    }

    // 如果截断位置为 0，说明第一个字符就超过了 max_bytes
    if cutoff == 0 {
        return "...".to_string();
    }

    format!("{}...", &s[..cutoff])
}

/// 按字符数（而非字节数）安全截断字符串
///
/// 该函数按照 Unicode 字符数进行截断，对中英文字符一视同仁。
///
/// # 参数
///
/// - `s`: 要截断的字符串
/// - `max_chars`: 最大字符数
///
/// # 返回
///
/// 截断后的字符串（如果需要截断，会添加 "..." 后缀）
///
/// # 示例
///
/// ```
/// use realconsole::utils::string::truncate_chars;
///
/// // 中英文字符统一按字符数计算
/// assert_eq!(truncate_chars("Hello, World!", 5), "Hello...");
/// assert_eq!(truncate_chars("你好世界", 2), "你好...");
/// assert_eq!(truncate_chars("Hello你好", 7), "Hello你好");
/// ```
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();

    if char_count <= max_chars {
        return s.to_string();
    }

    let truncated: String = s.chars().take(max_chars).collect();
    format!("{}...", truncated)
}

/// 智能截断：优先保持单词/词语完整性
///
/// 该函数会尝试在空格或标点符号处截断，避免截断单词/词语。
///
/// # 参数
///
/// - `s`: 要截断的字符串
/// - `max_bytes`: 最大字节长度（粗略估计）
///
/// # 返回
///
/// 截断后的字符串
pub fn truncate_smart(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    // 先安全截断到目标长度附近
    let mut cutoff = max_bytes.min(s.len());
    while cutoff > 0 && !s.is_char_boundary(cutoff) {
        cutoff -= 1;
    }

    let truncated = &s[..cutoff];

    // 尝试找到最后一个空格或标点符号
    if let Some(pos) = truncated.rfind(|c: char| c.is_whitespace() || c == ',' || c == '。' || c == '，') {
        if pos > max_bytes / 2 {  // 确保不会截得太短
            return format!("{}...", &s[..pos].trim_end());
        }
    }

    // 如果找不到合适的分割点，使用普通截断
    format!("{}...", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_safe_english() {
        assert_eq!(truncate_safe("Hello, World!", 5), "Hello...");
        assert_eq!(truncate_safe("Short", 10), "Short");
        assert_eq!(truncate_safe("", 5), "");
    }

    #[test]
    fn test_truncate_safe_chinese() {
        // 中文字符通常占 3 字节
        assert_eq!(truncate_safe("你好世界", 6), "你好...");
        assert_eq!(truncate_safe("你好世界", 9), "你好世...");
        assert_eq!(truncate_safe("你好世界", 12), "你好世界");
        assert_eq!(truncate_safe("中文测试字符串", 6), "中文...");
    }

    #[test]
    fn test_truncate_safe_mixed() {
        assert_eq!(truncate_safe("Hello你好", 8), "Hello你...");
        assert_eq!(truncate_safe("测试Test", 7), "测试T...");
    }

    #[test]
    fn test_truncate_safe_edge_cases() {
        // 第一个字符就超过限制
        assert_eq!(truncate_safe("你好", 2), "...");

        // 边界情况
        assert_eq!(truncate_safe("你", 3), "你");
        assert_eq!(truncate_safe("你", 2), "...");
    }

    #[test]
    fn test_truncate_chars() {
        assert_eq!(truncate_chars("Hello, World!", 5), "Hello...");
        assert_eq!(truncate_chars("你好世界", 2), "你好...");
        assert_eq!(truncate_chars("Hello你好", 7), "Hello你好");
        assert_eq!(truncate_chars("Short", 10), "Short");
    }

    #[test]
    fn test_truncate_smart() {
        // Smart truncate 会尝试在空格处截断，但如果空格位置不理想，会使用普通截断
        let result1 = truncate_smart("Hello World Test", 10);
        assert!(result1.starts_with("Hello") && result1.ends_with("..."));

        let result2 = truncate_smart("你好，世界", 9);
        assert!(result2.contains("你好") && result2.ends_with("..."));

        assert_eq!(truncate_smart("NoSpacesHere", 8), "NoSpaces...");
    }

    #[test]
    fn test_real_world_error_messages() {
        // 模拟实际的错误消息
        let error = "处理失败: 工具调用失败: LLM 调用失败: Parse error: Failed to parse JSON response";
        let truncated = truncate_safe(error, 40);

        // 确保不会 panic
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 43);  // 40 + "..." 的长度
    }
}
