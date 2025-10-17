//! 当前对话跟踪
//!
//! 用于在 REPL 会话中跟踪当前活跃的对话

use std::cell::RefCell;

thread_local! {
    /// 当前对话 ID（线程本地存储）
    static CURRENT_CONVERSATION: RefCell<Option<String>> = RefCell::new(None);
}

/// 获取当前对话 ID
pub fn get_current_conversation() -> Option<String> {
    CURRENT_CONVERSATION.with(|cell| cell.borrow().clone())
}

/// 设置当前对话 ID
pub fn set_current_conversation(id: Option<String>) {
    CURRENT_CONVERSATION.with(|cell| {
        *cell.borrow_mut() = id;
    });
}

/// 清除当前对话
pub fn clear_current_conversation() {
    set_current_conversation(None);
}

/// 检查是否有活跃对话
pub fn has_active_conversation() -> bool {
    get_current_conversation().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_tracking() {
        assert!(!has_active_conversation());

        set_current_conversation(Some("test-id".to_string()));
        assert!(has_active_conversation());
        assert_eq!(get_current_conversation(), Some("test-id".to_string()));

        clear_current_conversation();
        assert!(!has_active_conversation());
        assert_eq!(get_current_conversation(), None);
    }
}
