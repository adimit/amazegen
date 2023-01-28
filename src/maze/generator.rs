use super::*;

pub fn jarník(x_size: usize, y_size: usize, seed: u64) -> Maze {
    let mut maze = Maze::new((x_size, y_size));
    let mut vertices: Vec<(usize, usize)> = vec![];
    fastrand::seed(seed);

    {
        let start = (fastrand::usize(0..x_size), 0);
        maze.visit(start);
        vertices.push(start);
    }

    while !vertices.is_empty() {
        let e = vertices.pop().unwrap();
        let directions = maze.get_possible_paths(e);
        if !directions.is_empty() {
            let d = directions[fastrand::usize(..directions.len())];
            match maze.translate((e.0, e.1), d) {
                Some(t) if !maze.is_visited(t) => {
                    maze.move_from((e.0, e.1), d);
                    vertices.push(t);
                }
                _ => {}
            }
        }
    }

    maze.set_entrance(fastrand::usize(0..maze.extents.0));
    maze.set_exit(fastrand::usize(0..maze.extents.0));
    maze
}
