use criterion::{criterion_group, criterion_main, Criterion};
use maze::{generate_seed, make_svg_maze};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("maze_synthesis", |b| {
        b.iter(|| make_svg_maze(10, 10, generate_seed(), "000000".into()))
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
