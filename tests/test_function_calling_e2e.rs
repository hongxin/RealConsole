//! 端到端测试：Function Calling 完整流程
//!
//! 测试从 LLM 调用到工具执行再到结果反馈的完整流程

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use realconsole::llm::{ChatResponse, ClientStats, FunctionCall, LlmClient, LlmError, Message, ToolCall};
use realconsole::tool::{Parameter, ParameterType, Tool, ToolRegistry};
use realconsole::tool_executor::ToolExecutor;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock LLM Client 用于测试工具调用
struct MockLlmWithTools {
    /// 预定义的响应序列
    responses: Vec<ChatResponse>,
    /// 当前响应索引
    current_response: std::sync::Mutex<usize>,
}

impl MockLlmWithTools {
    fn new(responses: Vec<ChatResponse>) -> Self {
        Self {
            responses,
            current_response: std::sync::Mutex::new(0),
        }
    }

    /// 创建一个简单的工具调用场景：
    /// 1. LLM 返回工具调用
    /// 2. LLM 基于工具结果返回最终答案
    fn simple_tool_call_scenario() -> Self {
        let responses = vec![
            // 第一轮：LLM 决定调用 add 工具
            ChatResponse::with_tools(vec![ToolCall {
                id: "call_123".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "test_add".to_string(),
                    arguments: r#"{"a": 10, "b": 5}"#.to_string(),
                },
            }]),
            // 第二轮：LLM 基于工具结果返回最终答案
            ChatResponse::text("根据计算结果，10 + 5 = 15".to_string()),
        ];

        Self::new(responses)
    }

    /// 创建一个多轮工具调用场景
    fn multi_round_scenario() -> Self {
        let responses = vec![
            // 第一轮：调用 add
            ChatResponse::with_tools(vec![ToolCall {
                id: "call_1".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "test_add".to_string(),
                    arguments: r#"{"a": 10, "b": 5}"#.to_string(),
                },
            }]),
            // 第二轮：再次调用 multiply
            ChatResponse::with_tools(vec![ToolCall {
                id: "call_2".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "test_multiply".to_string(),
                    arguments: r#"{"a": 15, "b": 2}"#.to_string(),
                },
            }]),
            // 第三轮：返回最终答案
            ChatResponse::text("经过两次计算：(10 + 5) * 2 = 30".to_string()),
        ];

        Self::new(responses)
    }
}

#[async_trait]
impl LlmClient for MockLlmWithTools {
    async fn chat(&self, _messages: Vec<Message>) -> Result<String, LlmError> {
        Ok("Mock response without tools".to_string())
    }

    async fn chat_with_tools(
        &self,
        _messages: Vec<Message>,
        _tools: Vec<JsonValue>,
    ) -> Result<ChatResponse, LlmError> {
        let mut idx = self.current_response.lock().unwrap();
        let response = if *idx < self.responses.len() {
            self.responses[*idx].clone()
        } else {
            // 如果超出范围，返回错误
            return Err(LlmError::Other(format!(
                "Mock: 请求次数超出预期 (已请求 {} 次)",
                *idx + 1
            )));
        };
        *idx += 1;
        Ok(response)
    }

    fn model(&self) -> &str {
        "mock-model"
    }

    fn stats(&self) -> ClientStats {
        ClientStats::new()
    }

    async fn diagnose(&self) -> String {
        "Mock LLM is ready".to_string()
    }
}

/// 创建测试用的工具注册表
fn create_test_tool_registry() -> Arc<RwLock<ToolRegistry>> {
    let mut registry = ToolRegistry::new();

    // 注册 add 工具
    let add_tool = Tool::new(
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
    registry.register(add_tool);

    // 注册 multiply 工具
    let multiply_tool = Tool::new(
        "test_multiply",
        "测试乘法工具",
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
            Ok(format!("result: {}", a * b))
        },
    );
    registry.register(multiply_tool);

    Arc::new(RwLock::new(registry))
}

