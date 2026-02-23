use criterion::{criterion_group, criterion_main, Criterion};

fn monte_carlo_benchmarks(_c: &mut Criterion) {
    // TODO: Add Monte Carlo benchmarks
}

criterion_group!(benches, monte_carlo_benchmarks);
criterion_main!(benches);
