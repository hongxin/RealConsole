//! 多维向量决策系统 - 基于八卦哲学
//!
//! 基于 [docs/00-core/think.md](../../../../docs/00-core/think.md) 的易经智慧：
//! 将单一维度决策演化为**八卦场态决策**。
//!
//! # 八卦哲学基础
//!
//! 易经八卦：三爻组合，八种基本场态
//!
//! ```text
//! 乾 ☰ (111): 纯阳 - 最强状态
//! 坤 ☷ (000): 纯阴 - 最弱状态
//! 震 ☳ (001): 一阳在下 - 初动之象
//! 巽 ☴ (110): 一阴在下 - 渐进之象
//! 坎 ☵ (010): 阳被阴夹 - 险中有实
//! 离 ☲ (101): 阴被阳夹 - 外实内虚
//! 艮 ☶ (100): 一阳在上 - 止于至善
//! 兑 ☱ (011): 一阴在上 - 喜悦通达
//! ```
//!
//! # 三爻映射
//!
//! 将Intent决策映射为三爻系统：
//!
//! ```text
//! 上爻（天）：语义匹配维度  ━━━ 阳（高匹配） / ━ ━ 阴（低匹配）
//! 中爻（人）：风险安全维度  ━━━ 阳（低风险） / ━ ━ 阴（高风险）
//! 下爻（地）：上下文维度    ━━━ 阳（强相关） / ━ ━ 阴（弱相关）
//! ```
//!
//! # 八种基本场态
//!
//! | 卦 | 场态 | 语义 | 风险 | 上下文 | 决策建议 |
//! |-------|------|------|------|--------|----------|
//! | 乾 ☰ | 111 | 高 | 低 | 强 | **立即执行** - 完美匹配 |
//! | 兑 ☱ | 110 | 高 | 低 | 弱 | 执行（上下文不重要）|
//! | 离 ☲ | 101 | 高 | 高 | 强 | **需要确认** - 风险较高 |
//! | 震 ☳ | 100 | 高 | 高 | 弱 | 警告确认 - 高风险 |
//! | 巽 ☴ | 011 | 低 | 低 | 强 | 可尝试（语义弱但安全）|
//! | 坎 ☵ | 010 | 低 | 低 | 弱 | 不确定 - 建议澄清 |
//! | 艮 ☶ | 001 | 低 | 高 | 强 | **拒绝** - 不安全 |
//! | 坤 ☷ | 000 | 低 | 高 | 弱 | **拒绝** - 完全不匹配 |
//!
//! # 核心理念
//!
//! **不是单一分数**：而是三个维度构成的"卦象"
//! **不是 if/else 树**：而是八种基本场态的演化
//! **不是硬编码逻辑**：而是易经智慧的代码化

use serde::{Deserialize, Serialize};

/// 八卦场态（Trigram State）
///
/// 代表三爻组合的八种基本状态。
///
/// # 易经映射
///
/// 每个卦代表一种特定的决策场景：
/// - 乾 ☰: 完美匹配，立即执行
/// - 兑 ☱: 高质量匹配，可执行
/// - 离 ☲: 高匹配但高风险，需确认
/// - 震 ☳: 高风险，警告确认
/// - 巽 ☴: 低匹配但安全，可尝试
/// - 坎 ☵: 模糊不清，建议澄清
/// - 艮 ☶: 不安全，拒绝
/// - 坤 ☷: 完全不匹配，拒绝
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrigramState {
    /// 乾 ☰ (111): 纯阳 - 完美匹配
    Qian,

    /// 兑 ☱ (011): 一阴在上 - 喜悦通达
    Dui,

    /// 离 ☲ (101): 阴被阳夹 - 外实内虚
    Li,

    /// 震 ☳ (001): 一阳在下 - 初动之象
    Zhen,

    /// 巽 ☴ (110): 一阴在下 - 渐进之象
    Xun,

    /// 坎 ☵ (010): 阳被阴夹 - 险中有实
    Kan,

    /// 艮 ☶ (100): 一阳在上 - 止于至善
    Gen,

    /// 坤 ☷ (000): 纯阴 - 完全不匹配
    Kun,
}

/// 决策动作（基于卦象）
///
/// 根据八卦状态推荐的决策行为。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionAction {
    /// 立即执行 - 完美匹配，无需确认
    Execute,

    /// 需要确认 - 高匹配但有风险
    Confirm,

    /// 警告确认 - 风险较高，需要警告
    Warning,

    /// 可尝试 - 语义弱但安全
    Try,

    /// 建议澄清 - 不确定，需要更多信息
    Clarify,

    /// 拒绝执行 - 不匹配或不安全
    Reject,
}

