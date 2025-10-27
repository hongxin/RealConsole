//! 记忆条目定义
//!
//! 八维记忆空间的基本数据单元

use super::dimension::BaguaDimension;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 记忆条目
///
/// 八维记忆空间的基本存储单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// 唯一标识
    pub id: Uuid,

    /// 所属维度
    pub dimension: BaguaDimension,

    /// 记忆内容
    pub content: MemoryContent,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 相关性评分 (0.0-1.0)
    pub relevance: f64,

    /// 能量值 (0.0-1.0)
    ///
    /// 离维度高能量（显性），坎维度低能量（隐性）
    pub energy: f64,
}

impl MemoryEntry {
    /// 创建新的记忆条目
    pub fn new(dimension: BaguaDimension, content: MemoryContent) -> Self {
        // 根据维度设置默认能量值
        let energy = match dimension {
            BaguaDimension::Li => 0.8,   // 离：高能量（显性）
            BaguaDimension::Kan => 0.3,  // 坎：低能量（隐性）
            _ => 0.5,                     // 其他：中等能量
        };

        Self {
            id: Uuid::new_v4(),
            dimension,
            content,
            timestamp: Utc::now(),
            relevance: 1.0,
            energy,
        }
    }

    /// 设置相关性评分
    pub fn with_relevance(mut self, relevance: f64) -> Self {
        self.relevance = relevance.clamp(0.0, 1.0);
        self
    }

    /// 设置能量值
    pub fn with_energy(mut self, energy: f64) -> Self {
        self.energy = energy.clamp(0.0, 1.0);
        self
    }
}

/// 记忆内容（多态设计）
///
/// 每个维度对应不同类型的内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MemoryContent {
    /// 乾：意图目标
    Intent {
        goal: String,
        context: Option<String>,
        priority: f64,
    },

    /// 坤：对话记录
    Conversation {
        role: String,
        message: String,
        session_id: Option<String>,
    },

    /// 震：命令执行
    Action {
        command: String,
        result: ActionResult,
        duration_ms: u64,
    },

    /// 巽：趋势模式
    Trend {
        pattern: String,
        frequency: usize,
        change_rate: f64,
    },

    /// 坎：深层模式 ⭐
    Pattern {
        pattern_type: PatternType,
        confidence: f64,
        occurrences: usize,
    },

    /// 离：显性知识 ⭐
    Knowledge {
        fact: String,
        source: KnowledgeSource,
        confidence: f64,
    },

    /// 艮：系统快照
    Checkpoint {
        state: String,
        snapshot_id: String,
        metadata: Option<String>,
    },

    /// 兑：用户反馈
    Feedback {
        action: String,
        feedback_type: FeedbackType,
        score: f64,
    },
}

/// 命令执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    Success,
    Failure { error: String },
    Partial { message: String },
}

/// 模式类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// 频率模式：命令使用频率
    Frequency { command: String, count: usize },

    /// 序列模式：命令执行序列
    Sequence {
        commands: Vec<String>,
        occurrences: usize,
    },

    /// 修复模式：错误后的有效操作
    ErrorFix {
        error_pattern: String,
        fix_command: String,
        success_rate: f64,
    },
}

/// 知识来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// 从坎维度提取
    ExtractedFromKan,

    /// 用户明确告知
    UserProvided,

    /// LLM 推理
    LlmInferred,

    /// 系统观察
    SystemObserved,
}

/// 反馈类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackType {
    /// 接受建议
    Accept,

    /// 拒绝建议
    Reject,

    /// 修改建议
    Modify { original: String, modified: String },

    /// 显式反馈
    Explicit { rating: i32, comment: Option<String> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_entry_creation() {
        let content = MemoryContent::Intent {
            goal: "学习 Rust".to_string(),
            context: Some("编程学习".to_string()),
            priority: 0.8,
        };

        let entry = MemoryEntry::new(BaguaDimension::Qian, content);

        assert_eq!(entry.dimension, BaguaDimension::Qian);
        assert_eq!(entry.energy, 0.5);
        assert_eq!(entry.relevance, 1.0);
    }

    #[test]
    fn test_energy_defaults() {
        let li_entry = MemoryEntry::new(
            BaguaDimension::Li,
            MemoryContent::Knowledge {
                fact: "test".to_string(),
                source: KnowledgeSource::SystemObserved,
                confidence: 0.9,
            },
        );
        assert_eq!(li_entry.energy, 0.8); // 离：高能量

        let kan_entry = MemoryEntry::new(
            BaguaDimension::Kan,
            MemoryContent::Pattern {
                pattern_type: PatternType::Frequency {
                    command: "test".to_string(),
                    count: 10,
                },
                confidence: 0.9,
                occurrences: 10,
            },
        );
        assert_eq!(kan_entry.energy, 0.3); // 坎：低能量
    }
}
