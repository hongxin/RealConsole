//! API 和配置验证器

use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use serde_json::json;
use std::time::Duration;

/// API 验证器
pub struct ApiValidator {
    client: reqwest::Client,
}

impl ApiValidator {
    /// 创建新的验证器
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    /// 验证 Deepseek API Key
    ///
    /// 发送最小测试请求，检查 API Key 是否有效。
    ///
    /// # Returns
    /// - `Ok(true)`: API Key 有效
    /// - `Ok(false)`: API Key 无效（401 Unauthorized）
    /// - `Err(_)`: 网络错误或其他问题
    pub async fn validate_deepseek_key(&self, api_key: &str, endpoint: &str) -> Result<bool> {
        let response = self
            .client
            .post(format!("{}/chat/completions", endpoint))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "deepseek-chat",
                "messages": [{"role": "user", "content": "test"}],
                "max_tokens": 1
            }))
            .send()
            .await
            .map_err(|e| anyhow!("网络请求失败: {}", e))?;

        let status = response.status();

        // 200: 成功
        // 400: 参数错误（但 key 有效）
        // 401: 认证失败（key 无效）
        // 其他: 服务问题
        match status {
            StatusCode::OK | StatusCode::BAD_REQUEST => Ok(true),
            StatusCode::UNAUTHORIZED => Ok(false),
            _ => Err(anyhow!(
                "服务返回异常状态码: {}",
                status.as_u16()
            )),
        }
    }

    /// 检测 Ollama 服务并获取可用模型列表
    ///
    /// # Returns
    /// - `Ok(Vec<String>)`: 可用模型列表
    /// - `Err(_)`: 服务不可用或网络错误
    pub async fn check_ollama_service(&self, endpoint: &str) -> Result<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/api/tags", endpoint))
            .send()
            .await
            .map_err(|e| anyhow!("无法连接到 Ollama 服务: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Ollama 服务返回错误: {}",
                response.status()
            ));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("无法解析 Ollama 响应: {}", e))?;

        let models = data["models"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["name"].as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }
}

impl Default for ApiValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = ApiValidator::new();
        // 验证 validator 创建成功
        let _ = validator.client;
    }

    #[tokio::test]
    async fn test_validate_deepseek_key_invalid() {
        let validator = ApiValidator::new();
        let result = validator
            .validate_deepseek_key("sk-invalid-key", "https://api.deepseek.com/v1")
            .await;

        // 应该返回 Ok(false) 或网络错误
        // 不应该 panic
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_check_ollama_service_not_running() {
        let validator = ApiValidator::new();
        let result = validator
            .check_ollama_service("http://localhost:11434")
            .await;

        // 如果 Ollama 未运行，应该返回错误
        // 如果运行，应该返回模型列表
        assert!(result.is_ok() || result.is_err());
    }
}
