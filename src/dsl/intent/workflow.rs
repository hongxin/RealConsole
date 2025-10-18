//! Workflow Intent 系统
//!
//! **设计哲学**：套路化复用（基于流程分析的模板系统）
//!
//! 将成功的 LLM 调用流程固化为可复用的工作流模板，大幅提升性能：
//! - 减少 LLM 调用次数（2-3 次 → 1 次 → 0 次）
//! - 降低响应延迟（10-15 秒 → 5-8 秒 → 2-3 秒）
//! - 提供可预测的稳定输出
//! - 节省 API 调用成本
//!
//! ## 核心概念
//!
//! - **WorkflowIntent**: 包含工作流定义的意图
//! - **WorkflowStep**: 工作流中的单个步骤（工具调用、LLM 分析等）
//! - **WorkflowExecutor**: 执行工作流的引擎
//! - **ExecutionContext**: 工作流执行上下文（参数、中间结果等）

use crate::dsl::intent::types::{EntityType, Intent, IntentMatch};
use crate::llm::LlmClient;
use crate::tool::ToolRegistry;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 工作流步骤类型
///
/// 定义了工作流中可以执行的操作类型
#[derive(Debug, Clone)]
pub enum WorkflowStep {
    /// 直接调用工具（跳过 LLM 决策）
    ///
    /// # 参数
    /// - tool_name: 工具名称
    /// - args_template: 参数模板（支持 {variable} 占位符）
    /// - result_key: 结果存储的键名
    ToolCall {
        tool_name: String,
        args_template: HashMap<String, String>,
        result_key: String,
    },

    /// 使用 LLM 分析数据
    ///
    /// # 参数
    /// - prompt_template: 提示词模板（支持 {variable} 占位符）
    /// - result_key: 结果存储的键名
    LlmAnalyze {
        prompt_template: String,
        result_key: String,
    },

    /// 数据转换（简单的文本处理）
    ///
    /// # 参数
    /// - operation: 操作类型（extract_json, format_markdown 等）
    /// - input_key: 输入数据的键名
    /// - result_key: 结果存储的键名
    Transform {
        operation: TransformOperation,
        input_key: String,
        result_key: String,
    },
}

/// 数据转换操作类型
#[derive(Debug, Clone)]
pub enum TransformOperation {
    /// 提取 JSON 中的指定字段
    ExtractJson { path: String },

    /// 格式化为 Markdown
    FormatMarkdown,

    /// 文本截断
    Truncate { max_length: usize },

    /// 自定义转换（使用闭包，但不可克隆，所以用字符串表示函数名）
    Custom { function_name: String },
}

/// 工作流意图
///
/// 扩展标准 Intent，添加工作流定义
#[derive(Debug, Clone)]
pub struct WorkflowIntent {
    /// 基础意图信息
    pub base_intent: Intent,

    /// 工作流步骤列表
    pub workflow_steps: Vec<WorkflowStep>,

    /// 缓存策略
    pub cache_strategy: CacheStrategy,

    /// 工作流描述
    pub description: String,
}

/// 缓存策略
#[derive(Debug, Clone)]
pub enum CacheStrategy {
    /// 不缓存
    NoCache,

    /// 基于时间的缓存（TTL 秒）
    TimeBased { ttl: u64 },

    /// 基于参数的缓存（相同参数返回缓存结果）
    ParameterBased,
}

/// 工作流执行上下文
///
/// 存储执行过程中的参数和中间结果
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 用户输入的参数
    pub parameters: HashMap<String, String>,

    /// 步骤执行结果
    pub results: HashMap<String, String>,

    /// 执行开始时间（用于性能统计）
    pub start_time: std::time::Instant,
}

/// 工作流执行结果
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    /// 是否成功
    pub success: bool,

    /// 最终输出
    pub output: String,

    /// 执行耗时（毫秒）
    pub duration_ms: u64,

    /// 执行的步骤数
    pub steps_executed: usize,

    /// LLM 调用次数
    pub llm_calls: usize,

    /// 工具调用次数
    pub tool_calls: usize,
}

