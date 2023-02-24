pub mod generator;
pub mod paint;
pub mod regular;
pub mod solver;

pub trait Node: Copy {
    fn get_random_node(extents: Self) -> Self;
    fn get_all_nodes(extents: Self) -> Vec<Self>;
    fn get_all_edges(extents: Self) -> Vec<(Self, Self)>;
}

pub trait Maze: Clone {
    type Coords: Node;

    fn get_extents(&self) -> Self::Coords;

    fn get_entrance(&self) -> Self::Coords;
    fn get_exit(&self) -> Self::Coords;

    fn visit(&mut self, coords: Self::Coords);

    fn move_from_to(&mut self, from: Self::Coords, to: Self::Coords) -> bool;

    fn get_walkable_edges(
        &self,
        coords: Self::Coords,
    ) -> Box<dyn Iterator<Item = Self::Coords> + '_>;
    fn get_possible_targets(&self, coords: Self::Coords) -> Vec<Self::Coords>;
}
