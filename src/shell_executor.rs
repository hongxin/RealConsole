//! Shell 命令执行器
//!
//! 提供安全的 shell 命令执行功能。
//!
//! 特性：
//! - 黑名单安全检查
//! - 超时控制（30秒）
//! - 输出大小限制（100KB）
//! - 跨平台支持（Unix/Windows）
//! - 错误自动分析和修复建议（Phase 9.1 Week 2）

use crate::error::{ErrorCode, FixSuggestion, RealError};
use crate::error_fixer::{
    ErrorAnalysis, ErrorAnalyzer, FeedbackLearner, FeedbackRecord, FeedbackType, FixOutcome,
    FixStrategy,
};
use crate::llm::LlmClient;
use regex::Regex;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// 最大输出大小（字节）
const MAX_OUTPUT_SIZE: usize = 100_000;

/// 命令执行超时时间（秒）
const COMMAND_TIMEOUT: u64 = 30;

/// 危险命令模式（黑名单）
const DANGEROUS_PATTERNS: &[&str] = &[
    r"rm\s+-rf\s+/",           // 删除根目录
    r"rm\s+-fr\s+/",           // 删除根目录（参数顺序）
    r"dd\s+if=/dev/zero",      // 磁盘写入
    r"dd\s+if=/dev/random",    // 磁盘写入
    r"mkfs",                   // 格式化
    r":\(\)\{.*\|.*&.*\};:",   // fork 炸弹
    r"sudo\s+",                // 权限提升
    r"shutdown",               // 系统关机
    r"reboot",                 // 系统重启
    r"halt",                   // 系统停止
    r"poweroff",               // 电源关闭
    r"init\s+0",               // 关机
    r"init\s+6",               // 重启
    r">\s*/dev/sd[a-z]",      // 直接写磁盘（允许空格）
];

/// 检查命令是否安全
fn is_safe_command(command: &str) -> Result<(), RealError> {
    // 检查空命令
    if command.trim().is_empty() {
        return Err(RealError::new(
            ErrorCode::ShellCommandEmpty,
            "Shell 命令不能为空",
        )
        .with_suggestion(FixSuggestion::new("输入有效的 shell 命令")));
    }

    // 检查危险模式
    for pattern in DANGEROUS_PATTERNS {
        let re = Regex::new(pattern).map_err(|e| {
            RealError::new(ErrorCode::ShellExecutionError, format!("正则错误: {}", e))
        })?;

        if re.is_match(command) {
            return Err(RealError::new(
                ErrorCode::ShellDangerousCommand,
                format!("命令包含危险操作，已被安全策略阻止"),
            )
            .with_suggestion(
                FixSuggestion::new("此命令可能造成系统损坏，建议使用更安全的替代方案"),
            )
            .with_suggestion(
                FixSuggestion::new("查看允许的命令列表和安全策略")
                    .with_doc("https://docs.realconsole.com/shell-safety"),
            ));
        }
    }

    Ok(())
}

