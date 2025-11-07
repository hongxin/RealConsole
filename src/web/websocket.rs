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
use crate::i18n;
use crate::services::{LlmRequest, Service};
use crate::web::session::{ClientMessage, ServerMessage, Session};
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Instant;

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
            "{} [Session: {}]",
            i18n::t("web.websocket.connection_established"),
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
                                eprintln!("{}: {}", i18n::t("web.websocket.handle_message_error"), e);
                                let error_msg = ServerMessage::Error {
                                    content: format!("{}: {}", i18n::t("web.websocket.internal_error"), e),
                                };
                                let _ = sender
                                    .send(Message::Text(serde_json::to_string(&error_msg)?))
                                    .await;
                            }
                        }
                        Err(e) => {
                            eprintln!("{}: {}", i18n::t("web.websocket.parse_error"), e);
                            let error_msg = ServerMessage::Error {
                                content: i18n::t("web.websocket.invalid_message_format"),
                            };
                            let _ = sender
                                .send(Message::Text(serde_json::to_string(&error_msg)?))
                                .await;
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("{} [Session: {}]", i18n::t("web.websocket.client_closed"), session.id());
                    break;
                }
                Ok(Message::Ping(data)) => {
                    let _ = sender.send(Message::Pong(data)).await;
                }
                Ok(_) => {
                    // 忽略其他消息类型
                }
                Err(e) => {
                    eprintln!("{}: {}", i18n::t("web.websocket.connection_error"), e);
                    break;
                }
            }
        }

        println!(
            "{} [Session: {}, Duration: {}s]",
            i18n::t("web.websocket.connection_closed"),
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
            execute_system_command(&cmd_input, &agent, session, sender).await
        }
        CommandType::CommonShell(cmd) | CommandType::ForcedShell(cmd) => {
            // Shell 命令（常见命令自动识别 或 !前缀强制）
            execute_shell_command(&format!("!{}", cmd), &agent, session, sender).await
        }
        CommandType::NaturalLanguage(text) => {
            // 自然语言：先尝试 Intent 匹配，否则回退到 LLM 对话
            if let Some(intent_match) = try_match_intent(&text, &agent) {
                execute_intent(&intent_match, &text, &agent, session, sender).await
            } else {
                // 传递 session 以便访问 llm_init_error
                execute_llm_chat(&text, &agent, session, sender).await
            }
        }
    };

    if let Err(e) = result {
        let error_msg = ServerMessage::Error {
            content: format!("{}: {}", i18n::t("web.command.execution_failed"), e),
        };
        sender
            .send(Message::Text(serde_json::to_string(&error_msg)?))
            .await?;
    }

    Ok(())
}

/// 执行系统命令（v1.28.1: 统一回合系统）
///
/// ## 消息流程
/// ```
/// RoundStart(type: system) → 执行命令 → RoundComplete
/// ```
///
/// ## 与 LLM 的差异
/// - ❌ 没有 `Thinking` 消息
/// - ❌ 没有 `Stream` 消息（一次性返回）
/// - ✅ 有 `RoundStart` 和 `RoundComplete`
///
/// ## 传统模式显示
/// - 命令：`handleSubmit()` 显示（用户输入时）
/// - 输出：`round_complete` 时额外显示
///
/// ## 特殊处理
/// - `clear` 命令不创建回合，直接发送 `Clear` 消息
async fn execute_system_command(
    cmd_name: &str,
    agent: &crate::agent::Agent,
    session: &Arc<Session>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    // 解析命令和参数
    let parts: Vec<&str> = cmd_name.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    let cmd = parts[0];
    let args_str = parts[1..].join(" ");

    // 特殊处理清屏命令（不创建回合）
    if cmd == "clear" {
        let msg = ServerMessage::Clear;
        sender
            .send(Message::Text(serde_json::to_string(&msg)?))
            .await?;
        return Ok(());
    }

    // ===== v1.29.0: 特殊处理意图拆解命令 =====
    if cmd == "decompose" {
        eprintln!("🔍 [Web] Handling decompose command with args: {}", args_str);
        return execute_decompose_command(&args_str, agent, session, sender).await;
    }

    // ===== 创建回合 =====
    let round = session.create_round(
        crate::web::session::RoundType::System,
        cmd_name.to_string(),
        "system".to_string()
    ).await;
    let round_id = round.id.clone();

    // 发送 RoundStart 消息
    let round_start_msg = ServerMessage::RoundStart {
        round: round.clone(),
    };
    sender
        .send(Message::Text(serde_json::to_string(&round_start_msg)?))
        .await?;

    // 记录开始时间
    let start_time = Instant::now();

    // 执行命令
    match agent.registry.execute(cmd, &args_str) {
        Ok(output) => {
            let execution_time = start_time.elapsed().as_secs_f64();

            // 完成回合
            if let Some(completed_round) = session.complete_round(
                &round_id,
                output,
                execution_time,
                Vec::new(), // 系统命令没有工具使用
            ).await {
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: completed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }
        }
        Err(e) => {
            // 标记回合失败
            let error_msg = format!("{}: {}", i18n::t("web.command.execution_error"), e);
            if let Some(failed_round) = session.fail_round(&round_id, error_msg).await {
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: failed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }
        }
    }

    Ok(())
}

