//! Web 终端会话管理
//!
//! 为每个 WebSocket 连接维护独立的 Agent 实例和会话状态

use crate::agent::Agent;
use crate::command::CommandRegistry;
use crate::config::Config;
use crate::llm::{DeepseekClient, LlmClient, OllamaClient};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 会话 ID
pub type SessionId = String;

/// 消息类型（Client → Server）
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClientMessage {
    /// 用户输入命令
    Input { content: String },
    /// 中断信号（Ctrl+C）
    Interrupt { content: String },
}

/// 消息类型（Server → Client）
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerMessage {
    /// 思考中（显示飞轮）
    Thinking { model: String },
    /// 命令输出（一次性）
    Output { content: String },
    /// 流式输出（增量）
    Stream { content: String },
    /// 错误信息
    Error { content: String },
    /// 清屏
    Clear,
}

/// Web 终端会话
pub struct Session {
    /// 会话 ID
    pub id: SessionId,
    /// Agent 实例（独立）
    pub agent: Arc<RwLock<Agent>>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Session {
    /// 创建新会话（异步，会配置 LLM）
    pub async fn new(config: Config, registry: CommandRegistry) -> Self {
        let id = Uuid::new_v4().to_string();
        let mut agent = Agent::new(config.clone(), registry);

        // 配置 LLM（参考 main.rs）
        Self::configure_llm(&mut agent, &config).await;

        Self {
            id,
            agent: Arc::new(RwLock::new(agent)),
            created_at: chrono::Utc::now(),
        }
    }

    /// 配置 Agent 的 LLM
    async fn configure_llm(agent: &mut Agent, config: &Config) {
        let mut manager = agent.llm_manager.write().await;

        // 初始化 primary LLM
        if let Some(ref primary_cfg) = config.llm.primary {
            match Self::create_llm_client(primary_cfg) {
                Ok(client) => {
                    manager.set_primary(client.clone());

                    // 如果是 Deepseek，同时设置 deepseek_client 用于流式输出
                    if primary_cfg.provider == "deepseek" {
                        if let Some(api_key) = &primary_cfg.api_key {
                            let model = primary_cfg.model.as_deref().unwrap_or("deepseek-chat");
                            let endpoint = primary_cfg
                                .endpoint
                                .as_deref()
                                .unwrap_or("https://api.deepseek.com/v1");
                            if let Ok(deepseek_client) =
                                DeepseekClient::new(api_key, model, endpoint)
                            {
                                manager.set_deepseek(Arc::new(deepseek_client));
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠ Web Session - Primary LLM 初始化失败: {}", e);
                }
            }
        }

        // 初始化 fallback LLM
        if let Some(ref fallback_cfg) = config.llm.fallback {
            match Self::create_llm_client(fallback_cfg) {
                Ok(client) => {
                    manager.set_fallback(client);
                }
                Err(e) => {
                    eprintln!("⚠ Web Session - Fallback LLM 初始化失败: {}", e);
                }
            }
        }
    }

    /// 根据配置创建 LLM 客户端（参考 main.rs）
    fn create_llm_client(
        provider_config: &crate::config::LlmProvider,
    ) -> Result<Arc<dyn LlmClient>, String> {
        match provider_config.provider.as_str() {
            "ollama" => {
                let model = provider_config.model.as_deref().unwrap_or("qwen2.5:latest");
                let endpoint = provider_config
                    .endpoint
                    .as_deref()
                    .unwrap_or("http://localhost:11434");

                OllamaClient::new(model, endpoint)
                    .map(|client| Arc::new(client) as Arc<dyn LlmClient>)
                    .map_err(|e| format!("Ollama 客户端创建失败: {}", e))
            }
            "deepseek" => {
                let api_key = provider_config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| "Deepseek 需要 API Key".to_string())?;
                let model = provider_config.model.as_deref().unwrap_or("deepseek-chat");
                let endpoint = provider_config
                    .endpoint
                    .as_deref()
                    .unwrap_or("https://api.deepseek.com/v1");

                DeepseekClient::new(api_key, model, endpoint)
                    .map(|client| Arc::new(client) as Arc<dyn LlmClient>)
                    .map_err(|e| format!("Deepseek 客户端创建失败: {}", e))
            }
            other => Err(format!("未知的 LLM 提供商: {}", other)),
        }
    }

    /// 获取会话 ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取会话存活时长（秒）
    pub fn duration(&self) -> i64 {
        (chrono::Utc::now() - self.created_at).num_seconds()
    }
}
