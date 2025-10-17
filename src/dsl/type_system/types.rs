//! 类型系统 - 类型定义
//!
//! 实现 SimpleConsole DSL 的类型系统，包括：
//! - 原生类型（String, Integer, Boolean 等）
//! - 复合类型（List, Dict, Optional, Result）
//! - 领域类型（FilePath, FileList, CommandLine 等）
//! - 类型约束（Range, Pattern, Length）

use serde::{Deserialize, Serialize};
use std::fmt;

/// 原生类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    /// 字符串类型
    String,
    /// 整数类型
    Integer,
    /// 浮点数类型
    Float,
    /// 布尔类型
    Boolean,
    /// 日期类型
    Date,
    /// 空类型
    Unit,
}

/// 复合类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompositeType {
    /// 列表类型 List<T>
    List(Box<Type>),
    /// 字典类型 Dict<K, V>
    Dict(Box<Type>, Box<Type>),
    /// 可选类型 Optional<T>
    Optional(Box<Type>),
    /// 结果类型 Result<T, E>
    Result(Box<Type>, Box<Type>),
    /// 元组类型 (T1, T2, ...)
    Tuple(Vec<Type>),
}

/// 领域特定类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainType {
    /// 文件路径
    FilePath,
    /// 文件列表
    FileList,
    /// 命令行
    CommandLine,
    /// 管道数据
    PipelineData,
    /// 意图结果
    IntentResult,
    /// 规划结果
    PlanResult,
    /// Shell 输出
    ShellOutput,
}

/// 统一类型表示
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// 原生类型
    Primitive(PrimitiveType),
    /// 复合类型
    Composite(CompositeType),
    /// 领域类型
    Domain(DomainType),
    /// 泛型类型变量 (用于类型推导)
    TypeVar(String),
    /// 任意类型 (用于动态类型)
    Any,
}

/// 类型约束
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    /// 范围约束 (min, max)
    Range(ConstraintValue, ConstraintValue),
    /// 正则模式约束
    Pattern(String),
    /// 长度约束 (min, max)
    Length(usize, Option<usize>),
    /// 枚举约束（值必须在给定集合中）
    Enum(Vec<String>),
    /// 非空约束
    NonEmpty,
    /// 自定义约束（通过验证器名称引用）
    Custom(String),
}

/// 约束值（支持整数和浮点数）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintValue {
    Int(i64),
    Float(f64),
    Unbounded,
}

/// 带约束的类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstrainedType {
    /// 基础类型
    pub base_type: Type,
    /// 约束列表
    pub constraints: Vec<Constraint>,
}

impl Type {
    /// 创建字符串类型
    pub fn string() -> Self {
        Type::Primitive(PrimitiveType::String)
    }

    /// 创建整数类型
    pub fn integer() -> Self {
        Type::Primitive(PrimitiveType::Integer)
    }

    /// 创建浮点数类型
    pub fn float() -> Self {
        Type::Primitive(PrimitiveType::Float)
    }

    /// 创建布尔类型
    pub fn boolean() -> Self {
        Type::Primitive(PrimitiveType::Boolean)
    }

    /// 创建单元类型
    pub fn unit() -> Self {
        Type::Primitive(PrimitiveType::Unit)
    }

    /// 创建列表类型
    pub fn list(element_type: Type) -> Self {
        Type::Composite(CompositeType::List(Box::new(element_type)))
    }

    /// 创建可选类型
    pub fn optional(inner_type: Type) -> Self {
        Type::Composite(CompositeType::Optional(Box::new(inner_type)))
    }

    /// 创建结果类型
    pub fn result(ok_type: Type, err_type: Type) -> Self {
        Type::Composite(CompositeType::Result(
            Box::new(ok_type),
            Box::new(err_type),
        ))
    }

    /// 创建文件路径类型
    pub fn file_path() -> Self {
        Type::Domain(DomainType::FilePath)
    }

    /// 创建文件列表类型
    pub fn file_list() -> Self {
        Type::Domain(DomainType::FileList)
    }

    /// 创建命令行类型
    pub fn command_line() -> Self {
        Type::Domain(DomainType::CommandLine)
    }

    /// 创建管道数据类型
    pub fn pipeline_data() -> Self {
        Type::Domain(DomainType::PipelineData)
    }

