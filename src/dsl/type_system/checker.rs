//! 类型系统 - 类型检查器
//!
//! 实现类型兼容性检查、约束验证和错误报告

use super::types::{
    CompositeType, Constraint, ConstrainedType, ConstraintValue, PrimitiveType, Type,
};
use std::collections::HashMap;

/// 类型错误
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    /// 类型不兼容
    IncompatibleTypes {
        expected: Type,
        actual: Type,
        context: String,
    },
    /// 约束违反
    ConstraintViolation {
        constraint: String,
        value: String,
        reason: String,
    },
    /// 未定义的类型变量
    UndefinedTypeVar { name: String },
    /// 内层类型不匹配（用于复合类型）
    InnerTypeMismatch {
        expected: Type,
        actual: Type,
        context: String,
    },
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::IncompatibleTypes {
                expected,
                actual,
                context,
            } => {
                write!(
                    f,
                    "类型不兼容 [{}]: 期望 {}, 实际 {}",
                    context, expected, actual
                )
            }
            TypeError::ConstraintViolation {
                constraint,
                value,
                reason,
            } => {
                write!(f, "约束违反 [{}]: {} - {}", constraint, value, reason)
            }
            TypeError::UndefinedTypeVar { name } => {
                write!(f, "未定义的类型变量: ${}", name)
            }
            TypeError::InnerTypeMismatch {
                expected,
                actual,
                context,
            } => {
                write!(
                    f,
                    "内层类型不匹配 [{}]: 期望 {}, 实际 {}",
                    context, expected, actual
                )
            }
        }
    }
}

impl std::error::Error for TypeError {}

/// 类型检查器
pub struct TypeChecker {
    /// 类型变量绑定 (用于类型推导)
    type_bindings: HashMap<String, Type>,
}

impl TypeChecker {
    /// 创建新的类型检查器
    pub fn new() -> Self {
        Self {
            type_bindings: HashMap::new(),
        }
    }

    /// 绑定类型变量
    pub fn bind_type_var(&mut self, name: String, ty: Type) {
        self.type_bindings.insert(name, ty);
    }

    /// 获取类型变量的绑定
    pub fn get_type_var(&self, name: &str) -> Option<&Type> {
        self.type_bindings.get(name)
    }

    /// 清除所有类型变量绑定
    pub fn clear_bindings(&mut self) {
        self.type_bindings.clear();
    }

    /// 检查两个类型是否兼容（actual 能否赋值给 expected）
    pub fn check_assignable(
        &self,
        expected: &Type,
        actual: &Type,
        context: &str,
    ) -> Result<(), TypeError> {
        // 处理 Any 类型
        if matches!(expected, Type::Any) || matches!(actual, Type::Any) {
            return Ok(());
        }

        // 处理类型变量
        if let Type::TypeVar(name) = expected {
            if let Some(bound_type) = self.get_type_var(name) {
                return self.check_assignable(bound_type, actual, context);
            }
            return Err(TypeError::UndefinedTypeVar {
                name: name.clone(),
            });
        }

        if let Type::TypeVar(name) = actual {
            if let Some(bound_type) = self.get_type_var(name) {
                return self.check_assignable(expected, bound_type, context);
            }
            return Err(TypeError::UndefinedTypeVar {
                name: name.clone(),
            });
        }

        match (expected, actual) {
            // 相同原生类型
            (Type::Primitive(p1), Type::Primitive(p2)) if p1 == p2 => Ok(()),

            // Integer 可以赋值给 Float（隐式转换）
            (Type::Primitive(PrimitiveType::Float), Type::Primitive(PrimitiveType::Integer)) => {
                Ok(())
            }

            // 相同领域类型
            (Type::Domain(d1), Type::Domain(d2)) if d1 == d2 => Ok(()),

            // 复合类型检查
            (Type::Composite(c1), Type::Composite(c2)) => {
                self.check_composite_assignable(c1, c2, context)
            }

            // 不兼容
            _ => Err(TypeError::IncompatibleTypes {
                expected: expected.clone(),
                actual: actual.clone(),
                context: context.to_string(),
            }),
        }
    }

