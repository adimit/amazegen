use std::ops::{Index, IndexMut};

use crate::maze::{
    algorithms::{dijkstra, find_path},
    arengee::Arengee,
    interface::{Maze, Solution},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cartesian {
    x: usize,
    y: usize,
}

impl From<(usize, usize)> for Cartesian {
    fn from(val: (usize, usize)) -> Self {
        Cartesian { x: val.0, y: val.1 }
    }
}

impl Cartesian {
    fn regular_index(&self, row_size: usize) -> usize {
        row_size * self.y + self.x
    }

    fn get_random_contained_coordinate(&self, rng: &mut Arengee) -> Self {
        Self {
            x: rng.usize(0..self.x),
            y: rng.usize(0..self.y),
        }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    South,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Index<Direction> for Neighbours {
    type Output = Option<Cartesian>;

    fn index(&self, index: Direction) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<Direction> for Neighbours {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Neighbours(Vec<Option<Cartesian>>);

impl Neighbours {
    fn new() -> Self {
        Self(vec![None; 6])
    }
}

#[derive(Debug)]
pub struct SigmaCell {
    coordinates: Cartesian,
    pub accessible: Neighbours,
    inaccessible: Neighbours,
}

impl SigmaCell {
    fn new<C: Into<Cartesian>>(coordinates: C, size: usize) -> Self {
        let coordinates = coordinates.into();
        let Cartesian { x, y } = coordinates;
        let mut inaccessible = Neighbours::new();
        if y > 0 {
            inaccessible[Direction::North] = Some((x, y - 1).into());
        }
        if y < size - 1 {
            inaccessible[Direction::South] = Some((x, y + 1).into());
        }

        if x % 2 == 0 {
            if x < size - 1 {
                inaccessible[Direction::SouthEast] = Some((x + 1, y).into());
                if y > 0 {
                    inaccessible[Direction::NorthEast] = Some((x + 1, y - 1).into());
                }
            }
            if x > 0 {
                inaccessible[Direction::SouthWest] = Some((x - 1, y).into());
                if y > 0 {
                    inaccessible[Direction::NorthWest] = Some((x - 1, y - 1).into());
                }
            }
        } else {
            if x < size - 1 {
                inaccessible[Direction::NorthEast] = Some((x + 1, y).into());
                if y < size - 1 {
                    inaccessible[Direction::SouthEast] = Some((x + 1, y + 1).into());
                }
            }
            if x > 0 {
                inaccessible[Direction::NorthWest] = Some((x - 1, y).into());
                if y < size - 1 {
                    inaccessible[Direction::SouthWest] = Some((x - 1, y + 1).into());
                }
            }
        }
        Self {
            coordinates,
            accessible: Neighbours::new(),
            inaccessible,
        }
    }

    fn carve(&mut self, neighbour: Cartesian) {
        if let Some(index) = self
            .inaccessible
            .0
            .iter()
            .position(|&n| n == Some(neighbour))
        {
            self.inaccessible.0[index] = None;
            self.accessible.0[index] = Some(neighbour);
        }
    }
}

#[derive(Debug)]
pub struct SigmaMaze {
    pub size: usize,
    pub cells: Vec<SigmaCell>,
}

impl SigmaMaze {
    pub fn new(size: usize) -> Self {
        let cells = (0..size)
            .flat_map(|y| (0..size).map(move |x| (x, y)))
            .map(|coordinates| SigmaCell::new(coordinates, size))
            .collect();
        Self { size, cells }
    }

    fn set_exit(&mut self, x: usize, rng: &mut Arengee) {
        let y = self.size - 1;
        let index = Cartesian { x, y }.regular_index(self.size);
        let d = if x % 2 == 0 {
            &Direction::South
        } else {
            rng.choice(&[Direction::South, Direction::SouthEast, Direction::SouthWest])
        };
        self.cells[index].accessible[*d] = Some((x, y + 1).into());
    }

    fn set_entrance(&mut self, x: usize, rng: &mut Arengee) {
        let y = 0;
        let index = Cartesian { x, y }.regular_index(self.size);
        let d = if x % 2 == 0 {
            rng.choice(&[Direction::North, Direction::NorthEast, Direction::NorthWest])
        } else {
            &Direction::North
        };
        self.cells[index].accessible[*d] = Some((x, y).into());
    }

    pub fn has_path(&self, a: &Cartesian, d: Direction) -> bool {
        self.cells[self.get_index(*a)].accessible[d].is_some()
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Maze for SigmaMaze {
    type Idx = Cartesian;

    fn carve(&mut self, node: Self::Idx, neighbour: Self::Idx) {
        let a = self.get_index(node);
        let b = self.get_index(neighbour);
        self.cells[a].carve(neighbour);
        self.cells[b].carve(node);
    }

    fn get_walls(&self, node: Self::Idx) -> Vec<Self::Idx> {
        self.cells[self.get_index(node)]
            .inaccessible
            .0
            .iter()
            .filter_map(|&n| n)
            .collect()
    }

    fn get_paths(&self, node: Self::Idx) -> Vec<Self::Idx> {
        self.cells[self.get_index(node)]
            .accessible
            .0
            .iter()
            .filter_map(|&n| n)
            .collect()
    }

    fn get_random_node(&self, rng: &mut Arengee) -> Self::Idx {
        Cartesian {
            x: self.size,
            y: self.size,
        }
        .get_random_contained_coordinate(rng)
    }

    fn get_all_edges(&self) -> Vec<(Self::Idx, Self::Idx)> {
        self.cells
            .iter()
            .flat_map(|c| {
                [
                    c.inaccessible[Direction::South].map(|n| (c.coordinates, n)),
                    c.inaccessible[Direction::SouthEast].map(|n| (c.coordinates, n)),
                    c.inaccessible[Direction::SouthWest].map(|n| (c.coordinates, n)),
                ]
            })
            .flatten()
            .collect()
    }

    fn get_all_nodes(&self) -> Vec<Self::Idx> {
        self.cells.iter().map(|c| c.coordinates).collect()
    }

    fn get_index(&self, node: Self::Idx) -> usize {
        node.regular_index(self.size)
    }

    fn make_solution(&mut self, rng: &mut Arengee) -> Solution<Self::Idx> {
        let seed_topo = dijkstra(self, (rng.usize(0..self.size), 0).into());

        let exit: Cartesian = {
            let y = self.size - 1;
            (0..self.size)
                .map(|x| (x, y))
                .max_by_key(|&c| seed_topo.get(self.get_index(c.into())))
                .unwrap_or((rng.usize(0..self.size), y))
        }
        .into();

        let exit_topo = dijkstra(self, exit);
        let entrance: Cartesian = (0..self.size)
            .map(|x| (x, 0))
            .max_by_key(|&c| exit_topo.get(self.get_index(c.into())))
            .unwrap_or((rng.usize(0..self.size), 0))
            .into();

        let entrance_topo = dijkstra(self, entrance);
        self.set_entrance(entrance.x, rng);
        self.set_exit(exit.x, rng);
        let path = find_path(self, &exit_topo, entrance, exit);

        Solution {
            path,
            distances: entrance_topo,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_edge_case_00() {
        let cell = SigmaCell::new((0, 0), 3);
        assert_eq!(cell.inaccessible[Direction::North], None);
        assert_eq!(cell.inaccessible[Direction::NorthEast], None);
        assert_eq!(cell.inaccessible[Direction::NorthWest], None);
        assert_eq!(cell.inaccessible[Direction::SouthWest], None);

        assert_eq!(cell.inaccessible[Direction::South], Some((0, 1).into()));
        assert_eq!(cell.inaccessible[Direction::SouthEast], Some((1, 0).into()));
    }

    #[test]
    fn cell_edge_case_10() {
        let cell = SigmaCell::new((1, 0), 3);
        assert_eq!(cell.inaccessible[Direction::North], None);

        assert_eq!(cell.inaccessible[Direction::South], Some((1, 1).into()));
        assert_eq!(cell.inaccessible[Direction::NorthEast], Some((2, 0).into()));
        assert_eq!(cell.inaccessible[Direction::SouthEast], Some((2, 1).into()));
        assert_eq!(cell.inaccessible[Direction::NorthWest], Some((0, 0).into()));
        assert_eq!(cell.inaccessible[Direction::SouthWest], Some((0, 1).into()));
    }

    #[test]
    fn cell_edge_case_end0_even() {
        let cell = SigmaCell::new((3, 0), 4);
        assert_eq!(cell.inaccessible[Direction::North], None);
        assert_eq!(cell.inaccessible[Direction::NorthEast], None);
        assert_eq!(cell.inaccessible[Direction::SouthEast], None);

        assert_eq!(cell.inaccessible[Direction::South], Some((3, 1).into()));
        assert_eq!(cell.inaccessible[Direction::SouthWest], Some((2, 1).into()));
        assert_eq!(cell.inaccessible[Direction::NorthWest], Some((2, 0).into()));
    }

    #[test]
    fn cell_edge_case_end0_odd() {
        let cell = SigmaCell::new((2, 0), 3);
        assert_eq!(cell.inaccessible[Direction::North], None);
        assert_eq!(cell.inaccessible[Direction::NorthEast], None);
        assert_eq!(cell.inaccessible[Direction::NorthWest], None);
        assert_eq!(cell.inaccessible[Direction::SouthEast], None);

        assert_eq!(cell.inaccessible[Direction::South], Some((2, 1).into()));
        assert_eq!(cell.inaccessible[Direction::SouthWest], Some((1, 0).into()));
    }

    #[test]
    fn cell_edge_case_0end() {
        let cell = SigmaCell::new((0, 2), 3);
        assert_eq!(cell.inaccessible[Direction::South], None);
        assert_eq!(cell.inaccessible[Direction::SouthWest], None);
        assert_eq!(cell.inaccessible[Direction::NorthWest], None);

        assert_eq!(cell.inaccessible[Direction::North], Some((0, 1).into()));
        assert_eq!(cell.inaccessible[Direction::NorthEast], Some((1, 1).into()));
        assert_eq!(cell.inaccessible[Direction::SouthEast], Some((1, 2).into()));
    }

    #[test]
    fn cell_edge_case_1end() {
        let cell = SigmaCell::new((1, 2), 3);
        assert_eq!(cell.inaccessible[Direction::South], None);
        assert_eq!(cell.inaccessible[Direction::SouthWest], None);
        assert_eq!(cell.inaccessible[Direction::SouthEast], None);

        assert_eq!(cell.inaccessible[Direction::North], Some((1, 1).into()));
        assert_eq!(cell.inaccessible[Direction::NorthEast], Some((2, 2).into()));
        assert_eq!(cell.inaccessible[Direction::NorthWest], Some((0, 2).into()));
    }

    #[test]
    fn cell_edge_case_endend_odd() {
        let cell = SigmaCell::new((2, 2), 3);
        assert_eq!(cell.inaccessible[Direction::South], None);
        assert_eq!(cell.inaccessible[Direction::SouthEast], None);
        assert_eq!(cell.inaccessible[Direction::NorthEast], None);

        assert_eq!(cell.inaccessible[Direction::North], Some((2, 1).into()));
        assert_eq!(cell.inaccessible[Direction::NorthWest], Some((1, 1).into()));
        assert_eq!(cell.inaccessible[Direction::SouthWest], Some((1, 2).into()));
    }

    #[test]
    fn cell_edge_case_endend_even() {
        let cell = SigmaCell::new((3, 3), 4);
        assert_eq!(cell.inaccessible[Direction::South], None);
        assert_eq!(cell.inaccessible[Direction::SouthEast], None);
        assert_eq!(cell.inaccessible[Direction::NorthEast], None);
        assert_eq!(cell.inaccessible[Direction::SouthWest], None);

        assert_eq!(cell.inaccessible[Direction::North], Some((3, 2).into()));
        assert_eq!(cell.inaccessible[Direction::NorthWest], Some((2, 3).into()));
    }
}
