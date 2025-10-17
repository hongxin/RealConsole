//! 类型系统模块
//!
//! ⚠️ **状态**: 预留功能（暂未使用）
//!
//! 本模块为未来的 DSL 扩展预留，包含完整的类型系统实现：
//! - **类型定义** (types) - 基本类型、复合类型、领域类型
//! - **类型检查** (checker) - 类型赋值、约束验证
//! - **类型推导** (inference) - 类型统一、泛型实例化
//!
//! ## 计划用途
//!
//! - Pipeline DSL 的类型安全
//! - Tool DSL 的参数类型验证
//! - 复杂表达式的静态分析
//!
//! ## 重新评估时机
//!
//! - 实现 Pipeline DSL 时
//! - 扩展 Tool DSL 需要更强类型约束时
//! - 需要静态类型检查以提升安全性时
//!
//! **测试状态**: ✅ 23个测试全部通过，覆盖率 60-82%

#![allow(dead_code)]

pub mod checker;
pub mod inference;
pub mod types;

// 重新导出常用类型（暂未使用）
#[allow(unused_imports)]
pub use checker::{TypeError, TypeChecker};
#[allow(unused_imports)]
pub use inference::TypeInference;
#[allow(unused_imports)]
pub use types::{
    CompositeType, Constraint, ConstrainedType, ConstraintValue, DomainType, PrimitiveType, Type,
};
