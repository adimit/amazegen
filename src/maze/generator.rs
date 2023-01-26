use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Wall {
    x: usize,
    y: usize,
    d: Direction,
}

pub fn jarník(x_size: usize, y_size: usize, seed: u64) -> Maze {
    let mut maze = Maze::new((x_size, y_size));
    let mut vertices: Vec<Wall> = vec![];
    fastrand::seed(seed);

    let start = (fastrand::usize(0..x_size), 0);
    maze.visit(start);
    vertices.extend(maze.get_possible_paths(start).iter().map(|d| Wall {
        x: start.0,
        y: start.1,
        d: *d,
    }));

    while !vertices.is_empty() {
        let w = vertices.remove(fastrand::usize(
            (if vertices.len() > 4 {
                vertices.len() - 4
            } else {
                0
            })..vertices.len(),
        ));
        match maze.translate((w.x, w.y), w.d) {
            Some(t) if !maze.is_visited(t) => {
                maze.move_from((w.x, w.y), w.d);
                vertices.extend(maze.get_possible_paths(t).iter().map(|d| Wall {
                    x: t.0,
                    y: t.1,
                    d: *d,
                }))
            }
            _ => {}
        }
    }
    maze.remove_wall((fastrand::usize(0..maze.extents.0), 0), Direction::Up);
    maze.remove_wall(
        (fastrand::usize(0..maze.extents.0), maze.extents.1 - 1),
        Direction::Down,
    );
    maze
}
