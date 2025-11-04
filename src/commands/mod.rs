pub mod context_cmd; // ✨ Phase 对话上下文: 上下文管理命令
pub mod core;
pub mod git_cmd; // ✨ Phase 6: Git 智能助手命令
pub mod history_cmd; // ✨ Phase 8: 命令历史记录命令
pub mod liangyyi_cmd; // ✨ v1.9.4: 两仪演化系统可视化命令
pub mod likan; // ✨ Phase 4.3: 离坎炼化炉命令
pub mod llm;
pub mod llm_log; // ✨ LLM 交互日志命令
pub mod llm_prompt_cmd; // ✨ v1.23.1: LLM 系统提示词命令
pub mod log;
pub mod logfile_cmd; // ✨ Phase 6: 日志文件分析命令
pub mod memory;
pub mod project_cmd; // ✨ Phase 6: 项目上下文命令
pub mod stats_cmd; // ✨ Phase 9: 统计与可视化命令
pub mod system_cmd; // ✨ Phase 6: 系统监控命令
pub mod task_cmd; // ✨ Phase 10: 任务分解与规划命令
pub mod tool;
pub mod trace; // ✨ Phase 2 (Memory Redesign): 统一追踪命令
pub mod unified_dashboard_cmd; // ✨ v1.15.0 Phase 4: 统一Dashboard命令
pub mod voice_cmd; // ✨ 语音播报命令

pub use context_cmd::register_context_commands;
pub use core::register_core_commands;
pub use git_cmd::register_git_commands;
pub use history_cmd::register_history_commands;
pub use liangyyi_cmd::register_liangyyi_commands;
pub use likan::register_likan_commands;
pub use llm::register_llm_commands;
pub use llm_log::register_llm_log_commands;
pub use llm_prompt_cmd::register_llm_prompt_commands;
pub use log::register_log_commands;
pub use logfile_cmd::register_log_analysis_commands;
pub use memory::register_memory_commands;
pub use project_cmd::register_project_commands;
pub use stats_cmd::register_stats_commands;
pub use system_cmd::register_system_commands;
pub use task_cmd::register_task_commands;
pub use tool::register_tool_commands;
pub use trace::register_trace_commands;
pub use unified_dashboard_cmd::register_unified_dashboard_command;
pub use voice_cmd::register_voice_commands;
