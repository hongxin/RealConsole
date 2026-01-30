//! 知识图谱系统模块
//!
//! v2.3.0: 将 Claude Code 对话历史转化为可检索的知识库
//!
//! # 架构设计（一分为三）
//! - **数据层**：对话导入、索引存储
//! - **服务层**：主题提取、语义分析、检索引擎
//! - **展现层**：WebUI 知识浏览器
//!
//! # 核心功能
//! - 导入 Claude Code 对话历史（~/.claude/projects/）
//! - 自动提取主题和关键决策点
//! - 支持关键词搜索和时间线浏览
//! - 智能推荐相关对话

pub mod types;
pub mod importer;
pub mod topic;
pub mod retrieval;
pub mod service;

pub use types::*;
pub use importer::ConversationImporter;
pub use topic::TopicExtractor;
pub use retrieval::KnowledgeRetrieval;
pub use service::KnowledgeService;
