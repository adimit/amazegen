#![allow(mixed_script_confusables)]

use amazegen::pdf::PdfWriter;
use amazegen::{
    maze::{
        feature::{Algorithm, Configuration, Feature, Shape},
        paint::WebColour,
    },
    pdf::Font,
};
use clap::{Parser, ValueEnum};

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
#[command(version, about)]
struct Cli {
    #[arg(
        short,
        long,
        default_value = "\"000\"",
        help = "Stroke colour for the maze in Web compatible hex."
    )]
    colour: Option<String>,
    #[arg(
        short,
        long,
        value_enum,
        default_value = "growing-tree",
        help = "Selects the algorithm to genreate the maze.",
        long_help = "growing-tree is a backtracking algorithm that will generate long winding passages with few dead ends. The path through the maze will tend to be very long. kruskal will use Kruskal's algorithm, which tends to generate lots of dead ends but a relatively short path."
    )]
    algorithm: Option<CliAlgorithm>,
    #[arg(
        short,
        long,
        default_value = "20",
        help = "Size of the maze",
        long_help = "What size means may depend on the shape of the maze. For square mazes, it's the number of cells in each row and column. Theta mazes use size to determine the number of rows from the origin."
    )]
    size: Option<u32>,
    #[clap(
        long,
        short = 'b',
        default_value = "4",
        help = "Stroke width for the maze."
    )]
    stroke_width: Option<u32>,
    #[arg(
        short = 'S',
        long,
        value_enum,
        default_value = "rectilinear",
        help = "Shape of the maze.",
        long_help = "rectilinear will draw a square maze with square cells. theta will draw a circular maze with square-ish cells. sigma will draw a square (in cell count) maze with hexagonal cells."
    )]
    shape: Option<CliShape>,
    #[arg(long, default_value = "false", help = "Also draw a solution.")]
    solve: bool,
    #[arg(
        long,
        default_value = "false",
        help = "Stain cells",
        long_help = "Stain cells according to distance from origin."
    )]
    stain: bool,
    #[arg(
        long,
        short = 'i',
        help = "Initial seed for the maze",
        long_help = "This seed will be used to generate the first maze. If --pages is set, subsequent pages will reseed the RNG."
    )]
    initial_seed: Option<u64>,
    #[arg(long, short = 'f', help = "Output the maze(s) as a PDF.")]
    pdf: Option<String>,
    #[arg(long, help = "Output the maze as an SVG.")]
    svg: Option<String>,
    #[arg(
        long,
        short = 'u',
        help = "Base URL for QR code",
        long_help = "If provided, will add a QR code to reference each maze on the URL. E.g. https://aleks.bg/maze"
    )]
    url: Option<String>,
    #[arg(
        long,
        short = 'P',
        default_value = "1",
        help = "Number of PDF pages to fill",
        long_help = "Only works in combination with --pdf"
    )]
    pages: Option<u32>,
    #[arg(
        long,
        help = "Font file to use for metadata",
        long_help = "Only works in combination with --pdf. If provided, will use this font file to render metadata on the PDF. If this option is ommitted, we'll attempt to use the default font (Helvetica), which may not be available. If no metadata is visible in the PDF, try providing a font file here."
    )]
    font: Option<String>,
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
            seed: self.initial_seed.unwrap_or(fastrand::u64(..)),
            shape: match self.shape {
                Some(CliShape::Sigma) => Shape::Sigma(size),
                Some(CliShape::Theta) => Shape::Theta(size),
                _ => Shape::Rectilinear(size, size),
            },
            colour: self
                .colour
                .as_ref()
                .and_then(|s| WebColour::from_string(s).ok())
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
    let mut configuration = cli.get_configuration();

    let font_data = cli
        .font
        .map(|f| std::fs::read(&f).expect("Failed to read font"))
        .and_then(|data| Font::new(data));
    let font_name = font_data.as_ref().map(|f| f.name.clone());

    if let Some(svg_file) = cli.svg {
        let (maze, _) = configuration.execute_for_svg(&cli.url, &font_name);
        std::fs::write(svg_file, &maze.content).expect("Failed to write SVG");
    }

    if let Some(pdf_file) = cli.pdf {
        let mut writer = PdfWriter::new(font_data);
        let pages = cli.pages.unwrap_or(1);

        for _ in 0..pages {
            let (maze, new_seed) = configuration.execute_for_svg(&cli.url, &font_name);
            configuration.seed = new_seed;
            writer.append_maze(&maze);
        }
        writer.write_to_file(&pdf_file);
    }

    Ok(())
}
