//! Notebook Performance Benchmarks
//!
//! Benchmarks for notebook operations including:
//! - Dependency graph operations
//! - OT transformation
//! - Cell operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use realconsole::notebook::{
    Cell, CellType, Notebook,
    DependencyGraph, DependencyType, DependencyAnalyzer, ExecutionScheduler,
    CellOperation, OperationTransform, TextOperation,
    Collaborator, CollaborationSession,
};
use uuid::Uuid;

// ============================================================================
// Dependency Graph Benchmarks
// ============================================================================

fn bench_dependency_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_graph");

    // Benchmark adding cells to graph
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("add_cells", size), size, |b, &size| {
            b.iter(|| {
                let mut graph = DependencyGraph::new();
                for i in 0..size {
                    let cell = Cell::code(&format!("echo {}", i));
                    graph.add_cell(cell.id);
                }
                black_box(graph)
            });
        });
    }

    // Benchmark topological sort
    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("topological_sort", size), size, |b, &size| {
            let mut graph = DependencyGraph::new();
            let mut cells = Vec::new();

            // Create linear chain
            for i in 0..size {
                let cell = Cell::code(&format!("echo {}", i));
                cells.push(cell.id);
                graph.add_cell(cell.id);
            }

            // Add dependencies (linear chain)
            for i in 1..size {
                let _ = graph.add_dependency(cells[i], cells[i - 1], DependencyType::Sequential);
            }

            b.iter(|| {
                black_box(graph.topological_sort())
            });
        });
    }

    // Benchmark cycle detection
    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("has_cycle", size), size, |b, &size| {
            let mut graph = DependencyGraph::new();
            let mut cells = Vec::new();

            for i in 0..size {
                let cell = Cell::code(&format!("echo {}", i));
                cells.push(cell.id);
                graph.add_cell(cell.id);
            }

            // Add dependencies (linear chain - no cycle)
            for i in 1..size {
                let _ = graph.add_dependency(cells[i], cells[i - 1], DependencyType::Sequential);
            }

            b.iter(|| {
                black_box(graph.has_cycle())
            });
        });
    }

    group.finish();
}

// ============================================================================
// Execution Scheduler Benchmarks
// ============================================================================

fn bench_execution_scheduler(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution_scheduler");

    // Benchmark scheduling with diamond dependency
    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("schedule_diamond", size), size, |b, &size| {
            let mut graph = DependencyGraph::new();
            let mut cells = Vec::new();

            // Create cells
            for i in 0..size {
                let cell = Cell::code(&format!("echo {}", i));
                cells.push(cell.id);
                graph.add_cell(cell.id);
            }

            // Create diamond: 0 -> 1,2,3... -> last
            if size > 2 {
                for i in 1..(size - 1) {
                    let _ = graph.add_dependency(cells[i], cells[0], DependencyType::Sequential);
                    let _ = graph.add_dependency(cells[size - 1], cells[i], DependencyType::Sequential);
                }
            }

            let scheduler = ExecutionScheduler::new(graph.clone());

            b.iter(|| {
                black_box(scheduler.schedule())
            });
        });
    }

    group.finish();
}

// ============================================================================
// Dependency Analyzer Benchmarks
// ============================================================================

fn bench_dependency_analyzer(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_analyzer");

    // Benchmark variable analysis
    let test_sources = vec![
        "export FOO=bar",
        "echo $FOO",
        "BAR=${FOO}_suffix",
        "export PATH=$PATH:/usr/local/bin",
        "if [ -n \"$VAR\" ]; then echo yes; fi",
    ];

    group.bench_function("analyze_cell", |b| {
        let cells: Vec<Cell> = test_sources.iter()
            .map(|s| Cell::code(*s))
            .collect();

        b.iter(|| {
            let mut analyzer = DependencyAnalyzer::new();
            for cell in &cells {
                analyzer.analyze_cell(cell.id, &cell.source);
            }
            black_box(analyzer)
        });
    });

    // Benchmark building graph from cells
    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("build_graph", size), size, |b, &size| {
            let cells: Vec<Cell> = (0..size)
                .map(|i| {
                    if i % 3 == 0 {
                        Cell::code(format!("export VAR{}=value{}", i, i))
                    } else {
                        Cell::code(format!("echo $VAR{}", (i / 3) * 3))
                    }
                })
                .collect();

            // Prepare analyzer with all cells analyzed
            let mut analyzer = DependencyAnalyzer::new();
            let cell_ids: Vec<Uuid> = cells.iter().map(|c| {
                analyzer.analyze_cell(c.id, &c.source);
                c.id
            }).collect();

            b.iter(|| {
                black_box(analyzer.build_graph(&cell_ids))
            });
        });
    }

    group.finish();
}

// ============================================================================
// OT Transform Benchmarks
// ============================================================================

