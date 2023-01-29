use crate::maze::*;

pub fn solve(maze: &Maze) -> Vec<(usize, usize)> {
    find_shortest_path(maze)
}

fn find_shortest_path(maze: &Maze) -> Vec<(usize, usize)> {
    let distances = dijkstra(maze);
    let mut cursor = maze.get_exit();
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

pub fn dijkstra(maze: &Maze) -> Vec<Vec<usize>> {
    let mut distances = vec![vec![0usize; maze.extents.1]; maze.extents.0];
    let mut frontier: Vec<(usize, usize)> = vec![maze.get_entrance()];
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::maze::*;
    #[test]
    fn dijkstra_should_solve_a_trivial_maze() {
        let mut m = Maze::new((4, 4));
        m.set_entrance(0);
        m.set_exit(3);
        m.move_from((0, 0), Direction::Down);
        m.move_from((0, 1), Direction::Down);
        m.move_from((0, 2), Direction::Down);
        m.move_from((0, 3), Direction::Right);
        m.move_from((1, 3), Direction::Right);
        m.move_from((2, 3), Direction::Right);
    }
}
