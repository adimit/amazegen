pub mod generator;
pub mod paint;
pub mod regular;
pub mod solver;
pub mod theta;

pub trait Node: Copy {
    fn get_random_node(extents: Self) -> Self;
    fn get_all_nodes(extents: Self) -> Vec<Self>;
    fn get_all_edges(extents: Self) -> Vec<(Self, Self)>;
}

pub trait Maze: Clone {
    type NodeType: Node;

    fn get_extents(&self) -> Self::NodeType;

    fn get_entrance(&self) -> Self::NodeType;
    fn get_exit(&self) -> Self::NodeType;

    fn visit(&mut self, coords: Self::NodeType);

    fn move_from_to(&mut self, from: Self::NodeType, to: Self::NodeType) -> bool;

    fn get_walkable_edges(
        &self,
        coords: Self::NodeType,
    ) -> Box<dyn Iterator<Item = Self::NodeType> + '_>;
    fn get_possible_targets(&self, coords: Self::NodeType) -> Vec<Self::NodeType>;
}