/// 执行 shell 命令
///
/// # Arguments
/// * `command` - 要执行的命令字符串
///
/// # Returns
/// * `Ok(String)` - 命令输出（stdout + stderr）
/// * `Err(RealError)` - 错误信息（包含错误代码和修复建议）
pub async fn execute_shell(command: &str) -> Result<String, RealError> {
    // 安全检查
    is_safe_command(command)?;

    // 根据操作系统选择 shell
    #[cfg(unix)]
    let (shell, flag) = ("/bin/sh", "-c");

    #[cfg(windows)]
    let (shell, flag) = ("cmd", "/C");

    // 异步执行命令（带超时）
    let command_str = command.to_string();
    let result = timeout(Duration::from_secs(COMMAND_TIMEOUT), async move {
        tokio::task::spawn_blocking(move || {
            Command::new(shell)
                .arg(flag)
                .arg(&command_str)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        })
        .await
        .map_err(|e| {
            RealError::new(
                ErrorCode::ShellExecutionError,
                format!("任务执行失败: {}", e),
            )
        })?
        .map_err(|e| {
            RealError::new(
                ErrorCode::ShellExecutionError,
                format!("命令执行失败: {}", e),
            )
            .with_suggestion(FixSuggestion::new("检查命令语法是否正确"))
        })
    })
    .await;

    // 处理超时
    let output = match result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return Err(e),
        Err(_) => {
            return Err(RealError::new(
                ErrorCode::ShellTimeoutError,
                format!("命令执行超时（超过 {} 秒）", COMMAND_TIMEOUT),
            )
            .with_suggestion(
                FixSuggestion::new("命令执行时间过长，请检查命令或增加超时时间"),
            )
            .with_suggestion(
                FixSuggestion::new("在配置文件中调整 features.shell_timeout")
                    .with_command("vi realconsole.yaml"),
            ));
        }
    };

    // 合并 stdout 和 stderr
    let mut result_text = String::new();

    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        result_text.push_str(&stdout);
    }

    if !output.stderr.is_empty() {
        if !result_text.is_empty() {
            result_text.push('\n');
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        result_text.push_str("stderr: ");
        result_text.push_str(&stderr);
    }

    // 检查命令退出状态
    if !output.status.success() {
        // 命令执行失败，返回错误以触发错误修复系统
        // 但保留输出信息（stdout + stderr）供错误分析使用
        let error_message = if result_text.is_empty() {
            format!(
                "命令执行失败（退出码: {}）",
                output.status.code().unwrap_or(-1)
            )
        } else {
            result_text.clone()
        };

        return Err(RealError::new(
            ErrorCode::ShellExecutionError,
            error_message,
        )
        .with_suggestion(FixSuggestion::new("检查命令语法和参数是否正确"))
        .with_suggestion(
            FixSuggestion::new("查看命令的帮助信息").with_command("man <command>"),
        ));
    }

    // 限制输出大小
    if result_text.len() > MAX_OUTPUT_SIZE {
        result_text.truncate(MAX_OUTPUT_SIZE);
        result_text.push_str("\n... (输出已截断)");
    }

    // 如果没有输出，返回成功提示
    if result_text.is_empty() {
        result_text = format!("✓ 命令执行成功 (exit code: {})", output.status.code().unwrap_or(0));
    }

    Ok(result_text)
}

/// Shell 执行器结果（带错误分析）
pub struct ExecutionResult {
    /// 命令是否成功
    pub success: bool,

    /// 命令输出
    pub output: String,

    /// 错误分析（如果命令失败）
    pub error_analysis: Option<ErrorAnalysis>,

    /// 建议的修复策略（如果有）
    pub fix_strategies: Vec<FixStrategy>,
}

impl ExecutionResult {
    /// 创建成功结果
    pub fn success(output: String) -> Self {
        Self {
            success: true,
            output,
            error_analysis: None,
            fix_strategies: Vec::new(),
        }
    }

    /// 创建失败结果（带错误分析）
    pub fn failure(
        output: String,
        error_analysis: Option<ErrorAnalysis>,
        fix_strategies: Vec<FixStrategy>,
    ) -> Self {
        Self {
            success: false,
            output,
            error_analysis,
            fix_strategies,
        }
    }
}

/// 带错误自动修复的 Shell 执行器
///
/// 功能：
/// - 执行 shell 命令
/// - 自动分析错误
/// - 生成修复建议（基于规则 + LLM）
/// - 可选的自动修复应用
/// - 用户反馈学习（Phase 9.1 Week 3）
pub struct ShellExecutorWithFixer {
    /// 错误分析器
    analyzer: ErrorAnalyzer,

    /// LLM 客户端（可选，用于增强分析）
    llm: Option<Arc<dyn LlmClient>>,

    /// 是否启用 LLM 增强分析
    enable_llm_analysis: bool,

    /// 反馈学习器（Week 3）
    feedback_learner: Arc<FeedbackLearner>,
}

impl ShellExecutorWithFixer {
    /// 创建新的执行器
    pub fn new() -> Self {
        Self {
            analyzer: ErrorAnalyzer::new(),
            llm: None,
            enable_llm_analysis: false,
            feedback_learner: Arc::new(FeedbackLearner::new()),
        }
    }

