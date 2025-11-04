//! WebSocket 处理和 Agent 集成
//!
//! 负责：
//! - WebSocket 消息收发
//! - Agent 命令执行
//! - 流式输出转发
//! - Intent 意图识别和执行

use crate::command::CommandRegistry;
use crate::command_router::{CommandRouter, CommandType};
use crate::config::Config;
use crate::dsl::intent::IntentMatch;
use crate::web::session::{ClientMessage, ServerMessage, Session};
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;

/// WebSocket 会话处理器
pub struct WebSocketSession {
    socket: WebSocket,
    session: Arc<Session>,
}

impl WebSocketSession {
    /// 创建新的 WebSocket 会话（异步）
    pub async fn new(socket: WebSocket, config: Config, registry: CommandRegistry) -> Self {
        let session = Arc::new(Session::new(config, registry).await);

        Self { socket, session }
    }

    /// 运行会话主循环
    pub async fn run(self) -> anyhow::Result<()> {
        println!(
            "✅ WebSocket 连接建立 [Session: {}]",
            self.session.id()
        );

        // 克隆 session 以避免借用冲突
        let session = Arc::clone(&self.session);

        // 分离读写
        let (mut sender, mut receiver) = self.socket.split();

        // 处理客户端消息
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // 解析客户端消息
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(client_msg) => {
                            // 处理命令
                            if let Err(e) = handle_message(&session, client_msg, &mut sender).await {
                                eprintln!("❌ 处理消息失败: {}", e);
                                let error_msg = ServerMessage::Error {
                                    content: format!("内部错误: {}", e),
                                };
                                let _ = sender
                                    .send(Message::Text(serde_json::to_string(&error_msg)?))
                                    .await;
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ 解析消息失败: {}", e);
                            let error_msg = ServerMessage::Error {
                                content: "消息格式错误".to_string(),
                            };
                            let _ = sender
                                .send(Message::Text(serde_json::to_string(&error_msg)?))
                                .await;
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("🔌 客户端关闭连接 [Session: {}]", session.id());
                    break;
                }
                Ok(Message::Ping(data)) => {
                    let _ = sender.send(Message::Pong(data)).await;
                }
                Ok(_) => {
                    // 忽略其他消息类型
                }
                Err(e) => {
                    eprintln!("❌ WebSocket 错误: {}", e);
                    break;
                }
            }
        }

        println!(
            "👋 WebSocket 连接关闭 [Session: {}, Duration: {}s]",
            session.id(),
            session.duration()
        );

        Ok(())
    }
}

