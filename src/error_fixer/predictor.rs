//! 下一步预测系统 - Phase 2 (v1.16.0)
//!
//! 基于当前操作和上下文，预测用户接下来可能需要执行的操作

use serde::{Deserialize, Serialize};

/// 下一步预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextStep {
    /// 操作描述
    pub action: String,

    /// 具体命令（如果有）
    pub command: Option<String>,

    /// 预测概率 (0.0-1.0)
    pub probability: f64,

    /// 执行该步骤的好处
    pub benefit: String,

    /// 相关类别
    pub category: NextStepCategory,
}

impl NextStep {
    /// 创建新的下一步预测
    pub fn new(
        action: impl Into<String>,
        command: Option<impl Into<String>>,
        probability: f64,
    ) -> Self {
        Self {
            action: action.into(),
            command: command.map(|c| c.into()),
            probability: probability.clamp(0.0, 1.0),
            benefit: String::new(),
            category: NextStepCategory::General,
        }
    }

    /// 设置好处说明
    pub fn with_benefit(mut self, benefit: impl Into<String>) -> Self {
        self.benefit = benefit.into();
        self
    }

    /// 设置类别
    pub fn with_category(mut self, category: NextStepCategory) -> Self {
        self.category = category;
        self
    }

    /// 获取概率等级
    pub fn probability_level(&self) -> ProbabilityLevel {
        match self.probability {
            p if p >= 0.75 => ProbabilityLevel::VeryLikely,
            p if p >= 0.5 => ProbabilityLevel::Likely,
            p if p >= 0.25 => ProbabilityLevel::Possible,
            _ => ProbabilityLevel::Unlikely,
        }
    }
}

/// 概率等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbabilityLevel {
    /// 非常可能 (>= 0.75)
    VeryLikely,
    /// 可能 (>= 0.5)
    Likely,
    /// 也许 (>= 0.25)
    Possible,
    /// 不太可能 (< 0.25)
    Unlikely,
}

impl ProbabilityLevel {
    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::VeryLikely => "非常可能",
            Self::Likely => "可能",
            Self::Possible => "也许",
            Self::Unlikely => "不太可能",
        }
    }

    /// 获取符号
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::VeryLikely => "🔥",
            Self::Likely => "⭐",
            Self::Possible => "💡",
            Self::Unlikely => "·",
        }
    }
}

/// 下一步类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextStepCategory {
    /// 构建相关
    Build,
    /// 测试相关
    Test,
    /// 运行相关
    Run,
    /// 调试相关
    Debug,
    /// 部署相关
    Deploy,
    /// 清理相关
    Cleanup,
    /// 验证相关
    Verification,
    /// 一般操作
    General,
}

impl NextStepCategory {
    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Build => "构建",
            Self::Test => "测试",
            Self::Run => "运行",
            Self::Debug => "调试",
            Self::Deploy => "部署",
            Self::Cleanup => "清理",
            Self::Verification => "验证",
            Self::General => "一般操作",
        }
    }
}

/// 下一步预测器
#[derive(Clone)]
pub struct NextStepPredictor {
    /// 历史命令缓存
    history: Vec<String>,
}

impl NextStepPredictor {
    /// 创建新的预测器
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// 记录命令历史
    pub fn record_command(&mut self, command: &str) {
        self.history.push(command.to_string());
        // 保留最近50条
        if self.history.len() > 50 {
            self.history.remove(0);
        }
    }

    /// 预测下一步操作
    pub fn predict(&self, current_command: &str, success: bool) -> Vec<NextStep> {
        let mut predictions = Vec::new();

        // 根据当前命令和执行结果预测
        if success {
            predictions.extend(self.predict_after_success(current_command));
        } else {
            predictions.extend(self.predict_after_failure(current_command));
        }

        // 按概率排序
        predictions.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());

