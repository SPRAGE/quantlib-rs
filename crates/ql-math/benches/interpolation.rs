use criterion::{criterion_group, criterion_main, Criterion};

fn interpolation_benchmarks(_c: &mut Criterion) {
    // TODO: Add interpolation benchmarks
}

criterion_group!(benches, interpolation_benchmarks);
criterion_main!(benches);
