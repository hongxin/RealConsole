//! 存储后端性能基准测试
//!
//! v1.67.0: 完整存储层性能验证
//!
//! ## 测试目标
//!
//! 验证 v2.0 探路期目标：3-5x I/O 性能提升
//!
//! ## 测试场景
//!
//! - 基础性能：FileStorage vs MemoryStorage
//! - 缓存效果：CachedStorage 读取加速
//! - 压缩效果：CompressedStorage 空间/速度权衡
//! - 组合效果：StorageBuilder 预设配置
//! - 批量操作：批量读写性能
//!
//! ## 运行方式
//!
//! ```bash
//! cargo bench --bench storage_performance
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use realconsole::storage::{
    CachedStorage, CachedStorageConfig, CompressedStorage, FileStorage, MemoryStorage,
    StorageBackend, StorageBuilder, TieredCacheConfig,
};
use tempfile::tempdir;
use tokio::runtime::Runtime;

// ============================================================================
// 辅助函数
// ============================================================================

/// 创建测试数据（可压缩）
fn create_compressible_data(size: usize) -> Vec<u8> {
    // 重复模式，压缩效果好
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// 创建测试数据（随机，不可压缩）
#[allow(dead_code)]
fn create_random_data(size: usize) -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    (0..size)
        .map(|i| {
            let mut hasher = DefaultHasher::new();
            i.hash(&mut hasher);
            (hasher.finish() % 256) as u8
        })
        .collect()
}

// ============================================================================
// 基础性能基准
// ============================================================================

/// 基准测试：单次写入比较
fn bench_single_write(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("single_write");
    group.throughput(Throughput::Bytes(1024));

    let data = create_compressible_data(1024); // 1KB

    // MemoryStorage
    group.bench_function("memory", |b| {
        let storage = MemoryStorage::new();
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                storage
                    .write(black_box(&format!("key_{}", i)), black_box(&data))
                    .await
                    .unwrap();
            });
            i += 1;
        })
    });

    // FileStorage
    let temp_dir = tempdir().unwrap();
    let file_storage = FileStorage::new(temp_dir.path());
    group.bench_function("file", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                file_storage
                    .write(black_box(&format!("key_{}", i)), black_box(&data))
                    .await
                    .unwrap();
            });
            i += 1;
        })
    });

    // CachedStorage (Memory + Cache)
    let cached = CachedStorage::new(MemoryStorage::new());
    group.bench_function("cached", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                cached
                    .write(black_box(&format!("key_{}", i)), black_box(&data))
                    .await
                    .unwrap();
            });
            i += 1;
        })
    });

    // CompressedStorage
    let compressed = CompressedStorage::new(MemoryStorage::new());
    group.bench_function("compressed", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                compressed
                    .write(black_box(&format!("key_{}", i)), black_box(&data))
                    .await
                    .unwrap();
            });
            i += 1;
        })
    });

    group.finish();
}

/// 基准测试：单次读取比较
fn bench_single_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("single_read");
    group.throughput(Throughput::Bytes(1024));

    let data = create_compressible_data(1024);

    // MemoryStorage
    let mem_storage = MemoryStorage::new();
    rt.block_on(async { mem_storage.write("key", &data).await.unwrap() });
    group.bench_function("memory", |b| {
        b.iter(|| rt.block_on(async { mem_storage.read(black_box("key")).await.unwrap() }))
    });

    // FileStorage
    let temp_dir = tempdir().unwrap();
    let file_storage = FileStorage::new(temp_dir.path());
    rt.block_on(async { file_storage.write("key", &data).await.unwrap() });
    group.bench_function("file", |b| {
        b.iter(|| rt.block_on(async { file_storage.read(black_box("key")).await.unwrap() }))
    });

    // CachedStorage（缓存命中）
    let cached = CachedStorage::new(MemoryStorage::new());
    rt.block_on(async {
        cached.write("key", &data).await.unwrap();
        // 预热缓存
        cached.read("key").await.unwrap();
    });
    group.bench_function("cached_hit", |b| {
        b.iter(|| rt.block_on(async { cached.read(black_box("key")).await.unwrap() }))
    });

    // CompressedStorage
    let compressed = CompressedStorage::new(MemoryStorage::new());
    rt.block_on(async { compressed.write("key", &data).await.unwrap() });
    group.bench_function("compressed", |b| {
        b.iter(|| rt.block_on(async { compressed.read(black_box("key")).await.unwrap() }))
    });

    group.finish();
}

