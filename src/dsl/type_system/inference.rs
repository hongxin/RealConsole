//! 类型系统 - 类型推导引擎
//!
//! 实现类型推导、类型变量统一和泛型实例化

use super::checker::{TypeChecker, TypeError};
use super::types::{CompositeType, Type};
use std::collections::HashMap;

/// 类型推导引擎
pub struct TypeInference {
    /// 类型检查器
    checker: TypeChecker,
    /// 类型变量计数器 (用于生成唯一的类型变量名)
    var_counter: usize,
    /// 统一约束 (记录类型变量之间的等价关系)
    unification_map: HashMap<String, Type>,
}

impl TypeInference {
    /// 创建新的类型推导引擎
    pub fn new() -> Self {
        Self {
            checker: TypeChecker::new(),
            var_counter: 0,
            unification_map: HashMap::new(),
        }
    }

    /// 生成新的类型变量
    pub fn fresh_type_var(&mut self) -> Type {
        let name = format!("T{}", self.var_counter);
        self.var_counter += 1;
        Type::TypeVar(name)
    }

    /// 统一两个类型（解决类型变量）
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<Type, TypeError> {
        // 解析类型变量到具体类型
        let t1 = self.resolve_type(t1);
        let t2 = self.resolve_type(t2);

        match (&t1, &t2) {
            // 相同类型
            (Type::Primitive(p1), Type::Primitive(p2)) if p1 == p2 => Ok(t1.clone()),
            (Type::Domain(d1), Type::Domain(d2)) if d1 == d2 => Ok(t1.clone()),
            (Type::Any, _) => Ok(t2.clone()),
            (_, Type::Any) => Ok(t1.clone()),

            // 类型变量统一
            (Type::TypeVar(name), _) => {
                self.bind_type_var(name.clone(), t2.clone());
                Ok(t2.clone())
            }
            (_, Type::TypeVar(name)) => {
                self.bind_type_var(name.clone(), t1.clone());
                Ok(t1.clone())
            }

            // 复合类型统一
            (Type::Composite(c1), Type::Composite(c2)) => {
                self.unify_composite(c1, c2).map(Type::Composite)
            }

            // 不兼容的类型
            _ => Err(TypeError::IncompatibleTypes {
                expected: t1.clone(),
                actual: t2.clone(),
                context: "类型统一".to_string(),
            }),
        }
    }

