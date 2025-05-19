use super::interface::Maze;

pub struct MaskedMaze<T, I>
where
    T: Maze<Idx = I>,
{
    maze: T,
    masked: Vec<bool>,
    unmasked_nodes: Vec<I>,
}

impl<T, I> Maze for MaskedMaze<T, I>
where
    T: Maze<Idx = I>,
    I: Eq + PartialEq + Copy + Clone,
{
    type Idx = I;

    fn carve(&mut self, node: Self::Idx, neighbour: Self::Idx) {
        self.maze.carve(node, neighbour);
    }

    fn get_walls(&self, node: Self::Idx) -> Vec<Self::Idx> {
        self.maze
            .get_walls(node)
            .into_iter()
            .filter(|&neighbour| !self.masked[self.get_index(neighbour)])
            .collect()
    }

    fn get_paths(&self, node: Self::Idx) -> Vec<Self::Idx> {
        self.maze.get_paths(node)
    }

    fn get_random_node(&self, rng: &mut super::arengee::Arengee) -> Self::Idx {
        *rng.choice(&self.unmasked_nodes)
    }

    fn get_all_edges(&self) -> Vec<(Self::Idx, Self::Idx)> {
        self.maze
            .get_all_edges()
            .into_iter()
            .filter(|(node, neighbour)| {
                !self.masked[self.get_index(*node)] && !self.masked[self.get_index(*neighbour)]
            })
            .collect()
    }

    fn get_all_nodes(&self) -> Vec<Self::Idx> {
        self.unmasked_nodes.clone()
    }

    fn get_index(&self, node: Self::Idx) -> usize {
        self.maze.get_index(node)
    }

    fn make_solution(
        &mut self,
        rng: &mut super::arengee::Arengee,
    ) -> super::interface::Solution<Self::Idx> {
        self.maze.make_solution(rng)
    }
}

impl<T, I> MaskedMaze<T, I>
where
    T: Maze<Idx = I>,
    I: Eq + PartialEq + Copy + Clone,
{
    pub fn new<F>(maze: T, masker: F) -> Self
    where
        F: Fn(&T) -> Vec<bool>,
    {
        let mask = masker(&maze);
        let unmasked_nodes = maze
            .get_all_nodes()
            .into_iter()
            .filter(|&node| !mask[maze.get_index(node)])
            .collect();
        Self {
            maze,
            masked: mask,
            unmasked_nodes,
        }
    }
}
