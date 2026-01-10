//! 存储后端性能基准测试
//!
//! v1.58.0: 验证存储抽象层性能
//!
//! 测试场景：
//! - 单次读写性能
//! - 批量写入性能
//! - 批量读取性能
//! - 并发读写性能
//! - 对比：FileStorage vs MemoryStorage

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use realconsole::storage::{FileStorage, MemoryStorage, StorageBackend};
use tempfile::tempdir;
use tokio::runtime::Runtime;

/// 创建测试数据
fn create_test_data(size: usize) -> Vec<u8> {
    vec![b'x'; size]
}

/// 基准测试：单次写入
fn bench_single_write(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("single_write");

    let data = create_test_data(1024); // 1KB

    // MemoryStorage
    group.bench_function("memory_1kb", |b| {
        let storage = MemoryStorage::new();
        b.iter(|| {
            rt.block_on(async {
                storage.write(black_box("key"), black_box(&data)).await.unwrap();
            })
        })
    });

    // FileStorage
    let temp_dir = tempdir().unwrap();
    let file_storage = FileStorage::new(temp_dir.path());

    group.bench_function("file_1kb", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                file_storage.write(black_box(&format!("key_{}", i)), black_box(&data)).await.unwrap();
            });
            i += 1;
        })
    });

    group.finish();
}

/// 基准测试：单次读取
fn bench_single_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("single_read");

    let data = create_test_data(1024); // 1KB

    // MemoryStorage - 先写入数据
    let mem_storage = MemoryStorage::new();
    rt.block_on(async {
        mem_storage.write("key", &data).await.unwrap();
    });

    group.bench_function("memory_1kb", |b| {
        b.iter(|| {
            rt.block_on(async {
                mem_storage.read(black_box("key")).await.unwrap()
            })
        })
    });

    // FileStorage - 先写入数据
    let temp_dir = tempdir().unwrap();
    let file_storage = FileStorage::new(temp_dir.path());
    rt.block_on(async {
        file_storage.write("key", &data).await.unwrap();
    });

    group.bench_function("file_1kb", |b| {
        b.iter(|| {
            rt.block_on(async {
                file_storage.read(black_box("key")).await.unwrap()
            })
        })
    });

    group.finish();
}

/// 基准测试：批量写入
fn bench_batch_write(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("batch_write");

    let data = create_test_data(256); // 256 bytes

    for batch_size in [10, 100, 1000].iter() {
        // MemoryStorage
        group.bench_with_input(
            BenchmarkId::new("memory", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let storage = MemoryStorage::new();
                    rt.block_on(async {
                        for i in 0..size {
                            storage.write(&format!("key_{}", i), &data).await.unwrap();
                        }
                    })
                })
            },
        );

        // FileStorage
        group.bench_with_input(
            BenchmarkId::new("file", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempdir().unwrap();
                    let storage = FileStorage::new(temp_dir.path());
                    rt.block_on(async {
                        for i in 0..size {
                            storage.write(&format!("key_{}", i), &data).await.unwrap();
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

/// 基准测试：批量读取
fn bench_batch_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("batch_read");

    let data = create_test_data(256); // 256 bytes

    for batch_size in [10, 100, 1000].iter() {
        // MemoryStorage - 预填充
        let mem_storage = MemoryStorage::new();
        rt.block_on(async {
            for i in 0..*batch_size {
                mem_storage.write(&format!("key_{}", i), &data).await.unwrap();
            }
        });

        group.bench_with_input(
            BenchmarkId::new("memory", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        for i in 0..size {
                            mem_storage.read(&format!("key_{}", i)).await.unwrap();
                        }
                    })
                })
            },
        );

        // FileStorage - 预填充
        let temp_dir = tempdir().unwrap();
        let file_storage = FileStorage::new(temp_dir.path());
        rt.block_on(async {
            for i in 0..*batch_size {
                file_storage.write(&format!("key_{}", i), &data).await.unwrap();
            }
        });

        group.bench_with_input(
            BenchmarkId::new("file", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        for i in 0..size {
                            file_storage.read(&format!("key_{}", i)).await.unwrap();
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

/// 基准测试：存在性检查
fn bench_exists_check(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("exists_check");

    let data = create_test_data(256);

    // MemoryStorage
    let mem_storage = MemoryStorage::new();
    rt.block_on(async {
        mem_storage.write("key", &data).await.unwrap();
    });

    group.bench_function("memory", |b| {
        b.iter(|| {
            rt.block_on(async {
                mem_storage.exists(black_box("key")).await.unwrap()
            })
        })
    });

    // FileStorage
    let temp_dir = tempdir().unwrap();
    let file_storage = FileStorage::new(temp_dir.path());
    rt.block_on(async {
        file_storage.write("key", &data).await.unwrap();
    });

    group.bench_function("file", |b| {
        b.iter(|| {
            rt.block_on(async {
                file_storage.exists(black_box("key")).await.unwrap()
            })
        })
    });

    group.finish();
}

/// 基准测试：列表操作
fn bench_list_keys(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("list_keys");

    let data = create_test_data(64);

    // MemoryStorage - 预填充 1000 个键
    let mem_storage = MemoryStorage::new();
    rt.block_on(async {
        for i in 0..1000 {
            mem_storage.write(&format!("ns/key_{}", i), &data).await.unwrap();
        }
    });

    group.bench_function("memory_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                mem_storage.list(black_box("ns/")).await.unwrap()
            })
        })
    });

    // FileStorage - 预填充 1000 个键
    let temp_dir = tempdir().unwrap();
    let file_storage = FileStorage::new(temp_dir.path());
    rt.block_on(async {
        for i in 0..100 {
            file_storage.write(&format!("key_{}", i), &data).await.unwrap();
        }
    });

    group.bench_function("file_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                file_storage.list(black_box("")).await.unwrap()
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_write,
    bench_single_read,
    bench_batch_write,
    bench_batch_read,
    bench_exists_check,
    bench_list_keys,
);

criterion_main!(benches);
