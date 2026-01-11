//! Tab 补全系统
//!
//! 基于"一分为三"哲学的多维补全体系，提供：
//! - **Static Completion**: 静态补全（命令/路径/历史）
//! - **Semantic Completion**: 语义补全（Intent DSL/模糊匹配）
//! - **Intelligent Completion**: 智能补全（LLM 预测）
//!
//! # 设计哲学
//!
//! 补全不是简单的"匹配 vs 不匹配"二元对立，而是多维状态向量空间的演化：
//!
//! ```text
//! Static (确定性) → Semantic (灵活性) → Intelligent (预测性)
//!   <10ms             10-50ms              50-300ms
//! ```
//!
//! # 使用示例
//!
//! ```rust,no_run
//! use realconsole::completion::{MultiDimensionalCompleter, CompletionConfig};
//!
//! // 创建补全器
//! let completer = MultiDimensionalCompleter::new(
//!     command_registry,
//!     history,
//!     CompletionConfig::default(),
//! );
//!
//! // 补全输入
//! let candidates = completer.complete("/he", 3, &ctx)?;
//! ```

mod cache;
mod intelligent_completer;
mod semantic_completer;
mod static_completer;
mod types;

pub use cache::CompletionCache;
pub use intelligent_completer::IntelligentCompleter;
pub use semantic_completer::SemanticCompleter;
pub use static_completer::StaticCompleter;
pub use types::{
    Candidate, CompletionConfig, CompletionContext, CompletionSource, CompletionType,
    GitContext, // v1.85.0: Git 上下文感知补全
};

use crate::command::CommandRegistry;
use crate::dsl::intent::builtin::BuiltinIntents;
use crate::history::HistoryManager;
use crate::llm::LlmClient;
use rustyline::completion::{Completer, Pair};
use rustyline::Context;
use std::sync::{Arc, RwLock as StdRwLock};
use tokio::sync::RwLock as TokioRwLock;

/// 多维补全器 - 融合三态补全源的统一入口
///
/// # 架构
///
/// ```text
/// MultiDimensionalCompleter
///   ├─ StaticCompleter (Phase 1) ✅
///   ├─ SemanticCompleter (Phase 2) ✅
///   └─ IntelligentCompleter (Phase 3) ✅
/// ```
pub struct MultiDimensionalCompleter {
    /// Phase 1: 静态补全器
    static_completer: StaticCompleter,

    /// Phase 2: 语义补全器
    semantic_completer: Option<SemanticCompleter>,

    /// Phase 3: 智能补全器
    intelligent_completer: Option<Arc<IntelligentCompleter>>,

    /// 补全配置
    config: CompletionConfig,

    /// LRU 缓存（优化性能） - 使用 std::sync::RwLock 因为 Completer::complete 是同步的
    cache: Arc<StdRwLock<CompletionCache>>,
}

impl MultiDimensionalCompleter {
    /// 创建新的多维补全器
    ///
    /// # 参数
    ///
    /// * `command_registry` - 命令注册表
    /// * `history` - 历史管理器
    /// * `config` - 补全配置
    /// * `llm_client` - 可选的 LLM 客户端（用于智能补全）
    pub fn new(
        command_registry: Arc<CommandRegistry>,
        history: Arc<TokioRwLock<HistoryManager>>,
        config: CompletionConfig,
        llm_client: Option<Arc<dyn LlmClient>>,
    ) -> Self {
        // Phase 1: 静态补全器
        let static_completer = StaticCompleter::new(command_registry.clone(), history.clone());

        // Phase 2: 语义补全器（如果启用）
        let semantic_completer = if config.enable_semantic {
            // 创建 Intent 匹配器（使用内置意图库）
            let builtin = BuiltinIntents::new();
            let intent_matcher = Arc::new(builtin.create_matcher());

            Some(SemanticCompleter::new(
                command_registry.clone(),
                history.clone(),
                intent_matcher,
            ))
        } else {
            None
        };

        // Phase 3: 智能补全器（如果启用且提供了 LLM 客户端）
        let intelligent_completer = if config.enable_intelligent && llm_client.is_some() {
            llm_client.map(|client| {
                Arc::new(
                    IntelligentCompleter::new(client, history.clone())
                        .with_timeout(2000) // 2 秒超时
                        .with_max_candidates(3), // 最多 3 个 LLM 建议
                )
            })
        } else {
            None
        };

        Self {
            static_completer,
            semantic_completer,
            intelligent_completer,
            config,
            cache: Arc::new(StdRwLock::new(CompletionCache::new())),
        }
    }

    /// 将候选转换为 rustyline Pair
    fn format_candidates(&self, candidates: Vec<Candidate>) -> Vec<Pair> {
        candidates
            .into_iter()
            .take(self.config.max_candidates)
            .map(|c| Pair {
                display: if c.description.is_empty() {
                    c.text.clone()
                } else {
                    format!("{:<40} {}", c.text, c.description)
                },
                replacement: c.text,
            })
            .collect()
    }
}

impl Completer for MultiDimensionalCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), rustyline::error::ReadlineError> {
        let input = &line[..pos];

        // 1. 检查缓存
        if let Ok(mut cache) = self.cache.write() {
            if let Some(cached) = cache.get(input) {
                return Ok((0, cached.clone()));
            }
        }

        // 2. Phase 1: 静态补全
        let mut all_candidates = Vec::new();

        if self.config.enable_static {
            all_candidates.extend(self.static_completer.complete(input));
        }

        // 2.5. Phase 2: 语义补全
        if self.config.enable_semantic {
            if let Some(ref semantic_completer) = self.semantic_completer {
                all_candidates.extend(semantic_completer.complete(input));
            }
        }

        // 2.8. Phase 3: 智能补全（LLM 预测）
        if self.config.enable_intelligent {
            if let Some(ref intelligent_completer) = self.intelligent_completer {
                // 使用 block_in_place 在同步上下文中执行异步代码
                // 注意：这可能会阻塞当前线程，但由于设置了超时，影响有限
                let llm_candidates = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(intelligent_completer.complete(input))
                });
                all_candidates.extend(llm_candidates);
            }
        }

        // 3. 格式化输出
        let pairs = self.format_candidates(all_candidates);

        // 4. 缓存结果
        if let Ok(mut cache) = self.cache.write() {
            cache.put(input.to_string(), pairs.clone());
        }

        Ok((0, pairs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::RwLock;

    #[test]
    fn test_completer_creation() {
        let registry = Arc::new(CommandRegistry::new());
        let history = Arc::new(RwLock::new(HistoryManager::new("test_history.json", 100)));
        let completer =
            MultiDimensionalCompleter::new(registry, history, CompletionConfig::default(), None);

        assert!(completer.config.enable_static);
    }
}
