//! 参数定义与处理
//!
//! 支持多种参数类型，用于智能参数补全

use serde::{Deserialize, Serialize};

/// 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// 参数名称
    pub name: String,

    /// 参数类型
    pub param_type: ParameterType,

    /// 参数描述
    pub description: String,

    /// 是否必需
    pub required: bool,

    /// 默认值
    pub default_value: Option<ParameterValue>,

    /// 验证规则
    pub validation: Option<ValidationRule>,
}

impl Parameter {
    /// 创建新的必需参数
    pub fn new_required(name: String, param_type: ParameterType, description: String) -> Self {
        Self {
            name,
            param_type,
            description,
            required: true,
            default_value: None,
            validation: None,
        }
    }

    /// 创建新的可选参数
    pub fn new_optional(name: String, param_type: ParameterType, description: String) -> Self {
        Self {
            name,
            param_type,
            description,
            required: false,
            default_value: None,
            validation: None,
        }
    }

    /// 设置默认值
    pub fn with_default(mut self, value: ParameterValue) -> Self {
        self.default_value = Some(value);
        self
    }

    /// 设置验证规则
    pub fn with_validation(mut self, validation: ValidationRule) -> Self {
        self.validation = Some(validation);
        self
    }

    /// 验证参数值
    pub fn validate(&self, value: &ParameterValue) -> Result<(), String> {
        // 检查类型匹配
        if !self.param_type.matches(value) {
            return Err(format!(
                "参数 '{}' 类型不匹配：期望 {:?}，实际 {:?}",
                self.name, self.param_type, value
            ));
        }

        // 应用验证规则
        if let Some(ref validation) = self.validation {
            validation.validate(value)?;
        }

        Ok(())
    }
}

/// 参数类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    /// 字符串
    String,

    /// 整数
    Integer,

    /// 浮点数
    Float,

    /// 布尔值
    Boolean,

    /// 路径
    Path,

    /// 目录
    Directory,

    /// 文件
    File,

    /// 枚举值
    Enum(Vec<String>),

    /// 列表
    List(Box<ParameterType>),
}

impl ParameterType {
    /// 检查值是否匹配类型
    pub fn matches(&self, value: &ParameterValue) -> bool {
        match (self, value) {
            (ParameterType::String, ParameterValue::String(_)) => true,
            (ParameterType::Integer, ParameterValue::Integer(_)) => true,
            (ParameterType::Float, ParameterValue::Float(_)) => true,
            (ParameterType::Boolean, ParameterValue::Boolean(_)) => true,
            (ParameterType::Path, ParameterValue::String(_)) => true,
            (ParameterType::Directory, ParameterValue::String(_)) => true,
            (ParameterType::File, ParameterValue::String(_)) => true,
            (ParameterType::Enum(options), ParameterValue::String(s)) => options.contains(s),
            (ParameterType::List(inner_type), ParameterValue::List(values)) => {
                values.iter().all(|v| inner_type.matches(v))
            }
            _ => false,
        }
    }
}

/// 参数值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterValue {
    /// 字符串值
    String(String),

    /// 整数值
    Integer(i64),

    /// 浮点数值
    Float(f64),

    /// 布尔值
    Boolean(bool),

    /// 列表值
    List(Vec<ParameterValue>),
}

impl ParameterValue {
    /// 尝试转换为字符串
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ParameterValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// 尝试转换为整数
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ParameterValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// 尝试转换为浮点数
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ParameterValue::Float(f) => Some(*f),
            ParameterValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// 尝试转换为布尔值
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ParameterValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl std::fmt::Display for ParameterValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterValue::String(s) => write!(f, "{}", s),
            ParameterValue::Integer(i) => write!(f, "{}", i),
            ParameterValue::Float(fl) => write!(f, "{}", fl),
            ParameterValue::Boolean(b) => write!(f, "{}", b),
            ParameterValue::List(list) => {
                write!(f, "[")?;
                for (i, v) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
        }
    }
}

/// 验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    /// 范围验证（整数）
    IntRange { min: Option<i64>, max: Option<i64> },

    /// 范围验证（浮点数）
    FloatRange { min: Option<f64>, max: Option<f64> },

    /// 长度验证
    StringLength { min: Option<usize>, max: Option<usize> },

    /// 正则表达式验证
    Regex(String),

    /// 路径存在性验证
    PathExists,

    /// 目录存在性验证
    DirectoryExists,

    /// 文件存在性验证
    FileExists,

    /// 自定义验证
    Custom(String),
}

