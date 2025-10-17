//! Memory Search 性能基准测试
//!
//! 测试记忆系统的性能，包括：
//! - 关键词搜索
//! - 获取最近记忆
//! - 记忆追加
//! - 类型过滤

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use realconsole::memory::{EntryType, Memory};

/// 创建测试用 Memory（小规模：10 条）
fn create_small_memory() -> Memory {
    let mut mem = Memory::new(100);

    for i in 0..10 {
        mem.add(
            format!("这是测试记忆 {} 关于 Rust 编程", i),
            EntryType::User,
        );
    }

    mem
}

/// 创建测试用 Memory（中规模：100 条）
fn create_medium_memory() -> Memory {
    let mut mem = Memory::new(200);

    for i in 0..100 {
        let content = match i % 4 {
            0 => format!("用户消息 {}: 请帮我计算", i),
            1 => format!("助手回复 {}: 好的，我来帮你", i),
            2 => format!("系统消息 {}: 工具调用成功", i),
            3 => format!("Shell命令 {}: ls -la", i),
            _ => unreachable!(),
        };

        let entry_type = match i % 4 {
            0 => EntryType::User,
            1 => EntryType::Assistant,
            2 => EntryType::System,
            3 => EntryType::Shell,
            _ => unreachable!(),
        };

        mem.add(content, entry_type);
    }

    mem
}

/// 创建测试用 Memory（大规模：1000 条）
fn create_large_memory() -> Memory {
    let mut mem = Memory::new(1500);

    for i in 0..1000 {
        let content = format!("记忆条目 {}: 这是一段测试内容，包含各种关键词", i);
        mem.add(content, EntryType::User);
    }

    mem
}

/// 基准测试：小规模搜索（10 条）
fn bench_search_small(c: &mut Criterion) {
    let mem = create_small_memory();

    c.bench_function("memory_search_10", |b| {
        b.iter(|| {
            mem.search(black_box("Rust"));
        })
    });
}

/// 基准测试：中规模搜索（100 条）
fn bench_search_medium(c: &mut Criterion) {
    let mem = create_medium_memory();

    c.bench_function("memory_search_100", |b| {
        b.iter(|| {
            mem.search(black_box("计算"));
        })
    });
}

/// 基准测试：大规模搜索（1000 条）
fn bench_search_large(c: &mut Criterion) {
    let mem = create_large_memory();

    c.bench_function("memory_search_1000", |b| {
        b.iter(|| {
            mem.search(black_box("关键词"));
        })
    });
}

/// 基准测试：无匹配搜索
fn bench_search_no_match(c: &mut Criterion) {
    let mem = create_medium_memory();

    c.bench_function("memory_search_no_match", |b| {
        b.iter(|| {
            mem.search(black_box("不存在的关键词xyz123"));
        })
    });
}

/// 基准测试：获取最近记忆（10 条）
fn bench_recent_10(c: &mut Criterion) {
    let mem = create_medium_memory();

    c.bench_function("memory_recent_10", |b| {
        b.iter(|| {
            black_box(mem.recent(black_box(10)));
        })
    });
}

/// 基准测试：获取最近记忆（50 条）
fn bench_recent_50(c: &mut Criterion) {
    let mem = create_medium_memory();

    c.bench_function("memory_recent_50", |b| {
        b.iter(|| {
            black_box(mem.recent(black_box(50)));
        })
    });
}

/// 基准测试：记忆追加（单次）
fn bench_append_single(c: &mut Criterion) {
    c.bench_function("memory_append_single", |b| {
        b.iter_batched(
            || create_small_memory(),
            |mut mem| {
                mem.add(
                    black_box("新的记忆条目".to_string()),
                    black_box(EntryType::User),
                );
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// 基准测试：记忆追加（批量 10 条）
fn bench_append_batch_10(c: &mut Criterion) {
    c.bench_function("memory_append_batch_10", |b| {
        b.iter_batched(
            || create_small_memory(),
            |mut mem| {
                for i in 0..10 {
                    let content = format!("批量记忆 {}", i);
                    mem.add(
                        black_box(content),
                        black_box(EntryType::User),
                    );
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// 基准测试：按类型过滤
fn bench_filter_by_type(c: &mut Criterion) {
    let mem = create_medium_memory();

    c.bench_function("memory_filter_by_type", |b| {
        b.iter(|| {
            black_box(mem.filter_by_type(black_box(EntryType::User)));
        })
    });
}

/// 基准测试：导出所有记忆
fn bench_dump_all(c: &mut Criterion) {
    let mem = create_medium_memory();

    c.bench_function("memory_dump_all", |b| {
        b.iter(|| {
            black_box(mem.dump());
        })
    });
}

/// 基准测试：获取记忆数量
fn bench_len(c: &mut Criterion) {
    let mem = create_medium_memory();

    c.bench_function("memory_len", |b| {
        b.iter(|| {
            black_box(mem.len());
        })
    });
}

/// 基准测试：清空记忆
fn bench_clear(c: &mut Criterion) {
    c.bench_function("memory_clear", |b| {
        b.iter_batched(
            || create_small_memory(),
            |mut mem| {
                mem.clear();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// 基准测试：组合操作（追加 + 搜索 + 获取最近）
fn bench_combined_operations(c: &mut Criterion) {
    c.bench_function("memory_combined_ops", |b| {
        b.iter_batched(
            || create_small_memory(),
            |mut mem| {
                // 追加记忆
                mem.add("新记忆".to_string(), EntryType::User);
                // 搜索
                black_box(mem.search("测试"));
                // 获取最近
                black_box(mem.recent(5));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_search_small,
    bench_search_medium,
    bench_search_large,
    bench_search_no_match,
    bench_recent_10,
    bench_recent_50,
    bench_append_single,
    bench_append_batch_10,
    bench_filter_by_type,
    bench_dump_all,
    bench_len,
    bench_clear,
    bench_combined_operations
);

criterion_main!(benches);
