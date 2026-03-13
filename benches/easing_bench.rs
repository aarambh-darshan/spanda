use criterion::{black_box, criterion_group, criterion_main, Criterion};
use spanda::easing::*;

fn bench_easings(c: &mut Criterion) {
    let mut group = c.benchmark_group("easing_functions");

    group.bench_function("linear", |b| {
        b.iter(|| linear(black_box(0.5)))
    });

    group.bench_function("ease_out_cubic", |b| {
        b.iter(|| ease_out_cubic(black_box(0.5)))
    });

    group.bench_function("ease_in_out_cubic", |b| {
        b.iter(|| ease_in_out_cubic(black_box(0.5)))
    });

    group.bench_function("ease_out_elastic", |b| {
        b.iter(|| ease_out_elastic(black_box(0.5)))
    });

    group.bench_function("ease_out_bounce", |b| {
        b.iter(|| ease_out_bounce(black_box(0.5)))
    });

    group.bench_function("ease_in_out_expo", |b| {
        b.iter(|| ease_in_out_expo(black_box(0.5)))
    });

    group.finish();

    // Benchmark the enum dispatch path
    c.bench_function("Easing::apply (EaseOutCubic)", |b| {
        let easing = Easing::EaseOutCubic;
        b.iter(|| easing.apply(black_box(0.5)))
    });

    // Benchmark all named variants in a sweep
    c.bench_function("all_named_sweep", |b| {
        let variants = Easing::all_named();
        b.iter(|| {
            for easing in variants {
                let _ = easing.apply(black_box(0.5));
            }
        })
    });
}

criterion_group!(benches, bench_easings);
criterion_main!(benches);