#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_simple_tool_call() {
    // 准备测试环境
    let registry = create_test_tool_registry();
    let executor = ToolExecutor::with_defaults(Arc::clone(&registry));

    // 创建 Mock LLM（返回一次工具调用 + 一次最终答案）
    let llm = MockLlmWithTools::simple_tool_call_scenario();

    // 获取工具 schemas
    let tool_schemas = {
        let reg = registry.read().await;
        reg.get_function_schemas()
    };

    // 执行迭代工具调用
    let result = executor
        .execute_iterative(&llm, "请计算 10 + 5", tool_schemas)
        .await;

    // 验证结果
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("15"));
    assert!(response.contains("10 + 5"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_multi_round_tool_calls() {
    // 准备测试环境
    let registry = create_test_tool_registry();
    let executor = ToolExecutor::with_defaults(Arc::clone(&registry));

    // 创建 Mock LLM（多轮工具调用）
    let llm = MockLlmWithTools::multi_round_scenario();

    // 获取工具 schemas
    let tool_schemas = {
        let reg = registry.read().await;
        reg.get_function_schemas()
    };

    // 执行迭代工具调用
    let result = executor
        .execute_iterative(&llm, "请计算 (10 + 5) * 2", tool_schemas)
        .await;

    // 验证结果
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("30"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_no_tools_fallback() {
    // 创建一个没有工具的场景
    let mock_llm = MockLlmWithTools::new(vec![
        ChatResponse::text("这是一个普通的对话响应".to_string()),
    ]);

    let registry = create_test_tool_registry();
    let executor = ToolExecutor::with_defaults(registry);

    // 执行时不提供工具 schemas（空列表）
    let result = executor
        .execute_iterative(&mock_llm, "你好", vec![])
        .await;

    // 应该返回普通对话
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("普通的对话响应"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_tool_execution_error() {
    // 测试工具执行错误的情况
    let llm = MockLlmWithTools::new(vec![
        // LLM 调用一个不存在的工具
        ChatResponse::with_tools(vec![ToolCall {
            id: "call_err".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "non_existent_tool".to_string(),
                arguments: r#"{"arg": "value"}"#.to_string(),
            },
        }]),
        // LLM 仍然返回一个响应（基于错误信息）
        ChatResponse::text("抱歉，我无法执行该工具".to_string()),
    ]);

    let registry = create_test_tool_registry();
    let executor = ToolExecutor::with_defaults(Arc::clone(&registry));

    let tool_schemas = {
        let reg = registry.read().await;
        reg.get_function_schemas()
    };

    let result = executor
        .execute_iterative(&llm, "调用不存在的工具", tool_schemas)
        .await;

    // 执行应该成功（只是工具调用失败，LLM 仍可以回应）
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_max_iterations() {
    // 测试最大迭代次数限制
    let responses = vec![
        // 持续返回工具调用，直到达到最大次数
        ChatResponse::with_tools(vec![ToolCall {
            id: "call_1".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "test_add".to_string(),
                arguments: r#"{"a": 1, "b": 1}"#.to_string(),
            },
        }]),
        ChatResponse::with_tools(vec![ToolCall {
            id: "call_2".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "test_add".to_string(),
                arguments: r#"{"a": 2, "b": 2}"#.to_string(),
            },
        }]),
        ChatResponse::with_tools(vec![ToolCall {
            id: "call_3".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "test_add".to_string(),
                arguments: r#"{"a": 3, "b": 3}"#.to_string(),
            },
        }]),
        ChatResponse::with_tools(vec![ToolCall {
            id: "call_4".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "test_add".to_string(),
                arguments: r#"{"a": 4, "b": 4}"#.to_string(),
            },
        }]),
        ChatResponse::with_tools(vec![ToolCall {
            id: "call_5".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "test_add".to_string(),
                arguments: r#"{"a": 5, "b": 5}"#.to_string(),
            },
        }]),
        // 第6轮还想调用（但应该被限制）
        ChatResponse::with_tools(vec![ToolCall {
            id: "call_6".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "test_add".to_string(),
                arguments: r#"{"a": 6, "b": 6}"#.to_string(),
            },
        }]),
    ];

    let llm = MockLlmWithTools::new(responses);
    let registry = create_test_tool_registry();
    let executor = ToolExecutor::with_defaults(Arc::clone(&registry));

    let tool_schemas = {
        let reg = registry.read().await;
        reg.get_function_schemas()
    };

    let result = executor
        .execute_iterative(&llm, "持续调用工具", tool_schemas)
        .await;

    // 应该返回错误：达到最大迭代次数
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("最大迭代次数") || error.contains("循环"));
}
