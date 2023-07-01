use rand::{self, rngs::SmallRng, Rng as RngTrait, SeedableRng};

pub struct Rng(SmallRng);

impl Default for Rng {
    fn default() -> Self {
        Self(SmallRng::seed_from_u64(0))
    }
}

impl pincerhex_core::Rand for Rng {
    fn in_range(&mut self, a: i8, b: i8) -> i8 {
        self.0.gen_range(a..b)
    }

    fn next(&mut self) -> f32 {
        self.0.gen::<f32>()
    }
}
