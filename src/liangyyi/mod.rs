//! 两仪演化系统
//!
//! ## 哲学基础
//!
//! **先天八卦，竖看者也** - 时间维度的演化序列
//!
//! 太极生两仪，两仪生四象，四象生八卦。
//! 本模块实现"竖看"（时间维度）的状态演化系统，
//! 与"横看"（空间维度）的 Bagua Memory Palace 相辅相成，体用合一。
//!
//! ## 核心概念
//!
//! - **太极（Taiji）**: 系统的统一状态，阴阳能量的连续表示
//! - **两仪（Liangyyi）**: 阴阳二元状态（太阴☽、太阳☉）
//! - **四象（Sixiang）**: 四种状态类型（老阴、少阳、少阴、老阳）
//!
//! ## 体用关系
//!
//! ```text
//! 体（Liangyyi）      用（Bagua）
//!     ↓                  ↓
//!  时间演化           空间存储
//!     ↓                  ↓
//!  状态转换           数据记录
//!     ↓                  ↓
//!  "竖看"             "横看"
//! ```
//!
//! ## 使用示例
//!
//! ```rust
//! use realconsole::liangyyi::{Taiji, Liangyyi, Sixiang, Event};
//!
//! // 创建太极
//! let mut taiji = Taiji::new();
//!
//! // 更新状态
//! taiji.update_from_event(&Event::UserExecute);
//!
//! // 推导两仪
//! let liangyyi = Liangyyi::from_taiji(&taiji);
//!
//! // 推导四象
//! let sixiang = Sixiang::from_liangyyi_and_activity(liangyyi, 0.8);
//!
//! println!("{} {} {}", taiji.balance(), liangyyi.symbol(), sixiang.symbol());
//! ```

pub mod liangyyi;
pub mod sixiang;
pub mod taiji;
pub mod tracker; // ✨ Phase 2: 状态追踪器
pub mod state_vector; // ✨ v1.11.0: 多维状态空间
pub mod predictor; // ✨ v1.12.0: 状态预测
pub mod adaptive; // ✨ v1.13.0: 自适应系统

// Re-exports
pub use liangyyi::Liangyyi;
pub use sixiang::Sixiang;
pub use taiji::{EnergyType, Event, Taiji, TaijiContext};
pub use tracker::{
    LearningPhase, StateSnapshot, StateStats, StateTracker, StateTrackerConfig, StateTrend,
};
pub use state_vector::StateVector; // ✨ v1.11.0
pub use predictor::{DimensionTrend, StatePredictor, TrendDirection}; // ✨ v1.12.0
pub use adaptive::{AdaptiveStrategy, AdaptiveSystem, Recommendation, RecommendationAction, TargetState}; // ✨ v1.13.0