impl DecisionAction {
    /// 获取决策动作的描述
    pub fn description(&self) -> &'static str {
        match self {
            DecisionAction::Execute => "立即执行",
            DecisionAction::Confirm => "需要确认",
            DecisionAction::Warning => "警告确认",
            DecisionAction::Try => "可尝试",
            DecisionAction::Clarify => "建议澄清",
            DecisionAction::Reject => "拒绝执行",
        }
    }
}

impl TrigramState {
    /// 获取卦的Unicode符号
    pub fn symbol(&self) -> &'static str {
        match self {
            TrigramState::Qian => "☰",
            TrigramState::Dui => "☱",
            TrigramState::Li => "☲",
            TrigramState::Zhen => "☳",
            TrigramState::Xun => "☴",
            TrigramState::Kan => "☵",
            TrigramState::Gen => "☶",
            TrigramState::Kun => "☷",
        }
    }

    /// 获取卦名
    pub fn name(&self) -> &'static str {
        match self {
            TrigramState::Qian => "乾",
            TrigramState::Dui => "兑",
            TrigramState::Li => "离",
            TrigramState::Zhen => "震",
            TrigramState::Xun => "巽",
            TrigramState::Kan => "坎",
            TrigramState::Gen => "艮",
            TrigramState::Kun => "坤",
        }
    }

    /// 从三爻值构建卦象
    ///
    /// # 参数
    /// - `upper`: 上爻（true = 阳，false = 阴）
    /// - `middle`: 中爻
    /// - `lower`: 下爻
    pub fn from_yao(upper: bool, middle: bool, lower: bool) -> Self {
        match (upper, middle, lower) {
            (true, true, true) => TrigramState::Qian,   // 111
            (false, true, true) => TrigramState::Dui,   // 011
            (true, false, true) => TrigramState::Li,    // 101
            (false, false, true) => TrigramState::Zhen, // 001
            (true, true, false) => TrigramState::Xun,   // 110
            (false, true, false) => TrigramState::Kan,  // 010
            (true, false, false) => TrigramState::Gen,  // 100
            (false, false, false) => TrigramState::Kun, // 000
        }
    }
}

/// 意图向量（三维决策空间 - 对应三爻）
///
/// 基于易经八卦的三爻系统设计，而非任意多维。
///
/// # 三爻对应
///
/// ```text
/// 上爻（天）：semantic   - 语义匹配度
/// 中爻（人）：safety     - 安全性（1 - risk）
/// 下爻（地）：context    - 上下文相关性
/// ```
///
/// # 示例
/// ```
/// use realconsole::dsl::intent::vector::IntentVector;
///
/// // 高语义、低风险、强上下文 → 乾卦 ☰
/// let perfect = IntentVector::new(0.9, 0.1, 0.9);
/// assert_eq!(perfect.trigram().symbol(), "☰");
///
/// // 低语义、高风险、弱上下文 → 坤卦 ☷
/// let worst = IntentVector::new(0.2, 0.9, 0.1);
/// assert_eq!(worst.trigram().symbol(), "☷");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentVector {
    /// 上爻（天）：语义匹配度 (0.0 - 1.0)
    ///
    /// 用户输入与意图定义的语义相似度
    pub semantic: f64,

    /// 中爻（人）：风险等级 (0.0 - 1.0)
    ///
    /// 执行该意图的风险程度（越高越危险）
    /// **注意**: 在计算卦象时会转换为 safety = 1 - risk
    pub risk: f64,

    /// 下爻（地）：上下文相关性 (0.0 - 1.0)
    ///
    /// 该意图与当前对话上下文的相关程度
    pub context: f64,
}

impl IntentVector {
    /// 创建新的意图向量（三爻）
    ///
    /// # 参数
    /// - `semantic`: 语义匹配度 (0.0 - 1.0)
    /// - `risk`: 风险等级 (0.0 - 1.0)
    /// - `context`: 上下文相关性 (0.0 - 1.0)
    pub fn new(semantic: f64, risk: f64, context: f64) -> Self {
        Self {
            semantic: semantic.clamp(0.0, 1.0),
            risk: risk.clamp(0.0, 1.0),
            context: context.clamp(0.0, 1.0),
        }
    }

