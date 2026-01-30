//! Web 终端会话管理
//!
//! 为每个 WebSocket 连接维护独立的 Agent 实例和会话状态

use crate::agent::Agent;
use crate::command::CommandRegistry;
use crate::config::Config;
use crate::i18n;
use crate::llm::{DeepseekClient, LlmClient, OllamaClient};
use crate::visualization; // v1.44.0: 可视化系统
use crate::web::memory::SmartWebUIOrchestrator; // v1.54.0: Memory 2.0
use crate::web::notebook_ws::NotebookSession; // v2.1.0: Notebook WebSocket
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 会话 ID
pub type SessionId = String;

/// 对话回合类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoundType {
    /// LLM 对话
    Llm,
    /// Shell 命令
    Shell,
    /// 系统命令
    System,
}

/// 对话回合状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoundStatus {
    /// 等待执行
    Pending,
    /// 执行中
    Running,
    /// 执行成功
    Success,
    /// 执行失败
    Error { message: String },
}

/// 对话回合（v1.28.0 新增）
///
/// 每个对话轮次对应一个回合，包含用户输入、AI 响应、执行元数据等
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRound {
    /// 回合 ID
    pub id: String,

    /// 回合序号（从 1 开始）
    pub index: usize,

    /// 回合类型
    pub round_type: RoundType,

    /// 用户输入
    pub user_input: String,

    /// AI 响应（完整文本）
    pub ai_response: String,

    /// 使用的工具列表
    pub tools_used: Vec<String>,

    /// 执行时间（秒）
    pub execution_time: f64,

    /// 执行状态
    pub status: RoundStatus,

    /// 创建时间
    pub timestamp: DateTime<Utc>,

    /// 使用的模型
    pub model: String,
}

impl ConversationRound {
    /// 创建新回合
    pub fn new(index: usize, round_type: RoundType, user_input: String, model: String) -> Self {
        Self {
            id: format!("round-{}", Uuid::new_v4()),
            index,
            round_type,
            user_input,
            ai_response: String::new(),
            tools_used: Vec::new(),
            execution_time: 0.0,
            status: RoundStatus::Running,
            timestamp: Utc::now(),
            model,
        }
    }

    /// 完成回合（成功）
    pub fn complete(&mut self, response: String, execution_time: f64, tools_used: Vec<String>) {
        self.ai_response = response;
        self.execution_time = execution_time;
        self.tools_used = tools_used;
        self.status = RoundStatus::Success;
    }

    /// 标记失败
    pub fn fail(&mut self, error_message: String) {
        self.status = RoundStatus::Error {
            message: error_message,
        };
    }
}

