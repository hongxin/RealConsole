//! 对话上下文管理器
//!
//! 核心功能：
//! - 管理多轮对话上下文
//! - 智能检测是否需要上下文（Auto 模式）
//! - 自动清理过期上下文
//! - 构建发送给 LLM 的消息列表

use crate::config::{AutoClearConfig, ContextIncludeConfig, ContextMode, ConversationConfig};
use crate::conversation::Turn;
use crate::llm::Message;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// 轻量级上下文状态快照（用于 UI 显示，避免锁竞争）
#[derive(Debug, Clone)]
pub struct ContextSnapshot {
    /// 是否处于活跃状态
    pub is_active: bool,
    /// 当前轮次数
    pub turn_count: usize,
    /// 空闲时间（秒）
    pub idle_seconds: i64,
    /// 是否即将超时
    pub is_near_timeout: bool,
}

/// 对话上下文管理器
#[derive(Debug)]
pub struct ContextManager {
    /// 配置
    config: ConversationConfig,

    /// 对话轮次（双端队列，便于限制长度）
    turns: VecDeque<Turn>,

    /// 是否处于活跃状态（Manual 模式下由用户控制）
    is_active: bool,

    /// 最后活动时间
    last_activity: DateTime<Utc>,
}

impl ContextManager {
    /// 创建新的上下文管理器
    pub fn new(config: ConversationConfig) -> Self {
        Self {
            config,
            turns: VecDeque::new(),
            is_active: false,
            last_activity: Utc::now(),
        }
    }

    /// 获取当前模式
    pub fn mode(&self) -> ContextMode {
        self.config.mode
    }

