use crate::maze::{
    regular::{Rectilinear2DMap, RectilinearMaze},
    Maze, Node,
};
use std::ops::{Index, IndexMut};

use super::{make_random_longest_exit, MazeGenerator};
pub trait CoordinateMap<C: Node, T>: Index<C, Output = T> + IndexMut<C> {}

struct State<C: Node, M: CoordinateMap<C, Class>> {
    classes: M,
    cells: Vec<Vec<C>>,
}

impl<C, M> State<C, M>
where
    C: Node + Copy,
    M: CoordinateMap<C, Class>,
{
    fn classes_are_distinct(&self, a: C, b: C) -> bool {
        self.classes[a] != self.classes[b]
    }

    fn link(&mut self, a: C, b: C) {
        // to avoid the copy here we'd likely need unsafe
        let mut old: Vec<C> = self.cells[self.classes[a]].drain(..).collect();
        for c in &old {
            self.classes[*c] = self.classes[b];
        }
        self.cells[self.classes[b]].append(&mut old);
    }
}

impl<T> CoordinateMap<(usize, usize), T> for Rectilinear2DMap<T> {}

impl State<(usize, usize), Rectilinear2DMap<Class>> {
    fn new((ex, ey): (usize, usize)) -> Self {
        Self {
            cells: (0..ey)
                .flat_map(|y| (0..ex).map(move |x| vec![(x, y)]))
                .collect(),
            classes: Rectilinear2DMap::new((ex, ey), |(x, y)| Class(x + (ex * y))),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Class(usize);

impl<T> Index<Class> for Vec<T> {
    type Output = T;

    fn index(&self, index: Class) -> &Self::Output {
        &self[index.0]
    }
}

impl<T> IndexMut<Class> for Vec<T> {
    fn index_mut(&mut self, index: Class) -> &mut T {
        &mut self[index.0]
    }
}

pub struct Kruskal<C: Node, M: CoordinateMap<C, Class>> {
    extents: C,
    state: State<C, M>,
}

impl<C: Node + std::fmt::Debug, CM: CoordinateMap<C, Class>> Kruskal<C, CM> {
    fn get_randomized_edges(&self) -> Vec<(C, C)> {
        let mut edges: Vec<_> = C::get_all_edges(self.extents);
        fastrand::shuffle(&mut edges);
        edges
    }

    fn run_kruskal<M: Maze<NodeType = C>>(&mut self, mut maze: M) -> M {
        for (from, to) in self.get_randomized_edges() {
            if self.state.classes_are_distinct(from, to) {
                assert!(maze.move_from_to(from, to), "Invalid direction in move");
                self.state.link(from, to)
            }
        }
        maze
    }
}

impl Kruskal<(usize, usize), Rectilinear2DMap<Class>> {
    pub fn new(extents: (usize, usize), seed: u64) -> Self {
        fastrand::seed(seed);
        Self {
            extents,
            state: State::new(extents),
        }
    }
}

impl MazeGenerator<RectilinearMaze> for Kruskal<(usize, usize), Rectilinear2DMap<Class>> {
    fn generate(&mut self) -> RectilinearMaze {
        let mut maze = self.run_kruskal(RectilinearMaze::new(self.extents));
        make_random_longest_exit(&mut maze);
        maze
    }
}
