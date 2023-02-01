use crate::maze::*;

#[derive(Debug, Clone)]
pub struct Solver<'a> {
    distances: Vec<Vec<usize>>,
    maze: &'a Maze,
    origin: (usize, usize),
}

impl<'a> Solver<'a> {
    pub fn new(maze: &'a Maze, origin: (usize, usize)) -> Self {
        let distances = dijkstra(maze, origin);
        Self {
            maze,
            distances,
            origin,
        }
    }

    pub fn get_distances_from_origin(&self) -> &Vec<Vec<usize>> {
        &self.distances
    }

    pub fn solve_maze(&self) -> Vec<(usize, usize)> {
        find_shortest_path(self.maze, self.maze.get_entrance(), self.maze.get_exit())
    }

    pub fn find_shortest_path_from_origin(&self, to: (usize, usize)) -> Vec<(usize, usize)> {
        let mut cursor = to;
        let mut path: Vec<(usize, usize)> = vec![cursor];
        loop {
            cursor = self
                .maze
                .get_open_paths(cursor)
                .iter()
                .filter_map(|d| self.maze.translate(cursor, *d))
                .min_by_key(|(x, y)| self.distances[*x][*y])
                .unwrap();
            path.push(cursor);
            if self.distances[cursor.0][cursor.1] == 1 {
                break;
            }
        }

        path
    }

    fn dijkstra(maze: &Maze, origin: (usize, usize)) -> Vec<Vec<usize>> {
        let mut distances = vec![vec![0usize; maze.extents.1]; maze.extents.0];
        let mut frontier: Vec<(usize, usize)> = vec![origin];
        distances[frontier[0].0][frontier[0].1] = 1;
        while !frontier.is_empty() {
            let mut new_frontier: Vec<(usize, usize)> = frontier
                .drain(..)
                .flat_map(|cell| {
                    maze.get_open_paths(cell)
                        .iter()
                        .filter_map(|d| {
                            let new = maze.translate(cell, *d)?;
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
}

pub fn solve(maze: &Maze) -> Vec<(usize, usize)> {
    find_shortest_path(maze, maze.get_entrance(), maze.get_exit())
}

pub fn find_shortest_path(
    maze: &Maze,
    from: (usize, usize),
    to: (usize, usize),
) -> Vec<(usize, usize)> {
    let distances = dijkstra(maze, from);
    let mut cursor = to;
    let mut path: Vec<(usize, usize)> = vec![cursor];
    loop {
        cursor = maze
            .get_open_paths(cursor)
            .iter()
            .filter_map(|d| maze.translate(cursor, *d))
            .min_by_key(|(x, y)| distances[*x][*y])
            .unwrap();
        path.push(cursor);
        if distances[cursor.0][cursor.1] == 1 {
            break;
        }
    }

    path
}

pub fn dijkstra(maze: &Maze, origin: (usize, usize)) -> Vec<Vec<usize>> {
    let mut distances = vec![vec![0usize; maze.extents.1]; maze.extents.0];
    let mut frontier: Vec<(usize, usize)> = vec![origin];
    distances[frontier[0].0][frontier[0].1] = 1;
    while !frontier.is_empty() {
        let mut new_frontier: Vec<(usize, usize)> = frontier
            .drain(..)
            .flat_map(|cell| {
                maze.get_open_paths(cell)
                    .iter()
                    .filter_map(|d| {
                        let new = maze.translate(cell, *d)?;
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
