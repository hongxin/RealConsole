//! Web 会话持久化管理
//!
//! v1.40.0 新增：支持会话的保存、加载、列表、删除和导出功能

use crate::web::session::ConversationRound;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// 可序列化的会话数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableSession {
    /// 会话 ID（UUID）
    pub id: String,

    /// 会话名称（用户自定义或自动生成）
    pub name: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 最后更新时间
    pub updated_at: DateTime<Utc>,

    /// 对话 ID
    pub conversation_id: String,

    /// 对话回合列表
    pub rounds: Vec<ConversationRound>,

    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SessionMetadata>,

    /// 版本号（用于向后兼容）
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// 会话元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// 总回合数
    pub round_count: usize,

    /// 总执行时间（秒）
    pub total_execution_time: f64,

    /// 使用的模型列表
    pub models_used: Vec<String>,

    /// 使用的工具列表
    pub tools_used: Vec<String>,
}

impl SessionMetadata {
    /// 从回合列表计算元数据
    pub fn from_rounds(rounds: &[ConversationRound]) -> Self {
        let total_execution_time: f64 = rounds.iter().map(|r| r.execution_time).sum();

        let models_used: Vec<String> = rounds
            .iter()
            .map(|r| r.model.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let tools_used: Vec<String> = rounds
            .iter()
            .flat_map(|r| r.tools_used.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        Self {
            round_count: rounds.len(),
            total_execution_time,
            models_used,
            tools_used,
        }
    }
}

/// 会话列表项（轻量级，用于列表显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListItem {
    /// 会话 ID
    pub id: String,

    /// 会话名称
    pub name: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 最后更新时间
    pub updated_at: DateTime<Utc>,

    /// 回合数量
    pub round_count: usize,

    /// 最后一条消息预览
    pub last_message: String,
}

impl From<&SerializableSession> for SessionListItem {
    fn from(session: &SerializableSession) -> Self {
        let last_message = session
            .rounds
            .last()
            .map(|r| {
                let preview = &r.user_input;
                // 使用字符边界安全的截取方式，避免切割到 UTF-8 字符中间
                if preview.chars().count() > 50 {
                    let truncated: String = preview.chars().take(50).collect();
                    format!("{}...", truncated)
                } else {
                    preview.clone()
                }
            })
            .unwrap_or_else(|| "空会话".to_string());

        Self {
            id: session.id.clone(),
            name: session.name.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            round_count: session.rounds.len(),
            last_message,
        }
    }
}

/// 会话管理器
pub struct SessionManager {
    /// 会话存储目录
    sessions_dir: PathBuf,

    /// 导出文件目录
    exports_dir: PathBuf,
}

impl SessionManager {
    /// 创建会话管理器
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("无法获取用户主目录")?;
        let realconsole_dir = home_dir.join(".realconsole");

        let sessions_dir = realconsole_dir.join("sessions");
        let exports_dir = realconsole_dir.join("exports");

        // 创建目录（如果不存在）
        fs::create_dir_all(&sessions_dir)
            .context("无法创建 sessions 目录")?;
        fs::create_dir_all(&exports_dir)
            .context("无法创建 exports 目录")?;

        Ok(Self {
            sessions_dir,
            exports_dir,
        })
    }

    /// 保存会话
    pub fn save_session(&self, session: &SerializableSession) -> Result<()> {
        let file_path = self.sessions_dir.join(format!("session-{}.json", session.id));

        let json = serde_json::to_string_pretty(session)
            .context("序列化会话失败")?;

        fs::write(&file_path, json)
            .with_context(|| format!("写入会话文件失败: {:?}", file_path))?;

        eprintln!("✅ 会话已保存: {:?}", file_path);
        Ok(())
    }

    /// 加载会话
    pub fn load_session(&self, id: &str) -> Result<SerializableSession> {
        let file_path = self.sessions_dir.join(format!("session-{}.json", id));

        let json = fs::read_to_string(&file_path)
            .with_context(|| format!("读取会话文件失败: {:?}", file_path))?;

        let session: SerializableSession = serde_json::from_str(&json)
            .context("反序列化会话失败")?;

        eprintln!("✅ 会话已加载: {}", session.name);
        Ok(session)
    }

    /// 列出所有会话（返回轻量级列表项）
    pub fn list_sessions(&self) -> Result<Vec<SessionListItem>> {
        let mut sessions = Vec::new();

        let entries = fs::read_dir(&self.sessions_dir)
            .context("读取 sessions 目录失败")?;

        for entry in entries {
            let entry = entry.context("读取目录项失败")?;
            let path = entry.path();

            // 仅处理 .json 文件
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // 提取会话 ID
            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some(id) = file_name.strip_prefix("session-") {
                    // 加载完整会话（用于获取元数据）
                    if let Ok(session) = self.load_session(id) {
                        sessions.push(SessionListItem::from(&session));
                    }
                }
            }
        }

