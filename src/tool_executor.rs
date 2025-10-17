//! 工具执行引擎
//!
//! 负责：
//! - 迭代工具调用控制 (最多 5 轮)
//! - 单轮工具数量限制 (最多 3 个)
//! - LLM Function Calling 集成
//! - 工具结果反馈
//! - ✨ Phase 5.2: 并行工具执行 + 执行统计
//! - ✨ Phase 5.3 Week 3 Day 2: 工具响应缓存

use crate::llm::{LlmClient, LlmError, Message};
use crate::tool::ToolRegistry;
use crate::tool_cache::{ToolCache, CacheStats};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// 工具调用请求 (from LLM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// 调用 ID
    pub id: String,

    /// 工具名称
    pub name: String,

    /// 工具参数 (JSON)
    pub arguments: JsonValue,
}

/// 工具调用结果
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    /// 调用 ID
    pub call_id: String,

    /// 工具名称
    pub tool_name: String,

    /// 是否成功
    pub success: bool,

    /// 结果内容
    pub content: String,

    /// ✨ Phase 5.2: 执行耗时（毫秒）
    pub duration_ms: u64,
}

/// ✨ Phase 5.2: 工具执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// 串行执行（保持顺序，适合有依赖的工具链）
    Sequential,
    /// 并行执行（性能优先，适合独立工具）
    Parallel,
}

/// 工具执行引擎
pub struct ToolExecutor {
    /// 工具注册表
    registry: Arc<RwLock<ToolRegistry>>,

    /// 最大迭代轮数
    max_iterations: usize,

    /// 每轮最多工具数
    max_tools_per_round: usize,

    /// ✨ Phase 5.2: 执行模式
    execution_mode: ExecutionMode,

    /// ✨ Phase 5.3 Week 3 Day 2: 工具响应缓存
    cache: Option<Arc<ToolCache>>,
}

impl ToolExecutor {
    /// 创建新的工具执行引擎
    pub fn new(
        registry: Arc<RwLock<ToolRegistry>>,
        max_iterations: usize,
        max_tools_per_round: usize,
    ) -> Self {
        Self {
            registry,
            max_iterations,
            max_tools_per_round,
            execution_mode: ExecutionMode::Parallel, // 默认并行执行
            cache: None, // 默认不启用缓存
        }
    }

    /// 使用默认参数创建
    pub fn with_defaults(registry: Arc<RwLock<ToolRegistry>>) -> Self {
        Self::new(registry, 5, 3)
    }

    /// ✨ Phase 5.2: 设置执行模式
    pub fn with_execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    /// ✨ Phase 5.2: 获取当前执行模式
    pub fn execution_mode(&self) -> ExecutionMode {
        self.execution_mode
    }