impl ValidationRule {
    /// 验证参数值
    pub fn validate(&self, value: &ParameterValue) -> Result<(), String> {
        match self {
            ValidationRule::IntRange { min, max } => {
                if let Some(i) = value.as_integer() {
                    if let Some(min_val) = min {
                        if i < *min_val {
                            return Err(format!("值 {} 小于最小值 {}", i, min_val));
                        }
                    }
                    if let Some(max_val) = max {
                        if i > *max_val {
                            return Err(format!("值 {} 大于最大值 {}", i, max_val));
                        }
                    }
                    Ok(())
                } else {
                    Err("类型不是整数".to_string())
                }
            }

            ValidationRule::FloatRange { min, max } => {
                if let Some(f) = value.as_float() {
                    if let Some(min_val) = min {
                        if f < *min_val {
                            return Err(format!("值 {} 小于最小值 {}", f, min_val));
                        }
                    }
                    if let Some(max_val) = max {
                        if f > *max_val {
                            return Err(format!("值 {} 大于最大值 {}", f, max_val));
                        }
                    }
                    Ok(())
                } else {
                    Err("类型不是浮点数".to_string())
                }
            }

            ValidationRule::StringLength { min, max } => {
                if let Some(s) = value.as_string() {
                    let len = s.len();
                    if let Some(min_len) = min {
                        if len < *min_len {
                            return Err(format!("长度 {} 小于最小长度 {}", len, min_len));
                        }
                    }
                    if let Some(max_len) = max {
                        if len > *max_len {
                            return Err(format!("长度 {} 大于最大长度 {}", len, max_len));
                        }
                    }
                    Ok(())
                } else {
                    Err("类型不是字符串".to_string())
                }
            }

            ValidationRule::PathExists => {
                if let Some(path_str) = value.as_string() {
                    let path = std::path::Path::new(path_str);
                    if path.exists() {
                        Ok(())
                    } else {
                        Err(format!("路径不存在：{}", path_str))
                    }
                } else {
                    Err("类型不是字符串".to_string())
                }
            }

            ValidationRule::DirectoryExists => {
                if let Some(path_str) = value.as_string() {
                    let path = std::path::Path::new(path_str);
                    if path.is_dir() {
                        Ok(())
                    } else {
                        Err(format!("不是有效的目录：{}", path_str))
                    }
                } else {
                    Err("类型不是字符串".to_string())
                }
            }

            ValidationRule::FileExists => {
                if let Some(path_str) = value.as_string() {
                    let path = std::path::Path::new(path_str);
                    if path.is_file() {
                        Ok(())
                    } else {
                        Err(format!("不是有效的文件：{}", path_str))
                    }
                } else {
                    Err("类型不是字符串".to_string())
                }
            }

            ValidationRule::Regex(pattern) => {
                if let Some(s) = value.as_string() {
                    match regex::Regex::new(pattern) {
                        Ok(re) => {
                            if re.is_match(s) {
                                Ok(())
                            } else {
                                Err(format!("不匹配正则表达式：{}", pattern))
                            }
                        }
                        Err(e) => Err(format!("无效的正则表达式：{}", e)),
                    }
                } else {
                    Err("类型不是字符串".to_string())
                }
            }

            ValidationRule::Custom(msg) => Err(msg.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_type_matching() {
        let string_type = ParameterType::String;
        let int_type = ParameterType::Integer;

        assert!(string_type.matches(&ParameterValue::String("test".to_string())));
        assert!(!string_type.matches(&ParameterValue::Integer(42)));

        assert!(int_type.matches(&ParameterValue::Integer(42)));
        assert!(!int_type.matches(&ParameterValue::String("test".to_string())));
    }

    #[test]
    fn test_int_range_validation() {
        let validation = ValidationRule::IntRange {
            min: Some(1),
            max: Some(10),
        };

        assert!(validation.validate(&ParameterValue::Integer(5)).is_ok());
        assert!(validation.validate(&ParameterValue::Integer(0)).is_err());
        assert!(validation.validate(&ParameterValue::Integer(11)).is_err());
    }

    #[test]
    fn test_string_length_validation() {
        let validation = ValidationRule::StringLength {
            min: Some(3),
            max: Some(10),
        };

        assert!(validation.validate(&ParameterValue::String("hello".to_string())).is_ok());
        assert!(validation.validate(&ParameterValue::String("hi".to_string())).is_err());
        assert!(validation.validate(&ParameterValue::String("hello world!".to_string())).is_err());
    }

    #[test]
    fn test_enum_type() {
        let enum_type = ParameterType::Enum(vec![
            "small".to_string(),
            "medium".to_string(),
            "large".to_string(),
        ]);

        assert!(enum_type.matches(&ParameterValue::String("small".to_string())));
        assert!(!enum_type.matches(&ParameterValue::String("xlarge".to_string())));
    }
}
