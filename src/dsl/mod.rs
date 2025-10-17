//! DSL 模块
//!
//! SimpleConsole 领域特定语言系统，包括：
//! - 类型系统 (type_system): 类型定义、类型检查、类型推导 ✅
//! - 意图 DSL (intent): 意图识别与分析 ✅ Phase 3-6
//! - 管道 DSL (pipeline): 组合式命令执行计划 ✅ Phase 6.3 原型
//! - 工具 DSL (tool): 工具定义与安全策略 - 待实现

pub mod type_system;
pub mod intent;
pub mod pipeline;

// 重新导出常用类型（type_system 暂未使用，这些导出也暂不需要）
// pub use type_system::{
//     CompositeType, Constraint, ConstrainedType, ConstraintValue, DomainType, PrimitiveType, Type,
//     TypeChecker, TypeError, TypeInference,
// };
