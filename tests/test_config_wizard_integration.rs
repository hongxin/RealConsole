//! 配置向导系统集成测试
//!
//! 测试完整的配置向导流程：
//! - 环境检测
//! - 配置推荐
//! - 配置验证
//! - 配置生成
//!
//! 注意: 涉及文件系统操作的测试需要串行运行以避免竞态条件。
//! 运行方式: cargo test --test test_config_wizard_integration -- --test-threads=1

use realconsole::config::{
    ConfigRecommender, ConfigValidator, ConfigWizard, EnvironmentDetector, WizardMode,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

// 全局锁，确保文件系统操作测试串行执行
static FS_LOCK: Mutex<()> = Mutex::new(());

/// 测试环境检测 -> 推荐 -> 验证流程
#[tokio::test]
async fn test_detect_recommend_flow() {
    // 1. 环境检测
    let detector = EnvironmentDetector::new();
    let env = detector.detect_all().expect("环境检测失败");

    // 验证环境检测结果
    assert!(!env.os.os_type.is_empty(), "操作系统类型不应为空");
    assert!(!env.shell.shell_type.is_empty(), "Shell类型不应为空");
    println!("✓ 环境检测成功: {} {}", env.os.os_type, env.os.version);

    // 2. 配置推荐
    let recommender = ConfigRecommender::new();
    let recommendations = recommender.recommend_all(&env);

    // 验证推荐结果
    assert!(!recommendations.llm.provider.is_empty(), "LLM推荐不应为空");
    assert!(
        recommendations.llm.confidence >= 0.0 && recommendations.llm.confidence <= 1.0,
        "置信度应在0-1之间"
    );
    println!(
        "✓ 配置推荐成功: {} (置信度: {:.0}%)",
        recommendations.llm.provider,
        recommendations.llm.confidence * 100.0
    );

    // 3. 验证器创建
    let validator = ConfigValidator::new();
    // 验证一个可以创建的路径（父目录存在）
    let test_path = PathBuf::from("/tmp/test_realconsole_config.yaml");
    assert!(validator.validate_config_path(&test_path).success);
    println!("✓ 配置验证器创建成功");
}

/// 测试环境检测功能
#[test]
fn test_environment_detection() {
    let detector = EnvironmentDetector::new();

    // 检测操作系统
    let os = detector.detect_os().expect("操作系统检测失败");
    assert!(!os.os_type.is_empty());
    assert!(!os.arch.is_empty());
    println!("OS: {} {} ({})", os.os_type, os.version, os.arch);

    // 检测Shell
    let shell = detector.detect_shell().expect("Shell检测失败");
    assert!(!shell.shell_type.is_empty());
    assert!(shell.shell_path.exists());
    println!("Shell: {} at {:?}", shell.shell_type, shell.shell_path);

    // 检测工具
    let tools = detector.detect_tools();
    println!("找到 {} 个工具", tools.len());
    for tool in tools.iter().take(5) {
        println!("  - {}", tool.name);
    }

    // 完整检测
    let env = detector.detect_all().expect("完整环境检测失败");
    println!("用户画像: {:?}", env.user_profile);
}

/// 测试配置推荐功能
#[test]
fn test_configuration_recommendation() {
    let detector = EnvironmentDetector::new();
    let env = detector.detect_all().expect("环境检测失败");

    let recommender = ConfigRecommender::new();

    // LLM推荐
    let llm_rec = recommender.recommend_llm(&env);
    assert!(!llm_rec.provider.is_empty());
    assert!(llm_rec.confidence > 0.0 && llm_rec.confidence <= 1.0);
    println!("LLM推荐: {}", llm_rec.provider);
    println!("理由: {}", llm_rec.reason);

    // 模式推荐
    let mode_rec = recommender.recommend_mode(&env);
    assert!(!mode_rec.mode.is_empty());
    println!("模式推荐: {}", mode_rec.mode);

    // 建议频率推荐
    let freq_rec = recommender.recommend_suggestion_frequency(&env);
    assert!(!freq_rec.level.is_empty());
    println!("建议频率: {}", freq_rec.level);

    // 安全级别推荐
    let safety = recommender.recommend_safety_level(&env);
    assert!(!safety.is_empty());
    println!("安全级别: {}", safety);

    // 综合推荐
    let all_rec = recommender.recommend_all(&env);
    assert_eq!(all_rec.llm.provider, llm_rec.provider);
    assert_eq!(all_rec.mode.mode, mode_rec.mode);
}

/// 测试配置验证功能
#[tokio::test]
async fn test_configuration_validation() {
    let validator = ConfigValidator::new();

    // 测试路径验证
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let temp_path = temp_dir.path().join("test_config.yaml");

    // 不存在的文件（但父目录存在）
    let result = validator.validate_config_path(&temp_path);
    assert!(result.success, "应该允许创建新配置文件");

    // 测试空API Key验证
    let result = validator
        .validate_deepseek_api("")
        .await
        .expect("验证执行失败");
    assert!(!result.success, "空API Key应该验证失败");
    assert!(result.message.contains("不能为空"));
}

/// 测试配置文件生成
#[test]
fn test_config_file_generation() {
    use realconsole::config::wizard::WizardResult;

    // 获取文件系统锁，确保串行执行
    let _lock = FS_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let original_dir = std::env::current_dir().expect("获取当前目录失败");

    // 切换到临时目录
    std::env::set_current_dir(&temp_dir).expect("切换目录失败");

    let wizard = ConfigWizard::new(WizardMode::Minimal);

    let result = WizardResult {
        llm_provider: "deepseek".to_string(),
        llm_api_key: Some("test-api-key-12345".to_string()),
        llm_model: Some("deepseek-chat".to_string()),
        llm_endpoint: Some("https://api.deepseek.com/v1".to_string()),
        suggestion_frequency: "moderate".to_string(),
        safety_level: "standard".to_string(),
        data_collection: "anonymous".to_string(),
        keyboard_shortcuts: "default".to_string(),
    };

    // 生成配置文件
    wizard
        .generate_and_save(&result)
        .expect("配置文件生成失败");

    // 验证文件生成
    let yaml_path = temp_dir.path().join("realconsole.yaml");
    let env_path = temp_dir.path().join(".env");
    let gitignore_path = temp_dir.path().join(".gitignore");

    assert!(yaml_path.exists(), "realconsole.yaml 应该被创建");
    assert!(env_path.exists(), ".env 应该被创建");
    assert!(gitignore_path.exists(), ".gitignore 应该被创建");

    // 验证YAML内容
    let yaml_content = fs::read_to_string(&yaml_path).expect("读取YAML失败");
    assert!(yaml_content.contains("deepseek"), "应包含deepseek配置");
    assert!(
        yaml_content.contains("deepseek-chat"),
        "应包含模型名称"
    );
    assert!(
        yaml_content.contains("suggestion_frequency: moderate"),
        "应包含建议频率"
    );
    assert!(
        yaml_content.contains("safety_level: standard"),
        "应包含安全级别"
    );

    // 验证.env内容
    let env_content = fs::read_to_string(&env_path).expect("读取.env失败");
    assert!(
        env_content.contains("DEEPSEEK_API_KEY=test-api-key-12345"),
        "应包含API Key"
    );

    // 验证.gitignore内容
    let gitignore_content = fs::read_to_string(&gitignore_path).expect("读取.gitignore失败");
    assert!(
        gitignore_content.contains(".env"),
        ".gitignore应包含.env"
    );

    // 验证.env文件权限（Unix系统）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&env_path).expect("获取.env元数据失败");
        let permissions = metadata.permissions();
        let mode = permissions.mode() & 0o777;
        assert_eq!(mode, 0o600, ".env文件权限应为0600");
    }

    // 恢复原目录
    std::env::set_current_dir(&original_dir).expect("恢复目录失败");
}

