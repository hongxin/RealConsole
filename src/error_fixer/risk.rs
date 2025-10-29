//! 风险评估系统 - Phase 2 (v1.16.0)
//!
//! 提供命令和操作的风险分级评估：
//! - 安全（Safe）: 绿色，直接执行
//! - 低风险（Low）: 黄色，提示确认
//! - 中风险（Medium）: 橙色，详细说明
//! - 高风险（High）: 红色，强制二次确认

use serde::{Deserialize, Serialize};

/// 风险级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// 安全操作
    Safe,
    /// 低风险操作
    Low,
    /// 中风险操作
    Medium,
    /// 高风险操作
    High,
}

impl RiskLevel {
    /// 获取显示符号
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Safe => "✅",
            Self::Low => "⚠️",
            Self::Medium => "🔶",
            Self::High => "🔴",
        }
    }

    /// 获取颜色代码（用于终端显示）
    pub fn color(&self) -> &'static str {
        match self {
            Self::Safe => "green",
            Self::Low => "yellow",
            Self::Medium => "bright_yellow",
            Self::High => "red",
        }
    }

    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Safe => "安全操作",
            Self::Low => "低风险操作",
            Self::Medium => "中风险操作",
            Self::High => "高风险操作",
        }
    }

    /// 是否需要确认
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::Medium | Self::High)
    }

    /// 是否需要强制二次确认
    pub fn requires_double_confirmation(&self) -> bool {
        matches!(self, Self::High)
    }

    /// 从分数转换 (0.0-1.0, 越高越危险)
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.25 => Self::Safe,
            s if s < 0.5 => Self::Low,
            s if s < 0.75 => Self::Medium,
            _ => Self::High,
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.symbol(), self.description())
    }
}

/// 风险评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// 风险级别
    pub level: RiskLevel,

    /// 风险评分 (0.0-1.0, 越高越危险)
    pub score: f64,

    /// 风险因素
    pub factors: Vec<RiskFactor>,

    /// 警告信息
    pub warnings: Vec<String>,

    /// 建议
    pub recommendations: Vec<String>,
}

impl RiskAssessment {
    /// 创建新的风险评估
    pub fn new(level: RiskLevel, score: f64) -> Self {
        Self {
            level,
            score: score.clamp(0.0, 1.0),
            factors: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// 创建安全评估
    pub fn safe() -> Self {
        Self::new(RiskLevel::Safe, 0.0)
    }

    /// 添加风险因素
    pub fn add_factor(&mut self, factor: RiskFactor) {
        self.factors.push(factor);
    }

    /// 添加警告
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// 添加建议
    pub fn add_recommendation(&mut self, recommendation: impl Into<String>) {
        self.recommendations.push(recommendation.into());
    }

    /// 是否安全
    pub fn is_safe(&self) -> bool {
        self.level == RiskLevel::Safe
    }

    /// 是否需要确认
    pub fn requires_confirmation(&self) -> bool {
        self.level.requires_confirmation()
    }
}

/// 风险因素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// 因素类型
    pub factor_type: RiskFactorType,

    /// 描述
    pub description: String,

    /// 影响程度 (0.0-1.0)
    pub impact: f64,
}

impl RiskFactor {
    /// 创建新的风险因素
    pub fn new(factor_type: RiskFactorType, description: impl Into<String>, impact: f64) -> Self {
        Self {
            factor_type,
            description: description.into(),
            impact: impact.clamp(0.0, 1.0),
        }
    }
}

/// 风险因素类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskFactorType {
    /// 删除操作
    Deletion,
    /// 权限提升
    PrivilegeEscalation,
    /// 系统文件
    SystemFile,
    /// 网络操作
    Network,
    /// 递归操作
    Recursive,
    /// 强制操作
    Force,
    /// 数据丢失风险
    DataLoss,
    /// 不可逆操作
    Irreversible,
}

impl RiskFactorType {
    /// 获取基础风险分数
    pub fn base_risk(&self) -> f64 {
        match self {
            Self::Deletion => 0.6,
            Self::PrivilegeEscalation => 0.8,
            Self::SystemFile => 0.9,
            Self::Network => 0.4,
            Self::Recursive => 0.5,
            Self::Force => 0.7,
            Self::DataLoss => 0.9,
            Self::Irreversible => 0.85,
        }
    }

    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Deletion => "删除操作",
            Self::PrivilegeEscalation => "权限提升",
            Self::SystemFile => "系统文件操作",
            Self::Network => "网络操作",
            Self::Recursive => "递归操作",
            Self::Force => "强制操作",
            Self::DataLoss => "数据丢失风险",
            Self::Irreversible => "不可逆操作",
        }
    }
}

/// 风险评估器
pub struct RiskAssessor {
    /// 是否启用严格模式
    strict_mode: bool,
}

impl RiskAssessor {
    /// 创建新的风险评估器
    pub fn new() -> Self {
        Self {
            strict_mode: false,
        }
    }

    /// 启用严格模式
    pub fn with_strict_mode(mut self, enabled: bool) -> Self {
        self.strict_mode = enabled;
        self
    }

