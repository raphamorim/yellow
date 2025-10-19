use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use zaz::__bench::hash_line;
use zaz::{Attr, Cell, Color};

// Current hash implementation (multiplication-based)
fn hash_line_multiply(cells: &[Cell]) -> u64 {
    let mut hash = 0u64;
    for cell in cells {
        hash = hash.wrapping_mul(31).wrapping_add(cell.ch as u64);
        // Note: We can't access cell.attr.bits() without making it public
        // This is a simplified version for benchmarking
        hash = hash.wrapping_mul(31);
    }
    hash
}

// FNV-1a hash (proposed optimization)
fn hash_line_fnv1a(cells: &[Cell]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;

    for cell in cells {
        // Hash the character
        let ch_value = cell.ch as u32;
        hash ^= ch_value as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    hash
}

// Standard library hasher (baseline)
fn hash_line_std(cells: &[Cell]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for cell in cells {
        cell.ch.hash(&mut hasher);
    }
    hasher.finish()
}

// xxHash-inspired simple version
fn hash_line_xxhash_simple(cells: &[Cell]) -> u64 {
    const PRIME1: u64 = 11400714785074694791u64;
    const PRIME2: u64 = 14029467366897019727u64;

    let mut hash = PRIME1;

    for cell in cells {
        hash ^= (cell.ch as u64).wrapping_mul(PRIME2);
        hash = hash.rotate_left(31).wrapping_mul(PRIME1);
    }

    hash
}

fn bench_hash_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_functions");

    for size in [10, 50, 80, 200, 1000].iter() {
        let line: Vec<Cell> = (0..*size)
            .map(|i| {
                Cell::with_style(
                    (b'A' + (i % 26) as u8) as char,
                    if i % 2 == 0 { Attr::BOLD } else { Attr::NORMAL },
                    if i % 3 == 0 { Color::Red } else { Color::Reset },
                    if i % 5 == 0 {
                        Color::Blue
                    } else {
                        Color::Reset
                    },
                )
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("current", size), &line, |b, line| {
            b.iter(|| black_box(hash_line(line)));
        });

        group.bench_with_input(BenchmarkId::new("multiply", size), &line, |b, line| {
            b.iter(|| black_box(hash_line_multiply(line)));
        });

        group.bench_with_input(BenchmarkId::new("fnv1a", size), &line, |b, line| {
            b.iter(|| black_box(hash_line_fnv1a(line)));
        });

        group.bench_with_input(BenchmarkId::new("std", size), &line, |b, line| {
            b.iter(|| black_box(hash_line_std(line)));
        });

        group.bench_with_input(BenchmarkId::new("xxhash_simple", size), &line, |b, line| {
            b.iter(|| black_box(hash_line_xxhash_simple(line)));
        });
    }

    group.finish();
}

fn bench_hash_collision_rate(c: &mut Criterion) {
    c.bench_function("hash_collision_analysis", |b| {
        b.iter(|| {
            // Create 1000 different lines and check for hash collisions
            let mut hashes = std::collections::HashSet::new();
            let mut collisions = 0;

            for i in 0..1000 {
                let line: Vec<Cell> = (0..80)
                    .map(|j| Cell::new(((i + j) % 128) as u8 as char))
                    .collect();

                let hash = hash_line_fnv1a(&line);
                if !hashes.insert(hash) {
                    collisions += 1;
                }
            }

            black_box((hashes.len(), collisions))
        });
    });
}

fn bench_hash_consistency(c: &mut Criterion) {
    let line: Vec<Cell> = (0..80)
        .map(|i| {
            Cell::with_style(
                (b'A' + (i % 26) as u8) as char,
                Attr::BOLD,
                Color::Red,
                Color::Reset,
            )
        })
        .collect();

    c.bench_function("hash_consistency_check", |b| {
        b.iter(|| {
            // Hash the same line multiple times - should always be the same
            let h1 = hash_line_fnv1a(&line);
            let h2 = hash_line_fnv1a(&line);
            let h3 = hash_line_fnv1a(&line);
            black_box((h1, h2, h3, h1 == h2 && h2 == h3))
        });
    });
}

fn bench_hash_screen_lines(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_screen");

    for &(rows, cols) in &[(24, 80), (60, 200), (100, 300)] {
        let screen: Vec<Vec<Cell>> = (0..rows)
            .map(|row| {
                (0..cols)
                    .map(|col| Cell::new(((row + col) % 128) as u8 as char))
                    .collect()
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("full_screen", format!("{}x{}", rows, cols)),
            &screen,
            |b, screen: &Vec<Vec<Cell>>| {
                b.iter(|| {
                    let hashes: Vec<u64> = screen.iter().map(|line| hash_line(line)).collect();
                    black_box(hashes)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_hash_functions,
    bench_hash_collision_rate,
    bench_hash_consistency,
    bench_hash_screen_lines,
);
criterion_main!(benches);
