use super::{solver::Solver, *};

pub trait MazeGenerator<M: Maze> {
    fn generate(&self) -> M;
}

pub mod growing_tree {
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

        fn jarník<M: Maze>(&self, mut maze: M) -> M {
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
            let mut maze = self.jarník(RectilinearMaze::new(self.extents));
            make_random_longest_exit(&mut maze);
            maze
        }
    }
}

pub mod kruskal {
    use crate::maze::{Direction, Maze, RectilinearMaze};

    use super::{make_random_longest_exit, MazeGenerator};

    pub struct KruskalsAlgorithm {
        extents: (usize, usize),
    }

    struct State {
        classes: Vec<Vec<usize>>,
        cells: Vec<Vec<(usize, usize)>>,
    }

    impl State {
        fn new((ex, ey): (usize, usize)) -> Self {
            State {
                cells: (0..ey)
                    .flat_map(|y| (0..ex).map(move |x| vec![(x, y)]))
                    .collect(),
                classes: (0..(ey))
                    .map(|y| (0..ex).map(|x| x + (ex * y)).collect())
                    .collect(),
            }
        }

        fn get_class(&self, (x, y): (usize, usize)) -> usize {
            self.classes[y][x]
        }

        fn classes_are_distinct(&self, a: (usize, usize), b: (usize, usize)) -> bool {
            self.get_class(a) != self.get_class(b)
        }

        fn link(&mut self, a: (usize, usize), b: (usize, usize)) {
            let a_class = self.get_class(a);
            let b_class = self.get_class(b);
            // to avoid the copy here we'd likely need unsafe
            let drained: Vec<_> = self.cells[a_class].drain(..).collect();
            for (x, y) in &drained {
                self.classes[*y][*x] = b_class;
            }
            self.cells[b_class].extend(drained.iter());
        }
    }

    impl KruskalsAlgorithm {
        pub fn new(extents: (usize, usize), seed: u64) -> Self {
            fastrand::seed(seed);
            KruskalsAlgorithm { extents }
        }
        fn get_walls(&self) -> Vec<(usize, usize, Direction)> {
            let mut walls: Vec<_> = (0..(self.extents.1))
                .flat_map(|y| {
                    (0..(self.extents.0))
                        .flat_map(move |x| [(x, y, Direction::Down), (x, y, Direction::Right)])
                })
                .collect();
            fastrand::shuffle(&mut walls);
            walls
        }

        fn run_kruskal<M: Maze>(&self, mut maze: M) -> M {
            let mut state = State::new(self.extents);
            for (x, y, direction) in self.get_walls() {
                match maze.translate((x, y), direction) {
                    Some(target) if state.classes_are_distinct((x, y), target) => {
                        state.link((x, y), target);
                        maze.move_from((x, y), direction);
                    }
                    _ => {}
                }
            }
            maze
        }
    }

    impl MazeGenerator<RectilinearMaze> for KruskalsAlgorithm {
        fn generate(&self) -> RectilinearMaze {
            let mut maze = self.run_kruskal(RectilinearMaze::new(self.extents));
            make_random_longest_exit(&mut maze);
            maze
        }
    }
}

fn make_random_longest_exit(maze: &mut RectilinearMaze) {
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
