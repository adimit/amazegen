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
        // this is lifted straight from fastrand, but we reimplement
        // it here without reliance on generating usize randoms for
        // WASM portability.
        for i in 1..slice.len() {
            slice.swap(i, self.rng.u32(0..=i as u32) as usize);
        }
    }

    pub fn choice<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        let i = self.rng.u32(0..slice.len() as u32 - 1);
        &slice[i as usize]
    }

    pub fn get_current_seed(&self) -> u64 {
        self.rng.get_seed()
    }
}
