pub mod generator;
pub mod paint;
pub mod solver;

use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    ops::{Index, IndexMut},
};

use itertools::Itertools;

// UnorderedEq is a way to compare vectors without paying heed to the order of the elements.
// It's lifted from this SO answer: https://stackoverflow.com/a/42748484
#[derive(Debug, Copy, Clone)]
struct UnorderedEq<'a, T: 'a>(&'a [T]);

impl<'a, T> UnorderedEq<'a, T>
where
    T: Eq + Hash,
{
    fn count(&self) -> HashMap<&T, usize> {
        let mut cnt = HashMap::new();
        for i in self.0 {
            *cnt.entry(i).or_insert(0) += 1
        }
        cnt
    }
}

impl<'a, T> PartialEq for UnorderedEq<'a, T>
where
    T: Eq + Hash,
{
    fn eq(&self, other: &Self) -> bool {
        self.count() == other.count()
    }
}

#[derive(Debug, Clone)]
pub struct RectilinearMaze {
    fields: std::vec::Vec<Vec<u8>>,
    entrance: usize,
    exit: usize,
    pub extents: (usize, usize),
}

pub trait Coordinates: Copy {
    fn get_random(extents: Self) -> Self;
    fn get_all(extents: Self) -> Vec<Self>;
    fn is_in_extents(&self, extents: &Self) -> bool;
    fn get_all_edges(extents: Self) -> Vec<(Self, Self)>;
}

impl Coordinates for (usize, usize) {
    fn get_random(extents: Self) -> Self {
        (fastrand::usize(0..extents.0), fastrand::usize(0..extents.1))
    }

    fn get_all((ex, ey): Self) -> Vec<Self> {
        (0..(ey))
            .flat_map(|y| (0..(ex)).map(move |x| (x, y)))
            .collect()
    }

    fn is_in_extents(&self, extents: &Self) -> bool {
        self.0 < extents.0 && self.1 < extents.1
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
    fn new<F>((ex, ey): (usize, usize), f: F) -> Self
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

pub trait Maze: Clone {
    type Coords: Coordinates;

    fn get_extents(&self) -> Self::Coords;

    fn get_entrance(&self) -> Self::Coords;
    fn get_exit(&self) -> Self::Coords;

    fn visit(&mut self, coords: Self::Coords);
    fn is_visited(&self, coords: Self::Coords) -> bool;

    fn move_from(&mut self, coors: Self::Coords, direction: Direction) -> Option<Self::Coords>;
    fn move_from_to(&mut self, from: Self::Coords, to: Self::Coords) -> bool;

    fn get_open_paths(&self, coords: Self::Coords) -> Vec<Direction>;
    fn get_walkable_edges(
        &self,
        coords: Self::Coords,
    ) -> Box<dyn Iterator<Item = Self::Coords> + '_>;
    fn get_walls(&self, coords: Self::Coords) -> Vec<Direction>;
    fn has_wall(&self, coords: Self::Coords, direction: Direction) -> bool;
    fn get_possible_paths(&self, coords: Self::Coords) -> Vec<Direction>;
    fn get_possible_targets(&self, coords: Self::Coords) -> Vec<Self::Coords>;
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

    fn remove_wall(&mut self, (x, y): (usize, usize), direction: Direction) {
        self.fields[x][y] |= direction.bitmask()
    }

    fn coordinates_in_extents(&self, (x, y): (usize, usize)) -> bool {
        x < self.extents.0 && y < self.extents.1
    }

    fn set_entrance(&mut self, entrance: usize) {
        self.entrance = entrance;
        self.remove_wall((entrance, 0), Direction::Up);
    }

    fn set_exit(&mut self, exit: usize) {
        self.exit = exit;
        self.remove_wall((exit, self.extents.1 - 1), Direction::Down);
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

impl Maze for RectilinearMaze {
    type Coords = (usize, usize);

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

    fn is_visited(&self, (x, y): (usize, usize)) -> bool {
        self.coordinates_in_extents((x, y)) && self.fields[x][y] & VISIT > 0
    }

    fn move_from(
        &mut self,
        (x, y): (usize, usize),
        direction: Direction,
    ) -> Option<(usize, usize)> {
        let (tx, ty) = self.translate((x, y), direction)?;

        if !self.coordinates_in_extents((x, y)) || !self.coordinates_in_extents((tx, ty)) {
            panic!("origin or target out of bounds: ({x:?}, {y:?}) → {tx:?}, {ty:?}")
        }

        self.fields[x][y] |= VISIT | direction.bitmask();

        self.fields[tx][ty] |= VISIT | direction.reciprocal().bitmask();

        Some((tx, ty))
    }

    fn get_open_paths(&self, (x, y): (usize, usize)) -> Vec<Direction> {
        Direction::iterator()
            .filter(|direction| self.fields[x][y] & direction.bitmask() != 0)
            .collect()
    }

    fn get_walkable_edges(
        &self,
        (x, y): Self::Coords,
    ) -> Box<dyn Iterator<Item = Self::Coords> + '_> {
        Box::new(
            Direction::iterator()
                .filter(move |direction| self.fields[x][y] & direction.bitmask() != 0)
                .filter_map(move |direction| {
                    self.translate((x, y), direction).map(|target| (target))
                }),
        )
    }

    fn get_walls(&self, (x, y): (usize, usize)) -> Vec<Direction> {
        Direction::iterator()
            .filter(|direction| {
                let mask = direction.bitmask();
                self.fields[x][y] & mask == 0
            })
            .collect()
    }

    fn has_wall(&self, (x, y): (usize, usize), direction: Direction) -> bool {
        self.fields[x][y] & direction.bitmask() == 0
    }

    fn get_possible_paths(&self, (x, y): (usize, usize)) -> Vec<Direction> {
        Direction::iterator()
            .filter(|direction| match self.translate((x, y), *direction) {
                Some((tx, ty)) => {
                    self.fields[tx][ty] & VISIT == 0 && self.fields[x][y] & direction.bitmask() == 0
                }
                None => false,
            })
            .collect()
    }

    fn get_possible_targets(&self, (x, y): Self::Coords) -> Vec<Self::Coords> {
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

    fn move_from_to(&mut self, (fx, fy): Self::Coords, (tx, ty): Self::Coords) -> bool {
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

#[cfg(test)]
mod test {
    use super::*;
    use Direction::*;

    #[test]
    fn is_visited_should_return_true_when_field_has_been_visited() {
        let mut m = RectilinearMaze::new((12, 12));
        m.visit((3, 2));
        assert!(m.is_visited((3, 2)));
        assert!(!m.is_visited((0, 0)));
    }

    #[test]
    fn visit_is_idempotent() {
        let mut m = RectilinearMaze::new((12, 12));
        m.visit((5, 5));
        assert_eq!(m.fields[5][5], 1);
        m.visit((5, 5));
        assert_eq!(m.fields[5][5], 1);
    }

    #[test]
    fn move_marks_both_as_visited() {
        let mut m = RectilinearMaze::new((12, 12));
        m.move_from((1, 1), Down);
        assert!(m.is_visited((1, 1)));
        assert!(m.is_visited((1, 2)));
    }

    #[test]
    fn move_tears_down_the_walls_on_both_sides() {
        let mut m = RectilinearMaze::new((12, 12));
        m.move_from((1, 1), Down);
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
    fn get_open_paths_returns_where_there_are_no_walls() {
        let mut m = RectilinearMaze::new((12, 12));
        m.move_from((2, 2), Left);
        assert_eq!(m.get_open_paths((2, 2)), [Left]);
        assert_eq!(m.get_open_paths((1, 2)), [Right]);
    }

    #[test]
    fn get_walls_returns_where_there_are_walls() {
        let mut m = RectilinearMaze::new((12, 12));
        m.move_from((2, 2), Left);
        assert_eq!(
            UnorderedEq(&m.get_walls((2, 2))),
            UnorderedEq(&[Right, Up, Down])
        );
        assert_eq!(
            UnorderedEq(&m.get_walls((1, 2))),
            UnorderedEq(&[Left, Up, Down])
        );
    }

    #[test]
    fn get_walls_returns_walls_at_the_edges() {
        let mut m = RectilinearMaze::new((10, 10));
        m.move_from((0, 0), Down);
        m.move_from((0, 0), Right);
        assert_eq!(UnorderedEq(&m.get_walls((0, 0))), UnorderedEq(&[Left, Up]))
    }

    #[test]
    fn get_possible_paths_filters_edges_of_maze() {
        let m = RectilinearMaze::new((10, 10));
        assert_eq!(
            UnorderedEq(&m.get_possible_paths((0, 0))),
            UnorderedEq(&[Down, Right])
        );
        assert_eq!(
            UnorderedEq(&m.get_possible_paths((9, 0))),
            UnorderedEq(&[Down, Left])
        );
        assert_eq!(
            UnorderedEq(&m.get_possible_paths((9, 9))),
            UnorderedEq(&[Up, Left])
        );
        assert_eq!(
            UnorderedEq(&m.get_possible_paths((0, 9))),
            UnorderedEq(&[Up, Right])
        );
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
    fn get_possible_paths_filters_visited_cells() {
        let mut m = RectilinearMaze::new((10, 10));
        m.move_from((1, 1), Left);

        assert_eq!(
            UnorderedEq(&m.get_possible_paths((0, 1))),
            UnorderedEq(&[Up, Down])
        );
        assert_eq!(
            UnorderedEq(&m.get_possible_paths((1, 0))),
            UnorderedEq(&[Left, Right])
        );
    }

    #[test]
    fn move_with_two_coordinates_removes_walls_when_coordinates_valid() {
        use Direction::*;
        let mut m = RectilinearMaze::new((10, 10));

        assert!(m.move_from_to((0, 0), (1, 0))); // right
        assert!(m.move_from_to((1, 1), (0, 1))); // left
        assert!(m.move_from_to((2, 2), (2, 1))); // up
        assert!(m.move_from_to((3, 3), (3, 4))); // down

        assert_eq!(m.get_open_paths((0, 0)), vec![Right]);
        assert_eq!(m.get_open_paths((1, 0)), vec![Left]);
        assert_eq!(m.get_open_paths((1, 1)), vec![Left]);
        assert_eq!(m.get_open_paths((0, 1)), vec![Right]);
        assert_eq!(m.get_open_paths((2, 2)), vec![Up]);
        assert_eq!(m.get_open_paths((2, 1)), vec![Down]);
        assert_eq!(m.get_open_paths((3, 3)), vec![Down]);
        assert_eq!(m.get_open_paths((3, 4)), vec![Up]);
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

    #[test]
    fn move_with_two_coordinates_does_nothing_when_trying_to_move_off_extents() {
        let mut m = RectilinearMaze::new((10, 10));
        assert!(
            !m.move_from_to((9, 9), (10, 9)),
            "Can't move off to the right"
        );
        assert!(
            !m.move_from_to((9, 9), (9, 10)),
            "Can't move off to downwards"
        );
    }
}
