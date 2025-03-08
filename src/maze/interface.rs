use svg::Node;

use super::{
    feature::{Algorithm, Shape, Svg},
    paint::WebColour,
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
    fn render(&mut self, metadata: &Metadata) -> Svg;
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

    pub fn append_to_svg_document(&self, doc: &mut svg::Document, view_box: (u32, u32)) {
        let text_node = svg::node::element::Text::new(format!(
            "Algorithm: {:?},  Seed: {}",
            self.algorithm, self.seed
        ));

        

        println!("appending node {:?}", text_node);
        println!("document {:?}", doc);
    }
}
