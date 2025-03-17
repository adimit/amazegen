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

    /// Connect two neighbouring cells. May panic if the cells aren't actually neighbours.
    fn carve(&mut self, node: Self::Idx, neighbour: Self::Idx);

    /// Get the inaccessible neighbours of `node`, i.e. all neighbouring
    /// cells for which no connection has yet been carved using `carve`.
    fn get_walls(&self, node: Self::Idx) -> Vec<Self::Idx>;

    /// Get all the accessible neighours of `node`, i.e. all neighbouring
    /// cells for which a connection has been carved using `carve`.
    fn get_paths(&self, node: Self::Idx) -> Vec<Self::Idx>;

    /// Get any random node inside the maze without constraints.
    fn get_random_node(&self, rng: &mut Arengee) -> Self::Idx;

    /// Get all neighbour relations between cells, no matter whether paths have been
    /// carved or not.
    fn get_all_edges(&self) -> Vec<(Self::Idx, Self::Idx)>;

    /// Get all cells of the maze
    fn get_all_nodes(&self) -> Vec<Self::Idx>;

    /// Translate a coordinate to a unique index deterministically. Use this
    /// to save a maze in a one-dimensional data structure, such as a `Vec`.
    fn get_index(&self, node: Self::Idx) -> usize;

    /// Trace a path through a maze. Takes an `Arengee` because it needs
    /// to find a start, or could possibly try to find a random solution
    /// if there were more than one.
    fn make_solution(&mut self, rng: &mut Arengee) -> Solution<Self::Idx>;
}

pub trait MazeRenderer<M: Maze> {
    /// Colour each node of the maze according to its distance from the start by
    /// by mixing the two given colours.
    fn stain(&mut self, gradient: (WebColour, WebColour));

    /// Draw a solution path through the maze in the given stroke colour.
    fn solve(&mut self, stroke_colour: WebColour);

    /// Draw the maze's outline.
    fn paint(&mut self, border: WebColour);

    /// Finish drawing the maze.
    fn render(self) -> RenderedMaze;
}
