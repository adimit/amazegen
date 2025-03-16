use crate::maze::arengee::Arengee;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cartesian<N> {
    x: N,
    y: N,
}

impl<N> Cartesian<N>
where
    N: Copy + std::ops::Add<Output = N> + std::ops::Mul<Output = N>,
{
    pub fn new(x: N, y: N) -> Self {
        Cartesian { x, y }
    }
    pub fn regular_index(&self, row_size: N) -> N {
        row_size * self.y + self.x
    }

    pub fn x(&self) -> N {
        self.x
    }

    pub fn y(&self) -> N {
        self.y
    }

    pub fn get(&self) -> (N, N) {
        (self.x, self.y)
    }
}

impl From<(usize, usize)> for Cartesian<usize> {
    fn from(val: (usize, usize)) -> Self {
        Cartesian { x: val.0, y: val.1 }
    }
}

impl Cartesian<usize> {
    pub fn get_random_contained_coordinate(&self, rng: &mut Arengee) -> Self {
        Self {
            x: rng.u32(0..self.x as u32) as usize,
            y: rng.u32(0..self.y as u32) as usize,
        }
    }
}

impl Cartesian<u32> {
    pub fn get_random_contained_coordinate(&self, rng: &mut Arengee) -> Self {
        Self {
            x: rng.u32(0..self.x),
            y: rng.u32(0..self.y),
        }
    }
}
