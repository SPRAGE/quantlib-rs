use criterion::{criterion_group, criterion_main, Criterion};

fn pricing_benchmarks(_c: &mut Criterion) {
    // TODO: Add pricing engine benchmarks
}

criterion_group!(benches, pricing_benchmarks);
criterion_main!(benches);
