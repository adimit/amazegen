use qrcode::QrCode;
use svg::Node;

use super::{
    feature::{Algorithm, Shape},
    paint::{RenderedMaze, WebColour},
};

#[derive(Debug)]
pub struct Solution<T> {
    pub path: Vec<T>,
    pub distances: Vec<usize>,
}

pub trait Maze {
    type Idx: Eq + PartialEq + Copy + Clone;

    fn carve(&mut self, node: Self::Idx, neighbour: Self::Idx);

    fn get_walls(&self, node: Self::Idx) -> Vec<Self::Idx>;

    fn get_paths(&self, node: Self::Idx) -> Vec<Self::Idx>;

    fn get_random_node(&self) -> Self::Idx;

    fn get_all_edges(&self) -> Vec<(Self::Idx, Self::Idx)>;

    fn get_all_nodes(&self) -> Vec<Self::Idx>;

    fn get_index(&self, node: Self::Idx) -> usize;

    fn make_solution(&mut self) -> Solution<Self::Idx>;
}

pub trait MazeRenderer<M: Maze> {
    fn stain(&mut self, gradient: (WebColour, WebColour));
    fn solve(&mut self, stroke_colour: WebColour);
    fn paint(&mut self, border: WebColour);
    fn render(&self) -> RenderedMaze;
}

pub struct Metadata {
    algorithm: Algorithm,
    shape: Shape,
    seed: u64,
    maze_url: Option<String>,
}

const SCALE: f64 = 0.2;

impl Metadata {
    pub fn new(algorithm: Algorithm, shape: Shape, seed: u64, maze_url: Option<String>) -> Self {
        Self {
            algorithm,
            shape,
            seed,
            maze_url,
        }
    }

    // returns the y offset that the text will need in the document viewport
    pub fn append_to_svg_document(&self, doc: &mut svg::Document, (x, y): (u32, u32)) -> u32 {
        let font_size = y / 50;
        let text_node = svg::node::element::Text::new(format!(
            "Shape: {}, Algorithm: {:?}, Seed: {}",
            shape_to_str(&self.shape),
            self.algorithm,
            self.seed
        ))
        .set("x", x / 75)
        .set("y", y + (font_size * 2))
        .set("font-size", font_size) // todo: make the font size proportionate to the maze size
        .set("font-family", "sans-serif");

        let offset = if let Some(url) = &self.maze_url {
            let qrcode = QrCode::new(url.as_bytes()).expect("Failed to create QR code");
            let qr_svg = qrcode.render::<qrcode::render::svg::Color>().build();
            println!("QR code: {}", qr_svg);
            let qr_tree = ::svg::read(&qr_svg).expect("Failed to parse QR code SVG");
            let (qr_height, qr_width, qr_path) = {
                let mut height: u32 = 328;
                let mut width: u32 = 328;
                let mut d: String = String::new();
                for event in qr_tree {
                    use svg::node::element::tag;
                    use svg::parser::Event;
                    match event {
                        Event::Tag(tag::Path, _, attributes) => {
                            d = attributes
                                .get("d")
                                .expect("Failed to get QR code path")
                                .to_string();
                        }
                        Event::Tag(tag::Rectangle, _, attributes) => {
                            attributes.get("height").map(|h| {
                                height = h.parse().expect("Failed to parse QR code height")
                            });
                            attributes
                                .get("width")
                                .map(|w| width = w.parse().expect("Failed to parse QR code width"));
                        }
                        _ => {}
                    }
                }
                (height, width, d)
            };

            let qr_node = svg::node::element::Path::new()
                .set("d", qr_path)
                .set("shape-rendering", "crispEdges")
                .set("stroke", "black");
            let mut qr_group = svg::node::element::Group::new().set(
                "transform",
                format!(
                    "translate({},{}), scale({},{})",
                    x as f64 * 0.8,
                    y,
                    x as f64 / qr_width as f64 * SCALE,
                    y as f64 / qr_height as f64 * SCALE,
                ),
            );
            qr_group.append(qr_node);
            doc.append(qr_group);

            y as f64 * SCALE
        } else {
            (font_size * 3) as f64
        };

        doc.append(text_node);
        offset.floor() as u32
    }
}

fn shape_to_str(shape: &Shape) -> String {
    match shape {
        Shape::Rectilinear(x, y) => format!("Rectilinear {}×{}", x, y),
        Shape::Theta(size) => format!("Theta {}", size),
        Shape::Sigma(size) => format!("Sigma {}", size),
    }
    .to_string()
}
