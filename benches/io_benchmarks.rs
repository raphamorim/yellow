use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::io::Write;

// Benchmark comparing buffered stdout vs direct I/O

fn bench_stdout_buffered(c: &mut Criterion) {
    let mut group = c.benchmark_group("stdout_buffered");

    for size in [100, 1000, 10000].iter() {
        let data = "X".repeat(*size);

        group.bench_with_input(BenchmarkId::new("write_all", size), &data, |b, data| {
            b.iter(|| {
                std::io::stdout().write_all(data.as_bytes()).unwrap();
                black_box(());
            });
        });

        group.bench_with_input(
            BenchmarkId::new("write_all_flush", size),
            &data,
            |b, data| {
                b.iter(|| {
                    std::io::stdout().write_all(data.as_bytes()).unwrap();
                    std::io::stdout().flush().unwrap();
                    black_box(());
                });
            },
        );
    }

    group.finish();
}

fn bench_direct_io(c: &mut Criterion) {
    let mut group = c.benchmark_group("direct_io");

    for size in [100, 1000, 10000].iter() {
        let data = "X".repeat(*size);

        group.bench_with_input(BenchmarkId::new("write_stdout", size), &data, |b, data| {
            b.iter(|| {
                zaz::__bench_io::write_stdout(data.as_bytes()).unwrap();
                black_box(());
            });
        });

        group.bench_with_input(
            BenchmarkId::new("write_all_stdout", size),
            &data,
            |b, data| {
                b.iter(|| {
                    zaz::__bench_io::write_all_stdout(data.as_bytes()).unwrap();
                    black_box(());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_stdout_buffered, bench_direct_io);
criterion_main!(benches);
