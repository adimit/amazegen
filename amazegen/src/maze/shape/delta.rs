use std::ops::Index;

use crate::maze::{
    arengee::Arengee,
    interface::{Maze, Solution},
};

use super::coordinates::Cartesian;

pub enum Direction {
    ALPHA,
    WEST,
    EAST,
}

#[derive(Debug)]
struct Neighbours {
    alpha: Option<Cartesian<u32>>,
    west: Option<Cartesian<u32>>,
    east: Option<Cartesian<u32>>,
}

impl Neighbours {
    fn new() -> Self {
        Self {
            alpha: None,
            west: None,
            east: None,
        }
    }
}

impl Index<Direction> for Neighbours {
    fn index(&self, index: Direction) -> &Self::Output {
        match index {
            Direction::ALPHA => &self.alpha,
            Direction::WEST => &self.west,
            Direction::EAST => &self.east,
        }
    }

    type Output = Option<Cartesian<u32>>;
}

impl Cartesian<u32> {
    fn checked_north(&self) -> Option<Self> {
        let (x, y) = self.get();
        y.checked_sub(1).map(|y| Cartesian::new(x, y))
    }
    fn checked_west(&self) -> Option<Self> {
        let (x, y) = self.get();
        x.checked_sub(1).map(|x| Cartesian::new(x, y))
    }
    fn checked_east(&self, size: u32) -> Option<Self> {
        let (x, y) = self.get();
        x.checked_add(1).and_then(|x| {
            if x < size {
                Some(Cartesian::new(x, y))
            } else {
                None
            }
        })
    }
    fn checked_south(&self, size: u32) -> Option<Self> {
        let (x, y) = self.get();
        y.checked_add(1).and_then(|y| {
            if y < size {
                Some(Cartesian::new(x, y))
            } else {
                None
            }
        })
    }
}

#[derive(Debug)]
struct DeltaCell {
    coordinates: Cartesian<u32>,
    inaccessible: Neighbours,
    accessible: Neighbours,
}

fn is_top<C: Into<Cartesian<u32>>>(coordinates: C) -> bool {
    let (x, y) = coordinates.into().get();
    x + y % 2 == 0
}

impl DeltaCell {
    fn new<C: Into<Cartesian<u32>> + Copy>(coordinates: C, size: u32) -> Self {
        if is_top(coordinates) {
            Self::new_top(coordinates.into(), size)
        } else {
            Self::new_bottom(coordinates.into(), size)
        }
    }

    fn new_top(coordinates: Cartesian<u32>, size: u32) -> Self {
        Self {
            coordinates,
            inaccessible: Neighbours {
                alpha: coordinates.checked_north(),
                west: coordinates.checked_west(),
                east: coordinates.checked_east(size),
            },
            accessible: Neighbours::new(),
        }
    }

    fn new_bottom(coordinates: Cartesian<u32>, size: u32) -> Self {
        Self {
            coordinates,
            inaccessible: Neighbours {
                alpha: coordinates.checked_south(size),
                west: coordinates.checked_west(),
                east: coordinates.checked_east(size),
            },
            accessible: Neighbours::new(),
        }
    }

    fn carve(&mut self, neighbour: Cartesian<u32>) {
        self.inaccessible.alpha.inspect(|alpha| {
            if alpha == &neighbour {
                self.inaccessible.alpha = None;
                self.accessible.alpha = Some(neighbour);
            }
        });
        self.inaccessible.west.inspect(|west| {
            if west == &neighbour {
                self.inaccessible.west = None;
                self.accessible.west = Some(neighbour);
            }
        });
        self.inaccessible.east.inspect(|east| {
            if east == &neighbour {
                self.inaccessible.east = None;
                self.accessible.east = Some(neighbour);
            }
        });
    }
}

#[derive(Debug)]
pub struct DeltaMaze {
    size: u32,
    cells: Vec<DeltaCell>,
}

impl DeltaMaze {
    pub fn new(size: u32) -> Self {
        let mut cells = Vec::with_capacity((size * size) as usize);
        for x in 0..size {
            for y in 0..size {
                cells.push(DeltaCell::new(Cartesian::new(x, y), size));
            }
        }
        Self { size, cells }
    }
}

impl Maze for DeltaMaze {
    type Idx = Cartesian<u32>;

    fn carve(&mut self, node: Self::Idx, neighbour: Self::Idx) {
        let index = node.regular_index(self.size);
        self.cells[index as usize].carve(neighbour);
    }

    fn get_walls(&self, node: Self::Idx) -> Vec<Self::Idx> {
        let cell = &self.cells[node.regular_index(self.size) as usize];
        let mut walls = Vec::with_capacity(3);
        let neighbours = &cell.inaccessible;
        walls
    }

    fn get_paths(&self, node: Self::Idx) -> Vec<Self::Idx> {
        todo!()
    }

    fn get_random_node(&self, rng: &mut Arengee) -> Self::Idx {
        todo!()
    }

    fn get_all_edges(&self) -> Vec<(Self::Idx, Self::Idx)> {
        todo!()
    }

    fn get_all_nodes(&self) -> Vec<Self::Idx> {
        todo!()
    }

    fn get_index(&self, node: Self::Idx) -> usize {
        todo!()
    }

    fn make_solution(&mut self, rng: &mut Arengee) -> Solution<Self::Idx> {
        todo!()
    }
}
