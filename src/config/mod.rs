// Configuration module
//
// 提供配置管理、环境检测、智能推荐等功能

// 核心配置结构（原 config.rs）
pub mod settings;

// v1.16.0 新增：配置向导系统
pub mod detector;
pub mod recommender;
pub mod validator;
pub mod wizard;

// Re-export 核心配置类型（保持向后兼容）
pub use settings::*;

// Re-export 向导系统类型
pub use detector::{EnvironmentDetector, EnvironmentInfo, OsInfo, ShellInfo, UserProfile};
pub use recommender::{ConfigRecommender, LlmRecommendation, ModeRecommendation};
pub use validator::{ConfigValidator, ValidationResult};
pub use wizard::{ConfigWizard, WizardMode};
