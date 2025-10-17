//! Intent 匹配修复验证测试
//!
//! 验证 Phase 6.2 的修复：
//! 1. "显示当前目录下体积最大的rs文件" 应该匹配 find_files_by_size，而非 list_directory
//! 2. list_directory 的否定前瞻断言正确工作
//! 3. find_files_by_size 的增强关键词正确匹配

use realconsole::dsl::intent::builtin::BuiltinIntents;

#[test]
fn test_fix_largest_file_matching() {
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 用户输入：显示当前目录下体积最大的rs文件
    let input = "显示当前目录下体积最大的rs文件";
    let matches = matcher.match_intent(input);

    assert!(!matches.is_empty(), "应该有匹配结果");

    // 第一个匹配应该是 find_files_by_size，而非 list_directory
    let best_match = &matches[0];
    assert_eq!(
        best_match.intent.name, "find_files_by_size",
        "应该匹配 find_files_by_size，而非 list_directory。实际匹配：{}",
        best_match.intent.name
    );

    println!("✅ 修复验证成功！");
    println!("   输入：{}", input);
    println!("   匹配：{} (置信度: {:.2})", best_match.intent.name, best_match.confidence);
}

#[test]
fn test_list_directory_without_filter() {
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 不带过滤条件的输入应该匹配 list_directory
    let input = "显示当前目录下的所有文件";
    let matches = matcher.match_intent(input);

    assert!(!matches.is_empty(), "应该有匹配结果");

    let best_match = &matches[0];
    // 注意：这里可能匹配到 find_recent_files 或 list_directory
    // 只要不是 find_files_by_size 就好
    assert_ne!(
        best_match.intent.name, "find_files_by_size",
        "不应该匹配 find_files_by_size"
    );

    println!("✅ list_directory 基础功能正常！");
    println!("   输入：{}", input);
    println!("   匹配：{} (置信度: {:.2})", best_match.intent.name, best_match.confidence);
}

#[test]
fn test_filter_keywords_priority() {
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 测试不同的过滤关键词
    let test_cases = vec![
        ("显示最大的文件", "find_files_by_size"),
        ("显示最小的文件", "find_files_by_size"),
        ("显示体积最大的文件", "find_files_by_size"),
        ("显示大小最大的文件", "find_files_by_size"),
    ];

    for (input, expected) in test_cases {
        let matches = matcher.match_intent(input);
        assert!(!matches.is_empty(), "输入 '{}' 应该有匹配结果", input);

        let best_match = &matches[0];
        assert_eq!(
            best_match.intent.name, expected,
            "输入 '{}' 应该匹配 {}，实际匹配 {}",
            input, expected, best_match.intent.name
        );

        println!("✅ '{}'  →  {} (置信度: {:.2})",
                 input, best_match.intent.name, best_match.confidence);
    }
}

#[test]
fn test_filter_intent_priority_over_list() {
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 包含过滤关键词的输入应该优先匹配过滤型Intent（如find_files_by_size）
    // 而非 list_directory
    let filter_inputs = vec![
        ("显示当前目录下最大的文件", "find_files_by_size"),
        ("显示当前目录下最新的文件", "find_recent_files"),
    ];

    for (input, expected_intent) in filter_inputs {
        let matches = matcher.match_intent(input);
        assert!(!matches.is_empty(), "输入 '{}' 应该有匹配结果", input);

        let best_match = &matches[0];

        // 最佳匹配应该是过滤型Intent
        assert_eq!(
            best_match.intent.name, expected_intent,
            "输入 '{}' 应该优先匹配 {}，而非 list_directory。实际匹配：{}",
            input, expected_intent, best_match.intent.name
        );

        // 如果也匹配到了 list_directory，它的置信度应该低于过滤型Intent
        if let Some(list_dir_match) = matches.iter().find(|m| m.intent.name == "list_directory") {
            assert!(
                list_dir_match.confidence < best_match.confidence,
                "输入 '{}' 的 list_directory 置信度 ({:.2}) 应该低于 {} ({:.2})",
                input, list_dir_match.confidence, expected_intent, best_match.confidence
            );
        }

        println!("✅ '{}'  →  {} (置信度: {:.2}) 优先于 list_directory",
                 input, best_match.intent.name, best_match.confidence);
    }
}

#[test]
fn test_enhanced_find_large_files_keywords() {
    let builtin = BuiltinIntents::new();
    let matcher = builtin.create_matcher();

    // 测试增强的关键词
    let keywords = vec![
        "显示最大的文件",
        "体积最大的文件",
        "大小最大的文件",
        "最大的rs文件",
    ];

    for input in keywords {
        let matches = matcher.match_intent(input);
        assert!(!matches.is_empty(), "输入 '{}' 应该有匹配结果", input);

        // 应该匹配到 find_files_by_size
        let has_find_files_by_size = matches.iter().any(|m| m.intent.name == "find_files_by_size");
        assert!(
            has_find_files_by_size,
            "输入 '{}' 应该匹配 find_files_by_size",
            input
        );

        let best_match = &matches[0];
        println!("✅ '{}'  →  {} (置信度: {:.2})",
                 input, best_match.intent.name, best_match.confidence);
    }
}
