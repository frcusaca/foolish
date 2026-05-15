use std::sync::atomic::{AtomicU64, Ordering};

pub type Luid = u64;

pub struct LuidGenerator(AtomicU64);

impl LuidGenerator {
    pub fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub fn next(&self) -> Luid {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}

impl Default for LuidGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_zero() {
        let g = LuidGenerator::new();
        assert_eq!(g.next(), 0);
    }

    #[test]
    fn monotonic_uniqueness() {
        let g = LuidGenerator::new();
        let ids: Vec<Luid> = (0..100).map(|_| g.next()).collect();
        assert_eq!(ids[0], 0);
        assert_eq!(ids[99], 99);
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn default_impl_starts_at_zero() {
        let g = <LuidGenerator as Default>::default();
        assert_eq!(g.next(), 0);
        assert_eq!(g.next(), 1);
    }
}
