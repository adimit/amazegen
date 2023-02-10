use criterion::{criterion_group, criterion_main, Criterion};
use maze::{
    config::{Algorithm::GrowingTree, Configuration, Shape::Rectilinear},
    generate_seed,
};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("maze_synthesis", |b| {
        b.iter(|| {
            Configuration {
                algorithm: GrowingTree,
                colour: "000000".into(),
                features: vec![],
                seed: generate_seed(),
                shape: Rectilinear(10, 10),
            }
            .execute()
        })
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