/// 执行 Shell 命令（v1.28.1: 统一回合系统）
///
/// ## 消息流程
/// ```
/// RoundStart(type: shell) → 执行命令 → RoundComplete
/// ```
///
/// ## 与 LLM 的差异
/// - ❌ 没有 `Thinking` 消息
/// - ❌ 没有 `Stream` 消息（一次性返回）
/// - ✅ 有 `RoundStart` 和 `RoundComplete`
///
/// ## 传统模式显示
/// - 命令：`handleSubmit()` 显示（用户输入时）
/// - 输出：`round_complete` 时额外显示
///
/// ## 与 System 的差异
/// - Shell 命令通过 `sh -c` 执行
/// - System 命令通过工具注册表执行
async fn execute_shell_command(
    input: &str,
    _agent: &crate::agent::Agent,
    session: &Arc<Session>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let cmd = input.trim_start_matches('!').trim();

    // ===== 创建回合 =====
    let round = session.create_round(
        crate::web::session::RoundType::Shell,
        cmd.to_string(),
        "shell".to_string()
    ).await;
    let round_id = round.id.clone();

    // 发送 RoundStart 消息
    let round_start_msg = ServerMessage::RoundStart {
        round: round.clone(),
    };
    sender
        .send(Message::Text(serde_json::to_string(&round_start_msg)?))
        .await?;

    // 记录开始时间
    let start_time = Instant::now();

    // 使用 tokio 执行 shell 命令
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await;

    // 计算执行时间
    let execution_time = start_time.elapsed().as_secs_f64();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let result = if !stderr.is_empty() {
                format!("{}{}", stdout, stderr)
            } else {
                stdout.to_string()
            };

            // 完成回合
            if let Some(completed_round) = session.complete_round(
                &round_id,
                result,
                execution_time,
                Vec::new(), // Shell 命令没有工具使用
            ).await {
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: completed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }
        }
        Err(e) => {
            // 标记回合失败
            if let Some(failed_round) = session.fail_round(
                &round_id,
                format!("Shell command execution failed: {}", e),
            ).await {
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: failed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }
        }
    }

    Ok(())
}