    /// ✨ Phase 5.3 Week 3 Day 2: 启用工具缓存
    pub fn with_cache(mut self, cache: Arc<ToolCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// ✨ Phase 5.3 Week 3 Day 2: 获取缓存统计
    pub async fn cache_stats(&self) -> Option<CacheStats> {
        if let Some(cache) = &self.cache {
            Some(cache.stats().await)
        } else {
            None
        }
    }

    /// 执行单个工具调用
    /// ✨ Phase 5.3 Week 3 Day 2: 支持缓存
    pub async fn execute_tool_call(
        &self,
        call: &ToolCallRequest,
    ) -> ToolCallResult {
        let start = Instant::now();

        // ✨ 尝试从缓存获取
        if let Some(cache) = &self.cache {
            if let Some(cached_content) = cache.get(&call.name, &call.arguments).await {
                return ToolCallResult {
                    call_id: call.id.clone(),
                    tool_name: call.name.clone(),
                    success: true,
                    content: cached_content,
                    duration_ms: start.elapsed().as_millis() as u64, // 缓存命中很快
                };
            }
        }

        // 缓存未命中，执行工具
        let registry = self.registry.read().await;

        let result = match registry.execute(&call.name, call.arguments.clone()) {
            Ok(content) => {
                // ✨ 成功时写入缓存
                if let Some(cache) = &self.cache {
                    cache.set(&call.name, &call.arguments, content.clone()).await;
                }

                ToolCallResult {
                    call_id: call.id.clone(),
                    tool_name: call.name.clone(),
                    success: true,
                    content,
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            },
            Err(error) => ToolCallResult {
                call_id: call.id.clone(),
                tool_name: call.name.clone(),
                success: false,
                content: format!("工具执行失败: {}", error),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        };

        result
    }

    /// 执行多个工具调用
    /// ✨ Phase 5.2: 支持并行执行
    pub async fn execute_tool_calls(
        &self,
        calls: &[ToolCallRequest],
    ) -> Vec<ToolCallResult> {
        // 限制单轮工具数量
        let limited_calls = if calls.len() > self.max_tools_per_round {
            &calls[..self.max_tools_per_round]
        } else {
            calls
        };

        match self.execution_mode {
            ExecutionMode::Sequential => {
                // 串行执行（保持原有行为）
                let mut results = Vec::new();
                for call in limited_calls {
                    results.push(self.execute_tool_call(call).await);
                }
                results
            }
            ExecutionMode::Parallel => {
                // ✨ 并行执行（Phase 5.2 新增）
                let futures: Vec<_> = limited_calls
                    .iter()
                    .map(|call| self.execute_tool_call(call))
                    .collect();

                futures::future::join_all(futures).await
            }
        }
    }

    /// 执行迭代工具链
    ///
    /// 这个方法会：
    /// 1. 发送用户消息给 LLM (带工具 Schema)
    /// 2. 如果 LLM 返回工具调用，执行工具
    /// 3. 将工具结果发送回 LLM
    /// 4. 重复步骤 2-3，最多 max_iterations 轮
    /// 5. 返回最终的文本响应
    pub async fn execute_iterative(
        &self,
        llm: &dyn LlmClient,
        initial_message: &str,
        tool_schemas: Vec<JsonValue>,
    ) -> Result<String, String> {
        let mut messages = vec![Message::user(initial_message)];
        let mut iteration = 0;

        loop {
            iteration += 1;

            // 检查迭代次数限制
            if iteration > self.max_iterations {
                return Err(format!(
                    "达到最大迭代次数 ({}), 工具调用可能陷入循环",
                    self.max_iterations
                ));
            }

            // 调用 LLM with tools
            let response = llm
                .chat_with_tools(messages.clone(), tool_schemas.clone())
                .await
                .map_err(|e: LlmError| format!("LLM 调用失败: {}", e))?;

            // 如果是最终响应（没有工具调用），返回结果
            if response.is_final {
                return Ok(response.content.unwrap_or_default());
            }

            // 有工具调用，需要执行
            if response.tool_calls.is_empty() {
                // 不应该发生：is_final=false 但没有工具调用
                return Err("LLM 响应异常: is_final=false 但没有工具调用".to_string());
            }

            // 转换 LLM ToolCall 为内部 ToolCallRequest
            let tool_requests: Vec<ToolCallRequest> = response
                .tool_calls
                .iter()
                .map(|tc| ToolCallRequest {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments: serde_json::from_str(&tc.function.arguments)
                        .unwrap_or(JsonValue::Object(serde_json::Map::new())),
                })
                .collect();

            // ⚠️ 限制工具调用数量，确保 assistant 消息中的 tool_calls 与实际执行的一致
            let limited_tool_calls = if response.tool_calls.len() > self.max_tools_per_round {
                response.tool_calls[..self.max_tools_per_round].to_vec()
            } else {
                response.tool_calls.clone()
            };

            // 执行工具调用（execute_tool_calls 内部也会限制，这里保持一致）
            let tool_results = self.execute_tool_calls(&tool_requests).await;

            // 将助手的工具调用添加到消息历史（只包含实际执行的工具调用）
            messages.push(Message::assistant_with_tools(limited_tool_calls));

            // 将工具结果添加到消息历史
            for result in tool_results {
                messages.push(Message::tool_result(result.call_id, result.content));
            }

            // 继续下一轮迭代
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{Parameter, ParameterType, Tool};
    use serde_json::json;

    fn create_test_registry() -> Arc<RwLock<ToolRegistry>> {
        let mut registry = ToolRegistry::new();

        let test_tool = Tool::new(
            "test_add",
            "测试加法工具",
            vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: ParameterType::Number,
                    description: "第一个数".to_string(),
                    required: true,
                    default: None,
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: ParameterType::Number,
                    description: "第二个数".to_string(),
                    required: true,
                    default: None,
                },
            ],
            |args| {
                let a = args["a"].as_f64().ok_or("a 必须是数字")?;
                let b = args["b"].as_f64().ok_or("b 必须是数字")?;
                Ok(format!("result: {}", a + b))
            },
        );

        registry.register(test_tool);
        Arc::new(RwLock::new(registry))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execute_tool_call() {
        let registry = create_test_registry();
        let executor = ToolExecutor::with_defaults(registry);

        let call = ToolCallRequest {
            id: "call_123".to_string(),
            name: "test_add".to_string(),
            arguments: json!({"a": 10, "b": 5}),
        };

        let result = executor.execute_tool_call(&call).await;

        assert!(result.success);
        assert!(result.content.contains("15"));
        assert_eq!(result.call_id, "call_123");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execute_tool_calls_limit() {
        let registry = create_test_registry();
        let executor = ToolExecutor::new(registry, 5, 2); // 限制每轮最多 2 个工具

        let calls = vec![
            ToolCallRequest {
                id: "call_1".to_string(),
                name: "test_add".to_string(),
                arguments: json!({"a": 1, "b": 1}),
            },
            ToolCallRequest {
                id: "call_2".to_string(),
                name: "test_add".to_string(),
                arguments: json!({"a": 2, "b": 2}),
            },
            ToolCallRequest {
                id: "call_3".to_string(),
                name: "test_add".to_string(),
                arguments: json!({"a": 3, "b": 3}),
            },
        ];

        let results = executor.execute_tool_calls(&calls).await;

        // 应该只执行前 2 个
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].call_id, "call_1");
        assert_eq!(results[1].call_id, "call_2");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execute_tool_call_error() {
        let registry = create_test_registry();
        let executor = ToolExecutor::with_defaults(registry);

        // 缺少必需参数
        let call = ToolCallRequest {
            id: "call_err".to_string(),
            name: "test_add".to_string(),
            arguments: json!({"a": 10}), // 缺少 b
        };

        let result = executor.execute_tool_call(&call).await;

        assert!(!result.success);
        assert!(result.content.contains("失败") || result.content.contains("参数"));
    }

    // ========== ✨ Phase 5.2 新增测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_parallel_execution() {
        let registry = create_test_registry();
        let executor = ToolExecutor::with_defaults(registry);

        // 确认默认是并行模式
        assert_eq!(executor.execution_mode(), ExecutionMode::Parallel);

        let calls = vec![
            ToolCallRequest {
                id: "call_1".to_string(),
                name: "test_add".to_string(),
                arguments: json!({"a": 1, "b": 1}),
            },
            ToolCallRequest {
                id: "call_2".to_string(),
                name: "test_add".to_string(),
                arguments: json!({"a": 2, "b": 2}),
            },
            ToolCallRequest {
                id: "call_3".to_string(),
                name: "test_add".to_string(),
                arguments: json!({"a": 3, "b": 3}),
            },
        ];

        let results = executor.execute_tool_calls(&calls).await;

        // 应该执行所有 3 个工具
        assert_eq!(results.len(), 3);
        assert!(results[0].success);
        assert!(results[1].success);
        assert!(results[2].success);

        // 验证结果正确性
        assert!(results[0].content.contains("2")); // 1+1
        assert!(results[1].content.contains("4")); // 2+2
        assert!(results[2].content.contains("6")); // 3+3
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_sequential_execution() {
        let registry = create_test_registry();
        let executor = ToolExecutor::with_defaults(registry)
            .with_execution_mode(ExecutionMode::Sequential);

        assert_eq!(executor.execution_mode(), ExecutionMode::Sequential);

        let calls = vec![
            ToolCallRequest {
                id: "call_1".to_string(),
                name: "test_add".to_string(),
                arguments: json!({"a": 5, "b": 5}),
            },
            ToolCallRequest {
                id: "call_2".to_string(),
                name: "test_add".to_string(),
                arguments: json!({"a": 10, "b": 10}),
            },
        ];

        let results = executor.execute_tool_calls(&calls).await;

        assert_eq!(results.len(), 2);
        assert!(results[0].content.contains("10")); // 5+5
        assert!(results[1].content.contains("20")); // 10+10

        // 串行执行时，结果应该按顺序返回
        assert_eq!(results[0].call_id, "call_1");
        assert_eq!(results[1].call_id, "call_2");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execution_statistics() {
        let registry = create_test_registry();
        let executor = ToolExecutor::with_defaults(registry);

        let call = ToolCallRequest {
            id: "call_timing".to_string(),
            name: "test_add".to_string(),
            arguments: json!({"a": 100, "b": 200}),
        };

        let result = executor.execute_tool_call(&call).await;

        // 验证执行统计
        assert!(result.success);
        assert!(result.content.contains("300"));

        // 执行时间应该小于 1000ms（正常计算应该很快）
        assert!(result.duration_ms < 1000, "执行时间异常: {} ms", result.duration_ms);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execution_mode_switch() {
        // 测试并行和串行模式的切换
        let registry = create_test_registry();

        // 创建并行执行器
        let parallel_executor = ToolExecutor::with_defaults(Arc::clone(&registry))
            .with_execution_mode(ExecutionMode::Parallel);
        assert_eq!(parallel_executor.execution_mode(), ExecutionMode::Parallel);

        // 创建串行执行器
        let sequential_executor = ToolExecutor::with_defaults(Arc::clone(&registry))
            .with_execution_mode(ExecutionMode::Sequential);
        assert_eq!(sequential_executor.execution_mode(), ExecutionMode::Sequential);

        // 两种模式都应该能正确执行
        let calls = vec![
            ToolCallRequest {
                id: "call_1".to_string(),
                name: "test_add".to_string(),
                arguments: json!({"a": 1, "b": 2}),
            },
            ToolCallRequest {
                id: "call_2".to_string(),
                name: "test_add".to_string(),
                arguments: json!({"a": 3, "b": 4}),
            },
        ];

        let parallel_results = parallel_executor.execute_tool_calls(&calls).await;
        let sequential_results = sequential_executor.execute_tool_calls(&calls).await;

        // 两种模式结果应该一致
        assert_eq!(parallel_results.len(), 2);
        assert_eq!(sequential_results.len(), 2);

        assert!(parallel_results[0].content.contains("3")); // 1+2
        assert!(parallel_results[1].content.contains("7")); // 3+4
        assert!(sequential_results[0].content.contains("3"));
        assert!(sequential_results[1].content.contains("7"));

        println!("✓ 并行和串行模式功能验证通过");
        println!("注意：当前工具是同步的，真正的性能提升需要异步工具支持（Phase 5.3+）");
    }
}
