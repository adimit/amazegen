use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use itertools::Itertools;

use super::{feature::Configuration, interface::MazePath, Maze as OldMaze, Node};
use crate::maze::{algorithms::dijkstra, interface::Maze};

impl Node for (usize, usize) {
    fn get_random_node(extents: Self) -> Self {
        (fastrand::usize(0..extents.0), fastrand::usize(0..extents.1))
    }

    fn get_all_nodes((ex, ey): Self) -> Vec<Self> {
        (0..(ey))
            .flat_map(|y| (0..(ex)).map(move |x| (x, y)))
            .collect()
    }

    fn get_all_edges((ex, ey): Self) -> Vec<(Self, Self)> {
        (0..(ey - 1))
            .flat_map(|y| {
                (0..(ex - 1)).flat_map(move |x| [((x, y), (x + 1, y)), ((x, y), (x, y + 1))])
            })
            .merge((0..ey - 1).map(|y| ((ex - 1, y), (ex - 1, y + 1))))
            .merge((0..ex - 1).map(|x| ((x, ey - 1), (x + 1, ey - 1))))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct RectilinearMaze {
    fields: std::vec::Vec<Vec<u8>>,
    entrance: usize,
    exit: usize,
    pub extents: (usize, usize),
}

pub struct Rectilinear2DMap<T> {
    storage: Vec<Vec<T>>,
}

impl<T> Index<(usize, usize)> for Rectilinear2DMap<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.storage[y][x]
    }
}

impl<T> IndexMut<(usize, usize)> for Rectilinear2DMap<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.storage[y][x]
    }
}

impl<T> Rectilinear2DMap<T> {
    pub fn new<F>((ex, ey): (usize, usize), f: F) -> Self
    where
        F: Fn((usize, usize)) -> T,
    {
        Self {
            storage: (0..(ey))
                .map(|y| (0..ex).map(|x| f((x, y))).collect())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Direction::Up => "Up",
                Direction::Right => "Right",
                Direction::Down => "Down",
                Direction::Left => "Left",
            }
        )
    }
}

impl Direction {
    pub fn reciprocal(&self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    fn bitmask(&self) -> u8 {
        match self {
            Direction::Left => LEFT,
            Direction::Right => RIGHT,
            Direction::Up => UP,
            Direction::Down => DOWN,
        }
    }
    pub fn iterator() -> impl Iterator<Item = Direction> {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .iter()
        .copied()
    }
}

const VISIT: u8 = 1u8;
const LEFT: u8 = 2u8;
const UP: u8 = 4u8;
const RIGHT: u8 = 8u8;
const DOWN: u8 = 16u8;

impl RectilinearMaze {
    pub fn new(extents: (usize, usize)) -> Self {
        RectilinearMaze {
            extents,
            entrance: 0,
            exit: 0,
            fields: vec![vec![0u8; extents.1]; extents.0],
        }
    }
    pub fn set_entrance(&mut self, entrance: usize) {
        self.entrance = entrance;
        self.remove_wall((entrance, 0), Direction::Up);
    }

    pub fn set_exit(&mut self, exit: usize) {
        self.exit = exit;
        self.remove_wall((exit, self.extents.1 - 1), Direction::Down);
    }

    pub fn has_wall(&self, (x, y): (usize, usize), direction: Direction) -> bool {
        self.fields[x][y] & direction.bitmask() == 0
    }

    fn remove_wall(&mut self, (x, y): (usize, usize), direction: Direction) {
        self.fields[x][y] |= direction.bitmask()
    }

    fn coordinates_in_extents(&self, (x, y): (usize, usize)) -> bool {
        x < self.extents.0 && y < self.extents.1
    }

    fn translate(&self, (x, y): (usize, usize), direction: Direction) -> Option<(usize, usize)> {
        match direction {
            Direction::Left if x > 0 => Some((x - 1, y)),
            Direction::Right if x < self.extents.0 - 1 => Some((x + 1, y)),
            Direction::Up if y > 0 => Some((x, y - 1)),
            Direction::Down if y < self.extents.1 - 1 => Some((x, y + 1)),
            _ => None,
        }
    }
}

impl OldMaze for RectilinearMaze {
    type NodeType = (usize, usize);