    /// 从单一置信度创建（向后兼容）
    ///
    /// 用于从旧的单维度 confidence 平滑迁移到三维向量。
    ///
    /// # 参数
    /// - `confidence`: 原有的置信度
    ///
    /// # 返回值
    /// 一个三维向量，其中：
    /// - `semantic = confidence`
    /// - `risk = 0.3`（默认低风险）
    /// - `context = 0.5`（中等相关）
    pub fn from_confidence(confidence: f64) -> Self {
        Self {
            semantic: confidence.clamp(0.0, 1.0),
            risk: 0.3,    // 默认低风险
            context: 0.5, // 中等相关
        }
    }

    /// 计算当前向量对应的卦象
    ///
    /// 将连续值转换为阴阳（bool），使用阈值决定：
    /// - 阳（true）: 值 > 0.6
    /// - 阴（false）: 值 <= 0.6
    ///
    /// # 返回值
    /// - 对应的八卦之一
    ///
    /// # 示例
    /// ```
    /// use realconsole::dsl::intent::vector::IntentVector;
    ///
    /// // 高语义(0.9)、低风险(0.1)、强上下文(0.9) → 乾卦 ☰
    /// let perfect = IntentVector::new(0.9, 0.1, 0.9);
    /// assert_eq!(perfect.trigram().symbol(), "☰");
    /// ```
    pub fn trigram(&self) -> TrigramState {
        const THRESHOLD: f64 = 0.6;

        // 上爻（天）：语义匹配
        let upper = self.semantic > THRESHOLD;

        // 中爻（人）：安全性 = 1 - risk（风险越低越好）
        let middle = (1.0 - self.risk) > THRESHOLD;

        // 下爻（地）：上下文相关性
        let lower = self.context > THRESHOLD;

        TrigramState::from_yao(upper, middle, lower)
    }

    /// 基于卦象的决策建议
    ///
    /// 根据八卦状态返回推荐的决策动作。
    ///
    /// # 返回值
    /// - (action, score): 决策动作和对应的置信分数
    ///
    /// # 决策规则
    ///
    /// | 卦 | 场态 | 决策 | 分数 |
    /// |-------|------|------|------|
    /// | 乾 ☰ | 111 | Execute（立即执行） | 0.95 |
    /// | 兑 ☱ | 011 | Execute（可执行）| 0.80 |
    /// | 离 ☲ | 101 | Confirm（需确认）| 0.70 |
    /// | 震 ☳ | 001 | Warning（警告确认）| 0.60 |
    /// | 巽 ☴ | 110 | Try（可尝试）| 0.55 |
    /// | 坎 ☵ | 010 | Clarify（建议澄清）| 0.40 |
    /// | 艮 ☶ | 100 | Reject（拒绝）| 0.25 |
    /// | 坤 ☷ | 000 | Reject（拒绝）| 0.10 |
    pub fn decision_action(&self) -> (DecisionAction, f64) {
        match self.trigram() {
            TrigramState::Qian => (DecisionAction::Execute, 0.95),  // 乾：完美匹配
            TrigramState::Dui => (DecisionAction::Execute, 0.80),   // 兑：高质量
            TrigramState::Li => (DecisionAction::Confirm, 0.70),    // 离：需确认
            TrigramState::Zhen => (DecisionAction::Warning, 0.60),  // 震：警告
            TrigramState::Xun => (DecisionAction::Try, 0.55),       // 巽：可尝试
            TrigramState::Kan => (DecisionAction::Clarify, 0.40),   // 坎：澄清
            TrigramState::Gen => (DecisionAction::Reject, 0.25),    // 艮：拒绝
            TrigramState::Kun => (DecisionAction::Reject, 0.10),    // 坤：完全不匹配
        }
    }

    /// 计算综合决策分数（加权平均）
    ///
    /// 使用三维向量点积计算综合分数：
    /// ```text
    /// score = semantic × w_semantic
    ///       + (1 - risk) × w_safety    // 风险越低越好
    ///       + context × w_context
    /// ```
    ///
    /// # 参数
    /// - `weights`: 决策权重配置
    ///
    /// # 返回值
    /// - 综合决策分数 (0.0 - 1.0)
    pub fn decision_score(&self, weights: &DecisionWeights) -> f64 {
        let score = self.semantic * weights.semantic
            + (1.0 - self.risk) * weights.safety // 风险越低越好
            + self.context * weights.context;

        // 归一化到 [0, 1]
        let total_weight = weights.semantic + weights.safety + weights.context;
        (score / total_weight).clamp(0.0, 1.0)
    }

