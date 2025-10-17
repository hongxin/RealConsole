//! 多轮对话集成测试
//!
//! 端到端测试对话系统的完整流程

use realconsole::conversation::{
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
            ParameterValue::String("/var/log/test.log".to_string()),
        )
        .expect("收集参数失败");

    // 应该询问下一个参数
    assert!(matches!(response, Response::AskForParameter { .. }));

    // 4. 收集第二个参数
    let response = manager
        .collect_parameter(
            &conversation_id,
            "keyword",
            ParameterValue::String("ERROR".to_string()),
        )
        .expect("收集参数失败");

    // 所有参数已收集
    assert_eq!(response, Response::AllParametersCollected);

    // 5. 检测缺失参数（应该为空）
    let missing = manager
        .detect_missing_parameters(&conversation_id)
        .expect("检测失败");
    assert!(missing.is_empty());
}

/// 测试参数缺失检测
#[tokio::test]
async fn test_missing_parameter_detection() {
    let mut manager = ConversationManager::new(300);

    let conversation_id = manager
        .start_conversation("test_intent")
        .expect("启动对话失败");

    // 添加3个必需参数
    manager
        .add_parameter_spec(
            &conversation_id,
            ParameterSpec::new("param1", ParameterType::String, "参数1"),
        )
        .unwrap();

    manager
        .add_parameter_spec(
            &conversation_id,
            ParameterSpec::new("param2", ParameterType::String, "参数2"),
        )
        .unwrap();

    manager
        .add_parameter_spec(
            &conversation_id,
            ParameterSpec::new("param3", ParameterType::String, "参数3"),
        )
        .unwrap();

    // 只收集1个参数
    manager
        .collect_parameter(
            &conversation_id,
            "param1",
            ParameterValue::String("value1".to_string()),
        )
        .unwrap();

    // 检测缺失参数
    let missing = manager
        .detect_missing_parameters(&conversation_id)
        .expect("检测失败");

    assert_eq!(missing.len(), 2);
    assert_eq!(missing[0].name, "param2");
    assert_eq!(missing[1].name, "param3");
}

/// 测试可选参数
#[tokio::test]
async fn test_optional_parameters() {
    let mut manager = ConversationManager::new(300);

    let conversation_id = manager
        .start_conversation("test")
        .expect("启动对话失败");

    // 添加1个必需参数和1个可选参数
    manager
        .add_parameter_spec(
            &conversation_id,
            ParameterSpec::new("required", ParameterType::String, "必需参数"),
        )
        .unwrap();

    manager
        .add_parameter_spec(
            &conversation_id,
            ParameterSpec::new("optional", ParameterType::String, "可选参数").optional(),
        )
        .unwrap();

    // 只收集必需参数
    manager
        .collect_parameter(
            &conversation_id,
            "required",
            ParameterValue::String("value".to_string()),
        )
        .unwrap();

    // 检测缺失参数（可选参数不算缺失）
    let missing = manager
        .detect_missing_parameters(&conversation_id)
        .expect("检测失败");

    assert!(missing.is_empty());
}

/// 测试对话取消
#[tokio::test]
async fn test_conversation_cancellation() {
    let mut manager = ConversationManager::new(300);

    let conversation_id = manager
        .start_conversation("test")
        .expect("启动对话失败");

    // 取消对话
    manager
        .cancel_conversation(&conversation_id, "用户取消")
        .expect("取消失败");

    // 检查状态是否为已取消
    let context = manager.get_context(&conversation_id).unwrap();
    assert!(context.state.is_terminal());
}

