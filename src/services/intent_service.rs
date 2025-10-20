//! Intent 服务
//!
//! 负责处理自然语言意图识别，包括：
//! - Intent DSL 匹配
//! - 参数提取
//! - Pipeline 生成
//! - 命令验证
//!
//! ## 处理流程
//! 1. LLM 驱动的 Pipeline 生成（可选）
//! 2. IntentMatcher 匹配最佳意图
//! 3. 参数提取（支持 LLM 智能提取）
//! 4. Pipeline DSL 或模板引擎生成执行计划
//! 5. LLM 验证命令（可选）

use crate::config::Config;
use crate::dsl::intent::{
    EntityExtractor, ExecutionPlan, IntentMatch, IntentMatcher, IntentToPipeline, LlmToPipeline,
    TemplateEngine, ValidationResult, WorkflowExecutor, WorkflowIntent,
};
use crate::llm_manager::LlmManager;
use crate::services::{Service, ServiceResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Intent 服务请求
#[derive(Debug, Clone)]
pub struct IntentRequest {
    /// 用户输入文本
    pub text: String,
    /// 是否启用 LLM 生成
    pub llm_generation_enabled: bool,
    /// 是否启用 LLM 提取
    pub llm_extraction_enabled: bool,
    /// 是否启用 LLM 验证
    pub llm_validation_enabled: bool,
    /// 是否启用 Workflow
    pub workflow_enabled: bool,
}

impl IntentRequest {
    /// 从配置创建请求
    pub fn from_config(text: String, config: &Config) -> Self {
        Self {
            text,
            llm_generation_enabled: config.intent.llm_generation_enabled.unwrap_or(false),
            llm_extraction_enabled: config.intent.llm_extraction_enabled,
            llm_validation_enabled: config.intent.llm_validation_enabled,
            workflow_enabled: config.features.workflow_enabled.unwrap_or(false),
        }
    }
}

/// Intent 服务响应
#[derive(Debug, Clone)]
pub struct IntentResponse {
    /// 执行计划（如果匹配成功）
    pub plan: Option<ExecutionPlan>,
    /// 意图名称
    pub intent_name: Option<String>,
    /// 置信度（0.0-1.0）
    pub confidence: f64,
    /// 是否使用了 Workflow
    pub is_workflow: bool,
}

/// Intent 服务错误
#[derive(Debug, Clone)]
pub enum IntentError {
    /// 没有匹配的意图
    NoMatch,
    /// LLM 错误
    LlmError(String),
    /// 模板生成失败
    TemplateError(String),
    /// 验证失败
    ValidationFailed(String),
}

impl std::fmt::Display for IntentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentError::NoMatch => write!(f, "未匹配到任何意图"),
            IntentError::LlmError(e) => write!(f, "LLM 错误: {}", e),
            IntentError::TemplateError(e) => write!(f, "模板生成失败: {}", e),
            IntentError::ValidationFailed(e) => write!(f, "验证失败: {}", e),
        }
    }
}

impl std::error::Error for IntentError {}

/// Intent 服务
///
/// 处理自然语言意图识别和执行计划生成
pub struct IntentService {
    /// Intent 匹配器
    intent_matcher: IntentMatcher,
    /// 模板引擎
    template_engine: TemplateEngine,
    /// Pipeline 转换器
    pipeline_converter: IntentToPipeline,
    /// LLM Bridge（可选）
    llm_bridge: Option<Arc<LlmToPipeline>>,
    /// Workflow Intents
    workflow_intents: Vec<WorkflowIntent>,
    /// Workflow 执行器（可选）
    workflow_executor: Option<Arc<WorkflowExecutor>>,
    /// LLM Manager
    llm_manager: Arc<RwLock<LlmManager>>,
}

impl IntentService {
    /// 创建新的 Intent 服务
    pub fn new(
        intent_matcher: IntentMatcher,
        template_engine: TemplateEngine,
        pipeline_converter: IntentToPipeline,
        llm_bridge: Option<Arc<LlmToPipeline>>,
        workflow_intents: Vec<WorkflowIntent>,
        workflow_executor: Option<Arc<WorkflowExecutor>>,
        llm_manager: Arc<RwLock<LlmManager>>,
    ) -> Self {
        Self {
            intent_matcher,
            template_engine,
            pipeline_converter,
            llm_bridge,
            workflow_intents,
            workflow_executor,
            llm_manager,
        }
    }

    /// 尝试匹配 Workflow Intent
    ///
    /// TODO: 完整实现 Workflow 支持
    /// Workflow 处理比较复杂，返回 String 而不是 ExecutionPlan
    /// 暂时简化处理，后续完善
    async fn try_match_workflow(&self, _text: &str) -> Option<ExecutionPlan> {
        // TODO: 实现 Workflow 匹配逻辑
        // 参考 agent.rs:1218 的 try_match_workflow 实现
        None
    }