        // 按更新时间倒序排序（最新的在前）
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    /// 删除会话
    pub fn delete_session(&self, id: &str) -> Result<()> {
        let file_path = self.sessions_dir.join(format!("session-{}.json", id));

        fs::remove_file(&file_path)
            .with_context(|| format!("删除会话文件失败: {:?}", file_path))?;

        eprintln!("✅ 会话已删除: {}", id);
        Ok(())
    }

    /// 重命名会话
    pub fn rename_session(&self, id: &str, new_name: &str) -> Result<()> {
        let file_path = self.sessions_dir.join(format!("session-{}.json", id));

        // 读取会话文件
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("读取会话文件失败: {:?}", file_path))?;

        // 解析 JSON
        let mut session: SerializableSession = serde_json::from_str(&content)
            .with_context(|| "解析会话文件失败")?;

        // 更新名称
        session.name = new_name.to_string();

        // 保存回文件
        let updated_content = serde_json::to_string_pretty(&session)
            .with_context(|| "序列化会话失败")?;

        fs::write(&file_path, updated_content)
            .with_context(|| format!("写入会话文件失败: {:?}", file_path))?;

        eprintln!("✅ 会话已重命名: {} -> {}", id, new_name);
        Ok(())
    }

    /// 导出会话为 Markdown
    pub fn export_to_markdown(&self, session: &SerializableSession) -> Result<String> {
        let mut md = String::new();

        // 标题和元数据
        md.push_str(&format!("# 会话：{}\n\n", session.name));
        md.push_str(&format!("**创建时间**: {}\n", session.created_at.format("%Y-%m-%d %H:%M:%S")));
        md.push_str(&format!("**会话 ID**: {}\n", session.id));
        md.push_str(&format!("**回合数**: {}\n", session.rounds.len()));

        if let Some(ref metadata) = session.metadata {
            md.push_str(&format!("**总执行时间**: {:.2} 秒\n", metadata.total_execution_time));
            md.push_str(&format!("**使用的模型**: {}\n", metadata.models_used.join(", ")));
            if !metadata.tools_used.is_empty() {
                md.push_str(&format!("**使用的工具**: {}\n", metadata.tools_used.join(", ")));
            }
        }

        md.push_str("\n---\n\n");

        // 回合内容
        for round in &session.rounds {
            md.push_str(&format!("## 回合 {} - {:?}\n\n", round.index, round.round_type));
            md.push_str(&format!("**时间**: {}\n", round.timestamp.format("%Y-%m-%d %H:%M:%S")));
            md.push_str(&format!("**模型**: {}\n", round.model));
            md.push_str(&format!("**执行时间**: {:.2} 秒\n\n", round.execution_time));

            md.push_str("### 用户输入\n\n");
            md.push_str("```\n");
            md.push_str(&round.user_input);
            md.push_str("\n```\n\n");

            if !round.ai_response.is_empty() {
                md.push_str("### AI 响应\n\n");
                md.push_str("```\n");
                md.push_str(&round.ai_response);
                md.push_str("\n```\n\n");
            }

            if !round.tools_used.is_empty() {
                md.push_str(&format!("**使用的工具**: {}\n\n", round.tools_used.join(", ")));
            }

            md.push_str("---\n\n");
        }

        Ok(md)
    }

    /// 导出会话为 HTML
    pub fn export_to_html(&self, session: &SerializableSession) -> Result<String> {
        let mut html = String::new();

        // HTML 头部
        html.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("    <title>会话：{}</title>\n", session.name));
        html.push_str("    <style>\n");
        html.push_str(Self::html_style());
        html.push_str("    </style>\n");
        html.push_str("</head>\n<body>\n");

        // 标题和元数据
        html.push_str(&format!("    <h1>会话：{}</h1>\n", session.name));
        html.push_str("    <div class=\"metadata\">\n");
        html.push_str(&format!("        <p>📅 创建时间: {}</p>\n", session.created_at.format("%Y-%m-%d %H:%M:%S")));
        html.push_str(&format!("        <p>🆔 会话 ID: {}</p>\n", session.id));
        html.push_str(&format!("        <p>📊 回合数: {}", session.rounds.len()));

        if let Some(ref metadata) = session.metadata {
            html.push_str(&format!(" | ⏱️ 总执行时间: {:.2} 秒</p>\n", metadata.total_execution_time));
            html.push_str(&format!("        <p>🤖 使用的模型: {}</p>\n", metadata.models_used.join(", ")));
            if !metadata.tools_used.is_empty() {
                html.push_str(&format!("        <p>🔧 使用的工具: {}</p>\n", metadata.tools_used.join(", ")));
            }
        } else {
            html.push_str("</p>\n");
        }

        html.push_str("    </div>\n\n");

        // 回合内容
        for round in &session.rounds {
            html.push_str("    <div class=\"round\">\n");
            html.push_str(&format!("        <div class=\"round-header\">📍 回合 {} - {:?}</div>\n", round.index, round.round_type));
            html.push_str("        <div class=\"round-meta\">\n");
            html.push_str(&format!("            <span>⏰ {}</span>\n", round.timestamp.format("%Y-%m-%d %H:%M:%S")));
            html.push_str(&format!("            <span>🤖 {}</span>\n", round.model));
            html.push_str(&format!("            <span>⏱️ {:.2} 秒</span>\n", round.execution_time));
            html.push_str("        </div>\n\n");

            html.push_str("        <h3 class=\"user-input\">💬 用户输入</h3>\n");
            html.push_str("        <pre><code>");
            html.push_str(&Self::html_escape(&round.user_input));
            html.push_str("</code></pre>\n\n");

            if !round.ai_response.is_empty() {
                html.push_str("        <h3 class=\"ai-response\">🤖 AI 响应</h3>\n");
                html.push_str("        <pre><code>");
                html.push_str(&Self::html_escape(&round.ai_response));
                html.push_str("</code></pre>\n\n");
            }

            if !round.tools_used.is_empty() {
                html.push_str(&format!("        <div class=\"tools\">🔧 使用的工具: {}</div>\n", round.tools_used.join(", ")));
            }

            html.push_str("    </div>\n\n");
        }

        // HTML 尾部
        html.push_str("</body>\n</html>");

        Ok(html)
    }

    /// 保存导出文件
    pub fn save_export(&self, session_id: &str, content: &str, format: &str) -> Result<PathBuf> {
        let file_name = format!("session-{}.{}", session_id, format);
        let file_path = self.exports_dir.join(&file_name);

        fs::write(&file_path, content)
            .with_context(|| format!("写入导出文件失败: {:?}", file_path))?;

        eprintln!("✅ 导出文件已保存: {:?}", file_path);
        Ok(file_path)
    }

    /// HTML 样式（GitHub 暗色主题风格）
    fn html_style() -> &'static str {
        r#"
        body {
            background: #0D1117;
            color: #E6EDF3;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif;
            line-height: 1.6;
            max-width: 1200px;
            margin: 0 auto;
            padding: 32px;
        }

        h1 {
            color: #E6EDF3;
            border-bottom: 1px solid #30363D;
            padding-bottom: 16px;
            margin-bottom: 24px;
        }

        .metadata {
            background: #161B22;
            border: 1px solid #30363D;
            border-radius: 8px;
            padding: 16px;
            margin-bottom: 32px;
        }

        .metadata p {
            margin: 8px 0;
            color: #8B949E;
        }

        .round {
            background: #161B22;
            border: 1px solid #30363D;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 24px;
        }

        .round-header {
            color: #A371F7;
            font-size: 1.2em;
            font-weight: 600;
            margin-bottom: 12px;
        }

        .round-meta {
            color: #8B949E;
            font-size: 0.9em;
            margin-bottom: 16px;
        }

        .round-meta span {
            margin-right: 16px;
        }

        h3 {
            margin: 16px 0 8px 0;
        }

        .user-input {
            color: #F0B90B;
        }

        .ai-response {
            color: #51CF66;
        }

        pre {
            background: #0D1117;
            border: 1px solid #30363D;
            border-radius: 6px;
            padding: 16px;
            overflow-x: auto;
            margin: 8px 0;
        }

        code {
            color: #E6EDF3;
            font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
            font-size: 0.9em;
        }

        .tools {
            color: #8B949E;
            margin-top: 12px;
            font-size: 0.9em;
        }
        "#
    }

    /// HTML 转义
    fn html_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::session::{ConversationRound, RoundStatus, RoundType};

    #[test]
    fn test_session_metadata_from_rounds() {
        let rounds = vec![
            ConversationRound {
                id: "round-1".to_string(),
                index: 1,
                round_type: RoundType::Llm,
                user_input: "test".to_string(),
                ai_response: "response".to_string(),
                tools_used: vec!["Tool1".to_string()],
                execution_time: 1.5,
                status: RoundStatus::Success,
                timestamp: Utc::now(),
                model: "deepseek-chat".to_string(),
            },
            ConversationRound {
                id: "round-2".to_string(),
                index: 2,
                round_type: RoundType::Shell,
                user_input: "ls".to_string(),
                ai_response: "".to_string(),
                tools_used: vec!["Tool1".to_string(), "Tool2".to_string()],
                execution_time: 0.5,
                status: RoundStatus::Success,
                timestamp: Utc::now(),
                model: "deepseek-chat".to_string(),
            },
        ];

        let metadata = SessionMetadata::from_rounds(&rounds);

        assert_eq!(metadata.round_count, 2);
        assert_eq!(metadata.total_execution_time, 2.0);
        assert_eq!(metadata.models_used.len(), 1);
        assert_eq!(metadata.tools_used.len(), 2);
    }
}
