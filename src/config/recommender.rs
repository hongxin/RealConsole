// Configuration Recommender - 智能配置推荐
//
// 基于用户环境和画像，推荐最佳配置方案

use super::detector::{EnvironmentInfo, UserProfile};
use anyhow::Result;

/// LLM 推荐
#[derive(Debug, Clone)]
pub struct LlmRecommendation {
    pub provider: String,       // "deepseek", "ollama", "openai"
    pub reason: String,         // 推荐理由
    pub confidence: f64,        // 推荐置信度 (0.0-1.0)
    pub alternatives: Vec<String>, // 备选方案
}

/// 配置模式推荐
#[derive(Debug, Clone)]
pub struct ModeRecommendation {
    pub mode: String,           // "minimal", "standard", "advanced"
    pub reason: String,
}

/// 建议频率推荐
#[derive(Debug, Clone)]
pub struct SuggestionFrequency {
    pub level: String,          // "aggressive", "moderate", "conservative"
    pub reason: String,
}

/// 配置推荐器
pub struct ConfigRecommender;

impl ConfigRecommender {
    pub fn new() -> Self {
        Self
    }

    /// 推荐 LLM 后端
    pub fn recommend_llm(&self, env: &EnvironmentInfo) -> LlmRecommendation {
        // 基于用户画像和环境推荐
        match (&env.user_profile, env.os.os_type.as_str()) {
            // DevOps 用户，推荐稳定快速的 Deepseek
            (UserProfile::DevOps, _) => LlmRecommendation {
                provider: "deepseek".to_string(),
                reason: "DevOps 场景需要快速响应，Deepseek 延迟低且稳定".to_string(),
                confidence: 0.9,
                alternatives: vec!["openai".to_string()],
            },

            // 开发者，推荐 Deepseek（性价比高）
            (UserProfile::Developer, _) => LlmRecommendation {
                provider: "deepseek".to_string(),
                reason: "开发场景下 Deepseek 的代码理解能力强，价格实惠".to_string(),
                confidence: 0.85,
                alternatives: vec!["ollama".to_string(), "openai".to_string()],
            },

            // 学生，推荐 Ollama（本地免费）
            (UserProfile::Student, _) => LlmRecommendation {
                provider: "ollama".to_string(),
                reason: "学习环境推荐本地运行的 Ollama，完全免费".to_string(),
                confidence: 0.8,
                alternatives: vec!["deepseek".to_string()],
            },

            // 未知用户，默认推荐 Deepseek
            _ => LlmRecommendation {
                provider: "deepseek".to_string(),
                reason: "Deepseek 是平衡性能、价格和稳定性的最佳选择".to_string(),
                confidence: 0.75,
                alternatives: vec!["ollama".to_string(), "openai".to_string()],
            },
        }
    }

    /// 推荐配置模式
    pub fn recommend_mode(&self, env: &EnvironmentInfo) -> ModeRecommendation {
        match &env.user_profile {
            UserProfile::DevOps => ModeRecommendation {
                mode: "standard".to_string(),
                reason: "运维工程师推荐标准模式，平衡功能和简洁性".to_string(),
            },
            UserProfile::Developer => ModeRecommendation {
                mode: "standard".to_string(),
                reason: "开发者推荐标准模式，满足日常开发需求".to_string(),
            },
            UserProfile::Student => ModeRecommendation {
                mode: "minimal".to_string(),
                reason: "学习者推荐极简模式，快速上手".to_string(),
            },
            UserProfile::Unknown => ModeRecommendation {
                mode: "minimal".to_string(),
                reason: "首次使用推荐极简模式，3个问题快速配置".to_string(),
            },
        }
    }

    /// 推荐建议频率
    pub fn recommend_suggestion_frequency(&self, env: &EnvironmentInfo) -> SuggestionFrequency {
        match &env.user_profile {
            UserProfile::DevOps => SuggestionFrequency {
                level: "moderate".to_string(),
                reason: "运维场景下，适度的建议避免干扰关键操作".to_string(),
            },
            UserProfile::Developer => SuggestionFrequency {
                level: "aggressive".to_string(),
                reason: "开发场景下，积极的建议提升效率".to_string(),
            },
            UserProfile::Student => SuggestionFrequency {
                level: "aggressive".to_string(),
                reason: "学习者需要更多指导，积极建议模式".to_string(),
            },
            UserProfile::Unknown => SuggestionFrequency {
                level: "moderate".to_string(),
                reason: "默认适度建议，平衡帮助和干扰".to_string(),
            },
        }
    }

