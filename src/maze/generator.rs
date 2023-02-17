use super::{solver::Solver, *};

pub trait MazeGenerator<M: Maze> {
    fn generate(&mut self) -> M;
}

pub mod growing_tree {
    use crate::maze::{Coordinates, Maze, RectilinearMaze};

    use super::{make_random_longest_exit, MazeGenerator};

    pub struct GrowingTreeGenerator<C: Coordinates> {
        extents: C,
        seed: u64,
    }

    impl<C: Coordinates> GrowingTreeGenerator<C> {
        fn jarník<M: Maze<Coords = C>>(&self, mut maze: M) -> M {
            let mut vertices: Vec<C> = vec![];
            fastrand::seed(self.seed);

            {
                let start = C::get_random(self.extents);
                maze.visit(start);
                vertices.push(start);
            }

            while !vertices.is_empty() {
                let i = vertices.len() - 1;
                let e = vertices[i];
                let directions = maze.get_possible_paths(e);
                if !directions.is_empty() {
                    vertices.push(
                        maze.move_from(e, directions[fastrand::usize(..directions.len())])
                            .unwrap(),
                    );
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
}

pub mod kruskal {
    use crate::maze::{Coordinates, Direction, Maze, Rectilinear2DMap, RectilinearMaze};
    use std::ops::{Index, IndexMut};

    use super::{make_random_longest_exit, MazeGenerator};

    pub trait CoordinateMap<C: Coordinates, T>: Index<C, Output = T> + IndexMut<C> {}

    struct State<C: Coordinates, M: CoordinateMap<C, Class>> {
        classes: M,
        cells: Vec<Vec<C>>,
    }

    impl<C, M> State<C, M>
    where
        C: Coordinates + Copy,
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

    pub struct Kruskal<C: Coordinates, M: CoordinateMap<C, Class>> {
        extents: C,
        state: State<C, M>,
    }

    impl<C: Coordinates, CM: CoordinateMap<C, Class>> Kruskal<C, CM> {
        fn get_walls(&self) -> Vec<(C, Direction)> {
            let mut walls: Vec<_> = C::get_all(self.extents)
                .iter()
                .flat_map(|c| [(*c, Direction::Down), (*c, Direction::Right)])
                .collect();
            fastrand::shuffle(&mut walls);
            walls
        }

        fn run_kruskal<M: Maze<Coords = C>>(&mut self, mut maze: M) -> M {
            for (c, direction) in self.get_walls() {
                match maze.translate(c, direction) {
                    Some(target) if self.state.classes_are_distinct(c, target) => {
                        self.state.link(c, target);
                        maze.move_from(c, direction);
                    }
                    _ => {}
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
}

fn make_random_longest_exit(maze: &mut RectilinearMaze) {
    maze.set_entrance(fastrand::usize(0..maze.get_extents().0));
    maze.set_exit(find_exit_with_longest_path(maze).0);
}

fn find_exit_with_longest_path<M: Maze<Coords = (usize, usize)>>(maze: &M) -> (usize, usize) {
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
