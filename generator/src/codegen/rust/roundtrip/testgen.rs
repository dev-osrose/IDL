//! Random instances, as rose-proto's testgen makes them.

use crate::wire::BString;

pub struct Gen(u64);

impl Gen {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    pub fn letters(&mut self, len: usize) -> BString {
        (0..len).map(|_| b'a' + self.below(26) as u8).collect()
    }

    pub fn list<T: Sample>(&mut self) -> Vec<T> {
        let len = self.below(4);
        (0..len).map(|_| T::sample(self)).collect()
    }
}

pub trait Sample {
    fn sample(g: &mut Gen) -> Self;
}

macro_rules! sample_int {
    ($($t:ty),*) => {$(
        impl Sample for $t {
            fn sample(g: &mut Gen) -> Self {
                g.next_u64() as $t
            }
        }
    )*};
}
sample_int!(u8, i8, u16, i16, u32, i32, u64, i64);

impl Sample for f32 {
    fn sample(g: &mut Gen) -> Self {
        (g.next_u64() % 1000) as f32 / 4.0
    }
}

impl Sample for f64 {
    fn sample(g: &mut Gen) -> Self {
        (g.next_u64() % 1000) as f64 / 4.0
    }
}

impl Sample for BString {
    fn sample(g: &mut Gen) -> Self {
        let len = g.below(8);
        g.letters(len)
    }
}
