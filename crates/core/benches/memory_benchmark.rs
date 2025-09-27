use criterion::{black_box, criterion_group, criterion_main, Criterion};
use core::{read_mem_stats, MemStats};
use core::release::boost;
use core::processes::{get_all_processes, sort_and_take_processes};
use std::time::Duration;

fn benchmark_memory_stats_read(c: &mut Criterion) {
    c.bench_function("read_mem_stats", |b| {
        b.iter(|| {
            let stats = read_mem_stats();
            black_box(stats)
        })
    });
}

fn benchmark_process_listing(c: &mut Criterion) {
    c.bench_function("get_all_processes", |b| {
        b.iter(|| {
            let processes = get_all_processes();
            black_box(processes)
        })
    });

    c.bench_function("sort_and_take_processes_10", |b| {
        let processes = get_all_processes();
        b.iter(|| {
            let sorted = sort_and_take_processes(processes.clone(), 10);
            black_box(sorted)
        })
    });

    c.bench_function("sort_and_take_processes_50", |b| {
        let processes = get_all_processes();
        b.iter(|| {
            let sorted = sort_and_take_processes(processes.clone(), 50);
            black_box(sorted)
        })
    });
}

fn benchmark_boost_operation(c: &mut Criterion) {
    // 这个基准测试比较特殊，因为boost操作可能会失败（如果没有purge命令）
    // 我们首先测试是否可以执行boost
    let can_boost = match boost() {
        Ok(_) => true,
        Err(core::release::BoostError::Purge(core::release::PurgeError::CommandNotFound)) => false,
        Err(_) => false,
    };

    if can_boost {
        // 由于boost操作有副作用且耗时较长，我们使用较少的迭代次数
        c.bench_function("boost_cold", |b| {
            b.iter_batched(
                || {
                    // 等待一段时间让系统稳定
                    std::thread::sleep(Duration::from_secs(2));
                },
                |_| {
                    let result = boost();
                    black_box(result)
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn benchmark_memory_pressure_detection(c: &mut Criterion) {
    c.bench_function("memory_pressure_simulation", |b| {
        b.iter(|| {
            // 模拟不同内存压力情况的检测
            let test_stats = vec![
                MemStats {
                    total_mb: 16384,
                    free_mb: 8000,
                    active_mb: 4000,
                    inactive_mb: 2000,
                    wired_mb: 2000,
                    compressed_mb: 384,
                    pressure: core::PressureLevel::Normal,
                },
                MemStats {
                    total_mb: 16384,
                    free_mb: 2000,
                    active_mb: 8000,
                    inactive_mb: 2000,
                    wired_mb: 3000,
                    compressed_mb: 1384,
                    pressure: core::PressureLevel::Warning,
                },
                MemStats {
                    total_mb: 16384,
                    free_mb: 500,
                    active_mb: 10000,
                    inactive_mb: 1000,
                    wired_mb: 3500,
                    compressed_mb: 1384,
                    pressure: core::PressureLevel::Critical,
                },
            ];

            for stats in &test_stats {
                black_box(stats);
            }
        })
    });
}

fn benchmark_resident_memory_usage(c: &mut Criterion) {
    // 测试程序本身的常驻内存使用情况
    c.bench_function("resident_memory_footprint", |b| {
        b.iter(|| {
            // 执行一系列典型操作来测量内存占用
            let _stats = read_mem_stats();
            let _processes = get_all_processes();
            let _top_processes = sort_and_take_processes(_processes, 10);

            // 强制垃圾回收（如果有的话）
            black_box(())
        })
    });
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    use std::thread;

    c.bench_function("concurrent_memory_reads", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    thread::spawn(|| {
                        let stats = read_mem_stats();
                        black_box(stats)
                    })
                })
                .collect();

            for handle in handles {
                let _ = handle.join();
            }
        })
    });
}

criterion_group!(
    memory_benches,
    benchmark_memory_stats_read,
    benchmark_process_listing,
    benchmark_memory_pressure_detection,
    benchmark_resident_memory_usage,
    benchmark_concurrent_operations
);

criterion_group!(
    name = boost_benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(30));
    targets = benchmark_boost_operation
);

criterion_main!(memory_benches, boost_benches);