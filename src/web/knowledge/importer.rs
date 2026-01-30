//! 对话历史导入器
//!
//! 从 Claude Code 目录或备份文件导入对话历史

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::fs;

use super::types::*;

/// 对话导入器
pub struct ConversationImporter {
    /// Claude Code 项目目录
    claude_dir: PathBuf,
}

impl ConversationImporter {
    /// 创建导入器
    ///
    /// # Arguments
    /// * `claude_dir` - Claude Code 项目目录，如 ~/.claude/projects/-Users-xxx-project/
    pub fn new(claude_dir: PathBuf) -> Self {
        Self { claude_dir }
    }

    /// 从默认 Claude Code 目录创建导入器
    pub fn from_default_dir(project_path: &str) -> Result<Self> {
        let home = dirs::home_dir().context("无法获取用户主目录")?;
        let project_dir_name = project_path.replace('/', "-");
        let claude_dir = home.join(".claude/projects").join(project_dir_name);
        Ok(Self::new(claude_dir))
    }

    /// 导入 Claude Code 目录中的所有对话
    pub async fn import_from_claude_dir(&self) -> Result<(KnowledgeBase, ImportStats)> {
        let start = Instant::now();
        let mut stats = ImportStats::default();

        // 检查目录是否存在
        if !self.claude_dir.exists() {
            anyhow::bail!("Claude Code 目录不存在: {:?}", self.claude_dir);
        }

        // 读取会话索引
        let sessions_index = self.read_sessions_index().await?;

        // 读取所有 JSONL 文件
        let mut sessions = Vec::new();
        let mut agents = Vec::new();
        let mut all_messages = Vec::new();

        let mut entries = fs::read_dir(&self.claude_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                match self.parse_jsonl_file(&path).await {
                    Ok(messages) => {
                        let filename = path.file_name().unwrap().to_string_lossy().to_string();
                        let file_size = fs::metadata(&path).await?.len() as f64 / 1024.0 / 1024.0;
                        let session_id = path.file_stem().unwrap().to_string_lossy().to_string();

                        let is_agent = filename.starts_with("agent-");

                        // 获取时间范围
                        let (created_at, modified_at) = self.get_time_range(&messages);

                        let session = ConversationSession {
                            id: session_id.clone(),
                            summary: self.find_session_summary(&sessions_index, &session_id),
                            created_at,
                            modified_at,
                            message_count: messages.len(),
                            filename,
                            file_size_mb: (file_size * 100.0).round() / 100.0,
                            is_agent,
                            parent_session_id: None, // TODO: 从消息中提取
                        };

                        if is_agent {
                            agents.push(session);
                            stats.agents_imported += 1;
                        } else {
                            sessions.push(session);
                            stats.sessions_imported += 1;
                        }

                        stats.messages_imported += messages.len();
                        stats.total_size_mb += file_size;
                        all_messages.extend(messages);
                    }
                    Err(e) => {
                        eprintln!("解析文件失败 {:?}: {}", path, e);
                        stats.errors += 1;
                    }
                }
            }
        }

        // 计算时间范围
        if !all_messages.is_empty() {
            let mut timestamps: Vec<_> = all_messages.iter().map(|m| m.timestamp).collect();
            timestamps.sort();
            stats.date_range = Some((timestamps[0], *timestamps.last().unwrap()));
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;
        stats.total_size_mb = (stats.total_size_mb * 100.0).round() / 100.0;

        // 构建知识库
        let metadata = KnowledgeMetadata {
            project: "RealConsole".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            total_sessions: sessions.len(),
            total_agents: agents.len(),
            total_messages: all_messages.len(),
            total_topics: 0, // 后续通过 TopicExtractor 填充
            total_decisions: 0,
            source_path: self.claude_dir.to_string_lossy().to_string(),
        };

        // 合并 sessions 和 agents
        let mut all_sessions = sessions;
        all_sessions.extend(agents);

        let knowledge_base = KnowledgeBase {
            metadata,
            sessions: all_sessions,
            messages: all_messages,
            topics: Vec::new(),
            decisions: Vec::new(),
        };

        Ok((knowledge_base, stats))
    }

