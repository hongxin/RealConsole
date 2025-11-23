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
use crate::web::session::{ClientMessage, EnabledStep, ServerMessage, Session};
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde_json::json;
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
        ClientMessage::ExecutePlan { plan_id, enabled_steps } => {
            // v1.29.3: 执行计划
            execute_plan(session, &plan_id, &enabled_steps, sender).await?;
        }
        ClientMessage::RerunCell { round_id } => {
            // v1.38.0: 重新执行 Cell
            handle_rerun_cell(session, &round_id, sender).await?;
        }
        // v1.40.0: 会话管理消息
        ClientMessage::SaveSession { name } => {
            handle_save_session(session, name, sender).await?;
        }
        ClientMessage::LoadSession { session_id } => {
            handle_load_session(session, &session_id, sender).await?;
        }
        ClientMessage::ListSessions => {
            handle_list_sessions(sender).await?;
        }
        ClientMessage::DeleteSession { session_id } => {
            handle_delete_session(&session_id, sender).await?;
        }
        ClientMessage::RenameSession { session_id, new_name } => {
            handle_rename_session(&session_id, &new_name, sender).await?;
        }
        ClientMessage::ExportSession { session_id, format } => {
            handle_export_session(&session_id, &format, sender).await?;
        }
        // ===== v1.46.0: 文件上传 =====
        ClientMessage::UploadFile { filename, content } => {
            handle_upload_file(session, filename, content, sender).await?;
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

    // ===== v1.44.0: 特殊处理图表命令 =====
    if cmd.starts_with("chart ") || cmd == "chart" {
        return execute_chart_command(cmd, session, sender).await;
    }

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

/// ===== v1.45.0: 解析 CSV 图表命令 =====
///
/// 格式：`!chart csv <file_path|@file_id> --type <type> --x-col "col" --y-col "col1" --y-col "col2"`
///
/// v1.46.0: 支持 @file_id 语法（上传文件）
fn parse_csv_command(
    cmd: &str,
    session: &Arc<Session>,
) -> anyhow::Result<crate::visualization::ChartData> {
    use crate::visualization::{parse_csv_file, ChartType};

    // 移除 "chart csv" 前缀
    let cmd = cmd.trim_start_matches("chart").trim().trim_start_matches("csv").trim();

    // 提取文件路径/ID（第一个参数）
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("缺少 CSV 文件路径或文件 ID（如 @uploaded_001）"));
    }

    let file_path_or_id = parts[0];
    let args = &parts[1..].join(" ");

    // 提取参数
    let extract_arg = |name: &str| -> Option<String> {
        let pattern = format!("{} ", name);
        if let Some(start) = args.find(&pattern) {
            let value_start = start + pattern.len();
            let remaining = &args[value_start..];

            if remaining.starts_with('"') {
                if let Some(end) = remaining[1..].find('"') {
                    return Some(remaining[1..=end].to_string());
                }
            } else {
                return remaining.split_whitespace().next().map(|s| s.to_string());
            }
        }
        None
    };

    // 获取图表类型
    let chart_type_str = extract_arg("--type").unwrap_or_else(|| "line".to_string());
    let chart_type = ChartType::from_str(&chart_type_str)
        .ok_or_else(|| anyhow::anyhow!("无效的图表类型: {}", chart_type_str))?;

    // 获取标题
    let title = extract_arg("--title").unwrap_or_else(|| "CSV 数据图表".to_string());

    // 获取 X 轴列名
    let x_col = extract_arg("--x-col")
        .ok_or_else(|| anyhow::anyhow!("缺少 --x-col 参数"))?;

    // 获取所有 Y 轴列名
    let mut y_cols = Vec::new();
    let mut search_start = 0;
    while let Some(start) = args[search_start..].find("--y-col ") {
        let actual_start = search_start + start;
        let value_start = actual_start + "--y-col ".len();
        let remaining = &args[value_start..];

        let col_name = if remaining.starts_with('"') {
            if let Some(end) = remaining[1..].find('"') {
                remaining[1..=end].to_string()
            } else {
                return Err(anyhow::anyhow!("--y-col 参数引号未闭合"));
            }
        } else {
            remaining.split_whitespace().next().unwrap_or("").to_string()
        };

        y_cols.push(col_name);
        search_start = value_start + 1;
    }

    if y_cols.is_empty() {
        return Err(anyhow::anyhow!("至少需要一个 --y-col 参数"));
    }

    // v1.46.0: 支持 @file_id 语法
    let csv_data = if file_path_or_id.starts_with('@') {
        // 从上传文件中读取（去掉 @ 前缀）
        let file_id = file_path_or_id.trim_start_matches('@');

        // 获取文件内容
        let content = session.uploaded_files.get(file_id)
            .map_err(|e| anyhow::anyhow!("无法获取上传文件 {}: {}", file_id, e))?;

        // 解析 CSV 字符串
        let (headers, records) = parse_csv_string(&content)?;

        // 构建 CsvData
        crate::visualization::CsvData { headers, records }
    } else {
        // 从文件系统读取
        parse_csv_file(file_path_or_id)?
    };

    // 转换为 ChartData
    let y_col_refs: Vec<&str> = y_cols.iter().map(|s| s.as_str()).collect();
    csv_data.to_chart_data(chart_type, title, &x_col, &y_col_refs)
}

