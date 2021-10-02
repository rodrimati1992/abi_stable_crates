use super::LifetimeIndex;

use std::fmt::{self, Debug};

/// A set of lifetime indices.
pub(crate) struct LifetimeCounters {
    set: Vec<u8>,
}

const MASK: u8 = 0b11;
const MAX_VAL: u8 = 3;

impl LifetimeCounters {
    pub fn new() -> Self {
        Self { set: Vec::new() }
    }
    /// Increments the counter for the `lifetime` lifetime,stopping at 3.
    pub fn increment(&mut self, lifetime: LifetimeIndex) -> u8 {
        let (i, shift) = Self::get_index_shift(lifetime.bits);
        if i >= self.set.len() {
            self.set.resize(i + 1, 0);
        }
        let bits = &mut self.set[i];
        let mask = MASK << shift;
        if (*bits & mask) == mask {
            MAX_VAL
        } else {
            *bits += 1 << shift;
            (*bits >> shift) & MASK
        }
    }

    pub fn get(&self, lifetime: LifetimeIndex) -> u8 {
        let (i, shift) = Self::get_index_shift(lifetime.bits);
        match self.set.get(i) {
            Some(&bits) => (bits >> shift) & MASK,
            None => 0,
        }
    }

    fn get_index_shift(lt: usize) -> (usize, u8) {
        (lt >> 2, ((lt & 3) << 1) as u8)
    }
}

impl Debug for LifetimeCounters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.set.iter().cloned().map(U8Wrapper))
            .finish()
    }
}

#[repr(transparent)]
struct U8Wrapper(u8);

impl fmt::Debug for U8Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counting() {
        let mut counters = LifetimeCounters::new();

        let lts = vec![
            LifetimeIndex::Param(0),
            LifetimeIndex::Param(1),
            LifetimeIndex::Param(2),
            LifetimeIndex::Param(3),
            LifetimeIndex::Param(4),
            LifetimeIndex::Param(5),
            LifetimeIndex::Param(6),
            LifetimeIndex::Param(7),
            LifetimeIndex::Param(8),
            LifetimeIndex::Param(9),
            LifetimeIndex::Param(999),
            LifetimeIndex::ANONYMOUS,
            LifetimeIndex::STATIC,
            LifetimeIndex::NONE,
        ];

        for lt in lts {
            for i in 1..=3 {
                assert_eq!(counters.get(lt), i - 1);
                assert_eq!(counters.increment(lt), i);
                assert_eq!(counters.get(lt), i);
            }
        }
    }
}