    /// 获取简单的向后兼容置信度
    ///
    /// 优先使用卦象决策分数，如果需要加权则使用默认权重。
    ///
    /// # 返回值
    /// - 置信度分数 (0.0 - 1.0)
    pub fn as_confidence(&self) -> f64 {
        // 优先使用卦象决策的分数
        let (_, score) = self.decision_action();
        score
    }

    /// 获取卦象的可读描述
    ///
    /// # 返回值
    /// - (symbol, name, description): 符号、卦名、决策描述
    pub fn describe(&self) -> (String, String, String) {
        let trigram = self.trigram();
        let (action, score) = self.decision_action();

        let symbol = trigram.symbol().to_string();
        let name = trigram.name().to_string();
        let description = format!(
            "{} ({}): {} - 分数 {:.2}",
            symbol,
            name,
            action.description(),
            score
        );

        (symbol, name, description)
    }
}

/// 决策权重配置（三爻权重）
///
/// 控制三维向量决策时各个维度的重要性。
///
/// # 设计理念
///
/// - **三爻对应**：与易经八卦的三爻系统对应
/// - **可配置**：不同场景可以使用不同的权重
/// - **可演化**：未来可以通过机器学习自动调整
///
/// # 预设配置
///
/// - `default()`: 平衡模式（语义 50%，安全 30%，上下文 20%）
/// - `safety_first()`: 安全优先（安全 50%）
/// - `semantic_focused()`: 语义优先（语义 60%）
/// - `context_aware()`: 上下文感知（上下文 40%）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionWeights {
    /// 语义匹配权重（上爻 - 天）
    pub semantic: f64,

    /// 安全性权重（中爻 - 人，对应 1 - risk）
    pub safety: f64,

    /// 上下文相关性权重（下爻 - 地）
    pub context: f64,
}

impl Default for DecisionWeights {
    /// 默认权重：平衡模式
    ///
    /// - 语义匹配: 50%（最重要）
    /// - 安全性: 30%（次之）
    /// - 上下文: 20%（辅助）
    fn default() -> Self {
        Self {
            semantic: 0.5,
            safety: 0.3,
            context: 0.2,
        }
    }
}

impl DecisionWeights {
    /// 安全优先模式
    ///
    /// 适用于生产环境或高风险操作。
    pub fn safety_first() -> Self {
        Self {
            semantic: 0.3,
            safety: 0.5,  // 安全性权重最高
            context: 0.2,
        }
    }

    /// 语义优先模式
    ///
    /// 适用于熟练用户或交互式场景。
    pub fn semantic_focused() -> Self {
        Self {
            semantic: 0.6, // 语义匹配权重最高
            safety: 0.25,
            context: 0.15,
        }
    }

    /// 上下文感知模式
    ///
    /// 适用于多轮对话或任务追踪场景。
    pub fn context_aware() -> Self {
        Self {
            semantic: 0.4,
            safety: 0.2,
            context: 0.4, // 上下文权重最高
        }
    }

