//! HTTP 客户端公共层
//!
//! 为 LLM 客户端提供通用的 HTTP 功能：
//! - 客户端配置和构建
//! - 重试逻辑和退避策略
//! - 错误处理和统计记录
//! - JSON 请求/响应处理
//!
//! 设计原则（一分为三）：
//! - 业务逻辑（各客户端实现）
//! - 通用逻辑（本模块）
//! - 连接层（reqwest）

use super::{ClientStats, LlmError, RetryPolicy};
use reqwest::{Client, header::HeaderMap, Response};
use serde_json::Value;
use std::future::Future;
use std::time::Duration;

/// HTTP 客户端基础层
///
/// 提供通用的 HTTP 客户端功能，避免代码重复
#[derive(Clone)]
pub struct HttpClientBase {
    /// reqwest HTTP 客户端
    pub client: Client,

    /// API 端点（已规范化，无尾部斜杠）
    pub endpoint: String,

    /// 统计信息（线程安全）
    pub stats: ClientStats,

    /// 重试策略配置
    pub retry_policy: RetryPolicy,
}

impl HttpClientBase {
    /// 创建新的 HTTP 客户端基础层
    ///
    /// # 参数
    /// - `endpoint`: API 端点 URL
    /// - `timeout_secs`: 请求超时时间（秒）
    ///
    /// # 返回
    /// - `Ok(HttpClientBase)`: 成功创建
    /// - `Err(LlmError)`: 配置错误
    ///
    /// # 示例
    /// ```
    /// use realconsole::llm::http_base::HttpClientBase;
    ///
    /// let base = HttpClientBase::new("https://api.example.com", 60).unwrap();
    /// assert_eq!(base.endpoint, "https://api.example.com");
    /// ```
    pub fn new(endpoint: impl Into<String>, timeout_secs: u64) -> Result<Self, LlmError> {
        // 规范化 endpoint（移除尾部斜杠）
        let endpoint = endpoint.into().trim_end_matches('/').to_string();

        // 构建 HTTP 客户端
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| LlmError::Config(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            endpoint,
            stats: ClientStats::new(),
            retry_policy: RetryPolicy::default(),
        })
    }

    /// 使用自定义重试策略创建客户端
    ///
    /// # 参数
    /// - `endpoint`: API 端点 URL
    /// - `timeout_secs`: 请求超时时间（秒）
    /// - `retry_policy`: 自定义重试策略
    pub fn with_retry_policy(
        endpoint: impl Into<String>,
        timeout_secs: u64,
        retry_policy: RetryPolicy,
    ) -> Result<Self, LlmError> {
        let mut base = Self::new(endpoint, timeout_secs)?;
        base.retry_policy = retry_policy;
        Ok(base)
    }

    /// 发送 POST 请求（JSON payload）
    ///
    /// # 参数
    /// - `url`: 完整请求 URL
    /// - `payload`: JSON payload
    /// - `headers`: 可选的额外 headers（如认证）
    ///
    /// # 返回
    /// - `Ok(Response)`: HTTP 响应
    /// - `Err(LlmError)`: 请求错误
    pub async fn post_json(
        &self,
        url: &str,
        payload: Value,
        headers: Option<HeaderMap>,
    ) -> Result<Response, LlmError> {
        let mut request = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload);

        // 添加额外的 headers（如 Authorization）
        if let Some(headers) = headers {
            request = request.headers(headers);
        }

        request
            .send()
            .await
            .map_err(|e| LlmError::from(e))
    }

    /// 处理 HTTP 响应并提取 JSON
    ///
    /// # 参数
    /// - `resp`: HTTP 响应
    ///
    /// # 返回
    /// - `Ok(Value)`: 解析的 JSON 数据
    /// - `Err(LlmError)`: HTTP 错误或解析错误
    ///
    /// # 错误处理
    /// - HTTP 4xx/5xx → LlmError::Http
    /// - JSON 解析失败 → LlmError::Parse
    pub async fn handle_response(resp: Response) -> Result<Value, LlmError> {
        let status = resp.status();

        // 检查 HTTP 状态码
        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::Http {
                status: status.as_u16(),
                message: error_text,
            });
        }

        // 解析 JSON
        resp.json()
            .await
            .map_err(|e| LlmError::Parse(format!("Failed to parse JSON response: {}", e)))
    }

    /// 带重试的操作执行
    ///
    /// # 参数
    /// - `operation`: 异步操作闭包
    ///
    /// # 返回
    /// - `Ok(T)`: 操作成功结果
    /// - `Err(LlmError)`: 最终失败（重试耗尽）
    ///
    /// # 重试策略
    /// - 根据 `retry_policy` 判断错误是否可重试
    /// - 使用指数退避 + 随机抖动
    /// - 自动记录重试次数
    ///
    /// # 示例
    /// ```ignore
    /// let result = base.with_retry(|| async {
    ///     // 可能失败的操作
    ///     Ok("success".to_string())
    /// }).await?;
    /// ```
    pub async fn with_retry<F, Fut, T>(&self, mut operation: F) -> Result<T, LlmError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, LlmError>>,
    {
        let mut last_error = None;

        for attempt in 1..=self.retry_policy.max_attempts {
            match operation().await {
                Ok(result) => {
                    // 成功：记录重试次数（如果不是第一次）
                    if attempt > 1 {
                        self.stats.record_retry();
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e.clone());

                    // 判断是否应该重试
                    if attempt < self.retry_policy.max_attempts && self.retry_policy.is_retryable(&e) {
                        // 计算退避时间并等待
                        let backoff = self.retry_policy.backoff_duration(attempt);
                        tokio::time::sleep(backoff).await;
                        continue;
                    } else {
                        // 不可重试或重试耗尽
                        break;
                    }
                }
            }
        }

        // 返回最后的错误
        Err(last_error.unwrap_or_else(|| LlmError::Other("Unknown error after retries".into())))
    }

    /// 记录操作统计的包装器
    ///
    /// # 参数
    /// - `operation`: 异步操作闭包
    ///
    /// # 返回
    /// - `Ok(T)`: 操作成功结果
    /// - `Err(LlmError)`: 操作失败
    ///
    /// # 功能
    /// - 自动记录 total_calls
    /// - 自动记录 success/error
    /// - 透传原始结果
    ///
    /// # 示例
    /// ```ignore
    /// let result = base.record_operation(async {
    ///     // 业务逻辑
    ///     Ok("success".to_string())
    /// }).await?;
    /// ```
    pub async fn record_operation<F, Fut, T>(&self, operation: F) -> Result<T, LlmError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, LlmError>>,
    {
        self.stats.record_call();

        match operation().await {
            Ok(result) => {
                self.stats.record_success();
                Ok(result)
            }
            Err(e) => {
                self.stats.record_error();
                Err(e)
            }
        }
    }

    /// 组合：带重试 + 统计记录
    ///
    /// # 参数
    /// - `operation`: 异步操作闭包
    ///
    /// # 返回
    /// - `Ok(T)`: 操作成功结果
    /// - `Err(LlmError)`: 操作失败
    ///
    /// # 功能
    /// - 记录 total_calls（1次）
    /// - 重试逻辑（根据 retry_policy）
    /// - 记录 success/error（1次）
    /// - 记录 retries（如果有重试）
    ///
    /// # 示例
    /// ```ignore
    /// let result = base.with_retry_and_stats(|| async {
    ///     // 可能失败的操作
    ///     Ok("success".to_string())
    /// }).await?;
    /// ```
    pub async fn with_retry_and_stats<F, Fut, T>(&self, operation: F) -> Result<T, LlmError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, LlmError>>,
    {
        self.record_operation(|| self.with_retry(operation)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_base_creation() {
        let base = HttpClientBase::new("https://api.example.com", 60);
        assert!(base.is_ok());

        let base = base.unwrap();
        assert_eq!(base.endpoint, "https://api.example.com");
    }

    #[test]
    fn test_endpoint_normalization() {
        let base = HttpClientBase::new("https://api.example.com/", 60).unwrap();
        assert_eq!(base.endpoint, "https://api.example.com");

        let base = HttpClientBase::new("https://api.example.com///", 60).unwrap();
        assert_eq!(base.endpoint, "https://api.example.com");
    }

    #[test]
    fn test_custom_retry_policy() {
        let custom_policy = RetryPolicy {
            max_attempts: 5,
            initial_backoff_ms: 1000,
            max_backoff_ms: 10000,
            backoff_multiplier: 2.0,
        };

        let base = HttpClientBase::with_retry_policy(
            "https://api.example.com",
            60,
            custom_policy.clone(),
        )
        .unwrap();

        assert_eq!(base.retry_policy.max_attempts, 5);
        assert_eq!(base.retry_policy.initial_backoff_ms, 1000);
    }

    #[tokio::test]
    async fn test_with_retry_success_first_attempt() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let base = HttpClientBase::new("https://api.example.com", 60).unwrap();

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let result = base
            .with_retry(|| {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, LlmError>("success".to_string())
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Only called once
    }

    #[tokio::test]
    async fn test_with_retry_success_after_failures() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let base = HttpClientBase::new("https://api.example.com", 60).unwrap();

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let result = base
            .with_retry(|| {
                let count = call_count_clone.clone();
                async move {
                    let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                    if current < 3 {
                        Err(LlmError::Network("temporary error".to_string()))
                    } else {
                        Ok("success".to_string())
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.load(Ordering::SeqCst), 3); // Called 3 times
    }

    #[tokio::test]
    async fn test_with_retry_max_attempts_reached() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let base = HttpClientBase::new("https://api.example.com", 60).unwrap();

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let result = base
            .with_retry(|| {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>(LlmError::Network("persistent error".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), base.retry_policy.max_attempts as usize);
    }

    #[tokio::test]
    async fn test_with_retry_non_retryable_error() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let base = HttpClientBase::new("https://api.example.com", 60).unwrap();

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let result = base
            .with_retry(|| {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>(LlmError::Parse("parse error".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Only called once (non-retryable)
    }

    #[tokio::test]
    async fn test_record_operation_success() {
        let base = HttpClientBase::new("https://api.example.com", 60).unwrap();

        let stats_before = base.stats.clone();
        assert_eq!(stats_before.total_calls(), 0);

        let result = base
            .record_operation(|| async { Ok::<_, LlmError>("success".to_string()) })
            .await;

        assert!(result.is_ok());

        let stats_after = base.stats.clone();
        assert_eq!(stats_after.total_calls(), 1);
        assert_eq!(stats_after.total_success(), 1);
        assert_eq!(stats_after.total_errors(), 0);
    }

    #[tokio::test]
    async fn test_record_operation_failure() {
        let base = HttpClientBase::new("https://api.example.com", 60).unwrap();

        let result = base
            .record_operation(|| async { Err::<String, _>(LlmError::Network("error".to_string())) })
            .await;

        assert!(result.is_err());

        let stats = base.stats.clone();
        assert_eq!(stats.total_calls(), 1);
        assert_eq!(stats.total_success(), 0);
        assert_eq!(stats.total_errors(), 1);
    }

    #[tokio::test]
    async fn test_with_retry_and_stats() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let base = HttpClientBase::new("https://api.example.com", 60).unwrap();

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let result = base
            .with_retry_and_stats(|| {
                let count = call_count_clone.clone();
                async move {
                    let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                    if current < 2 {
                        Err(LlmError::Timeout)
                    } else {
                        Ok("success".to_string())
                    }
                }
            })
            .await;

        assert!(result.is_ok());

        let stats = base.stats.clone();
        assert_eq!(stats.total_calls(), 1); // Only recorded once
        assert_eq!(stats.total_success(), 1);
        assert_eq!(stats.total_retries(), 1); // One retry
    }

    #[tokio::test]
    async fn test_handle_response_success() {
        // This test requires a real HTTP response, which is difficult to mock without a server.
        // In practice, this is covered by integration tests with mock servers.
    }

    #[tokio::test]
    async fn test_post_json_without_headers() {
        // This test requires a real HTTP server or mock server.
        // Covered by integration tests.
    }
}