        predictions
    }

    /// 成功后的预测
    fn predict_after_success(&self, command: &str) -> Vec<NextStep> {
        let mut steps = Vec::new();

        // cargo build 成功后
        if command.contains("cargo build") {
            steps.push(
                NextStep::new("运行测试", Some("cargo test"), 0.8)
                    .with_benefit("验证代码正确性")
                    .with_category(NextStepCategory::Test),
            );

            steps.push(
                NextStep::new("运行程序", Some("cargo run"), 0.7)
                    .with_benefit("查看程序输出")
                    .with_category(NextStepCategory::Run),
            );

            if command.contains("--release") {
                steps.push(
                    NextStep::new(
                        "执行可执行文件",
                        Some("./target/release/[binary]"),
                        0.6,
                    )
                    .with_benefit("直接运行已编译的二进制")
                    .with_category(NextStepCategory::Run),
                );
            }
        }

        // cargo test 成功后
        if command.contains("cargo test") {
            steps.push(
                NextStep::new("运行程序", Some("cargo run"), 0.75)
                    .with_benefit("测试通过，可以运行程序")
                    .with_category(NextStepCategory::Run),
            );

            steps.push(
                NextStep::new("生成测试覆盖率报告", Some("cargo tarpaulin"), 0.4)
                    .with_benefit("了解测试覆盖情况")
                    .with_category(NextStepCategory::Verification),
            );
        }

        // git add 成功后
        if command.contains("git add") {
            steps.push(
                NextStep::new("提交更改", Some("git commit -m \"...\""), 0.9)
                    .with_benefit("保存更改到本地仓库")
                    .with_category(NextStepCategory::General),
            );
        }

        // git commit 成功后
        if command.contains("git commit") {
            steps.push(
                NextStep::new("推送到远程", Some("git push"), 0.85)
                    .with_benefit("同步到远程仓库")
                    .with_category(NextStepCategory::Deploy),
            );
        }

        // npm install 成功后
        if command.contains("npm install") {
            steps.push(
                NextStep::new("启动开发服务器", Some("npm run dev"), 0.8)
                    .with_benefit("开始开发")
                    .with_category(NextStepCategory::Run),
            );

            steps.push(
                NextStep::new("构建生产版本", Some("npm run build"), 0.5)
                    .with_benefit("准备生产部署")
                    .with_category(NextStepCategory::Build),
            );
        }

        // make 成功后
        if command == "make" || command.starts_with("make ") {
            steps.push(
                NextStep::new("运行可执行文件", Some("./[binary]"), 0.7)
                    .with_benefit("查看编译结果")
                    .with_category(NextStepCategory::Run),
            );

            steps.push(
                NextStep::new("安装", Some("make install"), 0.5)
                    .with_benefit("安装到系统")
                    .with_category(NextStepCategory::Deploy),
            );
        }

        // docker build 成功后
        if command.contains("docker build") {
            steps.push(
                NextStep::new("运行容器", Some("docker run"), 0.85)
                    .with_benefit("测试容器镜像")
                    .with_category(NextStepCategory::Run),
            );

            steps.push(
                NextStep::new("推送镜像", Some("docker push"), 0.6)
                    .with_benefit("部署到仓库")
                    .with_category(NextStepCategory::Deploy),
            );
        }

        steps
    }

    /// 失败后的预测
    fn predict_after_failure(&self, command: &str) -> Vec<NextStep> {
        let mut steps = Vec::new();

        // cargo build 失败后
        if command.contains("cargo build") {
            steps.push(
                NextStep::new("查看详细错误", Some("cargo build --verbose"), 0.6)
                    .with_benefit("获取更多错误信息")
                    .with_category(NextStepCategory::Debug),
            );

            steps.push(
                NextStep::new("运行 clippy 检查", Some("cargo clippy"), 0.7)
                    .with_benefit("发现代码问题")
                    .with_category(NextStepCategory::Verification),
            );

            steps.push(
                NextStep::new("清理并重新构建", Some("cargo clean && cargo build"), 0.4)
                    .with_benefit("解决缓存问题")
                    .with_category(NextStepCategory::Cleanup),
            );
        }

        // cargo test 失败后
        if command.contains("cargo test") {
            steps.push(
                NextStep::new("单独运行失败的测试", Some("cargo test [test_name]"), 0.8)
                    .with_benefit("隔离问题")
                    .with_category(NextStepCategory::Debug),
            );

            steps.push(
                NextStep::new("显示测试输出", Some("cargo test -- --nocapture"), 0.7)
                    .with_benefit("查看详细输出")
                    .with_category(NextStepCategory::Debug),
            );
        }

        // git push 失败后
        if command.contains("git push") {
            steps.push(
                NextStep::new("拉取最新更改", Some("git pull"), 0.85)
                    .with_benefit("同步远程更改")
                    .with_category(NextStepCategory::General),
            );

            steps.push(
                NextStep::new("强制推送", Some("git push --force"), 0.3)
                    .with_benefit("覆盖远程分支（危险）")
                    .with_category(NextStepCategory::General),
            );
        }

        // npm install 失败后
        if command.contains("npm install") {
            steps.push(
                NextStep::new("清理缓存", Some("npm cache clean --force"), 0.7)
                    .with_benefit("解决缓存问题")
                    .with_category(NextStepCategory::Cleanup),
            );

            steps.push(
                NextStep::new("删除 node_modules 重新安装", Some("rm -rf node_modules && npm install"), 0.6)
                    .with_benefit("完全重新安装依赖")
                    .with_category(NextStepCategory::Cleanup),
            );
        }

        steps
    }

    /// 清除历史
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// 获取历史命令数
    pub fn history_count(&self) -> usize {
        self.history.len()
    }
}