    /// 尝试使用 LLM 生成 Pipeline
    async fn try_llm_generation(&self, text: &str) -> Option<ExecutionPlan> {
        if let Some(llm_bridge) = &self.llm_bridge {
            match llm_bridge.understand_and_generate(text).await {
                Ok(pipeline_plan) => {
                    let command = pipeline_plan.to_shell_command();
                    return Some(ExecutionPlan {
                        command,
                        template_name: "llm_generated".to_string(),
                        bindings: HashMap::new(),
                    });
                }
                Err(_) => {
                    // LLM 失败，继续 fallback
                    return None;
                }
            }
        }
        None
    }

    /// 匹配传统 Intent
    fn match_intent(&self, text: &str) -> Option<IntentMatch> {
        self.intent_matcher.best_match(text)
    }

    /// 生成执行计划
    fn generate_plan(&self, intent_match: &IntentMatch) -> Result<ExecutionPlan, IntentError> {
        // 优先使用 Pipeline DSL
        if let Some(pipeline_plan) = self
            .pipeline_converter
            .convert(intent_match, &intent_match.extracted_entities)
        {
            let command = pipeline_plan.to_shell_command();

            // 转换实体为字符串绑定
            let mut bindings = HashMap::new();
            for (key, entity) in &intent_match.extracted_entities {
                let value = match entity {
                    crate::dsl::intent::EntityType::Path(p) => p.clone(),
                    crate::dsl::intent::EntityType::FileType(ft) => ft.clone(),
                    crate::dsl::intent::EntityType::Number(n) => n.to_string(),
                    crate::dsl::intent::EntityType::Custom(_, v) => v.clone(),
                    crate::dsl::intent::EntityType::Operation(op) => op.clone(),
                    crate::dsl::intent::EntityType::Date(d) => d.clone(),
                };
                bindings.insert(key.clone(), value);
            }

            return Ok(ExecutionPlan {
                command,
                template_name: intent_match.intent.name.clone(),
                bindings,
            });
        }

        // 回退到模板引擎
        self.template_engine
            .generate_from_intent(intent_match)
            .map_err(|e| IntentError::TemplateError(e.to_string()))
    }
}

#[async_trait]
impl Service for IntentService {
    type Request = IntentRequest;
    type Response = IntentResponse;
    type Error = IntentError;

    async fn process(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let text = &request.text;

        // 1. 尝试 Workflow（如果启用）
        if request.workflow_enabled {
            if let Some(plan) = self.try_match_workflow(text).await {
                return Ok(IntentResponse {
                    plan: Some(plan),
                    intent_name: Some("workflow".to_string()),
                    confidence: 1.0,
                    is_workflow: true,
                });
            }
        }

        // 2. 尝试 LLM 生成（如果启用）
        if request.llm_generation_enabled {
            if let Some(plan) = self.try_llm_generation(text).await {
                return Ok(IntentResponse {
                    plan: Some(plan),
                    intent_name: Some("llm_generated".to_string()),
                    confidence: 1.0,
                    is_workflow: false,
                });
            }
        }

        // 3. 传统 Intent 匹配
        let intent_match = self.match_intent(text).ok_or(IntentError::NoMatch)?;

        // TODO: LLM 参数提取（如果启用）
        // TODO: LLM 验证（如果启用）

        // 4. 生成执行计划
        let plan = self.generate_plan(&intent_match)?;

        Ok(IntentResponse {
            plan: Some(plan),
            intent_name: Some(intent_match.intent.name.clone()),
            confidence: intent_match.confidence,
            is_workflow: false,
        })
    }

    fn name(&self) -> &str {
        "IntentService"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::intent::BuiltinIntents;

    #[tokio::test]
    async fn test_intent_service_no_match() {
        let builtins = BuiltinIntents::new();
        let intent_matcher = builtins.create_matcher();
        let template_engine = builtins.create_engine();
        let pipeline_converter = IntentToPipeline::new();
        let llm_manager = Arc::new(RwLock::new(LlmManager::new()));

        let service = IntentService::new(
            intent_matcher,
            template_engine,
            pipeline_converter,
            None,
            vec![],
            None,
            llm_manager,
        );

        let request = IntentRequest {
            text: "随机无意义的输入xyz123".to_string(),
            llm_generation_enabled: false,
            llm_extraction_enabled: false,
            llm_validation_enabled: false,
            workflow_enabled: false,
        };

        let result = service.process(request).await;
        assert!(result.is_err());
        match result {
            Err(IntentError::NoMatch) => {} // 预期错误
            _ => panic!("Expected NoMatch error"),
        }
    }

    #[tokio::test]
    async fn test_intent_service_basic_match() {
        let builtins = BuiltinIntents::new();
        let intent_matcher = builtins.create_matcher();
        let template_engine = builtins.create_engine();
        let pipeline_converter = IntentToPipeline::new();
        let llm_manager = Arc::new(RwLock::new(LlmManager::new()));

        let service = IntentService::new(
            intent_matcher,
            template_engine,
            pipeline_converter,
            None,
            vec![],
            None,
            llm_manager,
        );

        let request = IntentRequest {
            text: "统计文件数量".to_string(),
            llm_generation_enabled: false,
            llm_extraction_enabled: false,
            llm_validation_enabled: false,
            workflow_enabled: false,
        };

        let result = service.process(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.plan.is_some());
        assert!(response.confidence > 0.0);
    }
}
