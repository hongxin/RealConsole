//! 参数分析器
//!
//! 使用 LLM 智能分析用户输入，提取参数并检测缺失项

use super::context::{ParameterSpec, ParameterType, ParameterValue};
use crate::llm::{LlmClient, Message};
use serde::{Deserialize, Serialize};

/// 参数分析器
pub struct ParameterAnalyzer;

impl ParameterAnalyzer {
    /// 创建新的参数分析器
    pub fn new() -> Self {
        Self
    }

    /// 从用户输入中提取参数
    ///
    /// 使用 LLM 分析用户输入，根据参数规格提取对应的参数值
    pub async fn extract_parameters(
        &self,
        user_input: &str,
        parameter_specs: &[ParameterSpec],
        llm: &dyn LlmClient,
    ) -> Result<Vec<(String, ParameterValue)>, String> {
        if parameter_specs.is_empty() {
            return Ok(Vec::new());
        }

        // 构建分析提示词
        let prompt = self.build_extraction_prompt(user_input, parameter_specs);

        // 调用 LLM 分析
        let response = llm
            .chat(vec![Message::user(prompt)])
            .await
            .map_err(|e| format!("LLM 调用失败: {}", e))?;

        // 解析 LLM 响应
        self.parse_extraction_response(&response, parameter_specs)
    }

    /// 检测缺失的参数
    ///
    /// 分析已收集的参数，确定还需要哪些参数
    pub fn detect_missing_parameters(
        &self,
        parameter_specs: &[ParameterSpec],
        collected_params: &std::collections::HashMap<String, ParameterValue>,
    ) -> Vec<ParameterSpec> {
        parameter_specs
            .iter()
            .filter(|spec| {
                // 必需参数且未收集
                !spec.is_optional && !collected_params.contains_key(&spec.name)
            })
            .cloned()
            .collect()
    }

    /// 生成智能提问
    ///
    /// 根据缺失的参数生成自然、友好的提问
    pub async fn generate_question(
        &self,
        parameter_spec: &ParameterSpec,
        context: &str,
        llm: &dyn LlmClient,
    ) -> Result<String, String> {
        let prompt = format!(
            r#"你是一个智能 CLI 助手，正在帮助用户完成一个任务。

当前上下文：{}

需要询问的参数：
- 名称：{}
- 类型：{:?}
- 描述：{}
{}{}

请生成一个自然、简洁、友好的提问，询问用户提供这个参数。
要求：
1. 使用口语化、自然的语言
2. 提问应该简短（不超过20个字）
3. 如果有提示信息或示例，可以适当引用
4. 不要使用"请提供"、"请输入"等生硬的表达

只返回提问文本，不要有任何额外说明。
"#,
            context,
            parameter_spec.name,
            parameter_spec.param_type,
            parameter_spec.description,
            parameter_spec
                .hint
                .as_ref()
                .map(|h| format!("\n- 提示：{}", h))
                .unwrap_or_default(),
            parameter_spec
                .example
                .as_ref()
                .map(|e| format!("\n- 示例：{}", e))
                .unwrap_or_default()
        );

        let question = llm
            .chat(vec![Message::user(prompt)])
            .await
            .map_err(|e| format!("LLM 调用失败: {}", e))?;

        Ok(question.trim().to_string())
    }

