use crate::maze::*;

#[derive(Debug, Clone)]
pub struct Solver<'a, M: Maze> {
    distances: Vec<Vec<usize>>,
    maze: &'a M,
}

impl<'a, M: Maze<NodeType = (usize, usize)>> Solver<'a, M> {
    pub fn new(maze: &'a M, origin: (usize, usize)) -> Self {
        let distances = dijkstra(maze, origin);
        Self { maze, distances }
    }

    pub fn get_distances_from_origin(&self) -> &Vec<Vec<usize>> {
        &self.distances
    }

    pub fn solve_maze(&self) -> Vec<(usize, usize)> {
        self.find_shortest_path_from_origin(self.maze.get_exit())
    }

    pub fn find_shortest_path_from_origin(&self, to: (usize, usize)) -> Vec<(usize, usize)> {
        let mut cursor = to;
        let mut path: Vec<(usize, usize)> = vec![cursor];
        loop {
            cursor = self
                .maze
                .get_walkable_edges(cursor)
                .min_by_key(|(x, y)| self.distances[*x][*y])
                .unwrap();
            path.push(cursor);
            if self.distances[cursor.0][cursor.1] == 1 {
                break;
            }
        }

        path
    }
}

fn dijkstra<M: Maze<NodeType = (usize, usize)>>(
    maze: &M,
    origin: (usize, usize),
) -> Vec<Vec<usize>> {
    let mut distances = vec![vec![0usize; maze.get_extents().1]; maze.get_extents().0];
    let mut frontier: Vec<(usize, usize)> = vec![origin];
    distances[frontier[0].0][frontier[0].1] = 1;
    while !frontier.is_empty() {
        let mut new_frontier: Vec<(usize, usize)> = frontier
            .drain(..)
            .flat_map(|cell| {
                maze.get_walkable_edges(cell)
                    .filter_map(|new| {
                        if distances[new.0][new.1] == 0 {
                            distances[new.0][new.1] = distances[cell.0][cell.1] + 1;
                            Some(new)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        frontier.append(&mut new_frontier);
    }

    distances
}