/// 消息类型（Client → Server）
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClientMessage {
    /// 用户输入命令
    Input { content: String },
    /// 中断信号（Ctrl+C）
    Interrupt { content: String },
    /// v1.29.3: 执行计划
    #[serde(rename = "execute_plan")]
    ExecutePlan {
        plan_id: String,
        enabled_steps: Vec<EnabledStep>,
    },
    /// v1.38.0: 重新执行 Cell
    #[serde(rename = "rerun_cell")]
    RerunCell { round_id: String },

    // ===== v1.40.0 新增：会话管理消息 =====
    /// 保存当前会话
    #[serde(rename = "save_session")]
    SaveSession {
        /// 可选的会话名称（如果不提供则自动生成）
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// 加载已保存的会话
    #[serde(rename = "load_session")]
    LoadSession { session_id: String },
    /// 列出所有已保存的会话
    #[serde(rename = "list_sessions")]
    ListSessions,
    /// 删除已保存的会话
    #[serde(rename = "delete_session")]
    DeleteSession { session_id: String },
    /// 重命名已保存的会话
    #[serde(rename = "rename_session")]
    RenameSession {
        session_id: String,
        new_name: String,
    },
    /// 导出会话（支持 markdown, html 格式）
    #[serde(rename = "export_session")]
    ExportSession {
        session_id: String,
        format: String, // "markdown" 或 "html"
    },

    // ===== v1.46.0 新增：文件上传功能 =====
    /// 上传 CSV 文件
    #[serde(rename = "upload_file")]
    UploadFile {
        /// 文件名
        filename: String,
        /// 文件内容（CSV 格式的文本）
        content: String,
    },

    // ===== v2.1.0 新增：Notebook 消息 =====
    /// 列出所有笔记本
    #[serde(rename = "list_notebooks")]
    ListNotebooks,
    /// 创建新笔记本
    #[serde(rename = "create_notebook")]
    CreateNotebook { name: String },
    /// 打开笔记本
    #[serde(rename = "open_notebook")]
    OpenNotebook { notebook_id: String },
    /// 保存笔记本
    #[serde(rename = "save_notebook")]
    SaveNotebook { notebook_id: String },
    /// 删除笔记本
    #[serde(rename = "delete_notebook")]
    DeleteNotebook { notebook_id: String },
    /// 重命名笔记本
    #[serde(rename = "rename_notebook")]
    RenameNotebook { notebook_id: String, new_name: String },
    /// 添加 Cell
    #[serde(rename = "add_cell")]
    AddCell {
        notebook_id: String,
        cell_type: String,
        source: String,
        #[serde(default)]
        index: Option<usize>,
    },
    /// 更新 Cell
    #[serde(rename = "update_cell")]
    UpdateCell {
        notebook_id: String,
        cell_id: String,
        source: String,
    },
    /// 删除 Cell
    #[serde(rename = "delete_cell")]
    DeleteCell {
        notebook_id: String,
        cell_id: String,
    },
    /// 移动 Cell
    #[serde(rename = "move_cell")]
    MoveCell {
        notebook_id: String,
        cell_id: String,
        new_index: usize,
    },
    /// 执行 Cell
    #[serde(rename = "execute_cell")]
    ExecuteCell {
        notebook_id: String,
        cell_id: String,
    },
    /// 执行所有 Cell
    #[serde(rename = "execute_all")]
    ExecuteAll { notebook_id: String },
    /// 清除 Cell 输出
    #[serde(rename = "clear_outputs")]
    ClearOutputs {
        notebook_id: String,
        cell_id: String,
    },
    /// 导出笔记本
    #[serde(rename = "export_notebook")]
    ExportNotebook {
        notebook_id: String,
        format: String, // "rcnb", "json", "markdown"
    },

    // ===== v2.2.0-beta.1 新增：Notebook 导入功能 =====
    /// 导入笔记本（支持 .ipynb, .rcnb, .json）
    #[serde(rename = "import_notebook")]
    ImportNotebook {
        /// 文件格式: "ipynb", "rcnb", "json"
        format: String,
        /// 文件内容（JSON 字符串）
        content: String,
        /// 原始文件名（用于提取笔记本名称）
        filename: String,
    },
}

/// v1.29.3: 启用的步骤信息
#[derive(Debug, Deserialize, Clone)]
pub struct EnabledStep {
    pub step_id: String,
    pub step_index: usize,
    pub description: String,
    pub tool: String,
    /// 工具参数（可选，JSON 格式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// v1.46.0: 文件预览数据
#[derive(Debug, Serialize, Clone)]
pub struct FilePreview {
    /// 列名（CSV header）
    pub headers: Vec<String>,
    /// 前 N 行数据（示例）
    pub rows: Vec<Vec<String>>,
    /// 总行数
    pub total_rows: usize,
    /// 列数
    pub total_columns: usize,
}

/// v1.51.0: 图表历史记录
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartHistoryEntry {
    /// 历史记录 ID
    pub id: String,
    /// 创建时间
    pub timestamp: DateTime<Utc>,
    /// 图表数据
    pub chart_data: visualization::ChartData,
    /// 关联的回合 ID
    pub round_id: Option<String>,
    /// 用于创建该图表的命令
    pub command: String,
    /// 图表标题（快速查看）
    pub title: String,
}

impl ChartHistoryEntry {
    /// 创建新的历史记录
    pub fn new(chart_data: visualization::ChartData, round_id: Option<String>, command: String) -> Self {
        let title = chart_data.title.clone();
        Self {
            id: format!("chart-{}", Uuid::new_v4()),
            timestamp: Utc::now(),
            chart_data,
            round_id,
            command,
            title,
        }
    }
}

/// v1.52.0: 图像历史记录
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageHistoryEntry {
    /// 历史记录 ID
    pub id: String,
    /// 创建时间
    pub timestamp: DateTime<Utc>,
    /// 图像数据
    pub image_data: visualization::ImageData,
    /// 关联的回合 ID
    pub round_id: Option<String>,
    /// 用于显示该图像的命令
    pub command: String,
}

impl ImageHistoryEntry {
    /// 创建新的图像历史记录
    pub fn new(image_data: visualization::ImageData, round_id: Option<String>, command: String) -> Self {
        Self {
            id: format!("image-{}", Uuid::new_v4()),
            timestamp: Utc::now(),
            image_data,
            round_id,
            command,
        }
    }
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

    // ===== v1.28.0 新增：对话回合消息 =====
    /// 回合开始（创建新回合）
    #[serde(rename = "round_start")]
    RoundStart { round: ConversationRound },

    /// 回合更新（状态变化）
    #[serde(rename = "round_update")]
    RoundUpdate {
        round_id: String,
        status: RoundStatus,
    },

    /// 回合完成（包含完整结果）
    #[serde(rename = "round_complete")]
    RoundComplete { round: ConversationRound },

    /// 历史回合列表（初始加载或重连）
    #[serde(rename = "round_history")]
    RoundHistory { rounds: Vec<ConversationRound> },

    // ===== v1.38.0 新增：Cell 重新执行消息 =====
    /// 清空 Cell 输出（重新执行前）
    #[serde(rename = "clear_cell")]
    ClearCell { round_id: String },

    // ===== v1.29.0 新增：意图拆解可视化消息 =====
    /// 意图理解（显示AI对意图的理解）
    #[serde(rename = "intent_understanding")]
    IntentUnderstanding {
        plan_id: String,
        understanding: String,
        step_count: usize,
        total_time: f64,
        /// v1.36.2 新增：态势分析结果（替换 v1.36.0 的 divination）
        #[serde(skip_serializing_if = "Option::is_none")]
        situation_analysis: Option<crate::agent::divination::SituationAnalysis>,
    },

    /// 步骤进度（执行中的步骤更新）
    #[serde(rename = "step_progress")]
    StepProgress {
        plan_id: String,
        step_index: usize,
        step_id: String,
        description: String,
        tool: String,
        /// v1.30.0: 工具参数
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<serde_json::Value>,
        status: String, // "pending" | "running" | "success" | "failed"
        elapsed_time: Option<f64>,
    },

    /// 步骤完成（整个计划执行完成）
    #[serde(rename = "step_complete")]
    StepComplete {
        plan_id: String,
        success: bool,
        total_time: f64,
        outputs: Vec<String>,
    },

    // ===== v1.29.3 新增：计划执行消息 =====
    /// 计划执行开始
    #[serde(rename = "plan_execution_start")]
    PlanExecutionStart {
        plan_id: String,
        enabled_count: usize,
        total_count: usize,
    },

    /// 步骤输出（执行中的步骤产生的输出）
    #[serde(rename = "step_output")]
    StepOutput {
        plan_id: String,
        step_id: String,
        output: String,
    },

    /// 计划执行完成
    #[serde(rename = "plan_execution_complete")]
    PlanExecutionComplete {
        plan_id: String,
        success: bool,
        executed_count: usize,
        skipped_count: usize,
        total_time: f64,
    },

    // ===== v1.40.0 新增：会话管理响应消息 =====
    /// 会话已保存
    #[serde(rename = "session_saved")]
    SessionSaved {
        session_id: String,
        name: String,
    },

    /// 会话已加载
    #[serde(rename = "session_loaded")]
    SessionLoaded {
        session: crate::web::session_manager::SerializableSession,
    },

    /// 会话列表
    #[serde(rename = "session_list")]
    SessionList {
        sessions: Vec<crate::web::session_manager::SessionListItem>,
    },

    /// 会话已删除
    #[serde(rename = "session_deleted")]
    SessionDeleted { session_id: String },

    /// 会话已重命名
    #[serde(rename = "session_renamed")]
    SessionRenamed {
        session_id: String,
        new_name: String,
    },

    /// 会话已导出
    #[serde(rename = "session_exported")]
    SessionExported {
        session_id: String,
        export_path: String,
        format: String,
        content: String,  // 文件内容，供前端下载
    },

    /// 会话操作错误
    #[serde(rename = "session_error")]
    SessionError { message: String },

    // ===== v1.44.0 新增：可视化消息 =====
    /// 图表数据（用于数据可视化）
    #[serde(rename = "chart")]
    Chart {
        /// 关联的回合 ID
        round_id: String,
        /// 图表数据
        chart_data: visualization::ChartData,
    },

    // ===== v1.52.0 新增：图像显示消息 =====
    /// 图像数据（用于远程运维场景）
    #[serde(rename = "image")]
    Image {
        /// 关联的回合 ID
        round_id: String,
        /// 图像数据
        image_data: visualization::ImageData,
    },

    // ===== v1.46.0 新增：文件上传响应 =====
    /// 文件上传成功（返回文件 ID 和预览数据）
    #[serde(rename = "file_uploaded")]
    FileUploaded {
        /// 文件 ID（用于后续图表命令）
        file_id: String,
        /// 文件名
        filename: String,
        /// 数据预览
        preview: FilePreview,
    },

    // ===== v1.36.0 占卜消息（v1.36.2 已废弃，保留用于向后兼容） =====
    // 以下消息类型已被 situation_analysis 字段替代，暂时注释掉
    /*
    /// 占卜开始（起卦）
    #[serde(rename = "divination_start")]
    DivinationStart { plan_id: String },

    /// 演算步骤（实时动画数据）
    #[serde(rename = "divination_step")]
    DivinationStep {
        plan_id: String,
        step: crate::agent::divination::YarrowStep,
    },

    /// 卦象生成
    #[serde(rename = "divination_hexagram")]
    DivinationHexagram {
        plan_id: String,
        hexagram: crate::agent::divination::Hexagram,
    },

    /// 占卜完成
    #[serde(rename = "divination_complete")]
    DivinationComplete {
        plan_id: String,
        result: crate::agent::divination::DivinationResult,
    },
    */
}

/// 上下文统计信息（v1.55.0）
#[derive(Debug, Clone, Serialize)]
pub struct ContextStats {
    /// 对话回合数量
    pub round_count: usize,
    /// 估算的 Token 数量
    pub estimated_tokens: usize,
}

/// Web 终端会话
pub struct Session {
    /// 会话 ID
    pub id: SessionId,
    /// Agent 实例（独立）
    pub agent: Arc<RwLock<Agent>>,
    /// Intent 路由器（v1.31.0 新增 - 快速识别简单意图）
    pub intent_router: crate::agent::decomposition::IntentRouter,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// LLM 初始化错误信息（用于诊断）
    pub llm_init_error: Option<String>,
    /// 对话 ID（用于多轮对话上下文）
    pub conversation_id: String,
    /// 对话回合列表（v1.28.0 新增）
    pub rounds: Arc<RwLock<Vec<ConversationRound>>>,
    /// 上传文件管理器（v1.46.0 新增）
    pub uploaded_files: crate::web::uploaded_files::UploadedFiles,
    /// 图表历史记录（v1.51.0 新增）
    pub chart_history: Arc<RwLock<Vec<ChartHistoryEntry>>>,
    /// 图像历史记录（v1.52.0 新增）
    pub image_history: Arc<RwLock<Vec<ImageHistoryEntry>>>,
    /// Memory 2.0 智能上下文编排器（v1.54.0 新增）
    pub memory_orchestrator: Option<Arc<SmartWebUIOrchestrator>>,
    /// Notebook 会话（v2.1.0 新增）
    pub notebook_session: Arc<NotebookSession>,
}

impl Session {
    /// 创建新会话（异步，会配置 LLM）
    pub async fn new(config: Config, registry: CommandRegistry) -> Self {
        let id = Uuid::new_v4().to_string();

        // ✨ 启用工具调用（Web 版本核心能力）
        let mut web_config = config.clone();
        web_config.features.tool_calling_enabled = Some(true);

        let mut agent = Agent::new(web_config.clone(), registry);

        // 配置 LLM（参考 main.rs），记录初始化错误
        let llm_init_error = Self::configure_llm(&mut agent, &web_config).await;

        // ✨ v1.29.0: 配置意图拆解器
        agent.configure_intent_decomposer();

        // ✨ 为每个 Web 会话创建独立的对话 ID
        let conversation_id = format!("web-{}", Uuid::new_v4());

        let chart_history = Arc::new(RwLock::new(Vec::new()));
        let image_history = Arc::new(RwLock::new(Vec::new()));

        // ✨ v1.54.0: 初始化 Memory 2.0 智能上下文编排器
        let memory_orchestrator = SmartWebUIOrchestrator::new(
            Arc::clone(&chart_history),
            Arc::clone(&image_history),
        )
        .ok()
        .map(Arc::new);

        if memory_orchestrator.is_some() {
            eprintln!("[Session] Memory 2.0 智能上下文编排器已初始化");
        } else {
            eprintln!("[Session] Memory 2.0 初始化失败，将以降级模式运行");
        }

        // v2.1.0: 初始化 Notebook 会话
        let notebook_base_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".realconsole");
        let notebook_session = Arc::new(NotebookSession::with_file_storage(notebook_base_dir));

        // v2.3.0: 配置 Notebook LLM（创建独立的 LLM 客户端用于 Natural Cell 执行）
        if let Some(ref primary_cfg) = web_config.llm.primary {
            if let Some(client) = Self::create_notebook_llm_client(primary_cfg) {
                notebook_session.configure_llm(client).await;
                eprintln!("[Session] Notebook LLM 已配置: {}", primary_cfg.provider);
            }
        }

        Self {
            id,
            agent: Arc::new(RwLock::new(agent)),
            intent_router: crate::agent::decomposition::IntentRouter::new(), // v1.31.0
            created_at: Utc::now(),
            llm_init_error,
            conversation_id,
            rounds: Arc::new(RwLock::new(Vec::new())),
            uploaded_files: crate::web::uploaded_files::UploadedFiles::new(), // v1.46.0
            chart_history: Arc::clone(&chart_history), // v1.51.0
            image_history: Arc::clone(&image_history), // v1.52.0
            memory_orchestrator, // v1.54.0
            notebook_session, // v2.1.0
        }
    }

    // ===== v1.28.0 新增：回合管理方法 =====

    /// 创建新回合
    pub async fn create_round(&self, round_type: RoundType, user_input: String, model: String) -> ConversationRound {
        let mut rounds = self.rounds.write().await;
        let index = rounds.len() + 1;
        let round = ConversationRound::new(index, round_type, user_input, model);
        rounds.push(round.clone());
        round
    }

    /// 获取当前回合（最后一个）
    pub async fn current_round(&self) -> Option<ConversationRound> {
        let rounds = self.rounds.read().await;
        rounds.last().cloned()
    }

    /// 更新回合状态
    pub async fn update_round_status(&self, round_id: &str, status: RoundStatus) -> bool {
        let mut rounds = self.rounds.write().await;
        if let Some(round) = rounds.iter_mut().find(|r| r.id == round_id) {
            round.status = status;
            true
        } else {
            false
        }
    }

    /// 完成回合（成功）
    pub async fn complete_round(
        &self,
        round_id: &str,
        response: String,
        execution_time: f64,
        tools_used: Vec<String>,
    ) -> Option<ConversationRound> {
        let mut rounds = self.rounds.write().await;
        if let Some(round) = rounds.iter_mut().find(|r| r.id == round_id) {
            round.complete(response, execution_time, tools_used);
            Some(round.clone())
        } else {
            None
        }
    }

    /// 标记回合失败
    pub async fn fail_round(&self, round_id: &str, error_message: String) -> Option<ConversationRound> {
        let mut rounds = self.rounds.write().await;
        if let Some(round) = rounds.iter_mut().find(|r| r.id == round_id) {
            round.fail(error_message);
            Some(round.clone())
        } else {
            None
        }
    }

    /// 获取所有回合
    pub async fn get_rounds(&self) -> Vec<ConversationRound> {
        let rounds = self.rounds.read().await;
        rounds.clone()
    }

    /// 获取回合数量
    pub async fn round_count(&self) -> usize {
        let rounds = self.rounds.read().await;
        rounds.len()
    }

    // ===== v1.51.0 新增：图表历史管理方法 =====

    /// 添加图表到历史记录
    pub async fn add_chart_to_history(&self, chart_data: visualization::ChartData, round_id: Option<String>, command: String) -> ChartHistoryEntry {
        let entry = ChartHistoryEntry::new(chart_data, round_id, command);
        let mut history = self.chart_history.write().await;
        history.push(entry.clone());
        entry
    }

    /// 获取所有图表历史记录
    pub async fn get_chart_history(&self) -> Vec<ChartHistoryEntry> {
        let history = self.chart_history.read().await;
        history.clone()
    }

    /// 根据 ID 获取图表历史记录
    pub async fn get_chart_by_id(&self, chart_id: &str) -> Option<ChartHistoryEntry> {
        let history = self.chart_history.read().await;
        history.iter().find(|entry| entry.id == chart_id).cloned()
    }

    /// 获取最近 N 个图表历史记录
    pub async fn get_recent_charts(&self, limit: usize) -> Vec<ChartHistoryEntry> {
        let history = self.chart_history.read().await;
        history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// 获取图表历史记录数量
    pub async fn chart_history_count(&self) -> usize {
        let history = self.chart_history.read().await;
        history.len()
    }

    // ===== v1.52.0 新增：图像历史管理方法 =====

    /// 添加图像到历史记录
    pub async fn add_image_to_history(
        &self,
        image_data: visualization::ImageData,
        round_id: Option<String>,
        command: String,
    ) -> ImageHistoryEntry {
        let entry = ImageHistoryEntry::new(image_data, round_id, command);
        let mut history = self.image_history.write().await;
        history.push(entry.clone());
        entry
    }

    /// 获取所有图像历史记录
    pub async fn get_image_history(&self) -> Vec<ImageHistoryEntry> {
        let history = self.image_history.read().await;
        history.clone()
    }

    /// 根据 ID 获取图像历史记录
    pub async fn get_image_by_id(&self, image_id: &str) -> Option<ImageHistoryEntry> {
        let history = self.image_history.read().await;
        history.iter().find(|entry| entry.id == image_id).cloned()
    }

    /// 获取最近 N 个图像历史记录
    pub async fn get_recent_images(&self, limit: usize) -> Vec<ImageHistoryEntry> {
        let history = self.image_history.read().await;
        history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// 获取图像历史记录数量
    pub async fn image_history_count(&self) -> usize {
        let history = self.image_history.read().await;
        history.len()
    }

    // ===== v1.55.0 新增：Memory 上下文管理 =====

    /// 清空对话上下文（强制重置）
    ///
    /// 清空所有对话回合，释放内存，让 LLM 从头开始
    pub async fn clear_context(&self) -> usize {
        let mut rounds = self.rounds.write().await;
        let count = rounds.len();
        rounds.clear();

        eprintln!("[Session] 清空了 {} 个对话回合", count);

        count
    }

    /// 获取当前上下文统计
    pub async fn get_context_stats(&self) -> ContextStats {
        let rounds = self.rounds.read().await;
        let round_count = rounds.len();

        // 计算 token 数量（简化估算）
        let estimated_tokens: usize = rounds.iter()
            .map(|round| {
                // 简化估算：user + ai 长度 / 4（平均每 4 字符 1 token）
                (round.user_input.len() + round.ai_response.len()) / 4
            })
            .sum();

        ContextStats {
            round_count,
            estimated_tokens,
        }
    }

    /// 配置 Agent 的 LLM
    ///
    /// 返回初始化错误信息（如果有）
    async fn configure_llm(agent: &mut Agent, config: &Config) -> Option<String> {
        let mut manager = agent.llm_manager.write().await;
        let mut error_messages = Vec::new();

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
                    let error_msg = format!("{}: {}", i18n::t("web.session.primary_llm_init_failed"), e);
                    eprintln!("{}", error_msg);
                    error_messages.push(error_msg);
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
                    let error_msg = format!("{}: {}", i18n::t("web.session.fallback_llm_init_failed"), e);
                    eprintln!("{}", error_msg);
                    error_messages.push(error_msg);
                }
            }
        }

        // 返回合并的错误信息
        if error_messages.is_empty() {
            None
        } else {
            Some(error_messages.join("\n"))
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
                    .map_err(|e| format!("{}: {}", i18n::t("web.session.ollama_client_creation_failed"), e))
            }
            "deepseek" => {
                let api_key = provider_config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| i18n::t("web.session.deepseek_requires_api_key"))?;
                let model = provider_config.model.as_deref().unwrap_or("deepseek-chat");
                let endpoint = provider_config
                    .endpoint
                    .as_deref()
                    .unwrap_or("https://api.deepseek.com/v1");

                DeepseekClient::new(api_key, model, endpoint)
                    .map(|client| Arc::new(client) as Arc<dyn LlmClient>)
                    .map_err(|e| format!("{}: {}", i18n::t("web.session.deepseek_client_creation_failed"), e))
            }
            other => Err(format!("{}: {}", i18n::t("web.session.unknown_llm_provider"), other)),
        }
    }

    /// 创建 Notebook 专用的 LLM 客户端（v2.3.0）
    ///
    /// 返回 Box<dyn LlmClient> 供 NotebookSession 使用
    fn create_notebook_llm_client(
        provider_config: &crate::config::LlmProvider,
    ) -> Option<Box<dyn LlmClient>> {
        match provider_config.provider.as_str() {
            "ollama" => {
                let model = provider_config.model.as_deref().unwrap_or("qwen2.5:latest");
                let endpoint = provider_config
                    .endpoint
                    .as_deref()
                    .unwrap_or("http://localhost:11434");

                OllamaClient::new(model, endpoint)
                    .map(|client| Box::new(client) as Box<dyn LlmClient>)
                    .ok()
            }
            "deepseek" => {
                let api_key = provider_config.api_key.as_ref()?;
                let model = provider_config.model.as_deref().unwrap_or("deepseek-chat");
                let endpoint = provider_config
                    .endpoint
                    .as_deref()
                    .unwrap_or("https://api.deepseek.com/v1");

                DeepseekClient::new(api_key, model, endpoint)
                    .map(|client| Box::new(client) as Box<dyn LlmClient>)
                    .ok()
            }
            _ => None,
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

    // ===== v1.40.0 新增：会话持久化方法 =====

    /// 转换为可序列化的会话数据
    pub async fn to_serializable(&self) -> crate::web::session_manager::SerializableSession {
        use crate::web::session_manager::{SerializableSession, SessionMetadata};

        let rounds = self.get_rounds().await;
        let name = Self::generate_session_name(&rounds);

        // 使用最后一个回合的时间作为更新时间，如果没有回合则使用创建时间
        let updated_at = rounds.last().map(|r| r.timestamp).unwrap_or(self.created_at);

        // 计算元数据
        let metadata = Some(SessionMetadata::from_rounds(&rounds));

        // v1.51.0: 获取图表历史
        let chart_history = self.chart_history.read().await.clone();
        // v1.52.0: 获取图像历史
        let image_history = self.image_history.read().await.clone();

        SerializableSession {
            id: self.id.clone(),
            name,
            created_at: self.created_at,
            updated_at,
            conversation_id: self.conversation_id.clone(),
            rounds,
            chart_history,
            image_history,
            metadata,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// 从可序列化会话数据恢复会话（异步构造器）
    pub async fn from_serializable(
        serializable: crate::web::session_manager::SerializableSession,
        config: Config,
        registry: CommandRegistry,
    ) -> Self {
        // 创建基础会话
        let mut session = Self::new(config, registry).await;

        // 恢复会话数据
        session.id = serializable.id;
        session.created_at = serializable.created_at;
        session.conversation_id = serializable.conversation_id;

        // 恢复回合历史（在独立作用域中，确保锁被释放）
        {
            let mut rounds = session.rounds.write().await;
            *rounds = serializable.rounds;
        }

        // v1.51.0: 恢复图表历史（在独立作用域中，确保锁被释放）
        {
            let mut chart_history = session.chart_history.write().await;
            *chart_history = serializable.chart_history;
        }

        // v1.52.0: 恢复图像历史（在独立作用域中，确保锁被释放）
        {
            let mut image_history = session.image_history.write().await;
            *image_history = serializable.image_history;
        }

        session
    }

    /// 生成会话名称（基于第一个回合的用户输入）
    fn generate_session_name(rounds: &[ConversationRound]) -> String {
        if let Some(first_round) = rounds.first() {
            let input = &first_round.user_input;
            // 截取前 30 个字符作为会话名称（使用字符边界安全的方式）
            if input.chars().count() > 30 {
                let truncated: String = input.chars().take(30).collect();
                format!("{}...", truncated)
            } else {
                input.to_string()
            }
        } else {
            // 如果没有回合，使用默认名称
            format!("新会话 {}", chrono::Utc::now().format("%Y-%m-%d %H:%M"))
        }
    }
}