    /// 检查复合类型的兼容性
    fn check_composite_assignable(
        &self,
        expected: &CompositeType,
        actual: &CompositeType,
        context: &str,
    ) -> Result<(), TypeError> {
        match (expected, actual) {
            // List<T>
            (CompositeType::List(t1), CompositeType::List(t2)) => {
                self.check_assignable(t1, t2, &format!("{}::List", context))
            }

            // Dict<K, V>
            (CompositeType::Dict(k1, v1), CompositeType::Dict(k2, v2)) => {
                self.check_assignable(k1, k2, &format!("{}::Dict::Key", context))?;
                self.check_assignable(v1, v2, &format!("{}::Dict::Value", context))?;
                Ok(())
            }

            // Optional<T>
            (CompositeType::Optional(t1), CompositeType::Optional(t2)) => {
                self.check_assignable(t1, t2, &format!("{}::Optional", context))
            }

            // Result<T, E>
            (CompositeType::Result(ok1, err1), CompositeType::Result(ok2, err2)) => {
                self.check_assignable(ok1, ok2, &format!("{}::Result::Ok", context))?;
                self.check_assignable(err1, err2, &format!("{}::Result::Err", context))?;
                Ok(())
            }

            // Tuple - 长度必须相同，每个元素类型必须兼容
            (CompositeType::Tuple(types1), CompositeType::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return Err(TypeError::IncompatibleTypes {
                        expected: Type::Composite(expected.clone()),
                        actual: Type::Composite(actual.clone()),
                        context: format!("{}::Tuple (长度不同)", context),
                    });
                }

                for (i, (t1, t2)) in types1.iter().zip(types2.iter()).enumerate() {
                    self.check_assignable(t1, t2, &format!("{}::Tuple[{}]", context, i))?;
                }
                Ok(())
            }

            // 不同的复合类型
            _ => Err(TypeError::IncompatibleTypes {
                expected: Type::Composite(expected.clone()),
                actual: Type::Composite(actual.clone()),
                context: context.to_string(),
            }),
        }
    }

    /// 检查带约束的类型
    pub fn check_constrained(
        &self,
        constrained: &ConstrainedType,
        value_type: &Type,
        context: &str,
    ) -> Result<(), TypeError> {
        // 首先检查基础类型兼容性
        self.check_assignable(&constrained.base_type, value_type, context)?;

        // 约束验证需要运行时值，这里只做类型级别的验证
        // 实际约束检查将在运行时执行
        Ok(())
    }

    /// 验证约束（需要运行时值）
    pub fn validate_constraint(
        &self,
        constraint: &Constraint,
        value: &dyn ConstraintValidator,
    ) -> Result<(), TypeError> {
        match constraint {
            Constraint::Range(min, max) => {
                value.validate_range(min, max)?;
            }
            Constraint::Pattern(pattern) => {
                value.validate_pattern(pattern)?;
            }
            Constraint::Length(min, max) => {
                value.validate_length(*min, max)?;
            }
            Constraint::Enum(allowed_values) => {
                value.validate_enum(allowed_values)?;
            }
            Constraint::NonEmpty => {
                value.validate_non_empty()?;
            }
            Constraint::Custom(name) => {
                value.validate_custom(name)?;
            }
        }
        Ok(())
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// 约束验证器接口（由具体值类型实现）
pub trait ConstraintValidator {
    fn validate_range(
        &self,
        _min: &ConstraintValue,
        _max: &ConstraintValue,
    ) -> Result<(), TypeError> {
        Err(TypeError::ConstraintViolation {
            constraint: "Range".to_string(),
            value: "<unknown>".to_string(),
            reason: "此类型不支持范围约束".to_string(),
        })
    }

    fn validate_pattern(&self, _pattern: &str) -> Result<(), TypeError> {
        Err(TypeError::ConstraintViolation {
            constraint: "Pattern".to_string(),
            value: "<unknown>".to_string(),
            reason: "此类型不支持模式约束".to_string(),
        })
    }

    fn validate_length(&self, _min: usize, _max: &Option<usize>) -> Result<(), TypeError> {
        Err(TypeError::ConstraintViolation {
            constraint: "Length".to_string(),
            value: "<unknown>".to_string(),
            reason: "此类型不支持长度约束".to_string(),
        })
    }

    fn validate_enum(&self, _allowed: &[String]) -> Result<(), TypeError> {
        Err(TypeError::ConstraintViolation {
            constraint: "Enum".to_string(),
            value: "<unknown>".to_string(),
            reason: "此类型不支持枚举约束".to_string(),
        })
    }

    fn validate_non_empty(&self) -> Result<(), TypeError> {
        Err(TypeError::ConstraintViolation {
            constraint: "NonEmpty".to_string(),
            value: "<unknown>".to_string(),
            reason: "此类型不支持非空约束".to_string(),
        })
    }

    fn validate_custom(&self, _name: &str) -> Result<(), TypeError> {
        Err(TypeError::ConstraintViolation {
            constraint: format!("Custom({})", _name),
            value: "<unknown>".to_string(),
            reason: "自定义约束验证器未实现".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_type_checking() {
        let checker = TypeChecker::new();

        // 相同类型兼容
        assert!(checker
            .check_assignable(&Type::string(), &Type::string(), "test")
            .is_ok());

        assert!(checker
            .check_assignable(&Type::integer(), &Type::integer(), "test")
            .is_ok());

        // 不同类型不兼容
        assert!(checker
            .check_assignable(&Type::string(), &Type::integer(), "test")
            .is_err());
    }

    #[test]
    fn test_implicit_conversion() {
        let checker = TypeChecker::new();

        // Integer -> Float 允许
        assert!(checker
            .check_assignable(&Type::float(), &Type::integer(), "test")
            .is_ok());

        // Float -> Integer 不允许
        assert!(checker
            .check_assignable(&Type::integer(), &Type::float(), "test")
            .is_err());
    }

    #[test]
    fn test_any_type() {
        let checker = TypeChecker::new();

        // Any 可以接受任何类型
        assert!(checker
            .check_assignable(&Type::Any, &Type::string(), "test")
            .is_ok());

        assert!(checker
            .check_assignable(&Type::Any, &Type::integer(), "test")
            .is_ok());

        // 任何类型可以赋值给 Any
        assert!(checker
            .check_assignable(&Type::string(), &Type::Any, "test")
            .is_ok());
    }

    #[test]
    fn test_list_type_checking() {
        let checker = TypeChecker::new();

        let list_string = Type::list(Type::string());
        let list_integer = Type::list(Type::integer());
        let list_any = Type::list(Type::Any);

        // 相同元素类型的列表兼容
        assert!(checker
            .check_assignable(&list_string, &list_string, "test")
            .is_ok());

        // 不同元素类型的列表不兼容
        assert!(checker
            .check_assignable(&list_string, &list_integer, "test")
            .is_err());

        // List<Any> 可以接受任何列表
        assert!(checker
            .check_assignable(&list_any, &list_string, "test")
            .is_ok());
    }

    #[test]
    fn test_optional_type_checking() {
        let checker = TypeChecker::new();

        let opt_string = Type::optional(Type::string());
        let opt_integer = Type::optional(Type::integer());

        assert!(checker
            .check_assignable(&opt_string, &opt_string, "test")
            .is_ok());

        assert!(checker
            .check_assignable(&opt_string, &opt_integer, "test")
            .is_err());
    }

    #[test]
    fn test_result_type_checking() {
        let checker = TypeChecker::new();

        let result1 = Type::result(Type::string(), Type::string());
        let result2 = Type::result(Type::string(), Type::string());
        let result3 = Type::result(Type::integer(), Type::string());

        assert!(checker
            .check_assignable(&result1, &result2, "test")
            .is_ok());

        assert!(checker
            .check_assignable(&result1, &result3, "test")
            .is_err());
    }

    #[test]
    fn test_tuple_type_checking() {
        let checker = TypeChecker::new();

        let tuple1 = Type::Composite(CompositeType::Tuple(vec![Type::string(), Type::integer()]));
        let tuple2 = Type::Composite(CompositeType::Tuple(vec![Type::string(), Type::integer()]));
        let tuple3 = Type::Composite(CompositeType::Tuple(vec![Type::integer(), Type::string()]));
        let tuple4 = Type::Composite(CompositeType::Tuple(vec![Type::string()]));

        // 相同结构的元组兼容
        assert!(checker.check_assignable(&tuple1, &tuple2, "test").is_ok());

        // 元素类型不同不兼容
        assert!(checker.check_assignable(&tuple1, &tuple3, "test").is_err());

        // 长度不同不兼容
        assert!(checker.check_assignable(&tuple1, &tuple4, "test").is_err());
    }

    #[test]
    fn test_domain_type_checking() {
        let checker = TypeChecker::new();

        // 相同领域类型兼容
        assert!(checker
            .check_assignable(&Type::file_path(), &Type::file_path(), "test")
            .is_ok());

        // 不同领域类型不兼容
        assert!(checker
            .check_assignable(&Type::file_path(), &Type::file_list(), "test")
            .is_err());
    }

    #[test]
    fn test_type_variable_binding() {
        let mut checker = TypeChecker::new();

        // 绑定类型变量
        checker.bind_type_var("T".to_string(), Type::string());

        // 使用类型变量
        let type_var = Type::TypeVar("T".to_string());
        assert!(checker
            .check_assignable(&type_var, &Type::string(), "test")
            .is_ok());

        // 未绑定的类型变量应该报错
        let unbound_var = Type::TypeVar("U".to_string());
        assert!(checker
            .check_assignable(&unbound_var, &Type::string(), "test")
            .is_err());
    }

    #[test]
    fn test_constrained_type() {
        let checker = TypeChecker::new();

        let constrained = ConstrainedType::new(Type::string())
            .with_constraint(Constraint::Length(1, Some(100)))
            .with_constraint(Constraint::NonEmpty);

        // 基础类型兼容则通过类型检查
        assert!(checker
            .check_constrained(&constrained, &Type::string(), "test")
            .is_ok());

        // 基础类型不兼容则失败
        assert!(checker
            .check_constrained(&constrained, &Type::integer(), "test")
            .is_err());
    }

    #[test]
    fn test_error_messages() {
        let checker = TypeChecker::new();

        let result = checker.check_assignable(&Type::string(), &Type::integer(), "assignment");

        match result {
            Err(TypeError::IncompatibleTypes {
                expected,
                actual,
                context,
            }) => {
                assert_eq!(expected, Type::string());
                assert_eq!(actual, Type::integer());
                assert_eq!(context, "assignment");
            }
            _ => panic!("期望得到 IncompatibleTypes 错误"),
        }
    }
}
