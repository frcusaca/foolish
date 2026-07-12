//! Nyes extension trait for UBCa — adds predicates to the core `Nyes` enum.
//!
//! Three categories:
//! - **Pre-constanic (nigh)**: PREMBRYONIC, EMBRYONIC, BRANING
//! - **Constanic**: ECONSTANIC, WOCONSTANIC (context-dependent terminal)
//! - **Constantew**: CONSTANT, INDEPENDENT, NK (constant everywhere)

use foolish_core::fir::Nyes;

/// Extension trait adding UBCa-specific predicates to `Nyes`.
pub trait NyesExt {
    /// All terminal states: ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, NK.
    /// constantew ⊂ constanic. Used as task-queue pop predicate and outer acceptance.
    fn is_constanic(&self) -> bool;

    /// Constant everywhere: CONSTANT, INDEPENDENT, or NK.
    fn is_constantew(&self) -> bool;

    /// Constanic but NOT NK — for code that needs "constanic but not NK".
    /// E.g., search results that should propagate NK separately.
    fn is_nnk_constanic(&self) -> bool;

    /// Exactly NK — provably unfindable, terminal.
    fn is_nk(&self) -> bool;
}

impl NyesExt for Nyes {
    fn is_constanic(&self) -> bool {
        matches!(self, Nyes::Econstanic | Nyes::Woconstanic) || self.is_constantew()
    }

    fn is_constantew(&self) -> bool {
        matches!(self, Nyes::Constant | Nyes::Independent | Nyes::Nk)
    }

    fn is_nnk_constanic(&self) -> bool {
        self.is_constanic() && *self != Nyes::Nk
    }

    fn is_nk(&self) -> bool {
        matches!(self, Nyes::Nk)
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
        for &nyes in ALL_NYES {
            let expected = matches!(nyes, Nyes::Constant | Nyes::Independent | Nyes::Nk);
            assert_eq!(nyes.is_constantew(), expected, "{nyes:?}");
        }
    }

    #[test]
    fn nnk_constanic_states() {
        for &nyes in ALL_NYES {
            let expected = nyes.is_constanic() && nyes != Nyes::Nk;
            assert_eq!(nyes.is_nnk_constanic(), expected, "{nyes:?}");
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
    fn pre_constanic_are_not_constanic() {
        for &nyes in &[Nyes::Prembrionic, Nyes::Embryonic, Nyes::Braning] {
            assert!(!nyes.is_constanic(), "{nyes:?} should not be constanic");
        }
    }

    #[test]
    fn nk_states() {
        for &nyes in ALL_NYES {
            let expected = matches!(nyes, Nyes::Nk);
            assert_eq!(nyes.is_nk(), expected, "{nyes:?}");
        }
    }

    #[test]
    fn nk_is_subset_of_constantew() {
        for &nyes in ALL_NYES {
            if nyes.is_nk() {
                assert!(nyes.is_constantew(), "{nyes:?} is NK but not constantew");
            }
        }
    }
}