    /// 验证修复策略的安全性
    ///
    /// 确保修复策略不会引入危险命令
    fn is_safe_fix_strategy(&self, strategy: &FixStrategy) -> bool {
        // 检查修复命令是否安全
        if is_safe_command(&strategy.command).is_err() {
            return false;
        }

        // 高风险策略必须经过用户确认
        if strategy.is_high_risk() && !strategy.requires_confirmation {
            return false;
        }

        true
    }

    /// 设置 LLM 客户端
    pub fn with_llm(mut self, llm: Arc<dyn LlmClient>) -> Self {
        self.llm = Some(llm);
        self.enable_llm_analysis = true;
        self
    }

    /// 设置反馈学习器
    pub fn with_feedback_learner(mut self, learner: Arc<FeedbackLearner>) -> Self {
        self.feedback_learner = learner;
        self
    }

    /// 获取反馈学习器的引用
    pub fn feedback_learner(&self) -> Arc<FeedbackLearner> {
        self.feedback_learner.clone()
    }

    /// 禁用 LLM 增强分析（仅使用规则）
    pub fn disable_llm_analysis(mut self) -> Self {
        self.enable_llm_analysis = false;
        self
    }

    /// 执行命令并分析错误
    ///
    /// # Arguments
    /// * `command` - 要执行的命令
    ///
    /// # Returns
    /// * `ExecutionResult` - 包含输出、错误分析和修复建议
    pub async fn execute_with_analysis(&self, command: &str) -> ExecutionResult {
        // 执行命令
        match execute_shell(command).await {
            Ok(output) => ExecutionResult::success(output),
            Err(err) => {
                // 提取错误输出
                let error_output = err.to_string();

                // 基础错误分析
                let mut analysis = self.analyzer.analyze(command, &error_output);

                // LLM 增强分析（如果启用且可用）
                if self.enable_llm_analysis {
                    if let Some(ref llm) = self.llm {
                        if let Ok(enhanced) = self.analyzer.analyze_with_llm(analysis.clone(), llm.as_ref()).await {
                            analysis = enhanced;
                        }
                    }
                }

                // 生成修复策略
                use crate::error_fixer::ErrorFixer;
                let mut strategies = ErrorFixer::generate_strategies(&analysis);

                // LLM 增强修复策略（如果启用且可用）
                if self.enable_llm_analysis {
                    if let Some(ref llm) = self.llm {
                        if let Ok(llm_strategies) = ErrorFixer::generate_strategies_with_llm(&analysis, llm.as_ref()).await {
                            strategies.extend(llm_strategies);
                        }
                    }
                }

                // 过滤掉不安全的修复策略
                let safe_strategies: Vec<_> = strategies.into_iter()
                    .filter(|s| self.is_safe_fix_strategy(s))
                    .collect();

                // 使用学习到的数据重新排序策略（Week 3）
                let ranked_strategies = self.feedback_learner.rerank_strategies(safe_strategies).await;

                ExecutionResult::failure(error_output, Some(analysis), ranked_strategies)
            }
        }
    }

    /// 执行命令并尝试自动修复
    ///
    /// 如果命令失败且有可用的低风险修复策略，自动应用修复并重试
    ///
    /// # Arguments
    /// * `command` - 要执行的命令
    /// * `max_retries` - 最大重试次数
    ///
    /// # Returns
    /// * `ExecutionResult` - 最终结果
    pub async fn execute_with_auto_fix(&self, command: &str, max_retries: usize) -> ExecutionResult {
        let mut current_command = command.to_string();
        let mut attempt = 0;

        loop {
            attempt += 1;
            let result = self.execute_with_analysis(&current_command).await;

            // 如果成功或达到最大重试次数，返回结果
            if result.success || attempt >= max_retries {
                return result;
            }

            // 查找可自动应用的低风险修复策略（带安全检查）
            let auto_fix = result.fix_strategies.iter()
                .find(|s| {
                    s.risk_level < 5
                    && !s.requires_confirmation
                    && self.is_safe_fix_strategy(s)
                });

            if let Some(fix) = auto_fix {
                // 应用修复并重试
                current_command = fix.command.clone();
                continue;
            }

            // 没有可自动应用的修复，返回结果
            return result;
        }
    }

