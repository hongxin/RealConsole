//! Command Validator Module (Phase 3)
//!
//! Provides LLM-based validation of generated commands to ensure correctness.

use crate::dsl::intent::ExecutionPlan;
use crate::llm::{LlmClient, Message};
use serde::{Deserialize, Serialize};

/// Command Validator
///
/// Uses LLM to validate if a generated command correctly reflects user intent.
pub struct CommandValidator;

impl CommandValidator {
    /// Create a new command validator
    pub fn new() -> Self {
        Self
    }

    /// 使用 LLM 验证生成的命令
    ///
    /// # Arguments
    ///
    /// * `user_input` - 原始用户输入
    /// * `plan` - 生成的执行计划
    /// * `intent_name` - 意图名称
    /// * `llm` - LLM 客户端
    ///
    /// # Returns
    ///
    /// ValidationResult 包含验证结果、置信度、原因和建议
    pub async fn validate(
        &self,
        user_input: &str,
        plan: &ExecutionPlan,
        intent_name: &str,
        llm: &dyn LlmClient,
    ) -> Result<ValidationResult, String> {
        let prompt = self.build_validation_prompt(user_input, plan, intent_name);

        let messages = vec![Message::user(prompt)];
        let response = llm.chat(messages).await
            .map_err(|e| format!("LLM 调用失败: {}", e))?;

        self.parse_validation_response(&response)
    }

    /// 构造验证 prompt
    fn build_validation_prompt(
        &self,
        user_input: &str,
        plan: &ExecutionPlan,
        intent_name: &str,
    ) -> String {
        format!(
            r#"请评估以下命令生成是否合理：

用户意图: "{}"
生成的命令: {}
匹配的意图: {}
模板名称: {}

请从以下角度评估:
1. 命令是否正确理解了用户意图？
2. 参数是否合理（如路径、数字等）？
3. 是否存在明显错误或安全风险？

请以 JSON 格式返回评估结果（只返回 JSON，不要其他解释）:
{{
  "is_valid": true/false,
  "confidence": 0.0-1.0,
  "reason": "评估理由",
  "suggestions": ["改进建议1", "改进建议2"]
}}

注意：
- confidence 应该是 0.0 到 1.0 之间的数字
- suggestions 是字符串数组，如果没有建议可以为空数组
- reason 应该简洁说明判断依据"#,
            user_input,
            plan.command,
            intent_name,
            plan.template_name
        )
    }

    /// 解析 LLM 验证响应
    fn parse_validation_response(&self, response: &str) -> Result<ValidationResult, String> {
        // 提取 JSON 块
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                return Err("未找到完整的 JSON".to_string());
            }
        } else {
            return Err("未找到 JSON 响应".to_string());
        };

        // 解析 JSON
        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON 解析失败: {}", e))?;

        Ok(ValidationResult {
            is_valid: parsed["is_valid"].as_bool().unwrap_or(true),
            confidence: parsed["confidence"].as_f64().unwrap_or(1.0),
            reason: parsed["reason"]
                .as_str()
                .unwrap_or("无法解析原因")
                .to_string(),
            suggestions: parsed["suggestions"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 命令是否有效
    pub is_valid: bool,

    /// 置信度 (0.0-1.0)
    pub confidence: f64,

    /// 评估原因
    pub reason: String,

    /// 改进建议
    pub suggestions: Vec<String>,
}

impl ValidationResult {
    /// 判断是否需要警告用户
    pub fn should_warn(&self, threshold: f64) -> bool {
        !self.is_valid || self.confidence < threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = CommandValidator::new();
        assert!(std::mem::size_of_val(&validator) == 0); // Zero-sized type
    }

    #[test]
    fn test_validation_result_should_warn() {
        let result = ValidationResult {
            is_valid: true,
            confidence: 0.9,
            reason: "看起来正确".to_string(),
            suggestions: vec![],
        };
        assert!(!result.should_warn(0.7));

        let result = ValidationResult {
            is_valid: true,
            confidence: 0.5,
            reason: "不太确定".to_string(),
            suggestions: vec!["检查路径是否存在".to_string()],
        };
        assert!(result.should_warn(0.7));

        let result = ValidationResult {
            is_valid: false,
            confidence: 0.9,
            reason: "命令有误".to_string(),
            suggestions: vec![],
        };
        assert!(result.should_warn(0.7));
    }

    #[test]
    fn test_parse_validation_response_valid() {
        let validator = CommandValidator::new();
        let json_response = r#"
        {
          "is_valid": true,
          "confidence": 0.95,
          "reason": "命令正确理解了用户意图",
          "suggestions": []
        }
        "#;

        let result = validator.parse_validation_response(json_response).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.confidence, 0.95);
        assert!(result.suggestions.is_empty());
    }

    #[test]
    fn test_parse_validation_response_invalid() {
        let validator = CommandValidator::new();
        let json_response = r#"
        {
          "is_valid": false,
          "confidence": 0.6,
          "reason": "路径参数可能不正确",
          "suggestions": ["检查目录是否存在", "使用绝对路径"]
        }
        "#;

        let result = validator.parse_validation_response(json_response).unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.confidence, 0.6);
        assert_eq!(result.suggestions.len(), 2);
    }

    #[test]
    fn test_parse_validation_response_with_markdown() {
        let validator = CommandValidator::new();
        let markdown_response = r#"
        根据评估，这个命令看起来是合理的。

        ```json
        {
          "is_valid": true,
          "confidence": 0.88,
          "reason": "参数提取正确",
          "suggestions": []
        }
        ```
        "#;

        let result = validator.parse_validation_response(markdown_response).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.confidence, 0.88);
    }
}