/// 测试完整的确认和执行流程
#[tokio::test]
async fn test_full_confirmation_execution_flow() {
    let mut manager = ConversationManager::new(300);

    let conversation_id = manager
        .start_conversation("test")
        .expect("启动对话失败");

    // 添加参数
    manager
        .add_parameter_spec(
            &conversation_id,
            ParameterSpec::new("param", ParameterType::String, "参数"),
        )
        .unwrap();

    // 收集参数
    let response = manager
        .collect_parameter(
            &conversation_id,
            "param",
            ParameterValue::String("value".to_string()),
        )
        .expect("收集参数失败");

    // 应该返回所有参数已收集
    assert_eq!(response, Response::AllParametersCollected);

    // collect_parameter 已经将状态转换到了 Validating
    // 现在需要转换到确认状态
    {
        let context = manager.get_context_mut(&conversation_id).unwrap();
        context
            .state
            .transition(realconsole::conversation::StateEvent::ValidationPassed)
            .unwrap();
    }

    // 用户确认
    let response = manager
        .confirm_execution(&conversation_id, true)
        .expect("确认失败");

    assert_eq!(response, Response::ReadyToExecute);

    // 完成执行
    let response = manager
        .complete_execution(&conversation_id, true, "执行成功".to_string())
        .expect("完成执行失败");

    assert!(matches!(
        response,
        Response::ExecutionResult {
            success: true,
            ..
        }
    ));
}

/// 测试对话超时
#[tokio::test]
async fn test_conversation_timeout() {
    let mut manager = ConversationManager::new(1); // 1秒超时

    let conversation_id = manager
        .start_conversation("test")
        .expect("启动对话失败");

    // 修改 last_active 模拟超时
    {
        let context = manager.get_context_mut(&conversation_id).unwrap();
        context.last_active = chrono::Utc::now() - chrono::Duration::seconds(2);
    }

    // 检查超时
    let timeout_ids = manager.check_timeouts();

    assert_eq!(timeout_ids.len(), 1);
    assert_eq!(timeout_ids[0], conversation_id);

    // 验证状态已转换为超时
    let context = manager.get_context(&conversation_id).unwrap();
    assert_eq!(
        context.state,
        realconsole::conversation::ConversationState::Timeout
    );
}

/// 测试参数构建器模式
#[tokio::test]
async fn test_parameter_spec_builder() {
    let spec = ParameterSpec::new("test_param", ParameterType::String, "测试参数")
        .with_hint("这是一个提示")
        .with_example("示例值")
        .optional();

    assert_eq!(spec.name, "test_param");
    assert_eq!(spec.description, "测试参数");
    assert!(spec.is_optional);
    assert_eq!(spec.hint, Some("这是一个提示".to_string()));
    assert_eq!(spec.example, Some("示例值".to_string()));
}

/// 测试并发对话管理
#[tokio::test]
async fn test_multiple_conversations() {
    let mut manager = ConversationManager::new(300);

    // 启动3个对话
    let id1 = manager.start_conversation("intent1").unwrap();
    let id2 = manager.start_conversation("intent2").unwrap();
    let id3 = manager.start_conversation("intent3").unwrap();

    // 验证都是不同的ID
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);

    // 验证活跃对话数量
    assert_eq!(manager.active_count(), 3);

    // 完成其中一个对话
    {
        let context = manager.get_context_mut(&id1).unwrap();
        context.state = realconsole::conversation::ConversationState::Completed {
            success: true,
            message: "完成".to_string(),
        };
    }

    // 活跃对话应该变为2个
    assert_eq!(manager.active_count(), 2);

    // 清理已完成的对话
    manager.cleanup_completed();

    // 验证只剩2个对话
    assert!(manager.get_context(&id1).is_err());
    assert!(manager.get_context(&id2).is_ok());
    assert!(manager.get_context(&id3).is_ok());
}

/// 测试参数类型匹配
#[tokio::test]
async fn test_parameter_type_validation() {
    use realconsole::conversation::analyzer::ParameterAnalyzer;

    let analyzer = ParameterAnalyzer::new();

    // 测试字符串类型
    let spec = ParameterSpec::new("name", ParameterType::String, "名称");
    let value = ParameterValue::String("测试".to_string());
    assert!(analyzer.validate_parameter(&spec, &value).is_ok());

    // 测试整数类型
    let spec = ParameterSpec::new("count", ParameterType::Integer, "数量");
    let value = ParameterValue::Integer(42);
    assert!(analyzer.validate_parameter(&spec, &value).is_ok());

    // 测试类型不匹配
    let spec = ParameterSpec::new("count", ParameterType::Integer, "数量");
    let value = ParameterValue::String("not a number".to_string());
    assert!(analyzer.validate_parameter(&spec, &value).is_err());
}