// ============================================================================
// 缓存效果基准
// ============================================================================

/// 基准测试：缓存命中 vs 未命中
fn bench_cache_effectiveness(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("cache_effectiveness");

    let data = create_compressible_data(4096); // 4KB

    // 配置小缓存以测试淘汰
    let small_cache_config = CachedStorageConfig {
        cache_config: TieredCacheConfig {
            hot_capacity: 10,
            warm_capacity: 20,
            cold_capacity: 30,
            ..Default::default()
        },
        ..Default::default()
    };

    // 缓存命中场景（重复读取同一键）
    let cached_hit = CachedStorage::with_config(MemoryStorage::new(), small_cache_config.clone());
    rt.block_on(async {
        cached_hit.write("hot_key", &data).await.unwrap();
        cached_hit.read("hot_key").await.unwrap(); // 预热
    });
    group.bench_function("cache_hit", |b| {
        b.iter(|| rt.block_on(async { cached_hit.read(black_box("hot_key")).await.unwrap() }))
    });

    // 缓存未命中场景（每次读不同键）
    let cached_miss = CachedStorage::with_config(MemoryStorage::new(), small_cache_config);
    rt.block_on(async {
        for i in 0..1000 {
            cached_miss
                .write(&format!("key_{}", i), &data)
                .await
                .unwrap();
        }
    });
    group.bench_function("cache_miss", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                // 轮询读取，超出缓存容量
                cached_miss
                    .read(black_box(&format!("key_{}", i % 1000)))
                    .await
                    .unwrap()
            });
            i += 1;
        })
    });

    // 无缓存基线
    let no_cache = MemoryStorage::new();
    rt.block_on(async {
        for i in 0..1000 {
            no_cache.write(&format!("key_{}", i), &data).await.unwrap();
        }
    });
    group.bench_function("no_cache_baseline", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                no_cache
                    .read(black_box(&format!("key_{}", i % 1000)))
                    .await
                    .unwrap()
            });
            i += 1;
        })
    });

    group.finish();
}

// ============================================================================
// 压缩效果基准
// ============================================================================

/// 基准测试：压缩级别比较
fn bench_compression_levels(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("compression_levels");

    // 使用可压缩数据
    let data = create_compressible_data(10240); // 10KB
    group.throughput(Throughput::Bytes(10240));

    // 无压缩基线
    let no_compress = MemoryStorage::new();
    group.bench_function("none", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                no_compress
                    .write(black_box(&format!("key_{}", i)), black_box(&data))
                    .await
                    .unwrap();
            });
            i += 1;
        })
    });

    // Fast 压缩
    let fast = CompressedStorage::with_fast(MemoryStorage::new());
    group.bench_function("fast", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                fast.write(black_box(&format!("key_{}", i)), black_box(&data))
                    .await
                    .unwrap();
            });
            i += 1;
        })
    });

    // Default 压缩
    let default = CompressedStorage::new(MemoryStorage::new());
    group.bench_function("default", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                default
                    .write(black_box(&format!("key_{}", i)), black_box(&data))
                    .await
                    .unwrap();
            });
            i += 1;
        })
    });

    // Best 压缩
    let best = CompressedStorage::with_best(MemoryStorage::new());
    group.bench_function("best", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                best.write(black_box(&format!("key_{}", i)), black_box(&data))
                    .await
                    .unwrap();
            });
            i += 1;
        })
    });

    group.finish();
}