/// 测试Ollama配置生成
#[test]
fn test_ollama_config_generation() {
    use realconsole::config::wizard::WizardResult;

    // 获取文件系统锁，确保串行执行
    let _lock = FS_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let original_dir = std::env::current_dir().expect("获取当前目录失败");

    std::env::set_current_dir(&temp_dir).expect("切换目录失败");

    let wizard = ConfigWizard::new(WizardMode::Standard);

    let result = WizardResult {
        llm_provider: "ollama".to_string(),
        llm_api_key: None,
        llm_model: Some("llama2".to_string()),
        llm_endpoint: Some("http://localhost:11434".to_string()),
        suggestion_frequency: "aggressive".to_string(),
        safety_level: "strict".to_string(),
        data_collection: "disabled".to_string(),
        keyboard_shortcuts: "default".to_string(),
    };

    wizard
        .generate_and_save(&result)
        .expect("Ollama配置文件生成失败");

    // 验证YAML内容
    let yaml_path = temp_dir.path().join("realconsole.yaml");
    let yaml_content = fs::read_to_string(&yaml_path).expect("读取YAML失败");

    assert!(yaml_content.contains("ollama"), "应包含ollama配置");
    assert!(yaml_content.contains("llama2"), "应包含模型名称");
    assert!(
        yaml_content.contains("http://localhost:11434"),
        "应包含端点地址"
    );
    assert!(
        !yaml_content.contains("api_key"),
        "Ollama配置不应包含API Key"
    );

    // 验证.env内容（应该基本为空）
    let env_path = temp_dir.path().join(".env");
    let env_content = fs::read_to_string(&env_path).expect("读取.env失败");
    assert!(
        !env_content.contains("DEEPSEEK_API_KEY"),
        "Ollama配置不应包含API Key"
    );

    std::env::set_current_dir(&original_dir).expect("恢复目录失败");
}