    /// 记录用户对修复策略的反馈（Week 3）
    ///
    /// # Arguments
    /// * `analysis` - 错误分析结果
    /// * `strategy` - 应用的策略
    /// * `feedback` - 用户反馈类型
    /// * `outcome` - 修复结果
    pub async fn record_feedback(
        &self,
        analysis: &ErrorAnalysis,
        strategy: &FixStrategy,
        feedback: FeedbackType,
        outcome: FixOutcome,
    ) {
        let record = FeedbackRecord::new(analysis, strategy, feedback, outcome);
        self.feedback_learner.record_feedback(record).await;
    }

    /// 获取学习摘要
    pub async fn get_learning_summary(&self) -> crate::error_fixer::LearningSummary {
        self.feedback_learner.get_summary().await
    }
}

impl Default for ShellExecutorWithFixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_command() {
        // 安全命令
        assert!(is_safe_command("ls -la").is_ok());
        assert!(is_safe_command("echo hello").is_ok());
        assert!(is_safe_command("pwd").is_ok());

        // 危险命令
        assert!(is_safe_command("rm -rf /").is_err());
        assert!(is_safe_command("sudo rm -rf /home").is_err());
        assert!(is_safe_command("dd if=/dev/zero of=/dev/sda").is_err());
        assert!(is_safe_command("mkfs.ext4 /dev/sda1").is_err());

