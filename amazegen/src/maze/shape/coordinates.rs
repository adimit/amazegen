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
    fn from((x, y): (usize, usize)) -> Self {
        Cartesian { x, y }
    }
}

impl From<(u32, u32)> for Cartesian<u32> {
    fn from((x, y): (u32, u32)) -> Self {
        Cartesian { x, y }
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

#[cfg(test)]
mod test {
    use super::Cartesian;
    #[test]
    fn indexing_is_correct() {
        assert_eq!(Cartesian::new(0, 0).regular_index(3), 0);
        assert_eq!(Cartesian::new(1, 0).regular_index(3), 1);
        assert_eq!(Cartesian::new(2, 0).regular_index(3), 2);
        assert_eq!(Cartesian::new(0, 1).regular_index(3), 3);
        assert_eq!(Cartesian::new(1, 1).regular_index(3), 4);
        assert_eq!(Cartesian::new(2, 1).regular_index(3), 5);
        assert_eq!(Cartesian::new(0, 2).regular_index(3), 6);
        assert_eq!(Cartesian::new(1, 2).regular_index(3), 7);
        assert_eq!(Cartesian::new(2, 2).regular_index(3), 8);
    }
}