/// 测试向导模式
#[test]
fn test_wizard_modes() {
    let minimal = ConfigWizard::new(WizardMode::Minimal);
    let standard = ConfigWizard::new(WizardMode::Standard);
    let advanced = ConfigWizard::new(WizardMode::Advanced);

    // 验证向导可以创建（测试构造函数）
    let _ = minimal;
    let _ = standard;
    let _ = advanced;

    // 默认模式应该是Standard
    let default = ConfigWizard::default();
    let _ = default;
}

/// 测试配置覆盖场景
#[test]
fn test_config_overwrite_scenario() {
    // 获取文件系统锁，确保串行执行
    let _lock = FS_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let original_dir = std::env::current_dir().expect("获取当前目录失败");

    std::env::set_current_dir(&temp_dir).expect("切换目录失败");

    // 先创建第一个配置
    let wizard = ConfigWizard::new(WizardMode::Minimal);

    let result1 = realconsole::config::wizard::WizardResult {
        llm_provider: "deepseek".to_string(),
        llm_api_key: Some("first-key".to_string()),
        llm_model: Some("deepseek-chat".to_string()),
        llm_endpoint: Some("https://api.deepseek.com/v1".to_string()),
        suggestion_frequency: "moderate".to_string(),
        safety_level: "standard".to_string(),
        data_collection: "anonymous".to_string(),
        keyboard_shortcuts: "default".to_string(),
    };

    wizard.generate_and_save(&result1).expect("第一次生成失败");

    // 再创建第二个配置（覆盖）
    let result2 = realconsole::config::wizard::WizardResult {
        llm_provider: "ollama".to_string(),
        llm_api_key: None,
        llm_model: Some("llama2".to_string()),
        llm_endpoint: Some("http://localhost:11434".to_string()),
        suggestion_frequency: "conservative".to_string(),
        safety_level: "relaxed".to_string(),
        data_collection: "full".to_string(),
        keyboard_shortcuts: "custom".to_string(),
    };

    wizard.generate_and_save(&result2).expect("第二次生成失败");

    // 验证配置已被覆盖
    let yaml_path = temp_dir.path().join("realconsole.yaml");
    let yaml_content = fs::read_to_string(&yaml_path).expect("读取YAML失败");

    assert!(
        yaml_content.contains("ollama"),
        "配置应已更新为ollama"
    );
    assert!(
        !yaml_content.contains("deepseek"),
        "旧的deepseek配置应被覆盖"
    );

    std::env::set_current_dir(&original_dir).expect("恢复目录失败");
}

/// 测试"稍后配置"选项
#[test]
fn test_skip_configuration() {
    use realconsole::config::wizard::WizardResult;

    // 获取文件系统锁，确保串行执行
    let _lock = FS_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let original_dir = std::env::current_dir().expect("获取当前目录失败");

    std::env::set_current_dir(&temp_dir).expect("切换目录失败");

    let wizard = ConfigWizard::new(WizardMode::Minimal);

    let result = WizardResult {
        llm_provider: "none".to_string(),
        llm_api_key: None,
        llm_model: None,
        llm_endpoint: None,
        suggestion_frequency: "moderate".to_string(),
        safety_level: "standard".to_string(),
        data_collection: "anonymous".to_string(),
        keyboard_shortcuts: "default".to_string(),
    };

    wizard
        .generate_and_save(&result)
        .expect("跳过配置生成失败");

    // 验证配置文件
    let yaml_path = temp_dir.path().join("realconsole.yaml");
    let yaml_content = fs::read_to_string(&yaml_path).expect("读取YAML失败");

    assert!(
        yaml_content.contains("# llm: (稍后配置)"),
        "应包含稍后配置标记"
    );

    std::env::set_current_dir(&original_dir).expect("恢复目录失败");
}

/// 性能测试：环境检测应该快速完成
#[test]
fn test_detection_performance() {
    use std::time::Instant;

    let start = Instant::now();
    let detector = EnvironmentDetector::new();
    let _ = detector.detect_all().expect("环境检测失败");
    let duration = start.elapsed();

    println!("环境检测耗时: {:?}", duration);

    // 环境检测应该在1秒内完成
    assert!(
        duration.as_secs() < 1,
        "环境检测耗时过长: {:?}",
        duration
    );
}

/// 集成测试：完整的推荐到验证流程
#[tokio::test]
async fn test_full_recommendation_pipeline() {
    // 1. 环境检测
    let detector = EnvironmentDetector::new();
    let env = detector.detect_all().expect("环境检测失败");
    println!("✓ Step 1: 环境检测完成");

    // 2. 获取推荐
    let recommender = ConfigRecommender::new();
    let recommendations = recommender.recommend_all(&env);
    println!(
        "✓ Step 2: 推荐生成完成 (LLM: {})",
        recommendations.llm.provider
    );

    // 3. 创建验证器
    let validator = ConfigValidator::new();
    println!("✓ Step 3: 验证器创建完成");

    // 4. 验证路径
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let config_path = temp_dir.path().join("realconsole.yaml");
    let validation = validator.validate_config_path(&config_path);
    assert!(validation.success);
    println!("✓ Step 4: 配置路径验证完成");

    println!("\n🎉 完整流程测试通过！");
}