/// 基准测试：压缩读取性能
fn bench_compression_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("compression_read");

    let data = create_compressible_data(10240); // 10KB
    group.throughput(Throughput::Bytes(10240));

    // 无压缩
    let no_compress = MemoryStorage::new();
    rt.block_on(async { no_compress.write("key", &data).await.unwrap() });
    group.bench_function("none", |b| {
        b.iter(|| rt.block_on(async { no_compress.read(black_box("key")).await.unwrap() }))
    });

    // Fast 解压
    let fast = CompressedStorage::with_fast(MemoryStorage::new());
    rt.block_on(async { fast.write("key", &data).await.unwrap() });
    group.bench_function("fast", |b| {
        b.iter(|| rt.block_on(async { fast.read(black_box("key")).await.unwrap() }))
    });

    // Default 解压
    let default = CompressedStorage::new(MemoryStorage::new());
    rt.block_on(async { default.write("key", &data).await.unwrap() });
    group.bench_function("default", |b| {
        b.iter(|| rt.block_on(async { default.read(black_box("key")).await.unwrap() }))
    });

    // Best 解压
    let best = CompressedStorage::with_best(MemoryStorage::new());
    rt.block_on(async { best.write("key", &data).await.unwrap() });
    group.bench_function("best", |b| {
        b.iter(|| rt.block_on(async { best.read(black_box("key")).await.unwrap() }))
    });

    group.finish();
}

// ============================================================================
// StorageBuilder 预设基准
// ============================================================================

/// 基准测试：StorageBuilder 预设性能
fn bench_builder_presets(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("builder_presets");

    let data = create_compressible_data(4096); // 4KB
    group.throughput(Throughput::Bytes(4096));

    // Development 预设（最简单）
    let dev = StorageBuilder::development();
    group.bench_function("development_write", |b| {
        let mut i = 0;
        b.iter(|| {
            rt.block_on(async {
                dev.write(black_box(&format!("key_{}", i)), black_box(&data))
                    .await
                    .unwrap();
            });
            i += 1;
        })
    });

    // Development 读取
    rt.block_on(async { dev.write("read_key", &data).await.unwrap() });
    group.bench_function("development_read", |b| {
        b.iter(|| rt.block_on(async { dev.read(black_box("read_key")).await.unwrap() }))
    });

    group.finish();
}

// ============================================================================
// 批量操作基准
// ============================================================================

