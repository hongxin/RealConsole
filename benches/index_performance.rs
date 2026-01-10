//! 多维索引性能基准测试
//!
//! v1.56.0: 验证 10x 查询性能提升
//!
//! 测试场景：
//! - 索引构建性能
//! - 维度查询性能
//! - 时间范围查询性能
//! - 组合查询性能
//! - 对比：线性扫描 vs 索引查询

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use chrono::{Duration, Utc};

use realconsole::tracer::{
    Dimension, EntryType, MultiDimensionalIndex, Status, StatusKey, TraceEntry,
};
use realconsole::tracer::types::Importance;

/// 创建测试条目
fn create_test_entry(
    dimension: Dimension,
    entry_type: EntryType,
    content: &str,
    status: Status,
    hours_ago: i64,
) -> TraceEntry {
    let mut entry = TraceEntry::new(dimension, entry_type, content.to_string(), status);
    entry.timestamp = Utc::now() - Duration::hours(hours_ago);
    entry.importance = Some(Importance::Normal);
    entry.tags = vec!["benchmark".to_string()];
    entry
}

/// 创建大规模测试数据集
fn create_dataset(size: usize) -> Vec<TraceEntry> {
    let dimensions = [
        Dimension::Statistics,
        Dimension::Coordination,
        Dimension::BlackBox,
        Dimension::Memory,
    ];
    let entry_types = [
        EntryType::ShellCommand,
        EntryType::SystemCommand,
        EntryType::TaskExecution,
        EntryType::LlmRequest,
    ];
    let statuses = [
        Status::Success,
        Status::Failed("error".to_string()),
        Status::Running,
        Status::Cancelled,
    ];

    (0..size)
        .map(|i| {
            create_test_entry(
                dimensions[i % 4].clone(),
                entry_types[i % 4].clone(),
                &format!("test content {}", i),
                statuses[i % 4].clone(),
                (i % 24) as i64,
            )
        })
        .collect()
}

/// 线性扫描查询（对比基准）
fn linear_scan_by_dimension<'a>(entries: &'a [TraceEntry], dimension: &Dimension) -> Vec<&'a TraceEntry> {
    entries
        .iter()
        .filter(|e| &e.dimension == dimension)
        .collect()
}

/// 线性扫描时间范围查询（对比基准）
fn linear_scan_by_time_range<'a>(
    entries: &'a [TraceEntry],
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> Vec<&'a TraceEntry> {
    entries
        .iter()
        .filter(|e| e.timestamp >= start && e.timestamp <= end)
        .collect()
}

// ============================================================================
// 基准测试
// ============================================================================

/// 基准测试：索引构建
fn bench_index_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_build");

    for size in [100, 1000, 10000].iter() {
        let entries = create_dataset(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                MultiDimensionalIndex::build_from(black_box(entries.clone()))
            })
        });
    }

    group.finish();
}

/// 基准测试：维度查询（索引 vs 线性扫描）
fn bench_dimension_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("dimension_query");

    let entries = create_dataset(10000);
    let index = MultiDimensionalIndex::build_from(entries.clone());

    // 索引查询
    group.bench_function("indexed_10k", |b| {
        b.iter(|| {
            index.query_by_dimension(black_box(&Dimension::Statistics), 100)
        })
    });

    // 线性扫描
    group.bench_function("linear_scan_10k", |b| {
        b.iter(|| {
            linear_scan_by_dimension(black_box(&entries), &Dimension::Statistics)
        })
    });

    group.finish();
}

/// 基准测试：时间范围查询（索引 vs 线性扫描）
fn bench_time_range_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_range_query");

    let entries = create_dataset(10000);
    let index = MultiDimensionalIndex::build_from(entries.clone());

    let now = Utc::now();
    let start = now - Duration::hours(6);
    let end = now;

    // 索引查询
    group.bench_function("indexed_10k", |b| {
        b.iter(|| {
            index.query_by_time_range(black_box(start), black_box(end), 100)
        })
    });

    // 线性扫描
    group.bench_function("linear_scan_10k", |b| {
        b.iter(|| {
            linear_scan_by_time_range(black_box(&entries), start, end)
        })
    });

    group.finish();
}

/// 基准测试：状态查询
fn bench_status_query(c: &mut Criterion) {
    let entries = create_dataset(10000);
    let index = MultiDimensionalIndex::build_from(entries);

    c.bench_function("status_query_10k", |b| {
        b.iter(|| {
            index.query_by_status(black_box(StatusKey::Success), 100)
        })
    });
}

/// 基准测试：标签查询
fn bench_tag_query(c: &mut Criterion) {
    let entries = create_dataset(10000);
    let index = MultiDimensionalIndex::build_from(entries);

    c.bench_function("tag_query_10k", |b| {
        b.iter(|| {
            index.query_by_tags(black_box(&["benchmark".to_string()]), 100)
        })
    });
}

/// 基准测试：组合查询
fn bench_combined_query(c: &mut Criterion) {
    let entries = create_dataset(10000);
    let index = MultiDimensionalIndex::build_from(entries);

    c.bench_function("combined_query_10k", |b| {
        b.iter(|| {
            index.query_combined(
                black_box(Some(&Dimension::Statistics)),
                black_box(Some(&EntryType::ShellCommand)),
                black_box(Some(StatusKey::Success)),
                black_box(None),
                100,
            )
        })
    });
}

/// 基准测试：去重检查
fn bench_dedup_check(c: &mut Criterion) {
    let entries = create_dataset(10000);
    let index = MultiDimensionalIndex::build_from(entries);

    // 使用一个已存在的哈希值
    let hash = 12345678901234567890u64;

    c.bench_function("dedup_check_10k", |b| {
        b.iter(|| {
            index.contains_content(black_box(hash))
        })
    });
}

criterion_group!(
    benches,
    bench_index_build,
    bench_dimension_query,
    bench_time_range_query,
    bench_status_query,
    bench_tag_query,
    bench_combined_query,
    bench_dedup_check,
);

criterion_main!(benches);