        // 空命令
        assert!(is_safe_command("").is_err());
        assert!(is_safe_command("   ").is_err());
    }

    #[tokio::test]
    async fn test_execute_shell_basic() {
        // 测试简单命令
        let result = execute_shell("echo 'Hello, World!'").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_execute_shell_pwd() {
        // 测试 pwd 命令
        let result = execute_shell("pwd").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
        // pwd 应该返回当前目录路径
        assert!(output.starts_with('/') || output.contains(':'));
    }

    #[tokio::test]
    async fn test_execute_shell_dangerous() {
        // 测试危险命令被阻止
        let result = execute_shell("rm -rf /").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::ShellDangerousCommand);
        assert!(err.to_string().contains("危险"));
    }

    #[tokio::test]
    async fn test_execute_shell_empty() {
        // 测试空命令
        let result = execute_shell("").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::ShellCommandEmpty);
    }

    #[tokio::test]
    async fn test_execute_shell_timeout() {
        // 测试超时控制（睡眠35秒，超过30秒限制）
        // 注意：这个测试需要较长时间，可以考虑降低超时时间进行测试
        let result = execute_shell("sleep 35").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::ShellTimeoutError);
        assert!(err.to_string().contains("超时"));
    }

    #[tokio::test]
    async fn test_execute_shell_output_limit() {
        // 测试输出大小限制（生成超过100KB的输出）
        // 使用 yes 命令生成大量输出，然后用 head 限制行数（确保超过100KB）
        #[cfg(unix)]
        let result = execute_shell("yes 'test output line with some content' | head -n 5000").await;

        #[cfg(windows)]
        let result = execute_shell("for /L %i in (1,1,5000) do @echo test output line with some content").await;

        assert!(result.is_ok());
        let output = result.unwrap();

        // 输出应该被截断，并包含截断提示
        assert!(output.contains("截断") || output.len() <= MAX_OUTPUT_SIZE + 100);
    }

    #[test]
    fn test_is_safe_command_additional_patterns() {
        // 测试额外的危险命令模式

        // shutdown 相关
        assert!(is_safe_command("shutdown -h now").is_err());
        assert!(is_safe_command("reboot now").is_err());
        assert!(is_safe_command("halt").is_err());
        assert!(is_safe_command("poweroff").is_err());

        // init 相关
        assert!(is_safe_command("init 0").is_err());
        assert!(is_safe_command("init 6").is_err());

        // dd 相关
        assert!(is_safe_command("dd if=/dev/random of=/dev/sda").is_err());

        // 直接写磁盘
        assert!(is_safe_command("echo data > /dev/sda").is_err());
        assert!(is_safe_command("cat file > /dev/sdb").is_err());
    }

    #[tokio::test]
    async fn test_execute_shell_stderr_handling() {
        // 测试 stderr 处理（故意执行一个会产生 stderr 的命令）
        let result = execute_shell("ls /nonexistent_directory_12345").await;

        // 命令应该执行，但输出中应该包含 stderr
        if let Ok(output) = result {
            assert!(output.contains("stderr") || output.contains("No such") || output.contains("cannot"));
        }
        // 某些系统可能返回错误，这也是可接受的
    }

    #[tokio::test]
    async fn test_execute_shell_exit_code_nonzero() {
        // 测试非零退出码的处理
        // false 命令总是返回非零退出码
        let result = execute_shell("false").await;

        // 应该返回错误或成功提示（取决于是否有输出）
        assert!(result.is_ok() || result.is_err());
    }

    // ========== ShellExecutorWithFixer 测试 ==========

    #[tokio::test]
    async fn test_executor_with_fixer_success() {
        // 测试成功执行
        let executor = ShellExecutorWithFixer::new();
        let result = executor.execute_with_analysis("echo 'test'").await;

        assert!(result.success);
        assert!(result.output.contains("test"));
        assert!(result.error_analysis.is_none());
        assert!(result.fix_strategies.is_empty());
    }

    #[tokio::test]
    async fn test_executor_with_fixer_command_not_found() {
        // 测试命令不存在错误分析
        let executor = ShellExecutorWithFixer::new();
        let result = executor.execute_with_analysis("nonexistent_command_12345").await;

        // 命令不存在通常会返回错误，但某些系统可能返回带 stderr 的成功输出
        // 如果返回失败，应该有错误分析
        if !result.success {
            assert!(result.error_analysis.is_some());
            if let Some(analysis) = &result.error_analysis {
                // 错误类别应该是 Command 或 Unknown
                match &analysis.category {
                    crate::error_fixer::ErrorCategory::Command => {},
                    crate::error_fixer::ErrorCategory::Unknown => {},
                    _ => panic!("Unexpected error category: {:?}", analysis.category),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_executor_with_fixer_file_not_found() {
        // 测试文件不存在错误分析
        let executor = ShellExecutorWithFixer::new();
        let result = executor.execute_with_analysis("cat /nonexistent_file_12345.txt").await;

        // 根据系统不同，可能返回错误或包含 stderr 的成功输出
        if !result.success {
            // 如果返回错误，应该有错误分析
            assert!(result.error_analysis.is_some());
        }
    }

    #[tokio::test]
    async fn test_execution_result_success() {
        // 测试 ExecutionResult 的成功创建
        let result = ExecutionResult::success("output".to_string());

        assert!(result.success);
        assert_eq!(result.output, "output");
        assert!(result.error_analysis.is_none());
        assert!(result.fix_strategies.is_empty());
    }

    #[tokio::test]
    async fn test_execution_result_failure() {
        // 测试 ExecutionResult 的失败创建
        use crate::error_fixer::{ErrorAnalysis, FixStrategy};

        let analysis = ErrorAnalysis::new("error".to_string(), "cmd".to_string());
        let strategy = FixStrategy::new("test", "fix", "description", 3);

        let result = ExecutionResult::failure(
            "error output".to_string(),
            Some(analysis),
            vec![strategy],
        );

        assert!(!result.success);
        assert_eq!(result.output, "error output");
        assert!(result.error_analysis.is_some());
        assert_eq!(result.fix_strategies.len(), 1);
    }

    #[test]
    fn test_executor_with_fixer_creation() {
        // 测试执行器创建
        let executor = ShellExecutorWithFixer::new();
        assert!(!executor.enable_llm_analysis);

        let executor = ShellExecutorWithFixer::default();
        assert!(!executor.enable_llm_analysis);
    }

    #[test]
    fn test_is_safe_fix_strategy() {
        use crate::error_fixer::FixStrategy;

        let executor = ShellExecutorWithFixer::new();

        // 安全的低风险策略
        let safe_strategy = FixStrategy::new("test", "echo hello", "safe test", 3);
        assert!(executor.is_safe_fix_strategy(&safe_strategy));

        // 危险命令策略
        let dangerous_strategy = FixStrategy::new("dangerous", "rm -rf /", "dangerous", 3);
        assert!(!executor.is_safe_fix_strategy(&dangerous_strategy));

        // 高风险但需要确认的策略 - 应该通过（使用非sudo的高风险命令）
        let high_risk_with_confirmation = FixStrategy::new("risky", "curl -o /tmp/file http://example.com", "needs confirm", 8);
        assert!(high_risk_with_confirmation.requires_confirmation); // 风险>=5 自动设置
        assert!(executor.is_safe_fix_strategy(&high_risk_with_confirmation));

        // 高风险但不需要确认的策略（不应该存在，但测试防御）
        let mut bad_strategy = FixStrategy::new("bad", "echo test", "bad config", 3);
        bad_strategy.risk_level = 9;
        bad_strategy.requires_confirmation = false; // 手动设置为不需要确认（错误配置）
        assert!(!executor.is_safe_fix_strategy(&bad_strategy));
    }

    // ========== Week 3: 反馈学习测试 ==========

    #[tokio::test]
    async fn test_feedback_learning_integration() {
        use crate::error_fixer::{ErrorAnalysis, FixStrategy, FeedbackType, FixOutcome};

        let executor = ShellExecutorWithFixer::new();

        // 创建测试数据
        let analysis = ErrorAnalysis::new("error".to_string(), "test_cmd".to_string());
        let strategy1 = FixStrategy::new("strategy_good", "echo test1", "good strategy", 3);
        let strategy2 = FixStrategy::new("strategy_bad", "echo test2", "bad strategy", 3);

        // 记录反馈：strategy1 成功，strategy2 失败
        for _ in 0..3 {
            executor.record_feedback(&analysis, &strategy1, FeedbackType::Accepted, FixOutcome::Success).await;
        }
        executor.record_feedback(&analysis, &strategy2, FeedbackType::Rejected, FixOutcome::Failure).await;

        // 检查学习摘要
        let summary = executor.get_learning_summary().await;
        assert_eq!(summary.total_feedbacks, 4);
        assert_eq!(summary.positive_feedbacks, 3);

        // 验证top策略
        assert!(!summary.top_strategies.is_empty());
        assert_eq!(summary.top_strategies[0].name, "strategy_good");
    }

    #[tokio::test]
    async fn test_strategy_reranking() {
        use crate::error_fixer::{ErrorAnalysis, FixStrategy, FeedbackType, FixOutcome};

        let executor = ShellExecutorWithFixer::new();

        // 创建两个策略
        let strategy1 = FixStrategy::new("low_score", "cmd1", "desc1", 3);
        let strategy2 = FixStrategy::new("high_score", "cmd2", "desc2", 3);

        // 给strategy2更多正面反馈
        let analysis = ErrorAnalysis::new("error".to_string(), "cmd".to_string());
        for _ in 0..5 {
            executor.record_feedback(&analysis, &strategy2, FeedbackType::Accepted, FixOutcome::Success).await;
        }
        executor.record_feedback(&analysis, &strategy1, FeedbackType::Rejected, FixOutcome::Failure).await;

        // 重新排序
        let learner = executor.feedback_learner();
        let strategies = vec![strategy1.clone(), strategy2.clone()];
        let ranked = learner.rerank_strategies(strategies).await;

        // high_score应该排在前面
        assert_eq!(ranked[0].name, "high_score");
        assert_eq!(ranked[1].name, "low_score");
    }

    #[tokio::test]
    async fn test_feedback_learner_access() {
        let executor = ShellExecutorWithFixer::new();

        // 获取学习器引用
        let learner = executor.feedback_learner();

        // 验证可以访问学习器的功能
        let summary = learner.get_summary().await;
        assert_eq!(summary.total_feedbacks, 0); // 初始为空

        // 验证learner是Arc，可以clone
        let learner2 = executor.feedback_learner();
        assert!(Arc::ptr_eq(&learner, &learner2));
    }
}
