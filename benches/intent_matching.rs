//! Intent Matching 性能基准测试
//!
//! 测试 Intent DSL 匹配器的性能，包括：
//! - 精确匹配
//! - 模糊匹配
//! - 缓存命中

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use realconsole::dsl::intent::builtin::BuiltinIntents;
use realconsole::dsl::intent::matcher::{FuzzyConfig, IntentMatcher};

/// 创建测试用 IntentMatcher（启用模糊匹配）
fn create_matcher_with_fuzzy() -> IntentMatcher {
    let fuzzy_config = FuzzyConfig::enabled(0.8, 0.7);
    let mut matcher = IntentMatcher::with_config(100, fuzzy_config);

    // 注册内置意图
    let builtin = BuiltinIntents::new();
    for intent in builtin.all_intents() {
        matcher.register(intent);
    }

    matcher
}

/// 创建测试用 IntentMatcher（不启用模糊匹配）
fn create_matcher() -> IntentMatcher {
    let mut matcher = IntentMatcher::new();

    // 注册内置意图
    let builtin = BuiltinIntents::new();
    for intent in builtin.all_intents() {
        matcher.register(intent);
    }

    matcher
}

/// 基准测试：精确匹配（最常见场景）
fn bench_exact_match(c: &mut Criterion) {
    let matcher = create_matcher();

    c.bench_function("intent_exact_match", |b| {
        b.iter(|| {
            // 测试精确匹配关键词
            matcher.match_intent(black_box("计算 1+1"))
        })
    });
}

/// 基准测试：模糊匹配
fn bench_fuzzy_match(c: &mut Criterion) {
    let matcher = create_matcher();

    c.bench_function("intent_fuzzy_match", |b| {
        b.iter(|| {
            // 测试模糊匹配（拼写错误）
            matcher.match_intent(black_box("帮我算一下 1+1"))
        })
    });
}

/// 基准测试：缓存命中（重复查询）
fn bench_cache_hit(c: &mut Criterion) {
    let matcher = create_matcher();

    // 预热缓存
    matcher.match_intent("计算 1+1");

    c.bench_function("intent_cache_hit", |b| {
        b.iter(|| {
            // 测试缓存命中（相同查询）
            matcher.match_intent(black_box("计算 1+1"))
        })
    });
}

/// 基准测试：无匹配场景
fn bench_no_match(c: &mut Criterion) {
    let matcher = create_matcher();

    c.bench_function("intent_no_match", |b| {
        b.iter(|| {
            // 测试完全不匹配的查询
            matcher.match_intent(black_box("这是一个完全无关的查询xyz123"))
        })
    });
}

/// 基准测试：长查询
fn bench_long_query(c: &mut Criterion) {
    let matcher = create_matcher();

    c.bench_function("intent_long_query", |b| {
        b.iter(|| {
            // 测试长查询字符串
            matcher.match_intent(black_box(
                "请帮我计算一下这个复杂的数学表达式 (123 + 456) * 789 - 100 / 2",
            ))
        })
    });
}

/// 基准测试：批量匹配
fn bench_batch_matching(c: &mut Criterion) {
    let matcher = create_matcher();

    let queries = vec![
        "计算 1+1",
        "读取文件 test.txt",
        "当前时间",
        "系统信息",
        "搜索 rust",
    ];

    c.bench_function("intent_batch_matching", |b| {
        b.iter(|| {
            for query in &queries {
                matcher.match_intent(black_box(query));
            }
        })
    });
}

/// 基准测试：缓存统计
fn bench_cache_stats(c: &mut Criterion) {
    let matcher = create_matcher();

    // 进行一些查询
    matcher.match_intent("计算 1+1");
    matcher.match_intent("计算 1+1"); // 缓存命中
    matcher.match_intent("读取文件 test.txt");

    c.bench_function("intent_cache_stats", |b| {
        b.iter(|| {
            black_box(matcher.cache_hits());
            black_box(matcher.cache_misses());
        })
    });
}

criterion_group!(
    benches,
    bench_exact_match,
    bench_fuzzy_match,
    bench_cache_hit,
    bench_no_match,
    bench_long_query,
    bench_batch_matching,
    bench_cache_stats
);

criterion_main!(benches);
