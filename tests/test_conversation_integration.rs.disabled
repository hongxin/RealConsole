//! 多轮对话集成测试
//!
//! 端到端测试对话系统的完整流程

use realconsole::conversation::{
    state::{ConversationState, StateEvent},
    ConversationManager, ParameterSpec, ParameterType, ParameterValue, Response,
};

/// 测试完整的日志分析对话流程
#[tokio::test]
async fn test_log_analysis_conversation_flow() {
    let mut manager = ConversationManager::new(300);

    // 1. 启动对话
    let conversation_id = manager
        .start_conversation("analyze_logs")
        .expect("应该成功启动对话");

    // 2. 添加参数规格
    manager
        .add_parameter_spec(
            &conversation_id,
            ParameterSpec::new("log_file", ParameterType::Path, "日志文件路径"),
        )
        .expect("添加参数失败");

    manager
        .add_parameter_spec(
            &conversation_id,
            ParameterSpec::new("keyword", ParameterType::String, "搜索关键词"),
        )
        .expect("添加参数失败");

    // 3. 收集第一个参数
    let response = manager
        .collect_parameter(
            &conversation_id,
            "log_file",
            ParameterValue::String("/var/log/app.log".to_string()),
        )
        .expect("收集参数失败");

    assert_eq!(response, Response::ParameterCollected);

    // 4. 收集第二个参数
    let response = manager
        .collect_parameter(
            &conversation_id,
            "keyword",
            ParameterValue::String("ERROR".to_string()),
        )
        .expect("收集参数失败");

    assert_eq!(response, Response::ParameterCollected);

    // 5. 验证所有参数
    let response = manager
        .validate_parameters(&conversation_id)
        .expect("验证失败");

    assert_eq!(response, Response::ValidationPassed);

    // 6. 确认执行
    let response = manager
        .confirm_execution(&conversation_id)
        .expect("确认失败");

    assert_eq!(response, Response::ExecutionConfirmed);

    // 7. 完成执行
    let response = manager
        .complete_execution(&conversation_id, true, "日志分析完成".to_string())
        .expect("完成失败");

    assert_eq!(response, Response::ExecutionCompleted);

    // 8. 验证最终状态
    let state = manager.get_state(&conversation_id).expect("获取状态失败");
    assert_eq!(
        state,
        ConversationState::Completed {
            success: true,
            message: "日志分析完成".to_string()
        }
    );
}

/// 测试参数缺失时的智能提问
#[tokio::test]
async fn test_missing_parameter_prompt() {
    let mut manager = ConversationManager::new(300);

    let conversation_id = manager
        .start_conversation("analyze_logs")
        .expect("应该成功启动对话");

    manager
        .add_parameter_spec(
            &conversation_id,
            ParameterSpec::new("log_file", ParameterType::Path, "日志文件路径"),
        )
        .expect("添加参数失败");

    // 尝试执行但缺少参数
    let response = manager
        .collect_parameter(
            &conversation_id,
            "log_file",
            ParameterValue::String("".to_string()),
        )
        .expect("收集参数失败");

    assert_eq!(response, Response::ParameterRequired);
}

/// 测试状态转换
#[tokio::test]
async fn test_state_transitions() {
    let mut manager = ConversationManager::new(300);

    let conversation_id = manager
        .start_conversation("test_task")
        .expect("应该成功启动对话");

    // 初始状态
    let state = manager.get_state(&conversation_id).expect("获取状态失败");
    assert_eq!(state, ConversationState::Initializing);

    // 添加参数后应该进入收集参数状态
    manager
        .add_parameter_spec(
            &conversation_id,
            ParameterSpec::new("param1", ParameterType::String, "测试参数"),
        )
        .expect("添加参数失败");

    // 手动触发状态转换
    manager
        .transition(&conversation_id, StateEvent::IntentRecognized)
        .expect("状态转换失败");

    let state = manager.get_state(&conversation_id).expect("获取状态失败");
    assert!(matches!(
        state,
        ConversationState::CollectingParameters { .. }
    ));

    // 提供参数
    manager
        .collect_parameter(
            &conversation_id,
            "param1",
            ParameterValue::String("test_value".to_string()),
        )
        .expect("收集参数失败");

    // 触发所有参数收集完成
    manager
        .transition(&conversation_id, StateEvent::AllParametersCollected)
        .expect("状态转换失败");

    let state = manager.get_state(&conversation_id).expect("获取状态失败");
    assert_eq!(state, ConversationState::Validating);

    // 验证通过
    manager
        .transition(&conversation_id, StateEvent::ValidationPassed)
        .expect("状态转换失败");

    let state = manager.get_state(&conversation_id).expect("获取状态失败");
    assert_eq!(state, ConversationState::Confirming);

    // 用户确认
    manager
        .transition(&conversation_id, StateEvent::UserConfirmed)
        .expect("状态转换失败");

    let state = manager.get_state(&conversation_id).expect("获取状态失败");
    assert_eq!(state, ConversationState::Executing);

    // 执行完成
    manager
        .transition(
            &conversation_id,
            StateEvent::ExecutionCompleted {
                success: true,
                message: "任务完成".to_string(),
            },
        )
        .expect("状态转换失败");

    let state = manager.get_state(&conversation_id).expect("获取状态失败");
    assert_eq!(
        state,
        ConversationState::Completed {
            success: true,
            message: "任务完成".to_string()
        }
    );
}