    /// 验证参数值
    ///
    /// 检查参数值是否符合类型和约束要求
    pub fn validate_parameter(
        &self,
        parameter_spec: &ParameterSpec,
        value: &ParameterValue,
    ) -> Result<(), String> {
        // 检查类型匹配
        let type_matches = match (&parameter_spec.param_type, value) {
            (ParameterType::String, ParameterValue::String(_)) => true,
            (ParameterType::Integer, ParameterValue::Integer(_)) => true,
            (ParameterType::Float, ParameterValue::Float(_)) => true,
            (ParameterType::Boolean, ParameterValue::Boolean(_)) => true,
            (ParameterType::Path, ParameterValue::String(_)) => true,
            (ParameterType::Directory, ParameterValue::String(_)) => true,
            (ParameterType::File, ParameterValue::String(_)) => true,
            (ParameterType::List(_), ParameterValue::List(_)) => true,
            (ParameterType::Enum(options), ParameterValue::String(s)) => options.contains(s),
            _ => false,
        };

        if !type_matches {
            return Err(format!(
                "参数类型不匹配：期望 {:?}，实际 {:?}",
                parameter_spec.param_type, value
            ));
        }

        // 路径验证
        match &parameter_spec.param_type {
            ParameterType::Path => {
                if let ParameterValue::String(path) = value {
                    if !std::path::Path::new(path).exists() {
                        return Err(format!("路径不存在：{}", path));
                    }
                }
            }
            ParameterType::Directory => {
                if let ParameterValue::String(path) = value {
                    let p = std::path::Path::new(path);
                    if !p.exists() {
                        return Err(format!("目录不存在：{}", path));
                    }
                    if !p.is_dir() {
                        return Err(format!("不是有效的目录：{}", path));
                    }
                }
            }
            ParameterType::File => {
                if let ParameterValue::String(path) = value {
                    let p = std::path::Path::new(path);
                    if !p.exists() {
                        return Err(format!("文件不存在：{}", path));
                    }
                    if !p.is_file() {
                        return Err(format!("不是有效的文件：{}", path));
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// 构建参数提取提示词
    fn build_extraction_prompt(&self, user_input: &str, parameter_specs: &[ParameterSpec]) -> String {
        let params_desc = parameter_specs
            .iter()
            .map(|spec| {
                format!(
                    "- {}: {:?} - {}{}",
                    spec.name,
                    spec.param_type,
                    spec.description,
                    spec.example
                        .as_ref()
                        .map(|e| format!(" (示例: {})", e))
                        .unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"你是一个参数提取专家。请从用户输入中提取以下参数的值：

用户输入："{}"

需要提取的参数：
{}

请以 JSON 格式返回提取到的参数，格式如下：
{{
    "parameters": {{
        "参数名": "参数值",
        ...
    }}
}}

注意：
1. 只返回你能确定提取到的参数
2. 如果某个参数在输入中找不到，就不要包含它
3. 参数值应该符合对应的类型要求
4. 只返回 JSON，不要有任何额外说明
"#,
            user_input, params_desc
        )
    }

    /// 解析 LLM 提取响应
    fn parse_extraction_response(
        &self,
        response: &str,
        parameter_specs: &[ParameterSpec],
    ) -> Result<Vec<(String, ParameterValue)>, String> {
        // 尝试提取 JSON
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        // 解析 JSON
        let parsed: ExtractionResult = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON 解析失败: {}", e))?;

        // 转换为 ParameterValue
        let mut extracted = Vec::new();
        for (name, value) in parsed.parameters {
            // 查找参数规格
            if let Some(spec) = parameter_specs.iter().find(|s| s.name == name) {
                // 根据类型转换值
                let param_value = self.convert_to_parameter_value(&value, &spec.param_type)?;
                extracted.push((name, param_value));
            }
        }

        Ok(extracted)
    }

    /// 将 JSON 值转换为 ParameterValue
    fn convert_to_parameter_value(
        &self,
        value: &serde_json::Value,
        param_type: &ParameterType,
    ) -> Result<ParameterValue, String> {
        match param_type {
            ParameterType::String
            | ParameterType::Path
            | ParameterType::Directory
            | ParameterType::File => {
                if let Some(s) = value.as_str() {
                    Ok(ParameterValue::String(s.to_string()))
                } else {
                    Err(format!("期望字符串类型，实际为 {:?}", value))
                }
            }
            ParameterType::Integer => {
                if let Some(i) = value.as_i64() {
                    Ok(ParameterValue::Integer(i))
                } else {
                    Err(format!("期望整数类型，实际为 {:?}", value))
                }
            }
            ParameterType::Float => {
                if let Some(f) = value.as_f64() {
                    Ok(ParameterValue::Float(f))
                } else {
                    Err(format!("期望浮点数类型，实际为 {:?}", value))
                }
            }
            ParameterType::Boolean => {
                if let Some(b) = value.as_bool() {
                    Ok(ParameterValue::Boolean(b))
                } else {
                    Err(format!("期望布尔类型，实际为 {:?}", value))
                }
            }
            ParameterType::Enum(options) => {
                if let Some(s) = value.as_str() {
                    if options.contains(&s.to_string()) {
                        Ok(ParameterValue::String(s.to_string()))
                    } else {
                        Err(format!("枚举值 '{}' 不在允许的选项中", s))
                    }
                } else {
                    Err(format!("期望枚举字符串，实际为 {:?}", value))
                }
            }
            ParameterType::List(inner_type) => {
                if let Some(arr) = value.as_array() {
                    let mut values = Vec::new();
                    for item in arr {
                        values.push(self.convert_to_parameter_value(item, inner_type)?);
                    }
                    Ok(ParameterValue::List(values))
                } else {
                    Err(format!("期望列表类型，实际为 {:?}", value))
                }
            }
        }
    }
}

impl Default for ParameterAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// LLM 提取结果
#[derive(Debug, Deserialize, Serialize)]
struct ExtractionResult {
    parameters: std::collections::HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = ParameterAnalyzer::new();
        assert!(true); // 只是测试能够创建
    }

    #[test]
    fn test_detect_missing_parameters() {
        let analyzer = ParameterAnalyzer::new();

        let specs = vec![
            ParameterSpec::new("param1", ParameterType::String, "First param"),
            ParameterSpec::new("param2", ParameterType::String, "Second param"),
            ParameterSpec::new("param3", ParameterType::String, "Third param").optional(),
        ];

        let mut collected = HashMap::new();
        collected.insert(
            "param1".to_string(),
            ParameterValue::String("value1".to_string()),
        );

        let missing = analyzer.detect_missing_parameters(&specs, &collected);

        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].name, "param2");
    }

    #[test]
    fn test_validate_parameter_string() {
        let analyzer = ParameterAnalyzer::new();

        let spec = ParameterSpec::new("test", ParameterType::String, "Test param");
        let value = ParameterValue::String("test_value".to_string());

        assert!(analyzer.validate_parameter(&spec, &value).is_ok());
    }

    #[test]
    fn test_validate_parameter_type_mismatch() {
        let analyzer = ParameterAnalyzer::new();

        let spec = ParameterSpec::new("test", ParameterType::Integer, "Test param");
        let value = ParameterValue::String("not_a_number".to_string());

        assert!(analyzer.validate_parameter(&spec, &value).is_err());
    }

    #[test]
    fn test_convert_to_parameter_value() {
        let analyzer = ParameterAnalyzer::new();

        // 测试字符串转换
        let json_value = serde_json::json!("test_string");
        let result = analyzer
            .convert_to_parameter_value(&json_value, &ParameterType::String)
            .unwrap();
        assert_eq!(result, ParameterValue::String("test_string".to_string()));

        // 测试整数转换
        let json_value = serde_json::json!(42);
        let result = analyzer
            .convert_to_parameter_value(&json_value, &ParameterType::Integer)
            .unwrap();
        assert_eq!(result, ParameterValue::Integer(42));

        // 测试浮点数转换
        let json_value = serde_json::json!(3.14);
        let result = analyzer
            .convert_to_parameter_value(&json_value, &ParameterType::Float)
            .unwrap();
        assert_eq!(result, ParameterValue::Float(3.14));

        // 测试布尔转换
        let json_value = serde_json::json!(true);
        let result = analyzer
            .convert_to_parameter_value(&json_value, &ParameterType::Boolean)
            .unwrap();
        assert_eq!(result, ParameterValue::Boolean(true));
    }

    #[test]
    fn test_parse_extraction_response() {
        let analyzer = ParameterAnalyzer::new();

        let specs = vec![
            ParameterSpec::new("name", ParameterType::String, "Name"),
            ParameterSpec::new("age", ParameterType::Integer, "Age"),
        ];

        let response = r#"{"parameters": {"name": "Alice", "age": 30}}"#;

        let result = analyzer.parse_extraction_response(response, &specs).unwrap();

        assert_eq!(result.len(), 2);

        // 由于 HashMap 迭代顺序不确定，需要按名称查找
        let name_param = result.iter().find(|(name, _)| name == "name").unwrap();
        assert_eq!(name_param.1, ParameterValue::String("Alice".to_string()));

        let age_param = result.iter().find(|(name, _)| name == "age").unwrap();
        assert_eq!(age_param.1, ParameterValue::Integer(30));
    }
}