/// 执行 LLM 对话（v1.28.1: 统一回合系统 + 工具调用）
///
/// ## 消息流程
/// ```
/// RoundStart(type: llm) → Thinking → Stream(流式) → RoundComplete
/// ```
///
/// ## 与 Shell/System 的差异
/// - ✅ 有 `Thinking` 消息（显示飞轮 + 模型名称）
/// - ✅ 有 `Stream` 消息（流式输出，逐步显示）
/// - ✅ 有 `RoundStart` 和 `RoundComplete`
///
/// ## 传统模式显示
/// - 命令：`handleSubmit()` 显示（用户输入时）
/// - 输出：`stream` 消息流式显示（**不需要** `round_complete` 重复显示）
///   - ⚠️ 关键：Shell/System 在 `round_complete` 时额外显示输出
///   - ⚠️ 但 LLM 已通过 `stream` 显示，`round_complete` 时跳过
///
/// ## 工具调用
/// - 从 `__DEBUG__` 部分提取工具名称
/// - 显示在回合元数据中
async fn execute_llm_chat(
    input: &str,
    agent: &crate::agent::Agent,
    session: &Arc<Session>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    // 获取 LLM 管理器
    let llm_manager = agent.llm_manager.read().await;

    // 检查是否配置了 LLM（优先 primary，其次 fallback）
    if llm_manager.primary().or(llm_manager.fallback()).is_none() {
        // 释放锁，以便访问 session
        drop(llm_manager);

        // 提供详细的诊断信息
        let error_content = if let Some(ref init_error) = session.llm_init_error {
            // 有初始化错误信息，显示详细错误
            format!(
                "{}\n\n{}\n{}",
                i18n::t("web.llm.not_configured"),
                i18n::t("web.llm.init_error_details"),
                init_error
            )
        } else {
            // 没有初始化错误，说明配置文件中未配置
            format!(
                "{}\n\n{}",
                i18n::t("web.llm.not_configured"),
                i18n::t("web.llm.config_missing_hint")
            )
        };

        let msg = ServerMessage::Error {
            content: error_content,
        };
        sender
            .send(Message::Text(serde_json::to_string(&msg)?))
            .await?;
        return Ok(());
    }

    // 获取模型名称并简化（与命令行版本一致，优先 primary，其次 fallback）
    let model_name = llm_manager
        .primary()
        .or(llm_manager.fallback())
        .map(|client| simplify_model_name(client.model()))
        .unwrap_or_else(|| "unknown".to_string());

    // 释放 llm_manager 锁
    drop(llm_manager);

    // ===== v1.28.0: 创建对话回合 =====
    let round = session.create_round(
        crate::web::session::RoundType::Llm,
        input.to_string(),
        model_name.clone()
    ).await;
    let round_id = round.id.clone();

    // 发送 RoundStart 消息
    let round_start_msg = ServerMessage::RoundStart {
        round: round.clone(),
    };
    sender
        .send(Message::Text(serde_json::to_string(&round_start_msg)?))
        .await?;

    // 记录开始时间
    let start_time = Instant::now();

    // 发送 Thinking 消息（显示飞轮）
    let thinking_msg = ServerMessage::Thinking {
        model: model_name.clone(),
    };
    sender
        .send(Message::Text(serde_json::to_string(&thinking_msg)?))
        .await?;

    // ✨ v1.27.0: 支持多轮对话上下文（参考 CLI 版本逻辑）
    let ctx_arc = agent.state_manager().conversation_context();
    let mut ctx_manager = ctx_arc.write().await;

    // 检查是否应该使用上下文
    let should_use_context = ctx_manager.should_use_context(input);

    // 构建消息列表（带上下文或不带）
    let messages = if should_use_context {
        ctx_manager.build_messages(input)
    } else {
        vec![crate::llm::Message::user(input)]
    };

    // 释放上下文管理器锁
    drop(ctx_manager);

    // 创建 LLM 请求（根据是否使用上下文选择方法）
    let request = if should_use_context {
        LlmRequest::with_tools_and_context(messages.clone())
    } else {
        LlmRequest::with_tools(input.to_string())
    };

    // 处理 LLM 请求
    match agent.llm_service().process(request).await {
        Ok(llm_response) => {
            // 计算执行时间
            let execution_time = start_time.elapsed().as_secs_f64();

            // 记录到上下文管理器（如果启用了上下文）
            if should_use_context {
                let ctx_arc = agent.state_manager().conversation_context();
                let mut ctx_manager = ctx_arc.write().await;
                let turn = crate::conversation::Turn::new(
                    input.to_string(),
                    llm_response.text.clone(),
                );
                ctx_manager.add_turn(turn);
            }

            // 🧹 清理 DEBUG 信息（Web 用户不需要看到）
            let clean_content = remove_debug_info(&llm_response.text);

            // ===== v1.28.0: 提取使用的工具 =====
            let tools_used = extract_tools_from_response(&llm_response);

            // ===== v1.28.0: 完成回合 =====
            if let Some(completed_round) = session
                .complete_round(
                    &round_id,
                    clean_content.clone(),
                    execution_time,
                    tools_used,
                )
                .await
            {
                // 发送 RoundComplete 消息
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: completed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }

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

            // ===== v1.28.0: 标记回合失败 =====
            if let Some(failed_round) = session.fail_round(&round_id, error_msg.clone()).await {
                // 发送 RoundComplete 消息（带错误状态）
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: failed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }

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
    session: &Arc<Session>,
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
            execute_llm_chat(original_text, agent, session, sender).await?;
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
        return i18n::t("web.error.response_parse_failed");
    }

    // 2. 网络超时
    if error_str.contains("timeout") || error_str.contains("Timeout") {
        return i18n::t("web.error.request_timeout");
    }

    // 3. 网络错误
    if error_str.contains("Network error") || error_str.contains("connection") {
        return i18n::t("web.error.network_error");
    }

    // 4. API 认证错误
    if error_str.contains("401") || error_str.contains("authentication") || error_str.contains("API key") {
        return i18n::t("web.error.auth_failed");
    }

    // 5. API 限流
    if error_str.contains("429") || error_str.contains("rate limit") {
        return i18n::t("web.error.rate_limit");
    }

    // 6. 工具调用失败
    if error_str.contains(i18n::t("web.error.tool_call_failed").as_str()) {
        // 提取实际的底层错误
        let llm_prefix = i18n::t("web.error.llm_call_failed_prefix");
        if let Some(start) = error_str.rfind(&format!("{}:", llm_prefix)) {
            let core_error = &error_str[start + llm_prefix.len() + 1..].trim();
            return i18n::t_with_args("web.error.tool_call_error", &[("error", core_error)]);
        }
    }

    // 7. 通用错误（清理嵌套的错误前缀）
    let llm_prefix = i18n::t("web.error.llm_call_failed_prefix");
    let tool_prefix = i18n::t("web.error.tool_call_failed");
    let clean_error = error_str
        .replace(&format!("{}: {}: {}: ", llm_prefix, tool_prefix, llm_prefix), "")
        .replace(&format!("{}: ", llm_prefix), "")
        .replace(&format!("{}: ", tool_prefix), "");

    i18n::t_with_args("web.error.generic_error", &[("error", &clean_error)])
}

/// 从 LLM 响应中提取使用的工具（v1.28.0 新增）
fn extract_tools_from_response(llm_response: &crate::services::LlmResponse) -> Vec<String> {
    // 从响应中的工具调用信息提取工具名称
    let mut tools = Vec::new();
    
    // 如果响应包含工具调用信息（从 DEBUG 部分提取）
    let text = &llm_response.text;
    if let Some(debug_start) = text.find("__DEBUG__") {
        let debug_section = &text[debug_start..];
        
        // 查找 "Tool:" 标记
        for line in debug_section.lines() {
            if line.trim().starts_with("Tool:") {
                if let Some(tool_name) = line.split(':').nth(1) {
                    let name = tool_name.trim().to_string();
                    if !name.is_empty() && !tools.contains(&name) {
                        tools.push(name);
                    }
                }
            }
        }
    }
    
    tools
}

/// ===== v1.29.0: 执行意图拆解命令 =====
///
/// 处理 `/decompose` 命令的 WebSocket 版本
///
/// ## 消息流程（v1.29.1 修复：加入回合系统）
/// ```
/// RoundStart → Thinking → IntentUnderstanding → StepProgress × N → Output → RoundComplete
/// ```
///
/// ## 与其他命令的一致性
/// - ✅ 有 `RoundStart` 和 `RoundComplete`（回合模式必需）
/// - ✅ 有 `Thinking` 消息（显示加载状态）
/// - ✅ 有自定义可视化消息（IntentUnderstanding/StepProgress）
async fn execute_decompose_command(
    query: &str,
    agent: &crate::agent::Agent,
    session: &Arc<Session>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use crate::agent::decomposition::StepStatus;

    eprintln!("📋 [Decompose] Starting with query: {}", query);

    // ===== 创建回合（v1.29.1 新增）=====
    let full_command = format!("/decompose {}", query);
    let round = session.create_round(
        crate::web::session::RoundType::System,
        full_command.clone(),
        "intent-decomposer".to_string(),
    ).await;
    let round_id = round.id.clone();

    // 发送 RoundStart 消息
    let round_start_msg = ServerMessage::RoundStart {
        round: round.clone(),
    };
    sender
        .send(Message::Text(serde_json::to_string(&round_start_msg)?))
        .await?;

    // 记录开始时间
    let start_time = std::time::Instant::now();

    // 检查拆解器是否已初始化
    let decomposer = match &agent.intent_decomposer {
        Some(d) => d,
        None => {
            let error_content = format!("{}\n{}",
                "⚠ 意图拆解系统未启用",
                "提示: 意图拆解系统需要配置 LLM 客户端");

            // 标记回合失败
            if let Some(failed_round) = session.fail_round(&round_id, error_content.clone()).await {
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: failed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }
            return Ok(());
        }
    };

    // 检查是否提供了查询
    if query.trim().is_empty() {
        let error_content = format!("{}\n{}\n{}",
            "❌ 请提供要拆解的自然语言查询",
            "用法: /decompose <自然语言查询>",
            "示例: /decompose 加载 data.csv 并显示前 10 行");

        // 标记回合失败
        if let Some(failed_round) = session.fail_round(&round_id, error_content.clone()).await {
            let round_complete_msg = ServerMessage::RoundComplete {
                round: failed_round,
            };
            sender
                .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                .await?;
        }
        return Ok(());
    }

    // 发送思考状态
    let thinking_msg = ServerMessage::Thinking {
        model: "意图拆解中...".to_string(),
    };
    sender
        .send(Message::Text(serde_json::to_string(&thinking_msg)?))
        .await?;

    // 调用拆解器
    match decomposer.decompose(query).await {
        Ok(plan) => {
            // 1. 发送意图理解消息
            let understanding_msg = ServerMessage::IntentUnderstanding {
                plan_id: plan.id.clone(),
                understanding: plan.understanding.clone(),
                step_count: plan.steps.len(),
                total_time: plan.total_estimated_time,
            };
            sender
                .send(Message::Text(serde_json::to_string(&understanding_msg)?))
                .await?;

            // 2. 发送每个步骤的初始状态（pending）
            for (index, step) in plan.steps.iter().enumerate() {
                let status_str = match step.status {
                    StepStatus::Pending => "pending",
                    StepStatus::Running => "running",
                    StepStatus::Success => "success",
                    StepStatus::Failed => "failed",
                    StepStatus::Skipped => "skipped",
                };

                let progress_msg = ServerMessage::StepProgress {
                    plan_id: plan.id.clone(),
                    step_index: index,
                    step_id: step.id.clone(),
                    description: step.description.clone(),
                    tool: step.tool.clone(),
                    status: status_str.to_string(),
                    elapsed_time: step.actual_time,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&progress_msg)?))
                    .await?;
            }

            // 3. 构建完成消息
            let output_content = format!(
                "\n💡 提示：v1.29.0 实现了意图拆解可视化，执行功能将在后续版本完善\n\
                📊 计划ID: {}\n\
                ⏰ 总预计时间: {:.1}s",
                plan.id,
                plan.total_estimated_time
            );

            // 计算执行时间
            let execution_time = start_time.elapsed().as_secs_f64();

            // 完成回合
            if let Some(completed_round) = session.complete_round(
                &round_id,
                output_content,
                execution_time,
                vec!["intent-decomposer".to_string()],
            ).await {
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: completed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }
        }
        Err(e) => {
            let error_content = format!("❌ 意图拆解失败\n详情: {}", e);

            // 标记回合失败
            if let Some(failed_round) = session.fail_round(&round_id, error_content).await {
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: failed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }
        }
    }

    Ok(())
}