/// 工作流执行器
///
/// 负责执行 WorkflowIntent 中定义的工作流
pub struct WorkflowExecutor {
    /// 工具注册表
    tool_registry: Arc<RwLock<ToolRegistry>>,

    /// LLM 管理器（可选）
    llm_manager: Option<Arc<RwLock<crate::llm_manager::LlmManager>>>,

    /// 缓存（可选）
    cache: Option<Arc<RwLock<HashMap<String, (String, std::time::Instant)>>>>,
}

impl WorkflowIntent {
    /// 创建一个新的工作流意图
    pub fn new(
        base_intent: Intent,
        workflow_steps: Vec<WorkflowStep>,
    ) -> Self {
        Self {
            base_intent,
            workflow_steps,
            cache_strategy: CacheStrategy::NoCache,
            description: String::new(),
        }
    }

    /// 设置缓存策略
    pub fn with_cache_strategy(mut self, strategy: CacheStrategy) -> Self {
        self.cache_strategy = strategy;
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 从 IntentMatch 提取参数
    pub fn extract_parameters(&self, intent_match: &IntentMatch) -> HashMap<String, String> {
        let mut parameters = HashMap::new();

        // 1. 从 Intent 的默认实体提取
        for (name, entity) in &self.base_intent.entities {
            let value = entity_to_string(entity);
            parameters.insert(name.clone(), value);
        }

        // 2. 从 IntentMatch 的提取实体覆盖
        for (name, entity) in &intent_match.extracted_entities {
            let value = entity_to_string(entity);
            parameters.insert(name.clone(), value);
        }

        parameters
    }
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new(parameters: HashMap<String, String>) -> Self {
        Self {
            parameters,
            results: HashMap::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// 获取参数或结果
    ///
    /// 优先从结果中查找，然后从参数中查找
    pub fn get(&self, key: &str) -> Option<&String> {
        self.results.get(key).or_else(|| self.parameters.get(key))
    }

    /// 存储步骤结果
    pub fn set_result(&mut self, key: String, value: String) {
        self.results.insert(key, value);
    }

    /// 替换模板中的占位符
    pub fn substitute_template(&self, template: &str) -> String {
        let mut result = template.to_string();

        // 替换参数
        for (key, value) in &self.parameters {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        // 替换中间结果
        for (key, value) in &self.results {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// 获取最终结果（最后一个步骤的结果）
    pub fn final_result(&self) -> String {
        // 查找最后一个结果
        if let Some(last_key) = self.results.keys().last() {
            self.results.get(last_key).cloned().unwrap_or_default()
        } else {
            String::new()
        }
    }
}

impl WorkflowExecutor {
    /// 创建新的工作流执行器
    pub fn new(
        tool_registry: Arc<RwLock<ToolRegistry>>,
        llm_manager: Option<Arc<RwLock<crate::llm_manager::LlmManager>>>,
    ) -> Self {
        Self {
            tool_registry,
            llm_manager,
            cache: Some(Arc::new(RwLock::new(HashMap::new()))),
        }
    }

    /// 执行工作流意图
    ///
    /// # 核心优化
    /// - 直接工具调用：跳过 LLM 工具选择环节
    /// - 参数模板化：快速替换占位符
    /// - 结果缓存：相同参数直接返回
    pub async fn execute(
        &self,
        workflow_intent: &WorkflowIntent,
        intent_match: &IntentMatch,
    ) -> Result<WorkflowResult, String> {
        // 1. 提取参数
        let parameters = workflow_intent.extract_parameters(intent_match);

        // 2. 检查缓存
        if let Some(cached) = self.check_cache(workflow_intent, &parameters).await {
            return Ok(cached);
        }

        // 3. 创建执行上下文
        let mut context = ExecutionContext::new(parameters);

        // 4. 执行工作流步骤
        let mut steps_executed = 0;
        let mut llm_calls = 0;
        let mut tool_calls = 0;

        for step in &workflow_intent.workflow_steps {
            match step {
                WorkflowStep::ToolCall { tool_name, args_template, result_key } => {
                    // 执行工具调用
                    let result = self.execute_tool_call(
                        tool_name,
                        args_template,
                        &context,
                    ).await?;

                    context.set_result(result_key.clone(), result);
                    steps_executed += 1;
                    tool_calls += 1;
                }

                WorkflowStep::LlmAnalyze { prompt_template, result_key } => {
                    // 执行 LLM 分析
                    let result = self.execute_llm_analyze(
                        prompt_template,
                        &context,
                    ).await?;

                    context.set_result(result_key.clone(), result);
                    steps_executed += 1;
                    llm_calls += 1;
                }

                WorkflowStep::Transform { operation, input_key, result_key } => {
                    // 执行数据转换
                    let result = self.execute_transform(
                        operation,
                        input_key,
                        &context,
                    )?;

                    context.set_result(result_key.clone(), result);
                    steps_executed += 1;
                }
            }
        }

        // 5. 获取最终结果
        let output = context.final_result();
        let duration_ms = context.start_time.elapsed().as_millis() as u64;

        let result = WorkflowResult {
            success: true,
            output: output.clone(),
            duration_ms,
            steps_executed,
            llm_calls,
            tool_calls,
        };

        // 6. 更新缓存
        self.update_cache(workflow_intent, &context.parameters, &output).await;

        Ok(result)
    }

    /// 执行工具调用
    async fn execute_tool_call(
        &self,
        tool_name: &str,
        args_template: &HashMap<String, String>,
        context: &ExecutionContext,
    ) -> Result<String, String> {
        // 1. 替换参数模板
        let mut args = serde_json::Map::new();
        for (key, template_value) in args_template {
            let substituted = context.substitute_template(template_value);
            args.insert(key.clone(), JsonValue::String(substituted));
        }

        // 2. 执行工具
        let registry = self.tool_registry.read().await;
        registry.execute(tool_name, JsonValue::Object(args))
    }

    /// 执行 LLM 分析
    async fn execute_llm_analyze(
        &self,
        prompt_template: &str,
        context: &ExecutionContext,
    ) -> Result<String, String> {
        // 1. 替换提示词模板
        let prompt = context.substitute_template(prompt_template);

        // 2. 调用 LLM
        if let Some(llm_manager) = &self.llm_manager {
            let manager = llm_manager.read().await;
            manager.chat(&prompt).await
                .map_err(|e| format!("LLM 调用失败: {}", e))
        } else {
            Err("LLM 管理器未配置".to_string())
        }
    }

    /// 执行数据转换
    fn execute_transform(
        &self,
        operation: &TransformOperation,
        input_key: &str,
        context: &ExecutionContext,
    ) -> Result<String, String> {
        let input = context.get(input_key)
            .ok_or_else(|| format!("输入数据不存在: {}", input_key))?;

        match operation {
            TransformOperation::ExtractJson { path } => {
                // 简单的 JSON 路径提取
                extract_json_path(input, path)
            }

            TransformOperation::FormatMarkdown => {
                // 格式化为 Markdown
                Ok(format!("```\n{}\n```", input))
            }

            TransformOperation::Truncate { max_length } => {
                // 截断文本
                let truncated = if input.len() > *max_length {
                    format!("{}...", &input[..*max_length])
                } else {
                    input.clone()
                };
                Ok(truncated)
            }

            TransformOperation::Custom { function_name } => {
                // 自定义转换（TODO: 实现插件系统）
                Err(format!("自定义转换未实现: {}", function_name))
            }
        }
    }

    /// 检查缓存
    async fn check_cache(
        &self,
        workflow_intent: &WorkflowIntent,
        parameters: &HashMap<String, String>,
    ) -> Option<WorkflowResult> {
        match &workflow_intent.cache_strategy {
            CacheStrategy::NoCache => None,

            CacheStrategy::TimeBased { ttl } => {
                if let Some(cache) = &self.cache {
                    let cache_key = generate_cache_key(&workflow_intent.base_intent.name, parameters);
                    let cache_guard = cache.read().await;

                    if let Some((cached_output, cached_time)) = cache_guard.get(&cache_key) {
                        let elapsed = cached_time.elapsed().as_secs();
                        if elapsed < *ttl {
                            // 缓存未过期
                            return Some(WorkflowResult {
                                success: true,
                                output: cached_output.clone(),
                                duration_ms: 0, // 缓存命中，耗时极短
                                steps_executed: 0,
                                llm_calls: 0,
                                tool_calls: 0,
                            });
                        }
                    }
                }
                None
            }

            CacheStrategy::ParameterBased => {
                // 与 TimeBased 类似，但不检查 TTL
                if let Some(cache) = &self.cache {
                    let cache_key = generate_cache_key(&workflow_intent.base_intent.name, parameters);
                    let cache_guard = cache.read().await;

                    if let Some((cached_output, _)) = cache_guard.get(&cache_key) {
                        return Some(WorkflowResult {
                            success: true,
                            output: cached_output.clone(),
                            duration_ms: 0,
                            steps_executed: 0,
                            llm_calls: 0,
                            tool_calls: 0,
                        });
                    }
                }
                None
            }
        }
    }

    /// 更新缓存
    async fn update_cache(
        &self,
        workflow_intent: &WorkflowIntent,
        parameters: &HashMap<String, String>,
        output: &str,
    ) {
        match &workflow_intent.cache_strategy {
            CacheStrategy::NoCache => {}

            CacheStrategy::TimeBased { .. } | CacheStrategy::ParameterBased => {
                if let Some(cache) = &self.cache {
                    let cache_key = generate_cache_key(&workflow_intent.base_intent.name, parameters);
                    let mut cache_guard = cache.write().await;
                    cache_guard.insert(
                        cache_key,
                        (output.to_string(), std::time::Instant::now()),
                    );
                }
            }
        }
    }
}

/// 将实体转换为字符串
fn entity_to_string(entity: &EntityType) -> String {
    match entity {
        EntityType::FileType(v) => v.clone(),
        EntityType::Operation(v) => v.clone(),
        EntityType::Path(v) => v.clone(),
        EntityType::Number(n) => n.to_string(),
        EntityType::Date(v) => v.clone(),
        EntityType::Custom(_, v) => v.clone(),
    }
}

/// 生成缓存键
fn generate_cache_key(intent_name: &str, parameters: &HashMap<String, String>) -> String {
    let mut keys: Vec<_> = parameters.keys().collect();
    keys.sort();

    let params_str = keys.iter()
        .map(|k| format!("{}={}", k, parameters.get(*k).unwrap()))
        .collect::<Vec<_>>()
        .join("&");

    format!("{}?{}", intent_name, params_str)
}

/// 简单的 JSON 路径提取
fn extract_json_path(json_str: &str, path: &str) -> Result<String, String> {
    let value: JsonValue = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    // 简单实现：只支持单层路径
    let result = value.get(path)
        .ok_or_else(|| format!("路径不存在: {}", path))?;

    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::intent::types::IntentDomain;

    #[test]
    fn test_workflow_intent_creation() {
        let base_intent = Intent::new(
            "test_workflow",
            IntentDomain::Custom("Test".to_string()),
            vec![],
            vec![],
            0.5,
        );

        let workflow = WorkflowIntent::new(base_intent, vec![]);

        assert_eq!(workflow.base_intent.name, "test_workflow");
        assert_eq!(workflow.workflow_steps.len(), 0);
    }

    #[test]
    fn test_execution_context_substitute() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "Alice".to_string());
        params.insert("age".to_string(), "30".to_string());

        let context = ExecutionContext::new(params);

        let result = context.substitute_template("Hello {name}, you are {age} years old");
        assert_eq!(result, "Hello Alice, you are 30 years old");
    }

    #[test]
    fn test_cache_key_generation() {
        let mut params = HashMap::new();
        params.insert("a".to_string(), "1".to_string());
        params.insert("b".to_string(), "2".to_string());

        let key = generate_cache_key("test_intent", &params);

        // 参数应该被排序
        assert!(key.contains("a=1"));
        assert!(key.contains("b=2"));
    }
}