    /// 评估命令风险
    pub fn assess_command(&self, command: &str) -> RiskAssessment {
        let mut factors = Vec::new();

        // 检查删除操作
        if command.contains("rm") {
            let impact = if command.contains("-rf") || command.contains("-fr") {
                0.9
            } else if command.contains("-r") || command.contains("-f") {
                0.7
            } else {
                0.5
            };

            factors.push(RiskFactor::new(
                RiskFactorType::Deletion,
                "包含删除命令",
                impact,
            ));

            if command.contains("-r") {
                factors.push(RiskFactor::new(
                    RiskFactorType::Recursive,
                    "递归删除",
                    0.8,
                ));
            }
        }

        // 检查权限提升
        if command.starts_with("sudo") || command.contains("sudo") {
            factors.push(RiskFactor::new(
                RiskFactorType::PrivilegeEscalation,
                "需要管理员权限",
                0.7,
            ));
        }

        // 检查系统文件
        if command.contains("/etc/")
            || command.contains("/var/")
            || command.contains("/sys/")
            || command.contains("/boot/")
        {
            factors.push(RiskFactor::new(
                RiskFactorType::SystemFile,
                "操作系统目录",
                0.9,
            ));
        }

        // 检查强制操作
        if command.contains(" -f") || command.contains("--force") {
            factors.push(RiskFactor::new(
                RiskFactorType::Force,
                "强制执行，跳过安全检查",
                0.7,
            ));
        }

        // 检查其他危险命令
        for dangerous_cmd in &["dd", "mkfs", "fdisk", "parted", "shutdown", "reboot"] {
            if command.contains(dangerous_cmd) {
                factors.push(RiskFactor::new(
                    RiskFactorType::DataLoss,
                    format!("包含危险命令: {}", dangerous_cmd),
                    0.9,
                ));
                break;
            }
        }

        // 计算风险分数：取最高风险因素，多个因素增加累积效应
        let mut score = if factors.is_empty() {
            0.0
        } else {
            // 找出最高的单个风险分数
            let max_single_risk = factors
                .iter()
                .map(|f| f.impact * f.factor_type.base_risk())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);

            // 多个因素会增加风险（每多一个因素增加15%）
            let factor_multiplier = 1.0 + (factors.len().saturating_sub(1) as f64 * 0.15);
            (max_single_risk * factor_multiplier).min(1.0)
        };

        // 严格模式提高风险等级
        if self.strict_mode && score > 0.0 {
            score = (score * 1.2).min(1.0);
        }

        let level = RiskLevel::from_score(score);
        let mut assessment = RiskAssessment::new(level, score);
        assessment.factors = factors;

        // 添加警告和建议
        match level {
            RiskLevel::High => {
                assessment.add_warning("这是一个高风险操作！");
                assessment.add_recommendation("请仔细检查命令是否正确");
                assessment.add_recommendation("建议先在测试环境验证");
            }
            RiskLevel::Medium => {
                assessment.add_warning("请注意操作风险");
                assessment.add_recommendation("确认操作对象是否正确");
            }
            RiskLevel::Low => {
                assessment.add_recommendation("建议检查命令参数");
            }
            RiskLevel::Safe => {}
        }

        assessment
    }
}

impl Default for RiskAssessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Safe < RiskLevel::Low);
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
    }

    #[test]
    fn test_risk_level_confirmation() {
        assert!(!RiskLevel::Safe.requires_confirmation());
        assert!(!RiskLevel::Low.requires_confirmation());
        assert!(RiskLevel::Medium.requires_confirmation());
        assert!(RiskLevel::High.requires_confirmation());

        assert!(!RiskLevel::Safe.requires_double_confirmation());
        assert!(!RiskLevel::Low.requires_double_confirmation());
        assert!(!RiskLevel::Medium.requires_double_confirmation());
        assert!(RiskLevel::High.requires_double_confirmation());
    }

    #[test]
    fn test_safe_command() {
        let assessor = RiskAssessor::new();
        let assessment = assessor.assess_command("ls -la");

        assert_eq!(assessment.level, RiskLevel::Safe);
        assert!(assessment.is_safe());
    }

    #[test]
    fn test_dangerous_rm_command() {
        let assessor = RiskAssessor::new();
        let assessment = assessor.assess_command("rm -rf /important/data");

        assert!(assessment.level >= RiskLevel::Medium);
        assert!(!assessment.is_safe());
        assert!(assessment.requires_confirmation());
    }

    #[test]
    fn test_sudo_command() {
        let assessor = RiskAssessor::new();
        let assessment = assessor.assess_command("sudo apt-get install something");

        assert!(assessment.level >= RiskLevel::Low);
        assert!(assessment
            .factors
            .iter()
            .any(|f| f.factor_type == RiskFactorType::PrivilegeEscalation));
    }

    #[test]
    fn test_system_file_command() {
        let assessor = RiskAssessor::new();
        let assessment = assessor.assess_command("sudo rm -rf /etc/config");

        assert_eq!(assessment.level, RiskLevel::High);
        assert!(assessment.level.requires_double_confirmation());
    }

    #[test]
    fn test_strict_mode() {
        let normal = RiskAssessor::new();
        let strict = RiskAssessor::new().with_strict_mode(true);

        let cmd = "rm file.txt";
        let normal_assessment = normal.assess_command(cmd);
        let strict_assessment = strict.assess_command(cmd);

        assert!(strict_assessment.score > normal_assessment.score);
    }

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(RiskLevel::from_score(0.0), RiskLevel::Safe);
        assert_eq!(RiskLevel::from_score(0.2), RiskLevel::Safe);
        assert_eq!(RiskLevel::from_score(0.3), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(0.6), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(0.9), RiskLevel::High);
    }
}