/// 基准测试：批量写入
fn bench_batch_write(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("batch_write");

    let data = create_compressible_data(256);

    for batch_size in [10, 100, 500].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));

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

        // CachedStorage
        group.bench_with_input(
            BenchmarkId::new("cached", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let storage = CachedStorage::new(MemoryStorage::new());
                    rt.block_on(async {
                        for i in 0..size {
                            storage.write(&format!("key_{}", i), &data).await.unwrap();
                        }
                    })
                })
            },
        );

        // CompressedStorage
        group.bench_with_input(
            BenchmarkId::new("compressed", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let storage = CompressedStorage::new(MemoryStorage::new());
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

    let data = create_compressible_data(256);

    for batch_size in [10, 100, 500].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));

        // MemoryStorage
        let mem_storage = MemoryStorage::new();
        rt.block_on(async {
            for i in 0..*batch_size {
                mem_storage
                    .write(&format!("key_{}", i), &data)
                    .await
                    .unwrap();
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

        // CachedStorage（预热缓存）
        let cached_storage = CachedStorage::new(MemoryStorage::new());
        rt.block_on(async {
            for i in 0..*batch_size {
                cached_storage
                    .write(&format!("key_{}", i), &data)
                    .await
                    .unwrap();
            }
            // 预热
            for i in 0..*batch_size {
                cached_storage.read(&format!("key_{}", i)).await.unwrap();
            }
        });
        group.bench_with_input(
            BenchmarkId::new("cached", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        for i in 0..size {
                            cached_storage.read(&format!("key_{}", i)).await.unwrap();
                        }
                    })
                })
            },
        );

        // CompressedStorage
        let compressed_storage = CompressedStorage::new(MemoryStorage::new());
        rt.block_on(async {
            for i in 0..*batch_size {
                compressed_storage
                    .write(&format!("key_{}", i), &data)
                    .await
                    .unwrap();
            }
        });
        group.bench_with_input(
            BenchmarkId::new("compressed", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        for i in 0..size {
                            compressed_storage.read(&format!("key_{}", i)).await.unwrap();
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// 数据大小基准
// ============================================================================

/// 基准测试：不同数据大小
fn bench_data_sizes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("data_sizes");

    // 测试不同数据大小：64B, 1KB, 10KB, 100KB
    for size in [64, 1024, 10240, 102400].iter() {
        let data = create_compressible_data(*size);
        group.throughput(Throughput::Bytes(*size as u64));

        // MemoryStorage 写入
        group.bench_with_input(BenchmarkId::new("memory_write", size), size, |b, _| {
            let storage = MemoryStorage::new();
            let mut i = 0;
            b.iter(|| {
                rt.block_on(async {
                    storage
                        .write(black_box(&format!("key_{}", i)), black_box(&data))
                        .await
                        .unwrap();
                });
                i += 1;
            })
        });

        // CompressedStorage 写入
        group.bench_with_input(BenchmarkId::new("compressed_write", size), size, |b, _| {
            let storage = CompressedStorage::new(MemoryStorage::new());
            let mut i = 0;
            b.iter(|| {
                rt.block_on(async {
                    storage
                        .write(black_box(&format!("key_{}", i)), black_box(&data))
                        .await
                        .unwrap();
                });
                i += 1;
            })
        });
    }

    group.finish();
}

// ============================================================================
// 混合工作负载基准
// ============================================================================

/// 基准测试：混合读写工作负载
fn bench_mixed_workload(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("mixed_workload");

    let data = create_compressible_data(1024);

    // 80% 读 20% 写（典型读多写少场景）
    // MemoryStorage
    let mem_storage = MemoryStorage::new();
    rt.block_on(async {
        for i in 0..100 {
            mem_storage
                .write(&format!("key_{}", i), &data)
                .await
                .unwrap();
        }
    });
    group.bench_function("memory_80r_20w", |b| {
        let mut op = 0;
        b.iter(|| {
            rt.block_on(async {
                if op % 5 == 0 {
                    // 20% 写入
                    mem_storage
                        .write(&format!("new_key_{}", op), &data)
                        .await
                        .unwrap();
                } else {
                    // 80% 读取
                    mem_storage
                        .read(&format!("key_{}", op % 100))
                        .await
                        .unwrap();
                }
            });
            op += 1;
        })
    });

    // CachedStorage
    let cached_storage = CachedStorage::new(MemoryStorage::new());
    rt.block_on(async {
        for i in 0..100 {
            cached_storage
                .write(&format!("key_{}", i), &data)
                .await
                .unwrap();
        }
        // 预热缓存
        for i in 0..100 {
            cached_storage.read(&format!("key_{}", i)).await.unwrap();
        }
    });
    group.bench_function("cached_80r_20w", |b| {
        let mut op = 0;
        b.iter(|| {
            rt.block_on(async {
                if op % 5 == 0 {
                    cached_storage
                        .write(&format!("new_key_{}", op), &data)
                        .await
                        .unwrap();
                } else {
                    cached_storage
                        .read(&format!("key_{}", op % 100))
                        .await
                        .unwrap();
                }
            });
            op += 1;
        })
    });

    group.finish();
}

// ============================================================================
// Criterion 配置
// ============================================================================

criterion_group!(
    benches,
    bench_single_write,
    bench_single_read,
    bench_cache_effectiveness,
    bench_compression_levels,
    bench_compression_read,
    bench_builder_presets,
    bench_batch_write,
    bench_batch_read,
    bench_data_sizes,
    bench_mixed_workload,
);

criterion_main!(benches);
