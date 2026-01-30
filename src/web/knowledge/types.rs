//! 知识系统数据类型定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// 消息 ID
    pub id: String,
    /// 消息类型：user, assistant, system
    pub message_type: MessageType,
    /// 消息内容
    pub content: String,
    /// 思考内容（仅 assistant）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 所属会话 ID
    pub session_id: String,
    /// 使用的工具
    #[serde(default)]
    pub tools_used: Vec<String>,
    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    User,
    Assistant,
    System,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    /// 会话 ID
    pub id: String,
    /// 会话摘要
    #[serde(default)]
    pub summary: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后修改时间
    pub modified_at: DateTime<Utc>,
    /// 消息数量
    pub message_count: usize,
    /// 文件名
    pub filename: String,
    /// 文件大小（MB）
    pub file_size_mb: f64,
    /// 是否为 Agent 子会话
    pub is_agent: bool,
    /// 父会话 ID（如果是 Agent）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<String>,
}

/// 主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    /// 主题 ID
    pub id: String,
    /// 主题名称
    pub name: String,
    /// 关键词列表
    pub keywords: Vec<String>,
    /// 关联的消息 ID 列表
    pub message_ids: Vec<String>,
    /// 时间范围
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
    /// 主题摘要
    pub summary: String,
    /// 消息数量
    pub message_count: usize,
}

/// 关键决策点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// 决策 ID
    pub id: String,
    /// 决策描述
    pub description: String,
    /// 相关消息 ID
    pub message_id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 决策类型
    pub decision_type: DecisionType,
    /// 影响的文件/模块
    #[serde(default)]
    pub affected_files: Vec<String>,
}

/// 决策类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    /// 架构设计决策
    Architecture,
    /// 实现方式选择
    Implementation,
    /// 技术选型
    TechChoice,
    /// Bug 修复策略
    BugFix,
    /// 重构方案
    Refactoring,
    /// 其他
    Other,
}

/// 代码变更关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    /// 变更 ID
    pub id: String,
    /// 文件路径
    pub file_path: String,
    /// 变更类型
    pub change_type: ChangeType,
    /// 关联的消息 ID
    pub message_id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 变更描述
    #[serde(default)]
    pub description: String,
}

/// 变更类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Create,
    Modify,
    Delete,
    Rename,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 消息
    pub message: ConversationMessage,
    /// 相关性分数
    pub relevance_score: f64,
    /// 匹配的关键词
    pub matched_keywords: Vec<String>,
    /// 上下文（前后消息）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<SearchContext>,
}

/// 搜索上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchContext {
    /// 前一条消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<Box<ConversationMessage>>,
    /// 后一条消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<Box<ConversationMessage>>,
}

/// 主题摘要（用于列表展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSummary {
    /// 主题 ID
    pub id: String,
    /// 主题名称
    pub name: String,
    /// 消息数量
    pub message_count: usize,
    /// 时间范围
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
}

/// 导入统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportStats {
    /// 导入的会话数
    pub sessions_imported: usize,
    /// 导入的 Agent 数
    pub agents_imported: usize,
    /// 导入的消息总数
    pub messages_imported: usize,
    /// 总文件大小（MB）
    pub total_size_mb: f64,
    /// 时间范围
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// 导入耗时（ms）
    pub duration_ms: u64,
    /// 跳过的文件数
    pub files_skipped: usize,
    /// 错误数
    pub errors: usize,
}

/// 更新统计（增量导入）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateStats {
    /// 新增的会话数
    pub new_sessions: usize,
    /// 更新的会话数
    pub updated_sessions: usize,
    /// 新增的消息数
    pub new_messages: usize,
    /// 更新耗时（ms）
    pub duration_ms: u64,
}

/// 知识库元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeMetadata {
    /// 项目名称
    pub project: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 会话总数
    pub total_sessions: usize,
    /// Agent 总数
    pub total_agents: usize,
    /// 消息总数
    pub total_messages: usize,
    /// 主题总数
    pub total_topics: usize,
    /// 决策点总数
    pub total_decisions: usize,
    /// 数据源路径
    pub source_path: String,
}

/// 知识库完整数据（用于导出/序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// 元数据
    pub metadata: KnowledgeMetadata,
    /// 会话列表
    pub sessions: Vec<ConversationSession>,
    /// 消息列表
    pub messages: Vec<ConversationMessage>,
    /// 主题列表
    pub topics: Vec<Topic>,
    /// 决策点列表
    pub decisions: Vec<Decision>,
}

/// WebSocket 知识系统消息（客户端 → 服务端）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KnowledgeClientMessage {
    /// 导入对话历史
    Import { source_path: Option<String> },
    /// 搜索
    Search { query: String, limit: Option<usize> },
    /// 按时间范围查询
    SearchByTime { start: DateTime<Utc>, end: DateTime<Utc> },
    /// 获取主题列表
    ListTopics,
    /// 获取主题详情
    GetTopic { topic_id: String },
    /// 获取会话列表
    ListSessions,
    /// 获取知识库元数据
    GetMetadata,
    /// 增量更新
    IncrementalUpdate,
}

/// WebSocket 知识系统消息（服务端 → 客户端）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KnowledgeServerMessage {
    /// 导入完成
    ImportComplete { stats: ImportStats },
    /// 搜索结果
    SearchResults { results: Vec<SearchResult>, total: usize },
    /// 主题列表
    TopicList { topics: Vec<TopicSummary> },
    /// 主题详情
    TopicDetail { topic: Topic },
    /// 会话列表
    SessionList { sessions: Vec<ConversationSession> },
    /// 知识库元数据
    Metadata { metadata: KnowledgeMetadata },
    /// 更新完成
    UpdateComplete { stats: UpdateStats },
    /// 错误
    Error { message: String },
    /// 进度更新
    Progress { current: usize, total: usize, stage: String },
}
