use crate::maze::arengee::Arengee;

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
    pub fn new(x: usize, y: usize) -> Self {
        Cartesian { x, y }
    }
    pub fn regular_index(&self, row_size: usize) -> usize {
        row_size * self.y + self.x
    }

    pub fn get_random_contained_coordinate(&self, rng: &mut Arengee) -> Self {
        Self {
            x: rng.u32(0..self.x as u32) as usize,
            y: rng.u32(0..self.y as u32) as usize,
        }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    pub fn get(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}