impl Default for NextStepPredictor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_step_creation() {
        let step = NextStep::new("Run tests", Some("cargo test"), 0.8)
            .with_benefit("Verify code correctness")
            .with_category(NextStepCategory::Test);

        assert_eq!(step.action, "Run tests");
        assert_eq!(step.command, Some("cargo test".to_string()));
        assert_eq!(step.probability, 0.8);
        assert_eq!(step.benefit, "Verify code correctness");
        assert_eq!(step.category, NextStepCategory::Test);
    }

    #[test]
    fn test_probability_level() {
        let very_likely = NextStep::new("test", None::<String>, 0.9);
        assert_eq!(very_likely.probability_level(), ProbabilityLevel::VeryLikely);

        let likely = NextStep::new("test", None::<String>, 0.6);
        assert_eq!(likely.probability_level(), ProbabilityLevel::Likely);

        let possible = NextStep::new("test", None::<String>, 0.3);
        assert_eq!(possible.probability_level(), ProbabilityLevel::Possible);

        let unlikely = NextStep::new("test", None::<String>, 0.1);
        assert_eq!(unlikely.probability_level(), ProbabilityLevel::Unlikely);
    }

    #[test]
    fn test_predict_after_cargo_build_success() {
        let predictor = NextStepPredictor::new();
        let steps = predictor.predict("cargo build", true);

        assert!(!steps.is_empty());
        assert!(steps.iter().any(|s| s.command.as_ref().is_some_and(|c| c.contains("cargo test"))));
        assert!(steps.iter().any(|s| s.command.as_ref().is_some_and(|c| c.contains("cargo run"))));
    }

    #[test]
    fn test_predict_after_cargo_build_failure() {
        let predictor = NextStepPredictor::new();
        let steps = predictor.predict("cargo build", false);

        assert!(!steps.is_empty());
        assert!(steps.iter().any(|s| s.category == NextStepCategory::Debug));
    }

    #[test]
    fn test_predict_after_git_commit() {
        let predictor = NextStepPredictor::new();
        let steps = predictor.predict("git commit -m 'test'", true);

        assert!(!steps.is_empty());
        assert!(steps.iter().any(|s| s.command.as_ref().is_some_and(|c| c.contains("git push"))));
    }

    #[test]
    fn test_history_management() {
        let mut predictor = NextStepPredictor::new();

        predictor.record_command("cargo build");
        predictor.record_command("cargo test");
        assert_eq!(predictor.history_count(), 2);

        predictor.clear_history();
        assert_eq!(predictor.history_count(), 0);
    }

    #[test]
    fn test_history_limit() {
        let mut predictor = NextStepPredictor::new();

        // 添加超过50条命令
        for i in 0..60 {
            predictor.record_command(&format!("command {}", i));
        }

        // 应该只保留最近50条
        assert_eq!(predictor.history_count(), 50);
    }

    #[test]
    fn test_probability_clamping() {
        let step = NextStep::new("test", None::<String>, 1.5); // 超过1.0
        assert_eq!(step.probability, 1.0);

        let step2 = NextStep::new("test", None::<String>, -0.5); // 小于0.0
        assert_eq!(step2.probability, 0.0);
    }
}