    /// 从备份 JSON 文件导入
    pub async fn import_from_backup(&self, backup_file: &Path) -> Result<(KnowledgeBase, ImportStats)> {
        let start = Instant::now();

        let content = fs::read_to_string(backup_file).await?;
        let backup: serde_json::Value = serde_json::from_str(&content)?;

        let mut stats = ImportStats::default();
        let mut sessions = Vec::new();
        let mut all_messages = Vec::new();

        // 解析 sessions
        if let Some(sessions_array) = backup.get("sessions").and_then(|v| v.as_array()) {
            for session_val in sessions_array {
                if let Ok(session) = self.parse_backup_session(session_val) {
                    stats.sessions_imported += 1;
                    stats.messages_imported += session.messages.len();
                    all_messages.extend(session.messages);
                    sessions.push(session.info);
                }
            }
        }

        // 解析 agents
        if let Some(agents_array) = backup.get("agents").and_then(|v| v.as_array()) {
            for agent_val in agents_array {
                if let Ok(session) = self.parse_backup_agent(agent_val) {
                    stats.agents_imported += 1;
                    stats.messages_imported += session.messages.len();
                    all_messages.extend(session.messages);
                    sessions.push(session.info);
                }
            }
        }

        // 从 metadata 获取统计
        if let Some(meta) = backup.get("metadata") {
            if let Some(size) = meta.get("total_size_mb").and_then(|v| v.as_f64()) {
                stats.total_size_mb = size;
            }
            if let Some(start_str) = meta.get("date_range").and_then(|v| v.get("start")).and_then(|v| v.as_str()) {
                if let Some(end_str) = meta.get("date_range").and_then(|v| v.get("end")).and_then(|v| v.as_str()) {
                    if let (Ok(start), Ok(end)) = (
                        DateTime::parse_from_rfc3339(start_str),
                        DateTime::parse_from_rfc3339(end_str),
                    ) {
                        stats.date_range = Some((start.with_timezone(&Utc), end.with_timezone(&Utc)));
                    }
                }
            }
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;

        let metadata = KnowledgeMetadata {
            project: "RealConsole".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            total_sessions: stats.sessions_imported,
            total_agents: stats.agents_imported,
            total_messages: stats.messages_imported,
            total_topics: 0,
            total_decisions: 0,
            source_path: backup_file.to_string_lossy().to_string(),
        };

        let knowledge_base = KnowledgeBase {
            metadata,
            sessions,
            messages: all_messages,
            topics: Vec::new(),
            decisions: Vec::new(),
        };

        Ok((knowledge_base, stats))
    }

    /// 解析 JSONL 文件
    async fn parse_jsonl_file(&self, path: &Path) -> Result<Vec<ConversationMessage>> {
        let content = fs::read_to_string(path).await?;
        let session_id = path.file_stem().unwrap().to_string_lossy().to_string();

        let mut messages = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(val) => {
                    if let Some(msg) = self.parse_message(&val, &session_id) {
                        messages.push(msg);
                    }
                }
                Err(e) => {
                    eprintln!("解析第 {} 行失败: {}", line_num + 1, e);
                }
            }
        }
        Ok(messages)
    }

    /// 解析单条消息
    fn parse_message(&self, val: &serde_json::Value, session_id: &str) -> Option<ConversationMessage> {
        let msg_type = val.get("type")?.as_str()?;
        let message_type = match msg_type {
            "user" => MessageType::User,
            "assistant" => MessageType::Assistant,
            "system" => MessageType::System,
            _ => return None,
        };

        // 提取内容
        let content = val
            .get("message")
            .and_then(|m| m.get("content"))
            .or_else(|| val.get("content"))
            .and_then(|c| {
                if c.is_string() {
                    c.as_str().map(|s| s.to_string())
                } else if c.is_array() {
                    // 处理数组形式的 content
                    c.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                } else {
                    None
                }
            })?;

        // 提取时间戳
        let timestamp = val
            .get("timestamp")
            .and_then(|t| t.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        // 提取 thinking
        let thinking = val
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_array())
            .and_then(|arr| {
                arr.iter()
                    .find(|item| item.get("type").and_then(|t| t.as_str()) == Some("thinking"))
                    .and_then(|item| item.get("thinking").and_then(|t| t.as_str()))
                    .map(|s| s.to_string())
            });

        // 提取使用的工具
        let tools_used = val
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        if item.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                            item.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // 生成 ID
        let id = val
            .get("uuid")
            .and_then(|u| u.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        Some(ConversationMessage {
            id,
            message_type,
            content,
            thinking,
            timestamp,
            session_id: session_id.to_string(),
            tools_used,
            metadata: HashMap::new(),
        })
    }

    /// 读取会话索引文件
    async fn read_sessions_index(&self) -> Result<serde_json::Value> {
        let index_path = self.claude_dir.join("sessions-index.json");
        if index_path.exists() {
            let content = fs::read_to_string(index_path).await?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(serde_json::json!({"sessions": []}))
        }
    }

    /// 从索引中查找会话摘要
    fn find_session_summary(&self, index: &serde_json::Value, session_id: &str) -> String {
        index
            .get("sessions")
            .and_then(|s| s.as_array())
            .and_then(|arr| {
                arr.iter()
                    .find(|s| s.get("id").and_then(|id| id.as_str()) == Some(session_id))
            })
            .and_then(|s| s.get("summary"))
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// 获取消息的时间范围
    fn get_time_range(&self, messages: &[ConversationMessage]) -> (DateTime<Utc>, DateTime<Utc>) {
        if messages.is_empty() {
            let now = Utc::now();
            return (now, now);
        }
        let mut timestamps: Vec<_> = messages.iter().map(|m| m.timestamp).collect();
        timestamps.sort();
        (timestamps[0], *timestamps.last().unwrap())
    }

    /// 解析备份文件中的 session
    fn parse_backup_session(&self, val: &serde_json::Value) -> Result<ParsedSession> {
        let session_id = val.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let messages_val = val.get("messages").and_then(|v| v.as_array());

        let mut messages = Vec::new();
        if let Some(msgs) = messages_val {
            for msg_val in msgs {
                if let Some(msg) = self.parse_message(msg_val, session_id) {
                    messages.push(msg);
                }
            }
        }

        let (created_at, modified_at) = self.get_time_range(&messages);

        let info = ConversationSession {
            id: session_id.to_string(),
            summary: val.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            created_at,
            modified_at,
            message_count: messages.len(),
            filename: val.get("filename").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            file_size_mb: val.get("file_size_mb").and_then(|v| v.as_f64()).unwrap_or(0.0),
            is_agent: false,
            parent_session_id: None,
        };

        Ok(ParsedSession { info, messages })
    }

    /// 解析备份文件中的 agent
    fn parse_backup_agent(&self, val: &serde_json::Value) -> Result<ParsedSession> {
        let agent_id = val.get("agent_id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let session_id = format!("agent-{}", agent_id);
        let messages_val = val.get("messages").and_then(|v| v.as_array());

        let mut messages = Vec::new();
        if let Some(msgs) = messages_val {
            for msg_val in msgs {
                if let Some(msg) = self.parse_message(msg_val, &session_id) {
                    messages.push(msg);
                }
            }
        }

        let (created_at, modified_at) = self.get_time_range(&messages);

        let info = ConversationSession {
            id: session_id,
            summary: String::new(),
            created_at,
            modified_at,
            message_count: messages.len(),
            filename: val.get("filename").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            file_size_mb: val.get("file_size_mb").and_then(|v| v.as_f64()).unwrap_or(0.0),
            is_agent: true,
            parent_session_id: None,
        };

        Ok(ParsedSession { info, messages })
    }
}

/// 解析后的会话数据
struct ParsedSession {
    info: ConversationSession,
    messages: Vec<ConversationMessage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_parsing() {
        let val = serde_json::json!({
            "type": "user",
            "content": "Hello",
            "timestamp": "2026-01-30T10:00:00Z"
        });

        let importer = ConversationImporter::new(PathBuf::from("/tmp"));
        let msg = importer.parse_message(&val, "test-session");
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert_eq!(msg.message_type, MessageType::User);
        assert_eq!(msg.content, "Hello");
    }
}
