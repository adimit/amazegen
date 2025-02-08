use amazegen::{
    generate_seed,
    maze::feature::{Algorithm, Configuration, Shape},
};
use criterion::{criterion_group, criterion_main, Criterion};

fn kruskal_rect(c: &mut Criterion) {
    c.bench_function("maze_synthesis", |b| {
        b.iter(|| {
            Configuration {
                algorithm: Algorithm::Kruskal,
                colour: "000000".into(),
                features: vec![],
                seed: generate_seed(),
                shape: Shape::Rectilinear(10, 10),
                stroke_width: 4.0,
            }
            .display_maze()
        })
    });
}

fn jarník_rect(c: &mut Criterion) {
    c.bench_function("maze_synthesis", |b| {
        b.iter(|| {
            Configuration {
                algorithm: Algorithm::GrowingTree,
                colour: "000000".into(),
                features: vec![],
                seed: generate_seed(),
                shape: Shape::Rectilinear(10, 10),
                stroke_width: 4.0,
            }
            .display_maze()
        })
    });
}

fn jarník_theta(c: &mut Criterion) {
    let mut group = c.benchmark_group("theta");
    group.warm_up_time(std::time::Duration::from_secs(6));
    group.measurement_time(std::time::Duration::from_secs(10));
    group.bench_function("maze_synthesis", |b| {
        b.iter(|| {
            Configuration {
                algorithm: Algorithm::GrowingTree,
                colour: "000000".into(),
                features: vec![],
                seed: generate_seed(),
                shape: Shape::Theta(10),
                stroke_width: 4.0,
            }
            .display_maze()
        })
    });
    group.finish();
}

fn kruskal_theta(c: &mut Criterion) {
    let mut group = c.benchmark_group("theta");
    group.warm_up_time(std::time::Duration::from_secs(6));
    group.measurement_time(std::time::Duration::from_secs(10));
    group.bench_function("maze_synthesis", |b| {
        b.iter(|| {
            Configuration {
                algorithm: Algorithm::Kruskal,
                colour: "000000".into(),
                features: vec![],
                seed: generate_seed(),
                shape: Shape::Theta(10),
                stroke_width: 4.0,
            }
            .display_maze()
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    kruskal_rect,
    jarník_rect,
    kruskal_theta,
    jarník_theta
);
criterion_main!(benches);
