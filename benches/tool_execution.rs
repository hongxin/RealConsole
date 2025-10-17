//! Tool System 性能基准测试
//!
//! 测试工具系统的性能，包括：
//! - 工具注册表查询
//! - 工具列表获取
//! - OpenAI Schema 生成

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use realconsole::tool::ToolRegistry;

/// 创建测试用 ToolRegistry
fn create_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // 注册内置工具
    realconsole::builtin_tools::register_builtin_tools(&mut registry);

    registry
}

/// 基准测试：获取工具列表
fn bench_list_tools(c: &mut Criterion) {
    let registry = create_tool_registry();

    c.bench_function("tool_list_all", |b| {
        b.iter(|| {
            black_box(registry.list_tools());
        })
    });
}

/// 基准测试：查询单个工具
fn bench_get_tool(c: &mut Criterion) {
    let registry = create_tool_registry();

    c.bench_function("tool_get_single", |b| {
        b.iter(|| {
            black_box(registry.get(black_box("calculator")));
        })
    });
}

/// 基准测试：工具存在性检查
fn bench_tool_exists(c: &mut Criterion) {
    let registry = create_tool_registry();

    c.bench_function("tool_exists_check", |b| {
        b.iter(|| {
            black_box(registry.get(black_box("calculator")).is_some());
        })
    });
}

/// 基准测试：工具执行（Calculator）
fn bench_execute_calculator(c: &mut Criterion) {
    use serde_json::json;

    let registry = create_tool_registry();

    c.bench_function("tool_execute_calculator", |b| {
        b.iter(|| {
            black_box(registry.execute(
                black_box("calculator"),
                black_box(json!({"expression": "2+2"})),
            ))
        })
    });
}

/// 基准测试：工具执行（Datetime）
fn bench_execute_datetime(c: &mut Criterion) {
    use serde_json::json;

    let registry = create_tool_registry();

    c.bench_function("tool_execute_datetime", |b| {
        b.iter(|| {
            black_box(registry.execute(
                black_box("datetime"),
                black_box(json!({"action": "now"})),
            ))
        })
    });
}

/// 基准测试：批量工具查询
fn bench_batch_get_tools(c: &mut Criterion) {
    let registry = create_tool_registry();
    let tool_names = vec!["calculator", "datetime", "read_file", "write_file", "http_get"];

    c.bench_function("tool_batch_get", |b| {
        b.iter(|| {
            for name in &tool_names {
                black_box(registry.get(black_box(name)));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_list_tools,
    bench_get_tool,
    bench_tool_exists,
    bench_execute_calculator,
    bench_execute_datetime,
    bench_batch_get_tools
);

criterion_main!(benches);
