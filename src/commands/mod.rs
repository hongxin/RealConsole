pub mod core;
pub mod git_cmd;      // ✨ Phase 6: Git 智能助手命令
pub mod history_cmd;  // ✨ Phase 8: 命令历史记录命令
pub mod llm;
pub mod log;
pub mod logfile_cmd;  // ✨ Phase 6: 日志文件分析命令
pub mod memory;
pub mod project_cmd;  // ✨ Phase 6: 项目上下文命令
pub mod stats_cmd;    // ✨ Phase 9: 统计与可视化命令
pub mod system_cmd;   // ✨ Phase 6: 系统监控命令
pub mod task_cmd;     // ✨ Phase 10: 任务分解与规划命令
pub mod tool;

pub use core::register_core_commands;
pub use git_cmd::register_git_commands;
pub use history_cmd::register_history_commands;
pub use llm::register_llm_commands;
pub use log::register_log_commands;
pub use logfile_cmd::register_log_analysis_commands;
pub use memory::register_memory_commands;
pub use project_cmd::register_project_commands;
pub use stats_cmd::register_stats_commands;
pub use system_cmd::register_system_commands;
pub use task_cmd::register_task_commands;
pub use tool::register_tool_commands;
