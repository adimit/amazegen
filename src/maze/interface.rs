use super::{
    arengee::Arengee,
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

    fn get_random_node(&self, rng: &mut Arengee) -> Self::Idx;

    fn get_all_edges(&self) -> Vec<(Self::Idx, Self::Idx)>;

    fn get_all_nodes(&self) -> Vec<Self::Idx>;

    fn get_index(&self, node: Self::Idx) -> usize;

    fn make_solution(&mut self, rng: &mut Arengee) -> Solution<Self::Idx>;
}

pub trait MazeRenderer<M: Maze> {
    fn stain(&mut self, gradient: (WebColour, WebColour));
    fn solve(&mut self, stroke_colour: WebColour);
    fn paint(&mut self, border: WebColour);
    fn render(&self) -> RenderedMaze;
}
