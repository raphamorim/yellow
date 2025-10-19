use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use zaz::__bench::{DirtyRegion, find_line_diff};
use zaz::{Attr, Cell, Color};

fn bench_find_line_diff(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_line_diff");

    for size in [10, 50, 80, 200, 1000].iter() {
        // Benchmark: no difference
        group.bench_with_input(BenchmarkId::new("identical", size), size, |b, &size| {
            let line1: Vec<Cell> = (0..size)
                .map(|i| Cell::new((b'A' + (i % 26) as u8) as char))
                .collect();
            let line2 = line1.clone();
            b.iter(|| black_box(find_line_diff(&line1, &line2)));
        });

        // Benchmark: single character different in middle
        group.bench_with_input(BenchmarkId::new("middle_diff", size), size, |b, &size| {
            let line1: Vec<Cell> = (0..size)
                .map(|i| Cell::new((b'A' + (i % 26) as u8) as char))
                .collect();
            let mut line2 = line1.clone();
            if size > 0 {
                line2[size / 2] = Cell::new('X');
            }
            b.iter(|| black_box(find_line_diff(&line1, &line2)));
        });

        // Benchmark: first character different
        group.bench_with_input(BenchmarkId::new("first_diff", size), size, |b, &size| {
            let line1: Vec<Cell> = (0..size)
                .map(|i| Cell::new((b'A' + (i % 26) as u8) as char))
                .collect();
            let mut line2 = line1.clone();
            if size > 0 {
                line2[0] = Cell::new('X');
            }
            b.iter(|| black_box(find_line_diff(&line1, &line2)));
        });

        // Benchmark: last character different
        group.bench_with_input(BenchmarkId::new("last_diff", size), size, |b, &size| {
            let line1: Vec<Cell> = (0..size)
                .map(|i| Cell::new((b'A' + (i % 26) as u8) as char))
                .collect();
            let mut line2 = line1.clone();
            if size > 0 {
                line2[size - 1] = Cell::new('X');
            }
            b.iter(|| black_box(find_line_diff(&line1, &line2)));
        });

        // Benchmark: half the line different
        group.bench_with_input(BenchmarkId::new("half_diff", size), size, |b, &size| {
            let line1: Vec<Cell> = (0..size)
                .map(|i| Cell::new((b'A' + (i % 26) as u8) as char))
                .collect();
            let mut line2 = line1.clone();
            for i in (size / 2)..size {
                line2[i] = Cell::new('X');
            }
            b.iter(|| black_box(find_line_diff(&line1, &line2)));
        });

        // Benchmark: style change only (same character)
        group.bench_with_input(BenchmarkId::new("style_diff", size), size, |b, &size| {
            let line1: Vec<Cell> = (0..size)
                .map(|i| Cell::new((b'A' + (i % 26) as u8) as char))
                .collect();
            let line2: Vec<Cell> = (0..size)
                .map(|i| Cell::with_style((b'A' + (i % 26) as u8) as char, Attr::BOLD, None, None))
                .collect();
            b.iter(|| black_box(find_line_diff(&line1, &line2)));
        });
    }

    group.finish();
}

fn bench_dirty_region_operations(c: &mut Criterion) {
    c.bench_function("dirty_region_create", |b| {
        b.iter(|| black_box(DirtyRegion::clean()));
    });

    c.bench_function("dirty_region_mark", |b| {
        b.iter(|| {
            let mut region = DirtyRegion::clean();
            region.mark(10, 20);
            black_box(region)
        });
    });

    c.bench_function("dirty_region_expand", |b| {
        b.iter(|| {
            let mut region = DirtyRegion::clean();
            region.mark(10, 20);
            region.mark(5, 30);
            black_box(region)
        });
    });
}

fn bench_buffer_swap(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_swap");

    for screen_size in [(24, 80), (60, 200), (100, 300)].iter() {
        let (rows, cols) = screen_size;
        group.bench_with_input(
            BenchmarkId::new("swap", format!("{}x{}", rows, cols)),
            screen_size,
            |b, &(rows, cols)| {
                let mut current: Vec<Vec<Cell>> = vec![vec![Cell::blank(); cols]; rows];
                let mut pending: Vec<Vec<Cell>> = vec![vec![Cell::blank(); cols]; rows];

                b.iter(|| {
                    std::mem::swap(&mut current, &mut pending);
                    black_box(&current);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("copy", format!("{}x{}", rows, cols)),
            screen_size,
            |b, &(rows, cols)| {
                let current: Vec<Vec<Cell>> = vec![vec![Cell::blank(); cols]; rows];
                let mut pending: Vec<Vec<Cell>> = vec![vec![Cell::blank(); cols]; rows];

                b.iter(|| {
                    for y in 0..rows {
                        pending[y].clone_from_slice(&current[y]);
                    }
                    black_box(&pending);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_find_line_diff,
    bench_dirty_region_operations,
    bench_buffer_swap,
);
criterion_main!(benches);
