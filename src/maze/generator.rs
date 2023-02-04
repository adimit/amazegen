use super::{solver::Solver, *};

pub trait MazeGenerator<M: Maze> {
    fn generate(&self) -> M;
}

pub mod growingTree {
    use crate::maze::{Maze, RectilinearMaze};

    use super::{make_random_longest_exit, MazeGenerator};

    pub struct GrowingTreeGenerator {
        extents: (usize, usize),
        seed: u64,
    }

    impl GrowingTreeGenerator {
        pub fn new(extents: (usize, usize), seed: u64) -> Self {
            GrowingTreeGenerator { extents, seed }
        }

        fn jarník(&self) -> RectilinearMaze {
            let mut maze = RectilinearMaze::new(self.extents);
            let mut vertices: Vec<(usize, usize)> = vec![];
            fastrand::seed(self.seed);

            {
                let start = (fastrand::usize(0..self.extents.0), 0);
                maze.visit(start);
                vertices.push(start);
            }

            while !vertices.is_empty() {
                let i = vertices.len() - 1;
                let e = vertices[i];
                let directions = maze.get_possible_paths(e);
                if !directions.is_empty() {
                    vertices.push(
                        maze.move_from((e.0, e.1), directions[fastrand::usize(..directions.len())])
                            .unwrap(),
                    );
                } else {
                    vertices.remove(i);
                }
            }

            maze
        }
    }

    impl MazeGenerator<RectilinearMaze> for GrowingTreeGenerator {
        fn generate(&self) -> RectilinearMaze {
            let mut maze = self.jarník();
            make_random_longest_exit(&mut maze);
            maze
        }
    }
}

fn make_random_longest_exit(maze: &mut impl Maze) {
    maze.set_entrance(fastrand::usize(0..maze.get_extents().0));
    maze.set_exit(find_exit_with_longest_path(maze).0);
}

fn find_exit_with_longest_path(maze: &impl Maze) -> (usize, usize) {
    let solver = Solver::new(maze, maze.get_entrance());
    let y = maze.get_extents().1 - 1;
    let best_exit = (0..maze.get_extents().0)
        .max_by_key(|x| {
            let exit = (*x, y);
            solver.find_shortest_path_from_origin(exit).len()
        })
        .unwrap_or_else(|| fastrand::usize(0..maze.get_extents().0));

    (best_exit, y)
}