    fn get_extents(&self) -> (usize, usize) {
        self.extents
    }

    fn get_entrance(&self) -> (usize, usize) {
        (self.entrance, 0)
    }

    fn get_exit(&self) -> (usize, usize) {
        (self.exit, self.extents.1 - 1)
    }

    fn visit(&mut self, (x, y): (usize, usize)) {
        if self.coordinates_in_extents((x, y)) {
            self.fields[x][y] |= VISIT
        } else {
            panic!("Can't visit coordinates ({x}, {y})")
        }
    }

    fn get_walkable_edges(
        &self,
        (x, y): Self::NodeType,
    ) -> Box<dyn Iterator<Item = Self::NodeType> + '_> {
        Box::new(
            Direction::iterator()
                .filter(move |direction| self.fields[x][y] & direction.bitmask() != 0)
                .filter_map(move |direction| self.translate((x, y), direction)),
        )
    }

    fn get_possible_targets(&self, (x, y): Self::NodeType) -> Vec<Self::NodeType> {
        Direction::iterator()
            .filter_map(|direction| match self.translate((x, y), direction) {
                Some((tx, ty))
                    if self.fields[tx][ty] & VISIT == 0
                        && self.fields[x][y] & direction.bitmask() == 0 =>
                {
                    Some((tx, ty))
                }
                _ => None,
            })
            .collect()
    }

    fn move_from_to(&mut self, (fx, fy): Self::NodeType, (tx, ty): Self::NodeType) -> bool {
        use Direction::*;
        // assert!(
        //     (tx < self.extents.0) && (ty < self.extents.1),
        //     "Attempted to move outside of extents"
        // );
        let direction = match ((fx.abs_diff(tx)), (fy.abs_diff(ty))) {
            (1, 0) if fx < tx => Some(Right),
            (1, 0) => Some(Left),
            (0, 1) if fy < ty => Some(Down),
            (0, 1) => Some(Up),
            _ => None,
        };
        direction
            .map(|d| {
                self.fields[fx][fy] |= VISIT | d.bitmask();
                self.fields[tx][ty] |= VISIT | d.reciprocal().bitmask();
            })
            .is_some()
    }
}

impl Maze for RectilinearMaze {
    type Idx = (usize, usize);

    fn carve(&mut self, node: Self::Idx, neighbour: Self::Idx) {
        self.move_from_to(node, neighbour);
    }

    fn get_walls(&self, node: Self::Idx) -> Vec<Self::Idx> {
        self.get_possible_targets(node)
    }

    fn get_paths(&self, node: Self::Idx) -> Vec<Self::Idx> {
        self.get_walkable_edges(node).collect()
    }

    fn get_random_node(&self) -> Self::Idx {
        <(usize, usize)>::get_random_node(self.extents)
    }

    fn get_all_edges(&self) -> Vec<(Self::Idx, Self::Idx)> {
        <(usize, usize)>::get_all_edges(self.extents)
    }

    fn get_all_nodes(&self) -> Vec<Self::Idx> {
        <(usize, usize)>::get_all_nodes(self.extents)
    }

    fn get_index(&self, (x, y): Self::Idx) -> usize {
        self.extents.0 * y + x
    }

