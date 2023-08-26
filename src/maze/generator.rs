use super::{regular::RectilinearMaze, solver::Solver, *};
pub mod growing_tree;
pub mod kruskal;

pub trait MazeGenerator<M: Maze> {
    fn generate(&mut self) -> M;
}

fn make_random_longest_exit(maze: &mut RectilinearMaze) {
    maze.set_entrance(fastrand::usize(0..maze.get_extents().0));
    maze.set_exit(find_exit_with_longest_path(maze).0);
}

fn find_exit_with_longest_path(maze: &RectilinearMaze) -> (usize, usize) {
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
