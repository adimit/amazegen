use svg::Node;

use super::{
    feature::{Algorithm, Shape, Svg},
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
    base_url: String,
}

impl Metadata {
    pub fn new(algorithm: Algorithm, shape: Shape, seed: u64, base_url: String) -> Self {
        Self {
            algorithm,
            shape,
            seed,
            base_url,
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

        doc.append(text_node);

        font_size * 3
    }

    pub fn add_qr_code() {
        // todo: qr code will also need to be proportionate to the maze size
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
