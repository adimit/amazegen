#![allow(mixed_script_confusables)]

use amazegen::maze::feature::{Algorithm, Configuration, Shape};

fn main() -> Result<(), ()> {
    let maze = Configuration {
        seed: 1,
        shape: Shape::Sigma(10),
        colour: "000000".to_string(),
        features: vec![],
        algorithm: Algorithm::GrowingTree,
        stroke_width: 8.0,
    }
    .run();

    println!("{}", maze.svg);
    Ok(())
}