    /// 判断是否为原生类型
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Primitive(_))
    }

    /// 判断是否为复合类型
    pub fn is_composite(&self) -> bool {
        matches!(self, Type::Composite(_))
    }

    /// 判断是否为领域类型
    pub fn is_domain(&self) -> bool {
        matches!(self, Type::Domain(_))
    }

    /// 判断是否为类型变量
    pub fn is_type_var(&self) -> bool {
        matches!(self, Type::TypeVar(_))
    }

    /// 判断是否为 Any 类型
    pub fn is_any(&self) -> bool {
        matches!(self, Type::Any)
    }

    /// 获取内层类型（对于 Optional, List 等）
    pub fn inner_type(&self) -> Option<&Type> {
        match self {
            Type::Composite(CompositeType::List(t)) => Some(t),
            Type::Composite(CompositeType::Optional(t)) => Some(t),
            _ => None,
        }
    }
}

impl ConstrainedType {
    /// 创建新的约束类型
    pub fn new(base_type: Type) -> Self {
        Self {
            base_type,
            constraints: Vec::new(),
        }
    }

    /// 添加约束
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// 添加多个约束
    pub fn with_constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints.extend(constraints);
        self
    }

    /// 判断是否有约束
    pub fn has_constraints(&self) -> bool {
        !self.constraints.is_empty()
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(p) => write!(f, "{}", p),
            Type::Composite(c) => write!(f, "{}", c),
            Type::Domain(d) => write!(f, "{}", d),
            Type::TypeVar(name) => write!(f, "${}", name),
            Type::Any => write!(f, "Any"),
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveType::String => write!(f, "String"),
            PrimitiveType::Integer => write!(f, "Integer"),
            PrimitiveType::Float => write!(f, "Float"),
            PrimitiveType::Boolean => write!(f, "Boolean"),
            PrimitiveType::Date => write!(f, "Date"),
            PrimitiveType::Unit => write!(f, "()"),
        }
    }
}

impl fmt::Display for CompositeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompositeType::List(t) => write!(f, "List<{}>", t),
            CompositeType::Dict(k, v) => write!(f, "Dict<{}, {}>", k, v),
            CompositeType::Optional(t) => write!(f, "Optional<{}>", t),
            CompositeType::Result(ok, err) => write!(f, "Result<{}, {}>", ok, err),
            CompositeType::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for DomainType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainType::FilePath => write!(f, "FilePath"),
            DomainType::FileList => write!(f, "FileList"),
            DomainType::CommandLine => write!(f, "CommandLine"),
            DomainType::PipelineData => write!(f, "PipelineData"),
            DomainType::IntentResult => write!(f, "IntentResult"),
            DomainType::PlanResult => write!(f, "PlanResult"),
            DomainType::ShellOutput => write!(f, "ShellOutput"),
        }
    }
}

impl fmt::Display for ConstrainedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base_type)?;
        if !self.constraints.is_empty() {
            write!(f, " where ")?;
            for (i, constraint) in self.constraints.iter().enumerate() {
                if i > 0 {
                    write!(f, " and ")?;
                }
                write!(f, "{:?}", constraint)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        assert_eq!(Type::string(), Type::Primitive(PrimitiveType::String));
        assert_eq!(Type::integer(), Type::Primitive(PrimitiveType::Integer));
        assert_eq!(Type::boolean(), Type::Primitive(PrimitiveType::Boolean));
    }

    #[test]
    fn test_composite_types() {
        let list_type = Type::list(Type::string());
        assert!(list_type.is_composite());
        assert_eq!(list_type.inner_type(), Some(&Type::string()));

        let optional_type = Type::optional(Type::integer());
        assert!(optional_type.is_composite());
        assert_eq!(optional_type.inner_type(), Some(&Type::integer()));
    }

    #[test]
    fn test_domain_types() {
        let file_path = Type::file_path();
        assert!(file_path.is_domain());

        let file_list = Type::file_list();
        assert!(file_list.is_domain());
    }

    #[test]
    fn test_constrained_type() {
        let constrained = ConstrainedType::new(Type::string())
            .with_constraint(Constraint::Length(1, Some(100)))
            .with_constraint(Constraint::NonEmpty);

        assert!(constrained.has_constraints());
        assert_eq!(constrained.constraints.len(), 2);
    }

    #[test]
    fn test_type_display() {
        assert_eq!(Type::string().to_string(), "String");
        assert_eq!(Type::list(Type::integer()).to_string(), "List<Integer>");
        assert_eq!(
            Type::optional(Type::boolean()).to_string(),
            "Optional<Boolean>"
        );
        assert_eq!(Type::file_list().to_string(), "FileList");
    }

    #[test]
    fn test_type_equality() {
        assert_eq!(Type::string(), Type::string());
        assert_ne!(Type::string(), Type::integer());
        assert_eq!(
            Type::list(Type::string()),
            Type::list(Type::string())
        );
        assert_ne!(
            Type::list(Type::string()),
            Type::list(Type::integer())
        );
    }
}
