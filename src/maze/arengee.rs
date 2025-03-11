use std::ops::Range;

pub struct Arengee {
    rng: fastrand::Rng,
}

impl Arengee {
    pub fn new(seed: u64) -> Self {
        let mut rng = fastrand::Rng::new();
        rng.seed(seed);
        Self { rng }
    }

    pub fn u32(&mut self, range: Range<u32>) -> u32 {
        self.rng.u32(range)
    }

    pub fn u64(&mut self, range: Range<u64>) -> u64 {
        self.rng.u64(range)
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.rng.shuffle(slice);
    }

    pub fn choice<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        self.rng
            .choice(slice)
            .expect("choice doesn't work on empty slices!")
    }
}
