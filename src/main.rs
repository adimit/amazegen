#![allow(mixed_script_confusables)]

use std::collections::HashMap;

use amazegen::maze::{
    feature::{Algorithm, Configuration, Feature, Shape},
    paint::WebColour,
};
use clap::{Parser, ValueEnum};
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref};
use svg2pdf::usvg::Tree;

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
    #[arg(long)]
    url: Option<String>,
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

fn write_pdf_file(filename: &str, tree: Tree, (width, height): (u32, u32)) {
    let mut alloc = Ref::new(1);
    let catalog_id = alloc.bump();
    let page_tree_id = alloc.bump();
    let page_id = alloc.bump();
    let font_id = alloc.bump();
    let content_id = alloc.bump();
    let font_name = Name(b"F1");
    let svg_name = Name(b"S1");

    let (svg_chunk, svg_id) = svg2pdf::to_chunk(&tree, svg2pdf::ConversionOptions::default())
        .expect("Failed to convert SVG to PDF chunk");
    let mut map = HashMap::new();
    let svg_chunk = svg_chunk.renumber(|old| *map.entry(old).or_insert_with(|| alloc.bump()));
    let svg_id = map
        .get(&svg_id)
        .expect("Failure while embedding SVG in PDF.");

    let mut pdf = Pdf::new();
    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    // Set up a simple A4 page.
    let mut page = pdf.page(page_id);
    let pdf_width = 595.0;
    let pdf_height = 842.0;
    let x_margin = 10.0;
    page.media_box(Rect::new(0.0, 0.0, pdf_width, pdf_height));
    page.parent(page_tree_id);
    page.contents(content_id);

    // compute width and height of the svg in the maze. The width is 595 - 20 = 575, the height is computed to keep the aspect ratio.
    let factor = (pdf_width - 2.0 * x_margin) / width as f32;
    let svg_width = width as f32 * factor;
    let svg_height = height as f32 * factor;

    // Add the font and, more importantly, the SVG to the resource dictionary
    // so that it can be referenced in the content stream.
    let mut resources = page.resources();
    resources.x_objects().pair(svg_name, svg_id);
    resources.fonts().pair(font_name, font_id);
    resources.finish();
    page.finish();

    // Set a predefined font, so we do not have to load anything extra.
    pdf.type1_font(font_id).base_font(Name(b"Helvetica"));

    // Add our graphic.
    let mut content = Content::new();
    content
        .transform([
            svg_width,
            0.0,
            0.0,
            svg_height,
            10.0,
            (822.0 - svg_height) / 2.0,
        ])
        .x_object(svg_name);

    pdf.stream(content_id, &content.finish());
    // Write the SVG chunk into the PDF page.
    pdf.extend(&svg_chunk);

    // Write the file to the disk.
    std::fs::write(filename, pdf.finish()).expect("Failed to write PDF file");
}

fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    let maze = cli.get_configuration().execute_for_svg(cli.url);

    if let Some(svg_file) = cli.svg {
        std::fs::write(svg_file, &maze.content).expect("Failed to write SVG");
    }

    if let Some(pdf_file) = cli.pdf {
        let mut options = svg2pdf::usvg::Options::default();
        let fontdb = options.fontdb_mut();
        fontdb.load_system_fonts();
        let tree =
            svg2pdf::usvg::Tree::from_str(&maze.content, &options).expect("Failed to parse SVG");
        write_pdf_file(&pdf_file, tree, maze.dimensions);
    }

    Ok(())
}