/// ===== v1.44.0: 执行图表命令 =====
///
/// 处理 `!chart` 命令，解析参数并生成图表
///
/// ## 消息流程
/// ```
/// RoundStart(type: shell) → Chart(chart_data) → RoundComplete
/// ```
async fn execute_chart_command(
    cmd: &str,
    session: &Arc<Session>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use crate::visualization::{parse_csv_file, ChartCommand, ChartCommandParser, ChartType, TemplateEngine};

    // 创建回合
    let round = session.create_round(
        crate::web::session::RoundType::Shell,
        cmd.to_string(),
        "chart".to_string(),
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

    // v1.50.0: 统一命令解析入口
    let command_result = if cmd.trim_start_matches("chart").trim().starts_with("csv ") {
        // CSV 命令：!chart csv <file|@file_id> --type <type> --x-col "col" --y-col "col1" --y-col "col2"
        parse_csv_command(cmd, session).map(ChartCommand::Create)
    } else {
        // 使用新的统一解析器（支持模板命令）
        ChartCommandParser::parse_command(cmd)
    };

    // 处理命令结果
    match command_result {
        Ok(ChartCommand::Create(chart_data)) => {
            // 图表创建命令：发送 Chart 消息
            let chart_msg = ServerMessage::Chart {
                round_id: round_id.clone(),
                chart_data,
            };
            sender
                .send(Message::Text(serde_json::to_string(&chart_msg)?))
                .await?;

            let execution_time = start_time.elapsed().as_secs_f64();
            let success_msg = format!("✅ 图表生成成功");
            if let Some(completed_round) = session.complete_round(
                &round_id,
                success_msg,
                execution_time,
                vec!["chart".to_string()],
            ).await {
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: completed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }
        }
        Ok(ChartCommand::ListTemplates { category }) => {
            // v1.50.0: 列出模板命令
            let engine = TemplateEngine::new();
            let templates = if let Some(cat) = category {
                engine.filter_by_category(cat)
            } else {
                engine.all_templates().iter().collect()
            };

            // 格式化输出
            let mut output = String::new();
            output.push_str("📊 **可用图表模板**\n\n");

            if let Some(cat) = category {
                output.push_str(&format!("**分类**: {:?}\n\n", cat));
            } else {
                let summary = engine.category_summary();
                output.push_str(&format!("**总计**: {} 个模板 (", templates.len()));
                output.push_str(&summary.iter()
                    .map(|(cat, count)| format!("{:?}: {}", cat, count))
                    .collect::<Vec<_>>()
                    .join(", "));
                output.push_str(")\n\n");
            }

            for template in templates {
                output.push_str(&format!("### `{}`\n", template.id));
                output.push_str(&format!("**{}** - {}\n", template.name, template.description));
                output.push_str(&format!("💡 {}\n", template.usage_hint));
                output.push_str(&format!("🏷️ {}\n", template.tags.join(", ")));
                output.push_str(&format!("📈 图表类型: {:?}\n\n", template.placeholder_data.chart_type));
                output.push_str(&format!("**使用**: `!chart use {}`\n\n", template.id));
                output.push_str("---\n\n");
            }

            output.push_str("\n💡 **提示**:\n");
            output.push_str("- 使用 `!chart templates <category>` 查看特定分类\n");
            output.push_str("- 分类: business, technical, team, academic, exploration\n");
            output.push_str("- 使用 `!chart use <template-id>` 应用模板创建图表\n");

            let execution_time = start_time.elapsed().as_secs_f64();
            if let Some(completed_round) = session.complete_round(
                &round_id,
                output,
                execution_time,
                vec!["chart".to_string(), "templates".to_string()],
            ).await {
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: completed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }
        }
        Ok(ChartCommand::UseTemplate { template_id }) => {
            // v1.50.0: 使用模板命令
            let engine = TemplateEngine::new();
            if let Some(template) = engine.find_by_id(&template_id) {
                // 发送 Chart 消息（使用模板的占位数据）
                let chart_msg = ServerMessage::Chart {
                    round_id: round_id.clone(),
                    chart_data: template.placeholder_data.clone(),
                };
                sender
                    .send(Message::Text(serde_json::to_string(&chart_msg)?))
                    .await?;

                let execution_time = start_time.elapsed().as_secs_f64();
                let success_msg = format!("✅ 已应用模板: **{}**\n\n{}\n\n💡 这是示例数据，请根据实际需求修改",
                    template.name, template.description);
                if let Some(completed_round) = session.complete_round(
                    &round_id,
                    success_msg,
                    execution_time,
                    vec!["chart".to_string(), "template".to_string()],
                ).await {
                    let round_complete_msg = ServerMessage::RoundComplete {
                        round: completed_round,
                    };
                    sender
                        .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                        .await?;
                }
            } else {
                // 模板不存在（理论上不会到这里，因为 parse_use_command 已经验证）
                let error_msg = format!("❌ 模板 '{}' 不存在", template_id);
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
        Err(e) => {
            // 解析失败，标记回合失败
            let error_msg = format!(
                "❌ 图表命令解析失败\n\n{}\n\n**使用示例**:\n\
                - 创建图表: `!chart line --title \"月度趋势\" --x-axis \"1月,2月,3月\" --series \"销售额:120,132,101\"`\n\
                - 查看模板: `!chart templates`\n\
                - 使用模板: `!chart use sales-trend`",
                e
            );
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
///
/// ## v1.40.0: 添加回合系统支持
/// 消息流程：RoundStart(type: llm) → Output(Intent名称) → Output(结果) → RoundComplete
async fn execute_intent(
    intent_match: &IntentMatch,
    original_text: &str,
    agent: &crate::agent::Agent,
    session: &Arc<Session>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use std::time::Instant;
    let start_time = Instant::now();

    // 创建新回合
    let round = session.create_round(
        crate::web::session::RoundType::Llm,
        original_text.to_string(),
        "intent".to_string()  // 标记为 Intent 执行
    ).await;
    let round_id = round.id.clone();

    // 发送 RoundStart 消息
    let round_start_msg = ServerMessage::RoundStart { round: round.clone() };
    sender
        .send(Message::Text(serde_json::to_string(&round_start_msg)?))
        .await?;

    // 1. 发送意图识别提示
    let info_msg = ServerMessage::Output {
        content: format!("🎯 {}\n", intent_match.intent.name),
    };
    sender
        .send(Message::Text(serde_json::to_string(&info_msg)?))
        .await?;

    let (ai_response, _status) = match agent.template_engine.generate_from_intent(intent_match) {
        Ok(plan) => {
            // 2. 执行计划中的命令
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
            let msg = ServerMessage::Output { content: result.clone() };
            sender
                .send(Message::Text(serde_json::to_string(&msg)?))
                .await?;

            // 返回完整响应（Intent 名称 + 结果）
            let full_response = format!("🎯 {}\n{}", intent_match.intent.name, result);
            (full_response, "success")
        }
        Err(e) => {
            // 如果生成执行计划失败，回退到 LLM 对话
            eprintln!("⚠️ Intent 执行计划生成失败: {}", e);
            execute_llm_chat(original_text, agent, session, sender).await?;
            return Ok(());  // LLM 对话会自己发送 RoundComplete
        }
    };

    // 完成回合
    let execution_time = start_time.elapsed().as_secs_f64();
    if let Some(completed_round) = session.complete_round(
        &round_id,
        ai_response,
        execution_time,
        vec![]  // Intent 执行没有工具使用
    ).await {
        let round_complete_msg = ServerMessage::RoundComplete {
            round: completed_round,
        };
        sender
            .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
            .await?;
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

    // v1.31.0: 先尝试 Intent 预识别（快速路径）
    if let Some(plan) = session.intent_router.try_match(query) {
        eprintln!("✨ [Intent] 快速识别成功，跳过 LLM 拆解");

        // v1.36.2: 生成态势测算分析结果（替换 v1.36.0 的占卜动画）
        let situation_analysis = {
            // 读取配置（克隆以避免长时间持有锁）
            let divination_config = {
                let agent = session.agent.read().await;
                agent.config.divination.clone()
            };

            if divination_config.enabled {
                use crate::agent::divination::SituationAnalyzer;

                // 执行态势分析（无动画，直接生成结果）
                let analysis = SituationAnalyzer::analyze(&plan);

                Some(analysis)
            } else {
                None
            }
        };

        // 1. 发送意图理解消息（包含态势测算分析结果）
        let understanding_msg = ServerMessage::IntentUnderstanding {
            plan_id: plan.id.clone(),
            understanding: plan.understanding.clone(),
            step_count: plan.steps.len(),
            total_time: plan.total_estimated_time,
            situation_analysis,  // v1.36.2: 替换 divination
        };
        sender
            .send(Message::Text(serde_json::to_string(&understanding_msg)?))
            .await?;

        // 2. 发送步骤状态（可视化）
        for (index, step) in plan.steps.iter().enumerate() {
            let progress_msg = ServerMessage::StepProgress {
                plan_id: plan.id.clone(),
                step_index: index,
                step_id: step.id.clone(),
                description: step.description.clone(),
                tool: step.tool.clone(),
                params: step.params.clone(),
                status: "pending".to_string(),
                elapsed_time: step.actual_time,
            };
            sender
                .send(Message::Text(serde_json::to_string(&progress_msg)?))
                .await?;
        }

        // v1.39.0: 自动执行计划（保留可视化价值）
        eprintln!("🚀 [Decompose] Auto-executing plan: {}", plan.id);

        // 转换 ExecutionStep -> EnabledStep
        let enabled_steps: Vec<EnabledStep> = plan
            .steps
            .iter()
            .enumerate()
            .map(|(index, step)| EnabledStep {
                step_id: step.id.clone(),
                step_index: index,
                description: step.description.clone(),
                tool: step.tool.clone(),
                params: step.params.clone(),
            })
            .collect();

        // 执行计划（调用已有的 execute_plan 函数）
        if let Err(e) = execute_plan(session, &plan.id, &enabled_steps, sender).await {
            eprintln!("❌ [Decompose] Plan execution failed: {}", e);
            // 标记回合失败
            if let Some(failed_round) = session.fail_round(&round_id, format!("❌ 计划执行失败: {}", e)).await {
                let round_complete_msg = ServerMessage::RoundComplete {
                    round: failed_round,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                    .await?;
            }
            return Ok(());
        }

        // 构建完成消息
        let output_content = "\n⚡ 通过 Intent DSL 快速识别并执行".to_string();

        // 完成回合
        let execution_time = start_time.elapsed().as_secs_f64();
        if let Some(completed_round) = session
            .complete_round(&round_id, output_content, execution_time, Vec::new())
            .await
        {
            let round_complete_msg = ServerMessage::RoundComplete {
                round: completed_round,
            };
            sender
                .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                .await?;
        }

        return Ok(());
    }

    // Intent 未匹配，继续 LLM 拆解（v1.30.0 原有逻辑）
    eprintln!("🤖 [LLM] Intent 未匹配，回退到 LLM 拆解");

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
            // v1.36.0: 生成并发送态势测算分析动画（如果启用）
            // v1.36.2: 生成态势测算分析结果（替换 v1.36.0 的占卜动画）
            let situation_analysis = {
                // 读取配置（克隆以避免长时间持有锁）
                let divination_config = {
                    let agent = session.agent.read().await;
                    agent.config.divination.clone()
                };

                if divination_config.enabled {
                    use crate::agent::divination::SituationAnalyzer;

                    // 执行态势分析（无动画，直接生成结果）
                    let analysis = SituationAnalyzer::analyze(&plan);

                    Some(analysis)
                } else {
                    None
                }
            };

            // 1. 发送意图理解消息（包含态势测算分析结果）
            let understanding_msg = ServerMessage::IntentUnderstanding {
                plan_id: plan.id.clone(),
                understanding: plan.understanding.clone(),
                step_count: plan.steps.len(),
                total_time: plan.total_estimated_time,
                situation_analysis,  // v1.36.2: 替换 divination
            };
            sender
                .send(Message::Text(serde_json::to_string(&understanding_msg)?))
                .await?;

            // 2. 发送每个步骤的初始状态（可视化）
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
                    params: step.params.clone(),  // v1.30.0: 包含工具参数
                    status: status_str.to_string(),
                    elapsed_time: step.actual_time,
                };
                sender
                    .send(Message::Text(serde_json::to_string(&progress_msg)?))
                    .await?;
            }

            // v1.39.0: 自动执行计划（保留可视化价值）
            eprintln!("🚀 [Decompose] Auto-executing LLM plan: {}", plan.id);

            // 转换 ExecutionStep -> EnabledStep
            let enabled_steps: Vec<EnabledStep> = plan
                .steps
                .iter()
                .enumerate()
                .map(|(index, step)| EnabledStep {
                    step_id: step.id.clone(),
                    step_index: index,
                    description: step.description.clone(),
                    tool: step.tool.clone(),
                    params: step.params.clone(),
                })
                .collect();

            // 执行计划（调用已有的 execute_plan 函数）
            if let Err(e) = execute_plan(session, &plan.id, &enabled_steps, sender).await {
                eprintln!("❌ [Decompose] LLM plan execution failed: {}", e);
                // 标记回合失败
                if let Some(failed_round) = session.fail_round(&round_id, format!("❌ 计划执行失败: {}", e)).await {
                    let round_complete_msg = ServerMessage::RoundComplete {
                        round: failed_round,
                    };
                    sender
                        .send(Message::Text(serde_json::to_string(&round_complete_msg)?))
                        .await?;
                }
                return Ok(());
            }

            // 构建完成消息
            let output_content = "\n🤖 通过 LLM 拆解并执行".to_string();

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

/// v1.29.3: 执行计划
async fn execute_plan(
    session: &Arc<Session>,
    plan_id: &str,
    enabled_steps: &[EnabledStep],
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use std::time::Instant;

    let total_count = enabled_steps.len();

    // 发送执行开始消息
    let start_msg = ServerMessage::PlanExecutionStart {
        plan_id: plan_id.to_string(),
        enabled_count: total_count,
        total_count,
    };
    sender
        .send(Message::Text(serde_json::to_string(&start_msg)?))
        .await?;

    let mut executed_count = 0;
    let mut has_error = false;
    let plan_start_time = Instant::now();

    // 执行每个启用的步骤
    for step in enabled_steps {
        let step_start_time = Instant::now();

        // 发送步骤开始运行
        let progress_msg = ServerMessage::StepProgress {
            plan_id: plan_id.to_string(),
            step_id: step.step_id.clone(),
            step_index: step.step_index,
            description: step.description.clone(),
            tool: step.tool.clone(),
            params: step.params.clone(),  // v1.30.0: 包含工具参数
            status: "running".to_string(),
            elapsed_time: None,
        };
        sender
            .send(Message::Text(serde_json::to_string(&progress_msg)?))
            .await?;

        // 执行步骤（模拟工具调用）
        let result = execute_step(session, step).await;

        let elapsed = step_start_time.elapsed().as_secs_f64();

        match result {
            Ok(output) => {
                // 发送步骤输出
                if !output.is_empty() {
                    let output_msg = ServerMessage::StepOutput {
                        plan_id: plan_id.to_string(),
                        step_id: step.step_id.clone(),
                        output: output.clone(),
                    };
                    sender
                        .send(Message::Text(serde_json::to_string(&output_msg)?))
                        .await?;
                }

                // 发送步骤成功
                let success_msg = ServerMessage::StepProgress {
                    plan_id: plan_id.to_string(),
                    step_id: step.step_id.clone(),
                    step_index: step.step_index,
                    description: step.description.clone(),
                    tool: step.tool.clone(),
                    params: step.params.clone(),  // v1.30.0: 包含工具参数
                    status: "success".to_string(),
                    elapsed_time: Some(elapsed),
                };
                sender
                    .send(Message::Text(serde_json::to_string(&success_msg)?))
                    .await?;

                executed_count += 1;
            }
            Err(e) => {
                // 发送步骤失败
                let error_msg = ServerMessage::StepOutput {
                    plan_id: plan_id.to_string(),
                    step_id: step.step_id.clone(),
                    output: format!("❌ 步骤执行失败: {}", e),
                };
                sender
                    .send(Message::Text(serde_json::to_string(&error_msg)?))
                    .await?;

                let failed_msg = ServerMessage::StepProgress {
                    plan_id: plan_id.to_string(),
                    step_id: step.step_id.clone(),
                    step_index: step.step_index,
                    description: step.description.clone(),
                    tool: step.tool.clone(),
                    params: step.params.clone(),  // v1.30.0: 包含工具参数
                    status: "failed".to_string(),
                    elapsed_time: Some(elapsed),
                };
                sender
                    .send(Message::Text(serde_json::to_string(&failed_msg)?))
                    .await?;

                has_error = true;
                // 继续执行其他步骤，不中断
            }
        }

        // 添加小延迟，避免消息过快
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let total_time = plan_start_time.elapsed().as_secs_f64();

    // 发送执行完成消息
    let complete_msg = ServerMessage::PlanExecutionComplete {
        plan_id: plan_id.to_string(),
        success: !has_error,
        executed_count,
        skipped_count: 0, // 当前版本没有跳过的步骤
        total_time,
    };
    sender
        .send(Message::Text(serde_json::to_string(&complete_msg)?))
        .await?;

    Ok(())
}

/// 执行单个步骤（v1.30.0: 调用 ToolRegistry）
async fn execute_step(
    session: &Arc<Session>,
    step: &EnabledStep,
) -> anyhow::Result<String> {
    // v1.30.0: 直接调用 ToolRegistry
    // 所有工具统一由 Tool 系统处理，包括：
    // - 参数验证
    // - 安全检查
    // - 错误处理

    // 获取 ToolRegistry
    let agent = session.agent.read().await;
    let registry = agent.tool_registry.read().await;

    // 解析参数（如果没有提供，使用空对象）
    let params = step.params.clone().unwrap_or(json!({}));

    // 调用工具
    match registry.execute(&step.tool, params) {
        Ok(output) => {
            Ok(format!("✅ 执行成功\n工具: {}\n\n{}", step.tool, output))
        }
        Err(e) => {
            Ok(format!("❌ 执行失败\n工具: {}\n错误: {}", step.tool, e))
        }
    }
}

/// 执行单个步骤（v1.29.4 废弃：手动命令提取）
/// 该函数已被 v1.30.0 的 ToolRegistry 方案替代
#[allow(dead_code)]
async fn execute_step_legacy(
    _session: &Arc<Session>,
    step: &EnabledStep,
) -> anyhow::Result<String> {
    // v1.29.4: 简化版真实执行（已废弃）
    // 保留此代码作为参考，展示命令提取的局限性

    match step.tool.as_str() {
        "shell" => {
            // 尝试从 description 中提取命令
            // 常见模式："检查 X 是否存在" -> "test -f X"
            //         "列出 X 目录" -> "ls X"
            //         "读取 X" -> "cat X"

            let cmd = extract_shell_command(&step.description);
            if let Some(command) = cmd {
                // 执行 Shell 命令（在当前目录）
                match tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .output()
                    .await
                {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        if output.status.success() {
                            if !stdout.is_empty() {
                                Ok(format!("✅ 执行成功\n$ {}\n\n{}", command, stdout.trim()))
                            } else {
                                Ok(format!("✅ 执行成功\n$ {}\n(无输出)", command))
                            }
                        } else if !stderr.is_empty() {
                            Ok(format!("❌ 执行失败\n$ {}\n\n{}", command, stderr.trim()))
                        } else {
                            Ok(format!("❌ 执行失败\n$ {}\n退出码: {}", command, output.status.code().unwrap_or(-1)))
                        }
                    }
                    Err(e) => {
                        Ok(format!("❌ 命令执行出错\n$ {}\n错误: {}", command, e))
                    }
                }
            } else {
                // 无法提取命令，返回说明
                Ok(format!("ℹ️ Shell 工具\n描述: {}\n提示: 无法自动提取命令", step.description))
            }
        }
        "file_read" => {
            // 尝试从 description 中提取文件路径
            if let Some(path) = extract_file_path(&step.description) {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        let lines: Vec<&str> = content.lines().collect();
                        let preview = if lines.len() > 20 {
                            format!("{}...\n(共 {} 行，显示前 20 行)",
                                lines[..20].join("\n"), lines.len())
                        } else {
                            content
                        };
                        Ok(format!("✅ 读取文件成功: {}\n\n{}", path, preview))
                    }
                    Err(e) => {
                        Ok(format!("❌ 读取文件失败: {}\n错误: {}", path, e))
                    }
                }
            } else {
                Ok(format!("ℹ️ 文件读取工具\n描述: {}\n提示: 无法自动提取文件路径", step.description))
            }
        }
        _ => {
            // 其他工具暂时返回说明性输出
            Ok(format!(
                "ℹ️ 工具: {}\n描述: {}\n提示: 此工具尚未实现真实执行",
                step.tool,
                step.description
            ))
        }
    }
}

/// 从描述中提取 Shell 命令
fn extract_shell_command(description: &str) -> Option<String> {
    let desc_lower = description.to_lowercase();

    // 首先尝试提取明确的文件名（带扩展名的）
    let filename = extract_filename_strict(description);

    // 优先级 1: "列出" 动作（因为列出目录也能看到文件是否存在）
    if desc_lower.contains("列出") {
        if desc_lower.contains("当前目录") || desc_lower.contains("文件") {
            return Some("ls -la".to_string());
        } else if desc_lower.contains("目录") {
            // 尝试查找目录名（如 src, docs 等）
            for word in description.split_whitespace() {
                if word.len() >= 2 && word.len() <= 10
                   && word.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '/')
                   && !word.chars().all(|c| c.is_ascii_digit()) {
                    return Some(format!("ls -la {}", word));
                }
            }
            return Some("ls -la".to_string());
        }
    }

    // 优先级 2: "检查/确认 X 是否存在"
    if (desc_lower.contains("检查") || desc_lower.contains("确认"))
       && desc_lower.contains("是否存在") {
        if let Some(file) = &filename {
            return Some(format!("test -e {} && echo '✅ 文件存在' || echo '❌ 文件不存在'", file));
        }
    }

    // 优先级 3: "读取 X"
    if desc_lower.contains("读取") || desc_lower.contains("read") {
        if let Some(file) = &filename {
            return Some(format!("cat {}", file));
        }
    }

    None
}

/// 严格提取文件名（只提取带扩展名的文件）
fn extract_filename_strict(description: &str) -> Option<String> {
    // 常见文件扩展名
    let extensions = vec![
        ".yaml", ".yml", ".json", ".txt", ".csv", ".toml", ".md", ".rs",
        ".py", ".js", ".ts", ".go", ".c", ".cpp", ".h", ".java", ".xml",
        ".ini", ".conf", ".cfg", ".log"
    ];

    for word in description.split_whitespace() {
        // 清理标点符号
        let cleaned = word.trim_matches(|c: char| {
            c.is_ascii_punctuation() && c != '.' && c != '/' && c != '_' && c != '-'
        });

        // 检查是否包含扩展名
        if extensions.iter().any(|ext| cleaned.ends_with(ext)) {
            return Some(cleaned.to_string());
        }
    }

    None
}

/// 从描述中提取文件/目录名
fn extract_filename(description: &str) -> Option<String> {
    // 常见文件扩展名
    let extensions = [".yaml", ".yml", ".json", ".txt", ".csv", ".toml", ".md", ".rs"];

    for word in description.split_whitespace() {
        // 检查是否包含扩展名
        if extensions.iter().any(|ext| word.contains(ext)) {
            return Some(word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '/' && c != '_' && c != '-').to_string());
        }

        // 检查是否是目录名（如 src, data 等）
        if word.len() > 2 && word.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '/') {
            return Some(word.to_string());
        }
    }

    None
}

/// 从描述中提取文件路径
fn extract_file_path(description: &str) -> Option<String> {
    extract_filename_strict(description)
}

/// v1.38.0: 处理 Cell 重新执行
async fn handle_rerun_cell(
    session: &Arc<Session>,
    round_id: &str,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    // 1. 从 session 中获取原始输入内容
    let original_input = {
        let rounds = session.rounds.read().await;
        rounds
            .iter()
            .find(|r| r.id == round_id)
            .map(|r| r.user_input.clone())
    };

    let input_content = match original_input {
        Some(content) => content,
        None => {
            // Round 不存在，返回错误
            let error_msg = ServerMessage::Error {
                content: format!("Round {} not found", round_id),
            };
            sender
                .send(Message::Text(serde_json::to_string(&error_msg)?))
                .await?;
            return Ok(());
        }
    };

    println!(
        "[v1.38.0] Rerunning cell: round_id={}, input={}",
        round_id,
        input_content.chars().take(50).collect::<String>()
    );

    // 2. 发送 ClearCell 消息，通知前端清空输出
    let clear_msg = ServerMessage::ClearCell {
        round_id: round_id.to_string(),
    };
    sender
        .send(Message::Text(serde_json::to_string(&clear_msg)?))
        .await?;

    // 3. 重新执行该输入（复用现有的 handle_input 逻辑）
    handle_input(session, &input_content, sender).await?;

    Ok(())
}

// ===== v1.40.0 新增：会话管理处理器 =====

/// 处理保存会话
async fn handle_save_session(
    session: &Arc<Session>,
    name: Option<String>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use crate::web::session_manager::SessionManager;

    let manager = SessionManager::new()?;
    let mut serializable = session.to_serializable().await;

    // 如果提供了名称，使用自定义名称
    if let Some(custom_name) = name {
        serializable.name = custom_name;
    }

    match manager.save_session(&serializable) {
        Ok(_) => {
            let response = ServerMessage::SessionSaved {
                session_id: serializable.id.clone(),
                name: serializable.name.clone(),
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
        Err(e) => {
            let response = ServerMessage::SessionError {
                message: format!("保存会话失败: {}", e),
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
    }

    Ok(())
}

/// 处理加载会话
async fn handle_load_session(
    session: &Arc<Session>,
    session_id: &str,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use crate::web::session_manager::SessionManager;

    let manager = SessionManager::new()?;

    match manager.load_session(session_id) {
        Ok(serializable) => {
            // 恢复会话数据到当前会话（只恢复回合历史）
            {
                let mut rounds = session.rounds.write().await;
                *rounds = serializable.rounds.clone();
            }

            let response = ServerMessage::SessionLoaded {
                session: serializable,
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
        Err(e) => {
            let response = ServerMessage::SessionError {
                message: format!("加载会话失败: {}", e),
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
    }

    Ok(())
}

/// 处理列出会话
async fn handle_list_sessions(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use crate::web::session_manager::SessionManager;

    let manager = SessionManager::new()?;

    match manager.list_sessions() {
        Ok(sessions) => {
            let response = ServerMessage::SessionList { sessions };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
        Err(e) => {
            let response = ServerMessage::SessionError {
                message: format!("列出会话失败: {}", e),
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
    }

    Ok(())
}

/// 处理重命名会话
async fn handle_rename_session(
    session_id: &str,
    new_name: &str,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use crate::web::session_manager::SessionManager;

    let manager = SessionManager::new()?;

    match manager.rename_session(session_id, new_name) {
        Ok(_) => {
            let response = ServerMessage::SessionRenamed {
                session_id: session_id.to_string(),
                new_name: new_name.to_string(),
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;

            // 重新发送会话列表
            handle_list_sessions(sender).await?;
        }
        Err(e) => {
            let response = ServerMessage::SessionError {
                message: format!("重命名会话失败: {}", e),
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
    }

    Ok(())
}

/// 处理删除会话
async fn handle_delete_session(
    session_id: &str,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use crate::web::session_manager::SessionManager;

    let manager = SessionManager::new()?;

    match manager.delete_session(session_id) {
        Ok(_) => {
            let response = ServerMessage::SessionDeleted {
                session_id: session_id.to_string(),
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
        Err(e) => {
            let response = ServerMessage::SessionError {
                message: format!("删除会话失败: {}", e),
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
    }

    Ok(())
}

/// 处理导出会话
async fn handle_export_session(
    session_id: &str,
    format: &str,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use crate::web::session_manager::SessionManager;

    let manager = SessionManager::new()?;

    // 1. 加载会话
    let session_result = manager.load_session(session_id);
    if let Err(e) = session_result {
        let response = ServerMessage::SessionError {
            message: format!("加载会话失败: {}", e),
        };
        sender
            .send(Message::Text(serde_json::to_string(&response)?))
            .await?;
        return Ok(());
    }

    let session_data = session_result.unwrap();

    // 2. 导出到指定格式
    let export_result = match format {
        "markdown" | "md" => manager.export_to_markdown(&session_data),
        "html" => manager.export_to_html(&session_data),
        _ => Err(anyhow::anyhow!("不支持的导出格式: {}", format)),
    };

    match export_result {
        Ok(content) => {
            // 3. 保存导出文件
            let file_format = if format == "md" { "markdown" } else { format };
            match manager.save_export(session_id, &content, file_format) {
                Ok(export_path) => {
                    let response = ServerMessage::SessionExported {
                        session_id: session_id.to_string(),
                        export_path: export_path.to_string_lossy().to_string(),
                        format: file_format.to_string(),
                        content: content.clone(),  // 返回文件内容供前端下载
                    };
                    sender
                        .send(Message::Text(serde_json::to_string(&response)?))
                        .await?;
                }
                Err(e) => {
                    let response = ServerMessage::SessionError {
                        message: format!("保存导出文件失败: {}", e),
                    };
                    sender
                        .send(Message::Text(serde_json::to_string(&response)?))
                        .await?;
                }
            }
        }
        Err(e) => {
            let response = ServerMessage::SessionError {
                message: format!("导出会话失败: {}", e),
            };
            sender
                .send(Message::Text(serde_json::to_string(&response)?))
                .await?;
        }
    }

    Ok(())
}

/// ===== v1.46.0: 处理文件上传 =====
///
/// 功能：
/// - 解析 CSV 内容
/// - 生成预览数据（前 10 行）
/// - 存储到内存（LRU 缓存）
/// - 返回文件 ID 和预览
async fn handle_upload_file(
    session: &Arc<Session>,
    filename: String,
    content: String,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    use crate::web::session::FilePreview;

    // 1. 验证文件格式（只支持 CSV）
    if !filename.to_lowercase().ends_with(".csv") {
        let error_msg = ServerMessage::Error {
            content: format!("❌ 不支持的文件格式: {}\n提示: 当前只支持 CSV 文件", filename),
        };
        sender
            .send(Message::Text(serde_json::to_string(&error_msg)?))
            .await?;
        return Ok(());
    }

    // 2. 解析 CSV 内容
    let csv_result = parse_csv_string(&content);
    let (headers, records) = match csv_result {
        Ok(data) => data,
        Err(e) => {
            let error_msg = ServerMessage::Error {
                content: format!("❌ CSV 解析失败: {}\n提示: 请检查文件格式是否正确", e),
            };
            sender
                .send(Message::Text(serde_json::to_string(&error_msg)?))
                .await?;
            return Ok(());
        }
    };

    // 3. 生成预览数据（前 10 行）
    let preview_rows = records
        .iter()
        .take(10)
        .cloned()
        .collect::<Vec<_>>();

    let preview = FilePreview {
        headers: headers.clone(),
        rows: preview_rows,
        total_rows: records.len(),
        total_columns: headers.len(),
    };

    // 4. 存储文件到内存
    let file_id = match session.uploaded_files.add(filename.clone(), content) {
        Ok(id) => id,
        Err(e) => {
            let error_msg = ServerMessage::Error {
                content: format!("❌ 文件存储失败: {}", e),
            };
            sender
                .send(Message::Text(serde_json::to_string(&error_msg)?))
                .await?;
            return Ok(());
        }
    };

    // 5. 返回成功消息
    let response = ServerMessage::FileUploaded {
        file_id: file_id.clone(),
        filename,
        preview,
    };

    sender
        .send(Message::Text(serde_json::to_string(&response)?))
        .await?;

    println!("✅ [FileUpload] File uploaded successfully: {}", file_id);

    Ok(())
}

/// 解析 CSV 字符串内容
///
/// # 返回
/// - `Result<(Vec<String>, Vec<Vec<String>>)>`: (headers, records)
fn parse_csv_string(content: &str) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
    use std::io::Cursor;

    // 使用 csv crate 读取字符串
    let cursor = Cursor::new(content);
    let mut reader = csv::Reader::from_reader(cursor);

    // 读取 headers
    let headers = reader
        .headers()
        .map_err(|e| anyhow::anyhow!("无法读取 CSV header: {}", e))?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    if headers.is_empty() {
        return Err(anyhow::anyhow!("CSV 文件为空"));
    }

    // 读取所有记录
    let mut records = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| anyhow::anyhow!("读取 CSV 记录失败: {}", e))?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();

        // 验证列数匹配
        if row.len() != headers.len() {
            return Err(anyhow::anyhow!(
                "CSV 数据列数不一致：期望 {} 列，实际 {} 列",
                headers.len(),
                row.len()
            ));
        }

        records.push(row);
    }

    Ok((headers, records))
}
