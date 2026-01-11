//! 工具执行引擎
//!
//! 负责：
//! - 迭代工具调用控制 (最多 5 轮)
//! - 单轮工具数量限制 (最多 3 个)
//! - LLM Function Calling 集成
//! - 工具结果反馈
//! - ✨ Phase 5.2: 并行工具执行 + 执行统计
//! - ✨ Phase 5.3 Week 3 Day 2: 工具响应缓存
//! - ✨ v1.90.0: 智能并行执行 + 依赖分析 + DAG可视化

pub mod parallel; // ✨ v1.90.0: 并行执行增强

use crate::llm::{LlmClient, LlmError, Message};
use crate::tool::ToolRegistry;
use crate::tool_cache::{CacheStats, ToolCache};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

// Re-export parallel module types
pub use parallel::{DependencyAnalyzer, ExecutionDAG, ExecutionStage, ToolDependency};

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

/// 对话轮次详情（用于 Debug 模式调试）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRound {
    /// 轮次编号（从 1 开始）
    pub round: usize,

    /// 用户消息（或上一轮的工具结果摘要）
    pub input_summary: String,

    /// LLM 响应内容
    pub assistant_response: Option<String>,

    /// 工具调用列表
    pub tool_calls: Vec<ToolCallSummary>,

    /// 工具执行结果
    pub tool_results: Vec<String>,

    /// 本轮消息数量（估算 tokens）
    pub message_count: usize,

    /// 本轮耗时（毫秒）
    pub duration_ms: u64,
}

/// 工具调用摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallSummary {
    pub name: String,
    pub arguments: String,
}

