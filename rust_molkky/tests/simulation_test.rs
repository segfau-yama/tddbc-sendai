use rand::{RngCore, SeedableRng, rngs::StdRng};
use rust_molkky::random_throw;

struct ConstantRng(u64);

impl RngCore for ConstantRng {
    fn next_u32(&mut self) -> u32 {
        self.0 as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.0
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            chunk.copy_from_slice(&self.0.to_le_bytes()[..chunk.len()]);
        }
    }
}

#[test]
fn すべてのピンが倒れる乱数なら12点になる() {
    assert_eq!(random_throw(&mut ConstantRng(0)).calc_point(), 12);
}

#[test]
fn どのピンも倒れない乱数なら0点になる() {
    assert_eq!(random_throw(&mut ConstantRng(u64::MAX)).calc_point(), 0);
}

#[test]
fn 同じシードなら同じ投てきの点数列を再現できる() {
    let mut first = StdRng::seed_from_u64(20261024);
    let mut second = StdRng::seed_from_u64(20261024);
    for _ in 0..100 {
        let points = random_throw(&mut first).calc_point();
        assert!((0..=12).contains(&points));
        assert_eq!(points, random_throw(&mut second).calc_point());
    }
}
