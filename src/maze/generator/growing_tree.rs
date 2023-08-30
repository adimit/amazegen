use super::{make_random_longest_exit, MazeGenerator};
use crate::maze::{regular::RectilinearMaze, Maze, Node};

pub struct GrowingTreeGenerator<C: Node> {
    extents: C,
    seed: u64,
}

impl<C: Node> GrowingTreeGenerator<C> {
    fn jarník<M: Maze<NodeType = C>>(&self, mut maze: M) -> M {
        let mut vertices: Vec<C> = vec![];
        fastrand::seed(self.seed);

        {
            let start = C::get_random_node(self.extents);
            maze.visit(start);
            vertices.push(start);
        }

        while !vertices.is_empty() {
            let i = vertices.len() - 1;
            let e = vertices[i];
            let targets = maze.get_possible_targets(e);
            if !targets.is_empty() {
                let target = targets[fastrand::usize(..targets.len())];
                assert!(maze.move_from_to(e, target));
                vertices.push(target);
            } else {
                vertices.remove(i);
            }
        }

        maze
    }
}

impl GrowingTreeGenerator<(usize, usize)> {
    pub fn new(extents: (usize, usize), seed: u64) -> Self {
        Self { extents, seed }
    }
}

impl MazeGenerator<RectilinearMaze> for GrowingTreeGenerator<(usize, usize)> {
    fn generate(&mut self) -> RectilinearMaze {
        let mut maze = self.jarník(RectilinearMaze::new(self.extents));
        make_random_longest_exit(&mut maze);
        maze
    }
}
