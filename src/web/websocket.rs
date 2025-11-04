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
use crate::services::{LlmRequest, Service};
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

/// 执行 LLM 对话（带工具调用）
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

    // 释放 llm_manager 锁
    drop(llm_manager);

    // 发送 Thinking 消息（显示飞轮）
    let thinking_msg = ServerMessage::Thinking { model: model_name };
    sender
        .send(Message::Text(serde_json::to_string(&thinking_msg)?))
        .await?;

    // ✨ 使用 LlmService 处理（带工具调用）
    let request = LlmRequest::with_tools(input.to_string());
    match agent.llm_service().process(request).await {
        Ok(llm_response) => {
            // 🧹 清理 DEBUG 信息（Web 用户不需要看到）
            let clean_content = remove_debug_info(&llm_response.text);

            let msg = ServerMessage::Output {
                content: clean_content,
            };
            sender
                .send(Message::Text(serde_json::to_string(&msg)?))
                .await?;
        }
        Err(e) => {
            // 🧹 改进错误提示，提供用户友好的错误信息
            let error_msg = format_user_friendly_error(&e.to_string());

            let msg = ServerMessage::Error {
                content: error_msg,
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

/// 移除响应中的 DEBUG 信息
///
/// Web 用户不需要看到工具调用的调试信息，只需要看到干净的最终结果
fn remove_debug_info(text: &str) -> String {
    if let Some(pos) = text.find("__DEBUG__") {
        // 移除 __DEBUG__ 及其后面的所有内容
        text[..pos].trim_end().to_string()
    } else {
        text.to_string()
    }
}

/// 格式化用户友好的错误信息
///
/// 将底层技术错误转换为用户可理解的友好提示
fn format_user_friendly_error(error_str: &str) -> String {
    // 1. 响应解码错误（最常见的问题）
    if error_str.contains("decoding response body") || error_str.contains("Failed to read response body") {
        return "⚠️ 响应解析失败\n\n可能原因：\n• 请求内容过于复杂，LLM 响应超出处理限制\n• 网络连接不稳定\n\n建议：\n• 尝试简化您的问题\n• 将复杂问题拆分为多个简单问题\n• 刷新页面后重试".to_string();
    }

    // 2. 网络超时
    if error_str.contains("timeout") || error_str.contains("Timeout") {
        return "⏱️ 请求超时\n\n原因：\n• LLM 响应时间过长（超过 60 秒）\n\n建议：\n• 简化问题描述\n• 减少工具调用次数\n• 稍后重试".to_string();
    }

    // 3. 网络错误
    if error_str.contains("Network error") || error_str.contains("connection") {
        return "🌐 网络连接错误\n\n可能原因：\n• 网络连接不稳定\n• API 服务暂时不可用\n\n建议：\n• 检查网络连接\n• 稍后重试".to_string();
    }

    // 4. API 认证错误
    if error_str.contains("401") || error_str.contains("authentication") || error_str.contains("API key") {
        return "🔑 API 认证失败\n\n原因：\n• API Key 无效或已过期\n\n建议：\n• 检查 API Key 配置\n• 联系管理员".to_string();
    }

    // 5. API 限流
    if error_str.contains("429") || error_str.contains("rate limit") {
        return "⚡ API 调用频率限制\n\n原因：\n• 短时间内请求过多\n\n建议：\n• 稍等片刻后重试\n• 降低使用频率".to_string();
    }

    // 6. 工具调用失败
    if error_str.contains("工具调用失败") {
        // 提取实际的底层错误
        if let Some(start) = error_str.rfind("LLM 调用失败:") {
            let core_error = &error_str[start + "LLM 调用失败:".len()..].trim();
            return format!("🔧 工具调用过程中出错\n\n错误详情：\n{}\n\n建议：\n• 尝试使用其他方式表达问题\n• 检查输入参数是否合理", core_error);
        }
    }

    // 7. 通用错误（清理嵌套的错误前缀）
    let clean_error = error_str
        .replace("LLM 调用失败: 工具调用失败: LLM 调用失败: ", "")
        .replace("LLM 调用失败: ", "")
        .replace("工具调用失败: ", "");

    format!("❌ 处理请求时出现错误\n\n错误信息：\n{}\n\n建议：\n• 尝试重新表述问题\n• 如果问题持续，请联系技术支持", clean_error)
}
