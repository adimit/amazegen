#![allow(mixed_script_confusables)]

use amazegen::maze::{
    feature::{Algorithm, Configuration, Feature, Shape},
    paint::WebColour,
};
use clap::{Parser, ValueEnum};
use svg2pdf::{ConversionOptions, PageOptions};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliAlgorithm {
    GrowingTree,
    Kruskal,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliShape {
    Rectilinear,
    Theta,
    Sigma,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CliFeature {
    Solve,
    Stain,
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long, default_value = "000")]
    colour: Option<String>,
    #[arg(short, long, value_enum, default_value = "growing-tree")]
    algorithm: Option<CliAlgorithm>,
    #[arg(short, long, default_value = "20")]
    size: Option<u32>,
    #[clap(long, default_value = "4")]
    stroke_width: Option<u32>,
    #[arg(short = 'S', long, value_enum, default_value = "rectilinear")]
    shape: Option<CliShape>,
    #[arg(long, default_value = "false")]
    solve: bool,
    #[arg(long, default_value = "false")]
    stain: bool,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long)]
    pdf: Option<String>,
    #[arg(long)]
    svg: Option<String>,
}

impl Cli {
    fn get_configuration(&self) -> Configuration {
        let size = self.size.unwrap_or(20) as usize;
        let features = {
            let mut features: Vec<Feature> = vec![];
            if self.solve {
                features.push(Feature::Solve);
            }
            if self.stain {
                features.push(Feature::Stain);
            }
            features
        };

        Configuration {
            seed: self.seed.unwrap_or(fastrand::u64(..)),
            shape: match self.shape {
                Some(CliShape::Sigma) => Shape::Sigma(size),
                Some(CliShape::Theta) => Shape::Theta(size),
                _ => Shape::Rectilinear(size, size),
            },
            colour: self
                .colour
                .as_ref()
                .and_then(|s| WebColour::from_string(&s).ok())
                .map(|c| c.to_web_string())
                .unwrap_or("000000".to_string()),
            features,
            algorithm: match self.algorithm {
                Some(CliAlgorithm::Kruskal) => Algorithm::Kruskal,
                _ => Algorithm::GrowingTree,
            },
            stroke_width: self.stroke_width.unwrap_or(2) as f64,
        }
    }
}

fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    let maze = cli.get_configuration().run();

    if let Some(svg_file) = cli.svg {
        std::fs::write(svg_file, &maze.svg).expect("Failed to write SVG");
    }

    if let Some(pdf_file) = cli.pdf {
        let mut options = svg2pdf::usvg::Options::default();
        options.fontdb_mut().load_system_fonts();
        let tree = svg2pdf::usvg::Tree::from_str(&maze.svg, &options).expect("Failed to parse SVG");
        let pdf = svg2pdf::to_pdf(&tree, ConversionOptions::default(), PageOptions::default())
            .expect("Failed to convert SVG to PDF");
        std::fs::write(pdf_file, pdf).expect("Failed to write PDF");
    }

    println!("{}", maze.hash);
    Ok(())
}
