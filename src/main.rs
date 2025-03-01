#![allow(mixed_script_confusables)]

use amazegen::maze::feature::{Algorithm, Configuration, Feature, Shape};
use svg2pdf::{ConversionOptions, PageOptions};

fn main() -> Result<(), ()> {
    let maze = Configuration {
        seed: 1,
        shape: Shape::Rectilinear(20, 20),
        colour: "000000".to_string(),
        features: vec![Feature::Solve],
        algorithm: Algorithm::GrowingTree,
        stroke_width: 8.0,
    }
    .run();
    std::fs::write("maze.svg", &maze.svg).expect("Failed to write SVG");
    let mut options = svg2pdf::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = svg2pdf::usvg::Tree::from_str(&maze.svg, &options).expect("Failed to parse SVG");
    let pdf = svg2pdf::to_pdf(&tree, ConversionOptions::default(), PageOptions::default())
        .expect("Failed to convert SVG to PDF");

    std::fs::write("maze.pdf", pdf).expect("Failed to write PDF");

    println!("{}", maze.svg);
    println!("{}", maze.hash);
    Ok(())
}
