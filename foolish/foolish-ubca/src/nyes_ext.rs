//! Nyes extension trait for UBCa — adds predicates to the core `Nyes` enum.
//!
//! Three categories:
//! - **Pre-constanic (nigh)**: PREMBRYONIC, EMBRYONIC, BRANING
//! - **Constanic**: ECONSTANIC, WOCONSTANIC (context-dependent terminal)
//! - **Constantew**: CONSTANT, INDEPENDENT, NK (constant everywhere)

use foolish_core::fir::Nyes;

/// Extension trait adding UBCa-specific predicates to `Nyes`.
pub trait NyesExt {
    /// Returns true when the NYES should be popped from the task queue.
    /// All constanic + constantew states are settled.
    fn is_settled(&self) -> bool;

    /// All terminal states: ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, NK.
    /// constantew ⊂ constanic.
    fn is_constanic(&self) -> bool;

    /// Constant everywhere: CONSTANT, INDEPENDENT, or NK.
    fn is_constantew(&self) -> bool;

    /// Constanic but NOT NK — for code that needs "constanic but not NK".
    /// E.g., search results that should propagate NK separately.
    fn is_nnk_constanic(&self) -> bool;
}

impl NyesExt for Nyes {
    fn is_settled(&self) -> bool {
        self.is_constanic() || self.is_constantew()
    }

    fn is_constanic(&self) -> bool {
        matches!(self, Nyes::Econstanic | Nyes::Woconstanic) || self.is_constantew()
    }

    fn is_constantew(&self) -> bool {
        matches!(self, Nyes::Constant | Nyes::Independent | Nyes::Nk)
    }

    fn is_nnk_constanic(&self) -> bool {
        self.is_constanic() && *self != Nyes::Nk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_NYES: &[Nyes] = &[
        Nyes::Prembrionic,
        Nyes::Embryonic,
        Nyes::Braning,
        Nyes::Econstanic,
        Nyes::Woconstanic,
        Nyes::Constant,
        Nyes::Independent,
        Nyes::Nk,
    ];

    #[test]
    fn constanic_states() {
        // constanic = ECONSTANIC | WOCONSTANIC | CONSTANT | INDEPENDENT | NK
        for &nyes in ALL_NYES {
            let expected = !matches!(nyes, Nyes::Prembrionic | Nyes::Embryonic | Nyes::Braning);
            assert_eq!(nyes.is_constanic(), expected, "{nyes:?}");
        }
    }

    #[test]
    fn constantew_states() {
        // constantew = CONSTANT | INDEPENDENT | NK
        for &nyes in ALL_NYES {
            let expected = matches!(nyes, Nyes::Constant | Nyes::Independent | Nyes::Nk);
            assert_eq!(nyes.is_constantew(), expected, "{nyes:?}");
        }
    }

    #[test]
    fn nnk_constanic_states() {
        // nnk_constanic = constanic && !NK
        for &nyes in ALL_NYES {
            let expected = nyes.is_constanic() && nyes != Nyes::Nk;
            assert_eq!(nyes.is_nnk_constanic(), expected, "{nyes:?}");
        }
    }

    #[test]
    fn settled_states() {
        // settled = constanic (which includes constantew)
        for &nyes in ALL_NYES {
            assert_eq!(nyes.is_settled(), nyes.is_constanic(), "{nyes:?}");
        }
    }

    #[test]
    fn constantew_is_subset_of_constanic() {
        for &nyes in ALL_NYES {
            if nyes.is_constantew() {
                assert!(
                    nyes.is_constanic(),
                    "{nyes:?} is constantew but not constanic"
                );
            }
        }
    }

    #[test]
    fn settled_is_either_constanic_or_constantew() {
        for &nyes in ALL_NYES {
            if nyes.is_settled() {
                assert!(
                    nyes.is_constanic() || nyes.is_constantew(),
                    "{nyes:?} is settled but neither constanic nor constantew"
                );
            }
        }
    }
}