    /// 推荐安全级别
    pub fn recommend_safety_level(&self, env: &EnvironmentInfo) -> String {
        match &env.user_profile {
            UserProfile::DevOps => "strict".to_string(),  // 运维场景，严格模式
            UserProfile::Developer => "standard".to_string(),
            UserProfile::Student => "standard".to_string(),
            UserProfile::Unknown => "strict".to_string(), // 未知用户，从严格开始
        }
    }

    /// 综合推荐
    pub fn recommend_all(&self, env: &EnvironmentInfo) -> AllRecommendations {
        AllRecommendations {
            llm: self.recommend_llm(env),
            mode: self.recommend_mode(env),
            suggestion_frequency: self.recommend_suggestion_frequency(env),
            safety_level: self.recommend_safety_level(env),
        }
    }
}

/// 所有推荐结果
#[derive(Debug, Clone)]
pub struct AllRecommendations {
    pub llm: LlmRecommendation,
    pub mode: ModeRecommendation,
    pub suggestion_frequency: SuggestionFrequency,
    pub safety_level: String,
}

impl Default for ConfigRecommender {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::detector::{OsInfo, ShellInfo};
    use std::path::PathBuf;

    fn create_test_env(profile: UserProfile) -> EnvironmentInfo {
        EnvironmentInfo {
            os: OsInfo {
                os_type: "macos".to_string(),
                version: "14.0".to_string(),
                arch: "arm64".to_string(),
            },
            shell: ShellInfo {
                shell_type: "zsh".to_string(),
                shell_path: PathBuf::from("/bin/zsh"),
                version: Some("5.9".to_string()),
            },
            tools: vec![],
            user_profile: profile,
            home_dir: PathBuf::from("/Users/test"),
            config_dir: PathBuf::from("/Users/test/.config/realconsole"),
        }
    }

    #[test]
    fn test_recommend_llm_for_developer() {
        let recommender = ConfigRecommender::new();
        let env = create_test_env(UserProfile::Developer);
        let rec = recommender.recommend_llm(&env);

        assert_eq!(rec.provider, "deepseek");
        assert!(rec.confidence > 0.8);
        assert!(!rec.alternatives.is_empty());
    }

    #[test]
    fn test_recommend_llm_for_devops() {
        let recommender = ConfigRecommender::new();
        let env = create_test_env(UserProfile::DevOps);
        let rec = recommender.recommend_llm(&env);

        assert_eq!(rec.provider, "deepseek");
        assert!(rec.confidence > 0.85);
    }

    #[test]
    fn test_recommend_llm_for_student() {
        let recommender = ConfigRecommender::new();
        let env = create_test_env(UserProfile::Student);
        let rec = recommender.recommend_llm(&env);

        assert_eq!(rec.provider, "ollama");
        assert_eq!(rec.alternatives[0], "deepseek");
    }

    #[test]
    fn test_recommend_mode() {
        let recommender = ConfigRecommender::new();

        let dev_env = create_test_env(UserProfile::Developer);
        let dev_mode = recommender.recommend_mode(&dev_env);
        assert_eq!(dev_mode.mode, "standard");

        let student_env = create_test_env(UserProfile::Student);
        let student_mode = recommender.recommend_mode(&student_env);
        assert_eq!(student_mode.mode, "minimal");
    }

    #[test]
    fn test_recommend_all() {
        let recommender = ConfigRecommender::new();
        let env = create_test_env(UserProfile::Developer);
        let recs = recommender.recommend_all(&env);

        assert_eq!(recs.llm.provider, "deepseek");
        assert_eq!(recs.mode.mode, "standard");
        assert_eq!(recs.suggestion_frequency.level, "aggressive");
        assert_eq!(recs.safety_level, "standard");
    }
}
