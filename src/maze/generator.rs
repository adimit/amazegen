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

    maze.set_entrance(fastrand::usize(0..maze.extents.0));
    maze.set_exit(fastrand::usize(0..maze.extents.0));
    maze
}

fn find_exit(maze: &Maze) -> usize {
    // let distances = dijkstra(maze, maze.get_entrance());

    let possible_exits = 0;

    0
}
