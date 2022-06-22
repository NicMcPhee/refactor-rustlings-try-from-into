use criterion::{black_box, criterion_group, criterion_main, Criterion};
use refactor_rustlings_try_from_into::original::Color as Original_Color;
use refactor_rustlings_try_from_into::refactored::Color as Refactored_Color;

fn bench_try_from(c: &mut Criterion) {
    let mut group = c.benchmark_group("TryFrom");
    group.bench_function("original from tuple", |b| b.iter(|| Original_Color::try_from(black_box((183, 65, 14))).unwrap()));
    group.bench_function("original from array", |b| b.iter(|| Original_Color::try_from(black_box([183, 65, 14])).unwrap()));
    let v = vec![183, 65, 14];
    // With slice we should use `try_from` function
    group.bench_function("original from slice", |b| b.iter(|| Original_Color::try_from(black_box(&v[..])).unwrap()));
    group.bench_function("refactored from tuple", |b| b.iter(|| Refactored_Color::try_from(black_box((183, 65, 14))).unwrap()));
    group.bench_function("refactored from array", |b| b.iter(|| Refactored_Color::try_from(black_box([183, 65, 14])).unwrap()));
    let v = vec![183, 65, 14];
    // With slice we should use `try_from` function
    group.bench_function("refactored from slice", |b| b.iter(|| Refactored_Color::try_from(black_box(&v[..])).unwrap()));
    group.finish();
}

criterion_group!(try_from, bench_try_from);
criterion_main!(try_from);