/// ✨ Phase 5.2: 工具执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// 串行执行（保持顺序，适合有依赖的工具链）
    Sequential,
    /// 并行执行（性能优先，适合独立工具）
    Parallel,
    /// ✨ v1.90.0: 智能执行（自动分析依赖，分阶段执行）
    Smart,
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

    /// ✨ v1.90.0: 依赖分析器
    dependency_analyzer: DependencyAnalyzer,
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
            cache: None,                             // 默认不启用缓存
            dependency_analyzer: DependencyAnalyzer::new(),
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
    /// ✨ Phase 5.2: 添加执行时间统计
    /// ✨ Phase 5.3 Week 3 Day 2: 添加缓存支持
    pub async fn execute_tool_call(&self, call: &ToolCallRequest) -> ToolCallResult {
        let start = Instant::now();

        // ✨ Phase 5.3: 检查缓存
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get(&call.name, &call.arguments).await {
                return ToolCallResult {
                    call_id: call.id.clone(),
                    tool_name: call.name.clone(),
                    success: true,
                    content: format!("[缓存] {}", cached),
                    duration_ms: start.elapsed().as_millis() as u64,
                };
            }
        }

        // 获取工具并执行
        let registry = self.registry.read().await;
        let result = match registry.execute(&call.name, call.arguments.clone()) {
            Ok(output) => {
                // ✨ Phase 5.3: 缓存成功结果
                if let Some(cache) = &self.cache {
                    cache.set(&call.name, &call.arguments, output.clone()).await;
                }

                ToolCallResult {
                    call_id: call.id.clone(),
                    tool_name: call.name.clone(),
                    success: true,
                    content: output,
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
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
    /// ✨ v1.90.0: 支持智能执行（依赖分析）
    pub async fn execute_tool_calls(&self, calls: &[ToolCallRequest]) -> Vec<ToolCallResult> {
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
            ExecutionMode::Smart => {
                // ✨ v1.90.0: 智能执行
                self.execute_smart(limited_calls).await
            }
        }
    }

    /// ✨ v1.90.0: 智能执行 - 分析依赖并分阶段执行
    async fn execute_smart(&self, calls: &[ToolCallRequest]) -> Vec<ToolCallResult> {
        // 分析依赖关系
        let dag = self.dependency_analyzer.analyze(calls);

        // 按阶段执行
        let mut all_results = Vec::new();
        let mut completed_results: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for stage in dag.stages() {
            let stage_calls: Vec<_> = stage
                .tool_ids
                .iter()
                .filter_map(|id| calls.iter().find(|c| &c.id == id))
                .cloned()
                .collect();

            if stage_calls.is_empty() {
                continue;
            }

            // 并行执行当前阶段的所有工具
            let futures: Vec<_> = stage_calls
                .iter()
                .map(|call| self.execute_tool_call(call))
                .collect();

            let stage_results = futures::future::join_all(futures).await;

            // 记录结果
            for result in &stage_results {
                completed_results.insert(result.call_id.clone(), result.content.clone());
            }

            all_results.extend(stage_results);
        }

        all_results
    }

    /// ✨ v1.90.0: 获取执行计划DAG（用于可视化）
    pub fn plan_execution(&self, calls: &[ToolCallRequest]) -> ExecutionDAG {
        self.dependency_analyzer.analyze(calls)
    }

    /// 执行迭代工具链
    ///
    /// 这个方法会：
    /// 1. 发送用户消息给 LLM (带工具 Schema)
    /// 2. 如果 LLM 返回工具调用，执行工具
    /// 3. 将工具结果发送回 LLM
    /// 4. 重复步骤 2-3，最多 max_iterations 轮
    /// 5. 返回最终的文本响应
    ///
    /// # 参数
    /// - `llm`: LLM 客户端
    /// - `messages`: 包含历史对话的消息列表（支持多轮上下文）
    /// - `tool_schemas`: 工具 schema 列表
    pub async fn execute_iterative(
        &self,
        llm: &dyn LlmClient,
        messages: Vec<Message>,
        tool_schemas: Vec<JsonValue>,
    ) -> Result<String, String> {
        let mut messages = messages;
        let mut iteration = 0;
        let mut conversation_rounds = Vec::new(); // ✨ 记录对话轮次（用于 debug）

        loop {
            iteration += 1;
            let round_start = Instant::now();

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
                .map_err(|e: LlmError| {
                    // ✨ 在错误时，将对话轮次信息附加到错误消息中
                    let error_msg = format!("LLM 调用失败: {}", e);
                    let debug_info = Self::encode_debug_info(&conversation_rounds);
                    format!("{}__DEBUG__{}", error_msg, debug_info)
                })?;

            // 如果是最终响应（没有工具调用），返回结果
            if response.is_final {
                let final_response = response.content.unwrap_or_default();

                // ✨ 在成功时也附加调试信息（用于 debug 模式显示）
                let debug_info = Self::encode_debug_info(&conversation_rounds);
                return Ok(format!("{}__DEBUG__{}", final_response, debug_info));
            }

            // 有工具调用，需要执行
            if response.tool_calls.is_empty() {
                // 不应该发生：is_final=false 但没有工具调用
                return Err("LLM 响应异常: is_final=false 但没有工具调用".to_string());
            }

            // 限制单轮工具调用数量
            let limited_tool_calls: Vec<_> = response
                .tool_calls
                .iter()
                .take(self.max_tools_per_round)
                .cloned()
                .collect();

            // 构建工具调用请求
            let tool_requests: Vec<ToolCallRequest> = limited_tool_calls
                .iter()
                .map(|tc| ToolCallRequest {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments: serde_json::from_str(&tc.function.arguments)
                        .unwrap_or(JsonValue::Null),
                })
                .collect();

            // 执行工具调用
            let tool_results = self.execute_tool_calls(&tool_requests).await;

            // 记录工具调用摘要
            let tool_summaries: Vec<_> = limited_tool_calls
                .iter()
                .map(|tc| ToolCallSummary {
                    name: tc.function.name.clone(),
                    arguments: tc.function.arguments.clone(),
                })
                .collect();

            // 记录工具执行结果
            let tool_result_strs: Vec<String> =
                tool_results.iter().map(|r| r.content.clone()).collect();

            // ✨ 记录对话轮次
            conversation_rounds.push(ConversationRound {
                round: iteration,
                input_summary: if iteration == 1 {
                    messages
                        .last()
                        .and_then(|m| m.content.clone())
                        .unwrap_or_else(|| "...".to_string())
                } else {
                    "工具结果".to_string()
                },
                assistant_response: response.content.clone(),
                tool_calls: tool_summaries,
                tool_results: tool_result_strs.clone(),
                message_count: messages.len() + 2 + tool_results.len(), // 估算
                duration_ms: round_start.elapsed().as_millis() as u64,
            });

            // 添加助手消息（包含工具调用）
            messages.push(Message {
                role: crate::llm::MessageRole::Assistant,
                content: response.content,
                tool_calls: Some(limited_tool_calls),
                tool_call_id: None,
            });

            // 添加工具结果消息
            for result in tool_results {
                messages.push(Message {
                    role: crate::llm::MessageRole::Tool,
                    content: Some(result.content),
                    tool_calls: None,
                    tool_call_id: Some(result.call_id),
                });
            }
        }
    }

    /// 将对话轮次信息编码为 JSON（用于 debug 模式）
    fn encode_debug_info(rounds: &[ConversationRound]) -> String {
        serde_json::to_string(rounds).unwrap_or_else(|_| "[]".to_string())
    }

    /// 解码调试信息（返回对话轮次，如果有的话）
    #[allow(dead_code)]
    pub fn decode_debug_info(response: &str) -> Option<Vec<ConversationRound>> {
        if let Some(pos) = response.find("__DEBUG__") {
            let debug_str = &response[pos + 9..];
            let rounds: Vec<ConversationRound> =
                serde_json::from_str(debug_str).unwrap_or_default();
            if rounds.is_empty() {
                None
            } else {
                Some(rounds)
            }
        } else {
            None
        }
    }

    /// 解码调试信息并提取内容和轮次
    #[allow(dead_code)]
    pub fn decode_debug_info_full(response: &str) -> (String, Vec<ConversationRound>) {
        if let Some(pos) = response.find("__DEBUG__") {
            let content = response[..pos].to_string();
            let debug_str = &response[pos + 9..];
            let rounds: Vec<ConversationRound> =
                serde_json::from_str(debug_str).unwrap_or_default();
            (content, rounds)
        } else {
            (response.to_string(), Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_mode() {
        let mode = ExecutionMode::Smart;
        assert_eq!(mode, ExecutionMode::Smart);
    }

    #[test]
    fn test_tool_call_request_serde() {
        let request = ToolCallRequest {
            id: "call_123".to_string(),
            name: "test_tool".to_string(),
            arguments: serde_json::json!({"key": "value"}),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: ToolCallRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "call_123");
        assert_eq!(parsed.name, "test_tool");
    }

    #[test]
    fn test_decode_debug_info() {
        let response = "Hello World__DEBUG__[{\"round\":1,\"input_summary\":\"\",\"tool_calls\":[],\"tool_results\":[],\"message_count\":0,\"duration_ms\":0}]";
        let rounds = ToolExecutor::decode_debug_info(response);
        assert!(rounds.is_some());
        assert_eq!(rounds.unwrap().len(), 1);
    }

    #[test]
    fn test_decode_debug_info_no_debug() {
        let response = "Hello World";
        let rounds = ToolExecutor::decode_debug_info(response);
        assert!(rounds.is_none());
    }

    #[test]
    fn test_decode_debug_info_full() {
        let response = "Hello World__DEBUG__[{\"round\":1,\"input_summary\":\"\",\"tool_calls\":[],\"tool_results\":[],\"message_count\":0,\"duration_ms\":0}]";
        let (content, rounds) = ToolExecutor::decode_debug_info_full(response);
        assert_eq!(content, "Hello World");
        assert_eq!(rounds.len(), 1);
    }
}