    /// 统一复合类型
    fn unify_composite(
        &mut self,
        c1: &CompositeType,
        c2: &CompositeType,
    ) -> Result<CompositeType, TypeError> {
        match (c1, c2) {
            // List<T>
            (CompositeType::List(t1), CompositeType::List(t2)) => {
                let unified = self.unify(t1, t2)?;
                Ok(CompositeType::List(Box::new(unified)))
            }

            // Dict<K, V>
            (CompositeType::Dict(k1, v1), CompositeType::Dict(k2, v2)) => {
                let unified_key = self.unify(k1, k2)?;
                let unified_value = self.unify(v1, v2)?;
                Ok(CompositeType::Dict(
                    Box::new(unified_key),
                    Box::new(unified_value),
                ))
            }

            // Optional<T>
            (CompositeType::Optional(t1), CompositeType::Optional(t2)) => {
                let unified = self.unify(t1, t2)?;
                Ok(CompositeType::Optional(Box::new(unified)))
            }

            // Result<T, E>
            (CompositeType::Result(ok1, err1), CompositeType::Result(ok2, err2)) => {
                let unified_ok = self.unify(ok1, ok2)?;
                let unified_err = self.unify(err1, err2)?;
                Ok(CompositeType::Result(
                    Box::new(unified_ok),
                    Box::new(unified_err),
                ))
            }

            // Tuple - 必须长度相同
            (CompositeType::Tuple(types1), CompositeType::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return Err(TypeError::IncompatibleTypes {
                        expected: Type::Composite(c1.clone()),
                        actual: Type::Composite(c2.clone()),
                        context: "Tuple 长度不匹配".to_string(),
                    });
                }

                let unified_types: Result<Vec<Type>, TypeError> = types1
                    .iter()
                    .zip(types2.iter())
                    .map(|(t1, t2)| self.unify(t1, t2))
                    .collect();

                Ok(CompositeType::Tuple(unified_types?))
            }

            // 不同的复合类型
            _ => Err(TypeError::IncompatibleTypes {
                expected: Type::Composite(c1.clone()),
                actual: Type::Composite(c2.clone()),
                context: "复合类型统一".to_string(),
            }),
        }
    }

    /// 绑定类型变量
    fn bind_type_var(&mut self, name: String, ty: Type) {
        // 避免循环引用
        if let Type::TypeVar(var_name) = &ty {
            if var_name == &name {
                return;
            }
        }

        self.unification_map.insert(name.clone(), ty.clone());
        self.checker.bind_type_var(name, ty);
    }

    /// 解析类型变量到具体类型
    pub fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::TypeVar(name) => {
                if let Some(resolved) = self.unification_map.get(name) {
                    // 递归解析
                    self.resolve_type(resolved)
                } else {
                    ty.clone()
                }
            }
            Type::Composite(composite) => {
                Type::Composite(self.resolve_composite(composite))
            }
            _ => ty.clone(),
        }
    }

    /// 解析复合类型中的类型变量
    fn resolve_composite(&self, composite: &CompositeType) -> CompositeType {
        match composite {
            CompositeType::List(t) => {
                CompositeType::List(Box::new(self.resolve_type(t)))
            }
            CompositeType::Dict(k, v) => {
                CompositeType::Dict(
                    Box::new(self.resolve_type(k)),
                    Box::new(self.resolve_type(v)),
                )
            }
            CompositeType::Optional(t) => {
                CompositeType::Optional(Box::new(self.resolve_type(t)))
            }
            CompositeType::Result(ok, err) => {
                CompositeType::Result(
                    Box::new(self.resolve_type(ok)),
                    Box::new(self.resolve_type(err)),
                )
            }
            CompositeType::Tuple(types) => {
                CompositeType::Tuple(
                    types.iter().map(|t| self.resolve_type(t)).collect()
                )
            }
        }
    }

    /// 实例化泛型类型（将类型变量替换为具体类型）
    pub fn instantiate(
        &mut self,
        generic_type: &Type,
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        match generic_type {
            Type::TypeVar(name) => {
                substitutions.get(name).cloned().unwrap_or_else(|| generic_type.clone())
            }
            Type::Composite(composite) => {
                Type::Composite(self.instantiate_composite(composite, substitutions))
            }
            _ => generic_type.clone(),
        }
    }

    /// 实例化复合类型
    fn instantiate_composite(
        &mut self,
        composite: &CompositeType,
        substitutions: &HashMap<String, Type>,
    ) -> CompositeType {
        match composite {
            CompositeType::List(t) => {
                CompositeType::List(Box::new(self.instantiate(t, substitutions)))
            }
            CompositeType::Dict(k, v) => {
                CompositeType::Dict(
                    Box::new(self.instantiate(k, substitutions)),
                    Box::new(self.instantiate(v, substitutions)),
                )
            }
            CompositeType::Optional(t) => {
                CompositeType::Optional(Box::new(self.instantiate(t, substitutions)))
            }
            CompositeType::Result(ok, err) => {
                CompositeType::Result(
                    Box::new(self.instantiate(ok, substitutions)),
                    Box::new(self.instantiate(err, substitutions)),
                )
            }
            CompositeType::Tuple(types) => {
                CompositeType::Tuple(
                    types.iter().map(|t| self.instantiate(t, substitutions)).collect()
                )
            }
        }
    }

    /// 获取类型检查器的引用
    pub fn checker(&self) -> &TypeChecker {
        &self.checker
    }

    /// 获取类型检查器的可变引用
    pub fn checker_mut(&mut self) -> &mut TypeChecker {
        &mut self.checker
    }

    /// 清除所有推导状态
    pub fn clear(&mut self) {
        self.checker.clear_bindings();
        self.unification_map.clear();
        self.var_counter = 0;
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_type_var() {
        let mut inference = TypeInference::new();

        let t1 = inference.fresh_type_var();
        let t2 = inference.fresh_type_var();

        assert!(matches!(t1, Type::TypeVar(_)));
        assert!(matches!(t2, Type::TypeVar(_)));
        assert_ne!(t1, t2); // 不同的类型变量
    }

    #[test]
    fn test_unify_primitive_types() {
        let mut inference = TypeInference::new();

        // 相同类型统一成功
        let result = inference.unify(&Type::string(), &Type::string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::string());

        // 不同类型统一失败
        let result = inference.unify(&Type::string(), &Type::integer());
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_type_variables() {
        let mut inference = TypeInference::new();

        let t_var = inference.fresh_type_var();

        // 类型变量与具体类型统一
        let result = inference.unify(&t_var, &Type::string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::string());

        // 解析类型变量应该得到 String
        let resolved = inference.resolve_type(&t_var);
        assert_eq!(resolved, Type::string());
    }

    #[test]
    fn test_unify_list_types() {
        let mut inference = TypeInference::new();

        let list_t = Type::list(inference.fresh_type_var());
        let list_string = Type::list(Type::string());

        // 统一 List<T> 和 List<String>
        let result = inference.unify(&list_t, &list_string);
        assert!(result.is_ok());

        // 解析后应该是 List<String>
        let resolved = inference.resolve_type(&result.unwrap());
        assert_eq!(resolved, list_string);
    }

    #[test]
    fn test_unify_optional_types() {
        let mut inference = TypeInference::new();

        let opt_t = Type::optional(inference.fresh_type_var());
        let opt_int = Type::optional(Type::integer());

        let result = inference.unify(&opt_t, &opt_int);
        assert!(result.is_ok());

        let resolved = inference.resolve_type(&result.unwrap());
        assert_eq!(resolved, opt_int);
    }

    #[test]
    fn test_unify_result_types() {
        let mut inference = TypeInference::new();

        let result_t_e = Type::result(
            inference.fresh_type_var(),
            inference.fresh_type_var(),
        );
        let result_string_string = Type::result(Type::string(), Type::string());

        let result = inference.unify(&result_t_e, &result_string_string);
        assert!(result.is_ok());

        let resolved = inference.resolve_type(&result.unwrap());
        assert_eq!(resolved, result_string_string);
    }

    #[test]
    fn test_unify_tuple_types() {
        let mut inference = TypeInference::new();

        let tuple_t = Type::Composite(CompositeType::Tuple(vec![
            inference.fresh_type_var(),
            Type::integer(),
        ]));

        let tuple_string_int = Type::Composite(CompositeType::Tuple(vec![
            Type::string(),
            Type::integer(),
        ]));

        let result = inference.unify(&tuple_t, &tuple_string_int);
        assert!(result.is_ok());

        let resolved = inference.resolve_type(&result.unwrap());
        assert_eq!(resolved, tuple_string_int);
    }

    #[test]
    fn test_instantiate_generic() {
        let mut inference = TypeInference::new();

        // 定义泛型类型 List<T>
        let generic_list = Type::list(Type::TypeVar("T".to_string()));

        // 实例化为 List<String>
        let mut substitutions = HashMap::new();
        substitutions.insert("T".to_string(), Type::string());

        let instantiated = inference.instantiate(&generic_list, &substitutions);

        assert_eq!(instantiated, Type::list(Type::string()));
    }

    #[test]
    fn test_complex_type_inference() {
        let mut inference = TypeInference::new();

        // 创建 Result<List<T>, String> 类型
        let t_var = inference.fresh_type_var();
        let result_list_t = Type::result(
            Type::list(t_var.clone()),
            Type::string(),
        );

        // 统一为 Result<List<Integer>, String>
        let result_list_int = Type::result(
            Type::list(Type::integer()),
            Type::string(),
        );

        let unified = inference.unify(&result_list_t, &result_list_int);
        assert!(unified.is_ok());

        // 解析 T 应该是 Integer
        let resolved_t = inference.resolve_type(&t_var);
        assert_eq!(resolved_t, Type::integer());
    }

    #[test]
    fn test_unify_with_any() {
        let mut inference = TypeInference::new();

        // Any 与任何类型统一都成功
        let result = inference.unify(&Type::Any, &Type::string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::string());

        let result = inference.unify(&Type::integer(), &Type::Any);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::integer());
    }

    #[test]
    fn test_clear() {
        let mut inference = TypeInference::new();

        let t_var = inference.fresh_type_var();
        inference.unify(&t_var, &Type::string()).unwrap();

        // 清除状态
        inference.clear();

        // 类型变量应该不再解析为 String
        let resolved = inference.resolve_type(&t_var);
        assert_eq!(resolved, t_var); // 未绑定
    }

    #[test]
    fn test_recursive_resolution() {
        let mut inference = TypeInference::new();

        let t1 = inference.fresh_type_var();
        let t2 = inference.fresh_type_var();

        // T1 -> T2 -> String
        inference.unify(&t1, &t2).unwrap();
        inference.unify(&t2, &Type::string()).unwrap();

        // T1 应该解析为 String
        let resolved = inference.resolve_type(&t1);
        assert_eq!(resolved, Type::string());
    }
}
