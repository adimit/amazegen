use super::{arengee::Arengee, interface::Maze};

struct Kruskal<'a, M>
where
    M: Maze,
{
    maze: &'a mut M,
    // we need a bijection between each node's class and all the nodes in a given class
    // a class is just an integer that happens to be the same one as the node's original
    // index from get_all_nodes()
    // each node has exactly one class
    // each class can have multiple nodes, but starts out with exactly one node
    class_members: Vec<Vec<M::Idx>>,
    classes: Vec<usize>,
}

impl<'a, M> Kruskal<'a, M>
where
    M: Maze,
{
    fn new(maze: &'a mut M) -> Self {
        let all_nodes = maze.get_all_nodes().to_vec();
        Self {
            maze,
            classes: all_nodes.iter().enumerate().map(|(i, _)| i).collect(),
            class_members: all_nodes
                .iter()
                .cloned()
                .map(|n| vec![n])
                .collect::<Vec<_>>(),
        }
    }

    fn link(&mut self, a: M::Idx, b: M::Idx) {
        self.maze.carve(a, b);
        let class_of_a = self.classes[self.maze.get_index(a)];
        let class_of_b = self.classes[self.maze.get_index(b)];
        let members_of_b = self.class_members[class_of_b].drain(..).collect::<Vec<_>>();
        for member_of_b in members_of_b {
            self.classes[self.maze.get_index(member_of_b)] = class_of_a;
            self.class_members[class_of_a].push(member_of_b);
        }
    }

    fn classes_are_distinct(&self, a: M::Idx, b: M::Idx) -> bool {
        self.classes[self.maze.get_index(a)] != self.classes[self.maze.get_index(b)]
    }
}

pub fn kruskal<M: Maze>(mut maze: M, rng: &mut Arengee) -> M {
    let mut edges = maze.get_all_edges();
    let mut state = Kruskal::<M>::new(&mut maze);
    rng.shuffle(&mut edges);

    for (a, b) in edges {
        if state.classes_are_distinct(a, b) {
            state.link(a, b);
        }
    }

    maze
}

pub fn jarník<M: Maze>(mut maze: M, rng: &mut Arengee) -> M {
    let start = maze.get_random_node(rng);
    let mut vertices: Vec<M::Idx> = vec![start];
    let mut visited = vec![false; maze.get_all_nodes().len()];
    visited[maze.get_index(start)] = true;

    while !vertices.is_empty() {
        let i = vertices.len() - 1;
        let e = vertices[i];
        let possible_targets = maze
            .get_walls(e)
            .iter()
            .cloned()
            .filter(|n| !visited[maze.get_index(*n)])
            .collect::<Vec<_>>();
        if !possible_targets.is_empty() {
            let target = possible_targets[rng.usize(0..possible_targets.len())];
            maze.carve(e, target);
            visited[maze.get_index(target)] = true;
            vertices.push(target);
        } else {
            vertices.swap_remove(i);
        }
    }

    maze
}

pub fn dijkstra<M: Maze>(maze: &M, origin: M::Idx) -> Vec<usize> {
    let mut distances = maze.get_all_nodes().iter().map(|_| 0).collect::<Vec<_>>();
    let mut frontier: Vec<M::Idx> = vec![origin];
    distances[maze.get_index(origin)] = 1;

    while !frontier.is_empty() {
        let mut new_frontier: Vec<M::Idx> = vec![];
        for cell in frontier.drain(..) {
            for new in maze.get_paths(cell) {
                if distances[maze.get_index(new)] == 0 {
                    distances[maze.get_index(new)] = distances[maze.get_index(cell)] + 1;
                    new_frontier.push(new);
                }
            }
        }
        frontier.append(&mut new_frontier);
    }
    distances
}

/// Find the shortest path from `entrance` to `exit` using `topo`, which is
/// the *exit topology* of `maze`, i.e. the result of `dijkstra` run with
/// `exit` as the origin.
pub fn find_path<M: Maze>(maze: &M, topo: &[usize], entrance: M::Idx, exit: M::Idx) -> Vec<M::Idx> {
    let mut cursor: M::Idx = entrance;
    let mut path = vec![cursor];
    loop {
        cursor = *maze
            .get_paths(cursor)
            .iter()
            .min_by_key(|n| topo[maze.get_index(**n)])
            .unwrap();
        path.push(cursor);
        if topo[maze.get_index(cursor)] == 1 {
            break;
        }
    }
    path.push(exit);
    path
}
