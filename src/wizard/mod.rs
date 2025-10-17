//! 配置向导模块
//!
//! 提供交互式配置向导，帮助用户快速完成初始配置。
//!
//! 特性：
//! - 交互式问答流程
//! - API Key 验证
//! - 配置文件自动生成
//! - 安全的敏感信息处理

mod generator;
mod validator;
mod wizard;

pub use wizard::{ConfigWizard, WizardMode};

// WizardConfig 仅用于内部，不导出
#[allow(unused_imports)]
pub(crate) use wizard::WizardConfig;