/// 处理客户端消息
async fn handle_message(
    session: &Arc<Session>,
    msg: ClientMessage,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    match msg {
        ClientMessage::Input { content } => {
            handle_input(session, &content, sender).await?;
        }
        ClientMessage::Interrupt { .. } => {
            // 中断信号暂不处理，未来可以支持任务取消
            let response = ServerMessage::Output {
                content: "^C".to_string(),
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
    }
    Ok(())
}

/// 处理用户输入
async fn handle_input(
    session: &Arc<Session>,
    input: &str,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(());
    }

    // 获取 Agent
    let agent = session.agent.read().await;

    // 使用 CommandRouter 进行智能路由
    let router = CommandRouter::new(agent.config.prefix.clone());
    let result = match router.route(input) {
        CommandType::SystemCommand(cmd, args) => {
            // 系统命令（/前缀）
            let cmd_input = if args.is_empty() {
                cmd
            } else {
                format!("{} {}", cmd, args)
            };
            execute_system_command(&cmd_input, &agent, sender).await
        }
        CommandType::CommonShell(cmd) | CommandType::ForcedShell(cmd) => {
            // Shell 命令（常见命令自动识别 或 !前缀强制）
            execute_shell_command(&format!("!{}", cmd), &agent, sender).await
        }
        CommandType::NaturalLanguage(text) => {
            // 自然语言：先尝试 Intent 匹配，否则回退到 LLM 对话
            if let Some(intent_match) = try_match_intent(&text, &agent) {
                execute_intent(&intent_match, &text, &agent, sender).await
            } else {
                execute_llm_chat(&text, &agent, sender).await
            }
        }
    };

    if let Err(e) = result {
        let error_msg = ServerMessage::Error {
            content: format!("执行失败: {}", e),
        };
        sender
            .send(Message::Text(serde_json::to_string(&error_msg)?))
            .await?;
    }

    Ok(())
}

/// 执行系统命令
async fn execute_system_command(
    cmd_name: &str,
    agent: &crate::agent::Agent,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    // 解析命令和参数
    let parts: Vec<&str> = cmd_name.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    let cmd = parts[0];
    let args_str = parts[1..].join(" ");

    // 特殊处理清屏命令
    if cmd == "clear" {
        let msg = ServerMessage::Clear;
        sender
            .send(Message::Text(serde_json::to_string(&msg)?))
            .await?;
        return Ok(());
    }

    // 执行命令（简化版本）
    match agent.registry.execute(cmd, &args_str) {
        Ok(output) => {
            let msg = ServerMessage::Output { content: output };
            sender
                .send(Message::Text(serde_json::to_string(&msg)?))
                .await?;
        }
        Err(e) => {
            let msg = ServerMessage::Error {
                content: format!("命令执行失败: {}", e),
            };
            sender
                .send(Message::Text(serde_json::to_string(&msg)?))
                .await?;
        }
    }

    Ok(())
}

/// 执行 Shell 命令
async fn execute_shell_command(
    input: &str,
    _agent: &crate::agent::Agent,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let cmd = input.trim_start_matches('!').trim();

    // 使用 tokio 执行 shell 命令
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let result = if !stderr.is_empty() {
        format!("{}{}", stdout, stderr)
    } else {
        stdout.to_string()
    };

    let msg = ServerMessage::Output { content: result };
    sender
        .send(Message::Text(serde_json::to_string(&msg)?))
        .await?;

    Ok(())
}

/// 执行 LLM 对话
async fn execute_llm_chat(
    input: &str,
    agent: &crate::agent::Agent,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    // 获取 LLM 管理器
    let llm_manager = agent.llm_manager.read().await;

    // 检查是否配置了 LLM
    if llm_manager.primary().is_none() {
        let msg = ServerMessage::Error {
            content: "未配置 LLM，无法进行对话".to_string(),
        };
        sender
            .send(Message::Text(serde_json::to_string(&msg)?))
            .await?;
        return Ok(());
    }

    // 获取模型名称并简化（与命令行版本一致）
    let model_name = llm_manager
        .primary()
        .map(|client| simplify_model_name(client.model()))
        .unwrap_or_else(|| "unknown".to_string());

    // 发送 Thinking 消息（显示飞轮）
    let thinking_msg = ServerMessage::Thinking { model: model_name };
    sender
        .send(Message::Text(serde_json::to_string(&thinking_msg)?))
        .await?;

    // 调用 LLM
    match llm_manager.chat(input).await {
        Ok(response) => {
            let msg = ServerMessage::Output { content: response };
            sender
                .send(Message::Text(serde_json::to_string(&msg)?))
                .await?;
        }
        Err(e) => {
            let msg = ServerMessage::Error {
                content: format!("LLM 调用失败: {}", e),
            };
            sender
                .send(Message::Text(serde_json::to_string(&msg)?))
                .await?;
        }
    }

    Ok(())
}

/// 尝试匹配 Intent 意图
fn try_match_intent(text: &str, agent: &crate::agent::Agent) -> Option<IntentMatch> {
    // 使用 Agent 的 IntentMatcher 进行匹配
    let matches = agent.intent_matcher.match_intent(text);

    // 返回最佳匹配（如果有）
    matches.into_iter().next()
}

/// 执行 Intent 意图
async fn execute_intent(
    intent_match: &IntentMatch,
    original_text: &str,
    agent: &crate::agent::Agent,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    // 1. 发送意图识别提示
    let info_msg = ServerMessage::Output {
        content: format!("🎯 {}\n", intent_match.intent.name),
    };
    sender
        .send(Message::Text(serde_json::to_string(&info_msg)?))
        .await?;

    // 2. 使用 TemplateEngine 生成执行计划
    match agent.template_engine.generate_from_intent(intent_match) {
        Ok(plan) => {
            // 3. 执行计划中的命令
            // 简化实现：直接执行 Shell 命令
            let cmd = &plan.command;

            // 执行 shell 命令
            let output = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .await?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let result = if !stderr.is_empty() {
                format!("{}{}", stdout, stderr)
            } else {
                stdout.to_string()
            };

            // 发送执行结果
            let msg = ServerMessage::Output { content: result };
            sender
                .send(Message::Text(serde_json::to_string(&msg)?))
                .await?;
        }
        Err(e) => {
            // 如果生成执行计划失败，回退到 LLM 对话
            eprintln!("⚠️ Intent 执行计划生成失败: {}", e);
            execute_llm_chat(original_text, agent, sender).await?;
        }
    }

    Ok(())
}

/// 简化模型名称（与命令行版本一致）
fn simplify_model_name(model: &str) -> String {
    // 移除常见后缀
    let simplified = model
        .trim()
        .replace(":latest", "")
        .replace(":stable", "")
        .replace("-chat", "")
        .replace("-turbo", "")
        .replace("-preview", "");

    // 移除日期后缀（如 20240229）
    let mut result = simplified.clone();
    if let Some(pos) = result.rfind('-') {
        if let Some(suffix) = result.get(pos + 1..) {
            // 检查是否是纯数字（日期）
            if suffix.len() >= 6 && suffix.chars().all(|c| c.is_ascii_digit()) {
                result = result[..pos].to_string();
            }
        }
    }

    // 限制长度（最多保留前两个部分，用 - 分隔）
    let parts: Vec<&str> = result.split('-').collect();
    if parts.len() > 2 {
        format!("{}-{}", parts[0], parts[1])
    } else {
        result
    }
}
