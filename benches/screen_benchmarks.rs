use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::fmt::Write as FmtWrite;
use zaz::{Attr, Cell, Color};

// Simulate output buffer operations

fn bench_ansi_sequence_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ansi_sequences");

    // Benchmark: cursor positioning
    group.bench_function("cursor_absolute", |b| {
        let mut buffer = String::with_capacity(100);
        b.iter(|| {
            buffer.clear();
            write!(buffer, "\x1b[{};{}H", 10, 20).unwrap();
            black_box(&buffer);
        });
    });

    group.bench_function("cursor_relative_forward", |b| {
        let mut buffer = String::with_capacity(100);
        b.iter(|| {
            buffer.clear();
            write!(buffer, "\x1b[{}C", 5).unwrap();
            black_box(&buffer);
        });
    });

    // Benchmark: style codes
    group.bench_function("style_simple", |b| {
        let mut buffer = String::with_capacity(100);
        b.iter(|| {
            buffer.clear();
            write!(buffer, "\x1b[1m").unwrap();
            black_box(&buffer);
        });
    });

    group.bench_function("style_complex", |b| {
        let mut buffer = String::with_capacity(100);
        let codes = vec!["1", "4", "38;2;255;0;0", "48;2;0;0;255"];
        b.iter(|| {
            buffer.clear();
            write!(buffer, "\x1b[{}m", codes.join(";")).unwrap();
            black_box(&buffer);
        });
    });

    // Benchmark: style code generation with pre-allocated buffer
    group.bench_function("style_prealloc", |b| {
        let mut buffer = String::with_capacity(100);
        let mut sequence_buf = String::with_capacity(50);
        let codes = vec!["1", "4", "38;2;255;0;0"];

        b.iter(|| {
            buffer.clear();
            sequence_buf.clear();
            sequence_buf.push_str("\x1b[");
            for (i, code) in codes.iter().enumerate() {
                if i > 0 {
                    sequence_buf.push(';');
                }
                sequence_buf.push_str(code);
            }
            sequence_buf.push('m');
            buffer.push_str(&sequence_buf);
            black_box(&buffer);
        });
    });

    group.finish();
}

fn bench_buffer_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_ops");

    for size in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("string_with_capacity", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut buffer = String::with_capacity(size);
                    for i in 0..100 {
                        write!(buffer, "Text{}", i).unwrap();
                    }
                    black_box(buffer);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("string_default", size),
            size,
            |b, _size| {
                b.iter(|| {
                    let mut buffer = String::new();
                    for i in 0..100 {
                        write!(buffer, "Text{}", i).unwrap();
                    }
                    black_box(buffer);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("string_clear_reuse", size),
            size,
            |b, &size| {
                let mut buffer = String::with_capacity(size);
                b.iter(|| {
                    buffer.clear();
                    for i in 0..100 {
                        write!(buffer, "Text{}", i).unwrap();
                    }
                    black_box(&buffer);
                });
            },
        );
    }

    group.finish();
}

fn bench_style_caching(c: &mut Criterion) {
    c.bench_function("style_comparison_overhead", |b| {
        let attr1 = Attr::BOLD;
        let attr2 = Attr::BOLD;
        let fg1 = Color::Red;
        let fg2 = Color::Red;
        let bg1 = Color::Blue;
        let bg2 = Color::Blue;

        b.iter(|| {
            let changed = attr1 != attr2 || fg1 != fg2 || bg1 != bg2;
            black_box(changed);
        });
    });

    c.bench_function("style_batch_check", |b| {
        let cells: Vec<Cell> = (0..80)
            .map(|i| {
                if i < 40 {
                    Cell::with_style('A', Attr::BOLD, Color::Red, Color::Reset)
                } else {
                    Cell::with_style('B', Attr::UNDERLINE, Color::Blue, Color::Reset)
                }
            })
            .collect();

        b.iter(|| {
            // Simulate finding runs of same style
            let mut runs = Vec::new();
            let mut current_start = 0;
            let mut current_style = (cells[0].attr, cells[0].fg(), cells[0].bg());

            for i in 1..cells.len() {
                let style = (cells[i].attr, cells[i].fg(), cells[i].bg());
                if style != current_style {
                    runs.push((current_start, i - 1, current_style));
                    current_start = i;
                    current_style = style;
                }
            }
            runs.push((current_start, cells.len() - 1, current_style));
            black_box(runs);
        });
    });
}

fn bench_rle_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("rle");

    for blank_count in [5, 8, 10, 20, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("spaces_literal", blank_count),
            blank_count,
            |b, &count| {
                let mut buffer = String::with_capacity(200);
                b.iter(|| {
                    buffer.clear();
                    for _ in 0..count {
                        buffer.push(' ');
                    }
                    black_box(&buffer);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("spaces_ech", blank_count),
            blank_count,
            |b, &count| {
                let mut buffer = String::with_capacity(200);
                b.iter(|| {
                    buffer.clear();
                    write!(buffer, "\x1b[{}X", count).unwrap();
                    black_box(&buffer);
                });
            },
        );
    }

    group.finish();
}

fn bench_line_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_rendering");

    for width in [40, 80, 132, 200].iter() {
        let cells: Vec<Cell> = (0..*width)
            .map(|i| {
                Cell::with_style(
                    (b'A' + (i % 26) as u8) as char,
                    if i % 10 == 0 {
                        Attr::BOLD
                    } else {
                        Attr::NORMAL
                    },
                    if i % 7 == 0 { Color::Red } else { Color::Reset },
                    Color::Reset,
                )
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("full_line", width), &cells, |b, cells| {
            let mut buffer = String::with_capacity(2000);
            let mut last_style = (Attr::NORMAL, None, None);

            b.iter(|| {
                buffer.clear();
                last_style = (Attr::NORMAL, None, None);

                for cell in cells {
                    let current_style = (cell.attr, cell.fg(), cell.bg());
                    if current_style != last_style {
                        // Simplified style application
                        write!(buffer, "\x1b[0m").unwrap();
                        last_style = current_style;
                    }
                    buffer.push(cell.ch);
                }
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

fn bench_full_screen_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_screen");

    for &(rows, cols) in &[(24, 80), (60, 200)] {
        let screen: Vec<Vec<Cell>> = (0..rows)
            .map(|row| {
                (0..cols)
                    .map(|col| {
                        Cell::with_style(
                            ((row + col) % 94 + 33) as u8 as char,
                            if (row + col) % 5 == 0 {
                                Attr::BOLD
                            } else {
                                Attr::NORMAL
                            },
                            None,
                            None,
                        )
                    })
                    .collect()
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("render", format!("{}x{}", rows, cols)),
            &screen,
            |b, screen: &Vec<Vec<Cell>>| {
                let mut buffer = String::with_capacity(rows * cols * 10);

                b.iter(|| {
                    buffer.clear();
                    for (y, row) in screen.iter().enumerate() {
                        write!(buffer, "\x1b[{};1H", y + 1).unwrap();
                        for cell in row {
                            buffer.push(cell.ch);
                        }
                    }
                    black_box(&buffer);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_ansi_sequence_generation,
    bench_buffer_operations,
    bench_style_caching,
    bench_rle_operations,
    bench_line_rendering,
    bench_full_screen_simulation,
);
criterion_main!(benches);
