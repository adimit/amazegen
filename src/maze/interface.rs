use super::{feature::Algorithm, paint::DrawingInstructions};

#[derive(Debug)]
pub struct MazePath<T> {
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

    fn find_path(&mut self) -> MazePath<Self::Idx>;
}

pub trait MazeGen {
    fn create_maze(
        &self,
        seed: u64,
        features: Vec<DrawingInstructions>,
        algorithm: &Algorithm,
    ) -> String;
}