    /// 自定义权重（会自动归一化）
    pub fn custom(semantic: f64, safety: f64, context: f64) -> Self {
        let total = semantic + safety + context;
        if total == 0.0 {
            return Self::default();
        }

        Self {
            semantic: semantic / total,
            safety: safety / total,
            context: context / total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_vector_creation() {
        let vector = IntentVector::new(0.9, 0.2, 0.7);

        assert_eq!(vector.semantic, 0.9);
        assert_eq!(vector.risk, 0.2);
        assert_eq!(vector.context, 0.7);
    }

    #[test]
    fn test_intent_vector_clamping() {
        // 测试自动裁剪到 [0, 1]
        let vector = IntentVector::new(1.5, -0.1, 1.2);

        assert_eq!(vector.semantic, 1.0);
        assert_eq!(vector.risk, 0.0);
        assert_eq!(vector.context, 1.0);
    }

    #[test]
    fn test_from_confidence() {
        let vector = IntentVector::from_confidence(0.8);

        assert_eq!(vector.semantic, 0.8);
        assert_eq!(vector.risk, 0.3);
        assert_eq!(vector.context, 0.5);
    }

    // ============ 八卦测试 ============

    #[test]
    fn test_trigram_qian() {
        // 乾 ☰ (111): 高语义、低风险、强上下文
        let vector = IntentVector::new(0.9, 0.1, 0.9);
        assert_eq!(vector.trigram(), TrigramState::Qian);

        let (action, score) = vector.decision_action();
        assert_eq!(action, DecisionAction::Execute);
        assert_eq!(score, 0.95);
    }

    #[test]
    fn test_trigram_dui() {
        // 兑 ☱ (011): 低语义、低风险、强上下文
        let vector = IntentVector::new(0.3, 0.1, 0.9);
        assert_eq!(vector.trigram(), TrigramState::Dui);

        let (action, score) = vector.decision_action();
        assert_eq!(action, DecisionAction::Execute);
        assert_eq!(score, 0.80);
    }

    #[test]
    fn test_trigram_li() {
        // 离 ☲ (101): 高语义、高风险、强上下文
        let vector = IntentVector::new(0.9, 0.9, 0.9);
        assert_eq!(vector.trigram(), TrigramState::Li);

        let (action, score) = vector.decision_action();
        assert_eq!(action, DecisionAction::Confirm);
        assert_eq!(score, 0.70);
    }

    #[test]
    fn test_trigram_zhen() {
        // 震 ☳ (001): 低语义、高风险、强上下文
        let vector = IntentVector::new(0.3, 0.9, 0.9);
        assert_eq!(vector.trigram(), TrigramState::Zhen);

        let (action, score) = vector.decision_action();
        assert_eq!(action, DecisionAction::Warning);
        assert_eq!(score, 0.60);
    }

    #[test]
    fn test_trigram_xun() {
        // 巽 ☴ (110): 高语义、低风险、弱上下文
        let vector = IntentVector::new(0.9, 0.1, 0.3);
        assert_eq!(vector.trigram(), TrigramState::Xun);

        let (action, score) = vector.decision_action();
        assert_eq!(action, DecisionAction::Try);
        assert_eq!(score, 0.55);
    }

    #[test]
    fn test_trigram_kan() {
        // 坎 ☵ (010): 低语义、低风险、弱上下文
        let vector = IntentVector::new(0.3, 0.1, 0.3);
        assert_eq!(vector.trigram(), TrigramState::Kan);

        let (action, score) = vector.decision_action();
        assert_eq!(action, DecisionAction::Clarify);
        assert_eq!(score, 0.40);
    }

    #[test]
    fn test_trigram_gen() {
        // 艮 ☶ (100): 高语义、高风险、弱上下文
        let vector = IntentVector::new(0.9, 0.9, 0.3);
        assert_eq!(vector.trigram(), TrigramState::Gen);

        let (action, score) = vector.decision_action();
        assert_eq!(action, DecisionAction::Reject);
        assert_eq!(score, 0.25);
    }

    #[test]
    fn test_trigram_kun() {
        // 坤 ☷ (000): 低语义、高风险、弱上下文
        let vector = IntentVector::new(0.3, 0.9, 0.3);
        assert_eq!(vector.trigram(), TrigramState::Kun);

        let (action, score) = vector.decision_action();
        assert_eq!(action, DecisionAction::Reject);
        assert_eq!(score, 0.10);
    }

    // ============ 决策分数测试 ============

    #[test]
    fn test_decision_score_default_weights() {
        // 高分向量：高语义、低风险、强上下文
        let high_vector = IntentVector::new(0.9, 0.1, 0.9);
        let weights = DecisionWeights::default();
        let high_score = high_vector.decision_score(&weights);

        assert!(high_score > 0.8);

        // 低分向量：低语义、高风险、弱上下文
        let low_vector = IntentVector::new(0.3, 0.8, 0.1);
        let low_score = low_vector.decision_score(&weights);

        assert!(low_score < 0.4);
    }

    #[test]
    fn test_decision_score_safety_first() {
        // 高语义匹配，但高风险
        let vector = IntentVector::new(0.9, 0.8, 0.9);

        let default_weights = DecisionWeights::default();
        let safety_weights = DecisionWeights::safety_first();

        let default_score = vector.decision_score(&default_weights);
        let safety_score = vector.decision_score(&safety_weights);

        // 安全优先模式下，高风险会导致更低的分数
        assert!(safety_score < default_score);
    }

    // ============ 权重测试 ============

    #[test]
    fn test_decision_weights_default() {
        let weights = DecisionWeights::default();

        // 验证权重之和接近 1.0
        let sum = weights.semantic + weights.safety + weights.context;
        assert!((sum - 1.0).abs() < 0.01);

        // 验证语义权重最高
        assert!(weights.semantic > weights.safety);
        assert!(weights.safety > weights.context);
    }

    #[test]
    fn test_decision_weights_safety_first() {
        let weights = DecisionWeights::safety_first();

        // 安全性权重最高
        assert!(weights.safety > weights.semantic);
        assert!(weights.safety > weights.context);
    }

    #[test]
    fn test_decision_weights_context_aware() {
        let weights = DecisionWeights::context_aware();

        // 上下文权重最高
        assert!(weights.context >= weights.semantic);
        assert!(weights.context > weights.safety);
    }

    #[test]
    fn test_decision_weights_custom() {
        let weights = DecisionWeights::custom(2.0, 1.0, 1.0);

        // 自动归一化
        let sum = weights.semantic + weights.safety + weights.context;
        assert!((sum - 1.0).abs() < 0.01);

        // 语义权重应该是 2/4 = 0.5
        assert!((weights.semantic - 0.5).abs() < 0.01);
    }

    // ============ 功能测试 ============

    #[test]
    fn test_as_confidence() {
        // 乾卦：应该返回最高分数
        let qian = IntentVector::new(0.9, 0.1, 0.9);
        assert_eq!(qian.as_confidence(), 0.95);

        // 坤卦：应该返回最低分数
        let kun = IntentVector::new(0.3, 0.9, 0.3);
        assert_eq!(kun.as_confidence(), 0.10);
    }

    #[test]
    fn test_describe() {
        let vector = IntentVector::new(0.9, 0.1, 0.9);
        let (symbol, name, description) = vector.describe();

        assert_eq!(symbol, "☰");
        assert_eq!(name, "乾");
        assert!(description.contains("立即执行"));
        assert!(description.contains("0.95"));
    }

    #[test]
    fn test_vector_comparison() {
        let weights = DecisionWeights::default();

        // 向量 A：高语义，低风险
        let vector_a = IntentVector::new(0.9, 0.2, 0.7);

        // 向量 B：中等语义，高风险
        let vector_b = IntentVector::new(0.7, 0.8, 0.7);

        let score_a = vector_a.decision_score(&weights);
        let score_b = vector_b.decision_score(&weights);

        // A 应该得分更高（低风险）
        assert!(score_a > score_b);
    }

    // ============ 边界测试 ============

    #[test]
    fn test_trigram_boundary() {
        // 测试阈值边界（0.6）
        let just_below = IntentVector::new(0.59, 0.39, 0.59);
        let just_above = IntentVector::new(0.61, 0.39, 0.61);

        // just_below: (0, 1, 0) 因为 safety = 1 - 0.39 = 0.61 > 0.6
        assert_eq!(just_below.trigram(), TrigramState::Kan); // 010

        // just_above: (1, 1, 1)
        assert_eq!(just_above.trigram(), TrigramState::Qian); // 111
    }

    #[test]
    fn test_trigram_state_symbols() {
        assert_eq!(TrigramState::Qian.symbol(), "☰");
        assert_eq!(TrigramState::Dui.symbol(), "☱");
        assert_eq!(TrigramState::Li.symbol(), "☲");
        assert_eq!(TrigramState::Zhen.symbol(), "☳");
        assert_eq!(TrigramState::Xun.symbol(), "☴");
        assert_eq!(TrigramState::Kan.symbol(), "☵");
        assert_eq!(TrigramState::Gen.symbol(), "☶");
        assert_eq!(TrigramState::Kun.symbol(), "☷");
    }

    #[test]
    fn test_trigram_state_names() {
        assert_eq!(TrigramState::Qian.name(), "乾");
        assert_eq!(TrigramState::Kun.name(), "坤");
        assert_eq!(TrigramState::Li.name(), "离");
    }

    #[test]
    fn test_decision_action_descriptions() {
        assert_eq!(DecisionAction::Execute.description(), "立即执行");
        assert_eq!(DecisionAction::Confirm.description(), "需要确认");
        assert_eq!(DecisionAction::Warning.description(), "警告确认");
        assert_eq!(DecisionAction::Try.description(), "可尝试");
        assert_eq!(DecisionAction::Clarify.description(), "建议澄清");
        assert_eq!(DecisionAction::Reject.description(), "拒绝执行");
    }
}
