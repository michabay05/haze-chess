#[derive(Clone)]
pub struct PRNG {
    seed: u128
}

impl PRNG {
    pub fn new(initial_seed: u128) -> Self {
        Self { seed: initial_seed }
    }

    pub fn rand64(self: &mut Self) -> u64 {
        let mut x = self.seed;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.seed = x;
        // var r = @truncate(u64, x);
        let r = x as u64;
        r ^ ((x >> 64) as u64)
    }

    pub fn sparse_rand64(self: &mut Self) -> u64 {
        self.rand64() & self.rand64() & self.rand64()
    }
}
