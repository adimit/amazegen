use crate::maze::*;

pub fn solve(maze: &Maze) -> Vec<Direction> {
    let paths = dijkstra(maze);
    find_shortest_path(paths)
}

fn find_shortest_path(paths: Vec<Vec<usize>>) -> Vec<Direction> {
    todo!()
}

pub fn dijkstra(maze: &Maze) -> Vec<Vec<usize>> {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::maze::*;
    #[test]
    fn dijkstra_should_solve_a_trivial_maze() {}
}