    /// 是否处于活跃状态
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// 获取当前轮次数
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }

    /// 获取当前上下文总长度（字符数）
    pub fn context_length(&self) -> usize {
        self.turns
            .iter()
            .map(|turn| turn.user_input.len() + turn.assistant_response.len())
            .sum()
    }

    /// 手动启动上下文（Manual 模式）
    pub fn start(&mut self) {
        if self.config.mode == ContextMode::Manual {
            self.is_active = true;
            self.last_activity = Utc::now();
        }
    }

    /// 手动停止并清除上下文（Manual 模式）
    pub fn stop(&mut self) {
        self.is_active = false;
        self.turns.clear();
        self.last_activity = Utc::now();
    }

    /// 清除上下文但不停止（Manual 模式）
    pub fn clear(&mut self) {
        self.turns.clear();
        self.last_activity = Utc::now();
    }

    /// 检查是否应该启用上下文（Auto 模式）
    ///
    /// 触发条件：
    /// - 检测到代词引用（它、这个、那个、this、that、it）
    /// - 检测到追问（为什么、继续、详细、why、continue、more、how）
    /// - 检测到上下文依赖词（刚才、之前、上面、现在、那么、earlier、previous、now、then）
    pub fn should_enable_context(&self, input: &str) -> bool {
        if self.config.mode != ContextMode::Auto {
            return false;
        }

        let input_lower = input.to_lowercase();

        // 代词检测（中英文）
        let pronouns = [
            "它", "这个", "那个", "这些", "那些", "this", "that", "it", "these", "those",
        ];
        if pronouns.iter().any(|p| input_lower.contains(p)) {
            return true;
        }

        // 追问检测（中英文）
        let followups = [
            "为什么", "为啥", "怎么", "继续", "详细", "更多", "why", "how", "continue",
            "more", "detail", "explain",
        ];
        if followups.iter().any(|f| input_lower.contains(f)) {
            return true;
        }

        // 上下文依赖词（中英文）
        let context_refs = [
            "刚才", "之前", "上面", "前面", // 回顾过去
            "现在", "那么", "所以", "因此", "这样", "那", // 承接转折
            "earlier", "previous", "above", "before", // 英文回顾
            "now", "then", "so", "thus", "therefore", // 英文承接
        ];
        if context_refs.iter().any(|c| input_lower.contains(c)) {
            return true;
        }

        false
    }

    /// 检查是否应该使用上下文
    ///
    /// 根据不同模式决策：
    /// - Disabled: 永不使用
    /// - Manual: 仅在手动启用时使用
    /// - Auto: 智能检测 + 激活后继续使用直到清除
    pub fn should_use_context(&mut self, input: &str) -> bool {
        // 先清理过期上下文
        self.cleanup_if_needed();

        match self.config.mode {
            ContextMode::Disabled => false,
            ContextMode::Manual => self.is_active,
            ContextMode::Auto => {
                // 如果已经激活或有上下文，继续使用
                if self.is_active || !self.turns.is_empty() {
                    self.is_active = true; // 确保激活状态
                    self.last_activity = Utc::now();
                    return true;
                }

                // 否则检测是否应该启用
                if self.should_enable_context(input) {
                    self.is_active = true;
                    self.last_activity = Utc::now();
                    return true;
                }

                false
            }
        }
    }

    /// 添加新的对话轮次
    pub fn add_turn(&mut self, turn: Turn) {
        // 更新活动时间
        self.last_activity = Utc::now();

        // 添加轮次
        self.turns.push_back(turn);

        // 限制轮次数量
        while self.turns.len() > self.config.max_turns {
            self.turns.pop_front();
        }

        // 限制总长度
        while self.context_length() > self.config.max_context_length && !self.turns.is_empty() {
            self.turns.pop_front();
        }
    }

    /// 构建发送给 LLM 的消息列表
    ///
    /// 将历史对话轮次转换为 Message 格式
    pub fn build_messages(&self, current_input: &str) -> Vec<Message> {
        let mut messages = Vec::new();

        // 添加历史轮次
        for turn in &self.turns {
            // 用户消息
            messages.push(Message::user(&turn.user_input));

            // 助手回复
            messages.push(Message::assistant(&turn.assistant_response));

            // 可选：包含工具调用信息
            if self.config.include.tool_calls && !turn.tools_called.is_empty() {
                // 简化工具调用信息（避免过长）
                let tool_summary = turn
                    .tools_called
                    .iter()
                    .map(|tc| format!("[Tool: {}]", tc.tool_name))
                    .collect::<Vec<_>>()
                    .join(" ");

                // 添加为系统消息（不影响对话流）
                if !tool_summary.is_empty() {
                    messages.push(Message::system(format!("工具调用: {}", tool_summary)));
                }
            }
        }

        // 添加当前输入
        messages.push(Message::user(current_input));

        messages
    }

    /// 清理过期上下文（如果启用了自动清除）
    pub fn cleanup_if_needed(&mut self) {
        if !self.config.auto_clear.enabled {
            return;
        }

        // 检查空闲时间
        let idle_duration = Utc::now() - self.last_activity;
        let idle_seconds = idle_duration.num_seconds();

        if idle_seconds > self.config.auto_clear.idle_timeout as i64 {
            // 空闲超时，清除上下文
            self.turns.clear();

            // Auto 模式下，清除后重置活跃状态
            if self.config.mode == ContextMode::Auto {
                self.is_active = false;
            }
        }
    }

    /// 获取空闲时间（秒）
    pub fn idle_seconds(&self) -> i64 {
        (Utc::now() - self.last_activity).num_seconds()
    }

    /// 是否即将超时（剩余时间 < 20%）
    pub fn is_near_timeout(&self) -> bool {
        if !self.config.auto_clear.enabled {
            return false;
        }

        let idle = self.idle_seconds();
        let timeout = self.config.auto_clear.idle_timeout as i64;
        let remaining = timeout - idle;

        remaining > 0 && remaining < (timeout / 5) // 剩余时间 < 20%
    }

    /// 获取配置的引用
    pub fn config(&self) -> &ConversationConfig {
        &self.config
    }

    /// 获取所有轮次（用于调试和展示）
    pub fn turns(&self) -> &VecDeque<Turn> {
        &self.turns
    }

    /// 获取轻量级快照（无需异步，避免锁竞争）
    ///
    /// 用于 UI 显示等高频场景，避免频繁获取 RwLock
    pub fn snapshot(&self) -> ContextSnapshot {
        ContextSnapshot {
            is_active: self.is_active,
            turn_count: self.turns.len(),
            idle_seconds: self.idle_seconds(),
            is_near_timeout: self.is_near_timeout(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ContextMode;
    use crate::conversation::{ToolCall, Turn};
    use std::collections::HashMap;

    fn default_config() -> ConversationConfig {
        ConversationConfig {
            mode: ContextMode::Auto,
            max_turns: 5,
            max_context_length: 1000,
            auto_clear: AutoClearConfig {
                enabled: true,
                idle_timeout: 300,
                on_task_complete: false,
            },
            include: ContextIncludeConfig {
                tool_calls: true,
                shell_output: false,
                errors: true,
            },
        }
    }

    #[test]
    fn test_context_manager_creation() {
        let config = default_config();
        let manager = ContextManager::new(config);

        assert_eq!(manager.mode(), ContextMode::Auto);
        assert!(!manager.is_active());
        assert_eq!(manager.turn_count(), 0);
        assert_eq!(manager.context_length(), 0);
    }

    #[test]
    fn test_manual_mode_control() {
        let mut config = default_config();
        config.mode = ContextMode::Manual;
        let mut manager = ContextManager::new(config);

        // 初始状态：未启用
        assert!(!manager.is_active());

        // 手动启动
        manager.start();
        assert!(manager.is_active());

        // 手动停止
        manager.stop();
        assert!(!manager.is_active());
    }

    #[test]
    fn test_should_enable_context_pronouns() {
        let config = default_config();
        let manager = ContextManager::new(config);

        // 中文代词
        assert!(manager.should_enable_context("显示它的内容"));
        assert!(manager.should_enable_context("这个怎么用"));
        assert!(manager.should_enable_context("那些文件在哪"));

        // 英文代词
        assert!(manager.should_enable_context("show me that"));
        assert!(manager.should_enable_context("what is it"));
        assert!(manager.should_enable_context("list these files"));

        // 非代词
        assert!(!manager.should_enable_context("列出当前目录"));
    }

    #[test]
    fn test_should_enable_context_followups() {
        let config = default_config();
        let manager = ContextManager::new(config);

        // 中文追问
        assert!(manager.should_enable_context("为什么会这样"));
        assert!(manager.should_enable_context("继续"));
        assert!(manager.should_enable_context("详细说明"));

        // 英文追问
        assert!(manager.should_enable_context("why is that"));
        assert!(manager.should_enable_context("continue"));
        assert!(manager.should_enable_context("more details"));
    }

    #[test]
    fn test_should_enable_context_refs() {
        let config = default_config();
        let manager = ContextManager::new(config);

        // 回顾过去的上下文引用
        assert!(manager.should_enable_context("刚才说的是什么"));
        assert!(manager.should_enable_context("之前的结果"));
        assert!(manager.should_enable_context("上面提到的"));
        assert!(manager.should_enable_context("check the previous result"));

        // 承接转折的上下文引用
        assert!(manager.should_enable_context("现在你叫什么名字"));
        assert!(manager.should_enable_context("那么现在怎么办"));
        assert!(manager.should_enable_context("所以我们应该"));
        assert!(manager.should_enable_context("what should we do now"));
        assert!(manager.should_enable_context("then what happens"));
    }

    #[test]
    fn test_add_turn_and_limits() {
        let mut config = default_config();
        config.max_turns = 3;
        let mut manager = ContextManager::new(config);

        // 添加轮次
        for i in 1..=5 {
            let turn = Turn::new(
                format!("input {}", i),
                format!("response {}", i),
            );
            manager.add_turn(turn);
        }

        // 应该只保留最后 3 轮
        assert_eq!(manager.turn_count(), 3);

        // 检查保留的是最后 3 轮
        let turns: Vec<_> = manager.turns().iter().collect();
        assert_eq!(turns[0].user_input, "input 3");
        assert_eq!(turns[1].user_input, "input 4");
        assert_eq!(turns[2].user_input, "input 5");
    }

    #[test]
    fn test_context_length_limit() {
        let mut config = default_config();
        config.max_context_length = 50; // 很小的限制
        config.max_turns = 10;
        let mut manager = ContextManager::new(config);

        // 添加一些较长的轮次
        for i in 1..=5 {
            let turn = Turn::new(
                format!("input number {}", i),
                format!("response number {}", i),
            );
            manager.add_turn(turn);
        }

        // 应该因为长度限制而少于 5 轮
        assert!(manager.turn_count() < 5);
        assert!(manager.context_length() <= 50);
    }

    #[test]
    fn test_build_messages() {
        let config = default_config();
        let mut manager = ContextManager::new(config);

        // 添加一些轮次
        manager.add_turn(Turn::new("hello".to_string(), "hi there".to_string()));
        manager.add_turn(Turn::new("how are you".to_string(), "I'm good".to_string()));

        // 构建消息
        let messages = manager.build_messages("what's next");

        // 应该有 5 条消息：user, assistant, user, assistant, user
        assert_eq!(messages.len(), 5);

        // 验证顺序
        assert_eq!(messages[0].content.as_ref().unwrap(), "hello");
        assert_eq!(messages[1].content.as_ref().unwrap(), "hi there");
        assert_eq!(messages[2].content.as_ref().unwrap(), "how are you");
        assert_eq!(messages[3].content.as_ref().unwrap(), "I'm good");
        assert_eq!(messages[4].content.as_ref().unwrap(), "what's next");
    }

    #[test]
    fn test_disabled_mode() {
        let mut config = default_config();
        config.mode = ContextMode::Disabled;
        let mut manager = ContextManager::new(config);

        // Disabled 模式下永不启用上下文
        assert!(!manager.should_use_context("显示它的内容"));
        assert!(!manager.should_use_context("为什么"));
        assert!(!manager.should_use_context("继续"));
    }

    #[test]
    fn test_auto_mode_activation() {
        let config = default_config();
        let mut manager = ContextManager::new(config);

        // 初始状态未激活
        assert!(!manager.is_active());

        // 检测到代词后自动激活
        assert!(manager.should_use_context("显示它的内容"));
        assert!(manager.is_active());

        // 后续输入继续使用上下文
        assert!(manager.should_use_context("列出文件"));
    }

    #[test]
    fn test_snapshot() {
        let mut config = default_config();
        config.mode = ContextMode::Manual;
        let mut manager = ContextManager::new(config);

        // 初始状态快照
        let snapshot = manager.snapshot();
        assert!(!snapshot.is_active);
        assert_eq!(snapshot.turn_count, 0);

        // 启动上下文
        manager.start();

        // 添加轮次
        manager.add_turn(Turn::new("hello".to_string(), "hi".to_string()));
        manager.add_turn(Turn::new("how are you".to_string(), "fine".to_string()));

        // 获取快照
        let snapshot = manager.snapshot();

        // 验证快照数据
        assert!(snapshot.is_active);
        assert_eq!(snapshot.turn_count, 2);
        assert!(snapshot.idle_seconds >= 0);
        assert!(!snapshot.is_near_timeout);

        // 快照应该是轻量级的，可以多次调用
        let snapshot2 = manager.snapshot();
        assert_eq!(snapshot2.turn_count, snapshot.turn_count);
    }
}