fn bench_ot_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("ot_transform");

    // Benchmark insert vs insert transform
    group.bench_function("insert_insert", |b| {
        let cell_a = Cell::natural("Test A");
        let cell_b = Cell::natural("Test B");
        let op_a = CellOperation::insert(0, cell_a);
        let op_b = CellOperation::insert(0, cell_b);

        b.iter(|| {
            black_box(OperationTransform::transform(&op_a, &op_b))
        });
    });

    // Benchmark insert vs delete transform
    group.bench_function("insert_delete", |b| {
        let cell = Cell::natural("Test");
        let delete_id = Uuid::new_v4();
        let op_a = CellOperation::insert(5, cell);
        let op_b = CellOperation::delete(delete_id, 3);

        b.iter(|| {
            black_box(OperationTransform::transform(&op_a, &op_b))
        });
    });

    // Benchmark transform_list
    for size in [5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::new("transform_list", size), size, |b, &size| {
            let ops: Vec<CellOperation> = (0..size)
                .map(|i| CellOperation::insert(i, Cell::natural(&format!("Cell {}", i))))
                .collect();
            let against = CellOperation::insert(0, Cell::natural("Against"));

            b.iter(|| {
                black_box(OperationTransform::transform_list(&ops, &against))
            });
        });
    }

    group.finish();
}

// ============================================================================
// Text Operation Benchmarks
// ============================================================================

fn bench_text_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_operations");

    let base_text = "Hello, World! This is a test string for benchmarking.";

    group.bench_function("text_insert", |b| {
        let op = TextOperation::Insert {
            position: 7,
            text: "Beautiful ".to_string(),
        };

        b.iter(|| {
            black_box(op.apply(base_text))
        });
    });

    group.bench_function("text_delete", |b| {
        let op = TextOperation::Delete {
            position: 7,
            length: 6,
        };

        b.iter(|| {
            black_box(op.apply(base_text))
        });
    });

    group.finish();
}

// ============================================================================
// Collaboration Session Benchmarks
// ============================================================================

fn bench_collaboration_session(c: &mut Criterion) {
    let mut group = c.benchmark_group("collaboration_session");

    // Benchmark adding collaborators
    group.bench_function("add_collaborators", |b| {
        let owner = Collaborator::new("Owner");
        let notebook_id = Uuid::new_v4();

        b.iter(|| {
            let mut session = CollaborationSession::new(notebook_id, owner.clone());
            for i in 0..10 {
                session.add_collaborator(Collaborator::new(&format!("User{}", i)));
            }
            black_box(session)
        });
    });

    // Benchmark applying operations
    group.bench_function("apply_operations", |b| {
        let owner = Collaborator::new("Owner");
        let owner_id = owner.id;
        let notebook_id = Uuid::new_v4();
        let mut session = CollaborationSession::new(notebook_id, owner);

        b.iter(|| {
            let cell = Cell::natural("Test");
            let op = CellOperation::insert(0, cell);
            black_box(session.apply_local(op, owner_id))
        });
    });

    group.finish();
}

// ============================================================================
// Notebook Operations Benchmarks
// ============================================================================

fn bench_notebook_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("notebook_operations");

    // Benchmark adding cells
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("add_cells", size), size, |b, &size| {
            b.iter(|| {
                let mut notebook = Notebook::new("Test Notebook");
                for i in 0..size {
                    notebook.add_cell(Cell::code(&format!("echo {}", i)));
                }
                black_box(notebook)
            });
        });
    }

    // Benchmark cell lookup by ID
    group.bench_function("get_cell_by_id", |b| {
        let mut notebook = Notebook::new("Test");
        let mut cell_ids = Vec::new();
        for i in 0..100 {
            let cell = Cell::code(&format!("echo {}", i));
            cell_ids.push(cell.id);
            notebook.add_cell(cell);
        }
        let target_id = cell_ids[50];

        b.iter(|| {
            black_box(notebook.get_cell(target_id))
        });
    });

    // Benchmark cells by type
    group.bench_function("cells_by_type", |b| {
        let mut notebook = Notebook::new("Test");
        for i in 0..100 {
            match i % 4 {
                0 => notebook.add_cell(Cell::natural(&format!("Natural {}", i))),
                1 => notebook.add_cell(Cell::code(&format!("!echo {}", i))),
                2 => notebook.add_cell(Cell::command(&format!("/help {}", i))),
                _ => notebook.add_cell(Cell::markdown(&format!("# Heading {}", i))),
            }
        }

        b.iter(|| {
            black_box(notebook.cells_by_type(CellType::Code))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_dependency_graph,
    bench_execution_scheduler,
    bench_dependency_analyzer,
    bench_ot_transform,
    bench_text_operations,
    bench_collaboration_session,
    bench_notebook_operations,
);

criterion_main!(benches);
