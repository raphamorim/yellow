use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use zaz::{Cell, Attr, Color};

fn bench_cell_creation(c: &mut Criterion) {
    c.bench_function("cell_new", |b| {
        b.iter(|| {
            black_box(Cell::new('A'))
        });
    });

    c.bench_function("cell_blank", |b| {
        b.iter(|| {
            black_box(Cell::blank())
        });
    });

    c.bench_function("cell_with_style", |b| {
        b.iter(|| {
            black_box(Cell::with_style(
                'X',
                Attr::BOLD | Attr::UNDERLINE,
                Some(Color::Red),
                Some(Color::Blue),
            ))
        });
    });
}

fn bench_cell_clone(c: &mut Criterion) {
    let cell = Cell::with_style(
        'X',
        Attr::BOLD | Attr::UNDERLINE,
        Some(Color::Red),
        Some(Color::Blue),
    );

    c.bench_function("cell_clone", |b| {
        b.iter(|| {
            black_box(cell.clone())
        });
    });
}

fn bench_cell_comparison(c: &mut Criterion) {
    let cell1 = Cell::with_style('A', Attr::BOLD, Some(Color::Red), None);
    let cell2 = Cell::with_style('A', Attr::BOLD, Some(Color::Red), None);
    let cell3 = Cell::with_style('B', Attr::UNDERLINE, Some(Color::Blue), None);

    c.bench_function("cell_eq_same", |b| {
        b.iter(|| {
            black_box(cell1 == cell2)
        });
    });

    c.bench_function("cell_eq_different", |b| {
        b.iter(|| {
            black_box(cell1 == cell3)
        });
    });

    c.bench_function("cell_same_style", |b| {
        b.iter(|| {
            black_box(cell1.same_style(&cell2))
        });
    });
}

fn bench_cell_line_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell_line_ops");

    for size in [10, 50, 80, 200, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("create_line", size), size, |b, &size| {
            b.iter(|| {
                let line: Vec<Cell> = (0..size).map(|_| Cell::blank()).collect();
                black_box(line)
            });
        });

        group.bench_with_input(BenchmarkId::new("clone_line", size), size, |b, &size| {
            let line: Vec<Cell> = (0..size).map(|_| Cell::blank()).collect();
            b.iter(|| {
                black_box(line.clone())
            });
        });

        group.bench_with_input(BenchmarkId::new("compare_identical_lines", size), size, |b, &size| {
            let line1: Vec<Cell> = (0..size).map(|i| Cell::new((b'A' + (i % 26) as u8) as char)).collect();
            let line2 = line1.clone();
            b.iter(|| {
                black_box(line1 == line2)
            });
        });

        group.bench_with_input(BenchmarkId::new("compare_different_lines", size), size, |b, &size| {
            let line1: Vec<Cell> = (0..size).map(|i| Cell::new((b'A' + (i % 26) as u8) as char)).collect();
            let mut line2 = line1.clone();
            line2[size / 2] = Cell::new('X');
            b.iter(|| {
                black_box(line1 == line2)
            });
        });
    }

    group.finish();
}

fn bench_cell_memory_size(c: &mut Criterion) {
    c.bench_function("cell_memory_footprint", |b| {
        b.iter(|| {
            // Measure memory impact by creating many cells
            let cells: Vec<Cell> = (0..10000)
                .map(|i| Cell::with_style(
                    (b'A' + (i % 26) as u8) as char,
                    if i % 2 == 0 { Attr::BOLD } else { Attr::NORMAL },
                    if i % 3 == 0 { Some(Color::Red) } else { None },
                    if i % 5 == 0 { Some(Color::Blue) } else { None },
                ))
                .collect();
            black_box(cells)
        });
    });

    // Report actual size
    let cell_size = std::mem::size_of::<Cell>();
    println!("Cell size: {} bytes", cell_size);
    println!("80x24 screen buffer: {} KB", (cell_size * 80 * 24) / 1024);
    println!("200x60 screen buffer: {} KB", (cell_size * 200 * 60) / 1024);
}

criterion_group!(
    benches,
    bench_cell_creation,
    bench_cell_clone,
    bench_cell_comparison,
    bench_cell_line_operations,
    bench_cell_memory_size,
);
criterion_main!(benches);