    fn find_path(&mut self) -> MazePath<Self::Idx> {
        self.set_entrance(fastrand::usize(0..self.get_extents().0));
        let entrance_topo = dijkstra(self, self.get_entrance());
        let y = self.get_extents().1 - 1;
        let exit = (0..self.get_extents().0)
            .map(|x| (x, y))
            .max_by_key(|node| entrance_topo[self.get_index(*node)])
            .unwrap_or((
                fastrand::usize(0..self.get_extents().0),
                self.get_extents().1 - 1,
            ));
        let exit_topo = dijkstra(self, exit);
        let entrance = (0..self.get_extents().0)
            .map(|x| (x, 0))
            .max_by_key(|node| exit_topo[self.get_index(*node)])
            .unwrap_or((fastrand::usize(0..self.get_extents().0), 0));
        self.set_entrance(entrance.0);
        self.set_exit(exit.0);

        let mut cursor = exit;
        let mut path = vec![cursor];
        loop {
            cursor = *self
                .get_paths(cursor)
                .iter()
                .min_by_key(|n| exit_topo[self.get_index(**n)])
                .unwrap();
            path.push(cursor);
            if exit_topo[self.get_index(cursor)] == 1 {
                break;
            }
        }
        path.push(entrance);
        MazePath {
            path,
            distances: exit_topo,
        }
    }
}

pub fn test_maze() {
    let svg = Configuration {
        seed: 0,
        shape: super::feature::Shape::Rectilinear(10, 10),
        colour: "000000".to_string(),
        features: [].to_vec(),
        algorithm: crate::maze::feature::Algorithm::GrowingTree,
        stroke_width: 4.0,
    }
    .execute();

    println!("{}", svg.0);
}

#[cfg(test)]
mod test {
    use super::Direction::*;
    use crate::maze::regular::{RectilinearMaze, DOWN, LEFT, RIGHT, UP};
    use crate::maze::Maze;

    #[test]
    fn visit_is_idempotent() {
        let mut m = RectilinearMaze::new((12, 12));
        m.visit((5, 5));
        assert_eq!(m.fields[5][5], 1);
        m.visit((5, 5));
        assert_eq!(m.fields[5][5], 1);
    }

    #[test]
    fn move_tears_down_the_walls_on_both_sides() {
        let mut m = RectilinearMaze::new((12, 12));
        m.move_from_to((1, 1), (1, 2));
        assert_eq!(m.fields[1][1] & DOWN, DOWN);
        assert_eq!(m.fields[1][1] & LEFT, 0);
        assert_eq!(m.fields[1][1] & RIGHT, 0);
        assert_eq!(m.fields[1][1] & UP, 0);

        assert_eq!(m.fields[1][2] & UP, UP);
        assert_eq!(m.fields[1][2] & LEFT, 0);
        assert_eq!(m.fields[1][2] & RIGHT, 0);
        assert_eq!(m.fields[1][2] & DOWN, 0);
    }

    #[test]
    fn translate_does_not_allow_going_off_grid() {
        let m = RectilinearMaze::new((10, 10));
        assert_eq!(m.translate((0, 0), Left), None);
        assert_eq!(m.translate((0, 0), Up), None);
        assert_eq!(m.translate((9, 9), Down), None);
        assert_eq!(m.translate((9, 9), Right), None);
    }

    #[test]
    fn move_with_two_coordinates_removes_walls_when_coordinates_valid() {
        let mut m = RectilinearMaze::new((10, 10));

        assert!(m.move_from_to((0, 0), (1, 0))); // right
        assert!(m.move_from_to((1, 1), (0, 1))); // left
        assert!(m.move_from_to((2, 2), (2, 1))); // up
        assert!(m.move_from_to((3, 3), (3, 4))); // down
    }

    #[test]
    fn move_with_two_coordinates_does_not_remove_walls_when_coordinates_invalid() {
        let mut m = RectilinearMaze::new((10, 10));
        assert!(!m.move_from_to((0, 0), (2, 0)), "Can't skip cell"); // skip
        assert!(!m.move_from_to((1, 1), (0, 0)), "Can't move diagonally"); // Diagonal
        assert!(!m.move_from_to((1, 1), (2, 2)), "Can't move diagonally"); // Diagonal
        assert!(!m.move_from_to((1, 1), (0, 2)), "Can't move diagonally"); // Diagonal
        assert!(!m.move_from_to((1, 1), (2, 0)), "Can't move diagonally"); // Diagonal
    }
}
