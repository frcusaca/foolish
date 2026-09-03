//! Nyes extension trait for UBCa — adds predicates to the core `Nyes` enum.
//!
//! Four groups:
//! - **Pre-constanic (nigh)**: PREMBRYONIC, EMBRYONIC, BRANING — `is_preconstanic()`,
//!   with `is_nye()` as an alias for the traditional name.
//! - **Constanic**: ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, NK — `is_constanic()`.
//! - **Constantew**: CONSTANT, INDEPENDENT, NK (constant everywhere) — `is_constantew()`.
//! - **Conclusive**: CONSTANT, INDEPENDENT (a value was reached) — `is_conclusive()`.
//!
//! Conclusive and constantew differ exactly on NK: NK is constant everywhere yet never
//! produced a value, so it is constantew but not conclusive.

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

    /// Pre-constanic (nigh): PREMBRYONIC, EMBRYONIC, BRANING — still stepping.
    fn is_preconstanic(&self) -> bool {
        !self.is_constanic()
    }

    /// Not Yet Evaluated — the older name for the same group. An alias, kept so
    /// the traditional Foolish vocabulary still reads.
    fn is_nye(&self) -> bool {
        self.is_preconstanic()
    }

    /// Conclusive: the FIR reached a value — CONSTANT or INDEPENDENT.
    /// Distinct from `is_constantew()`, which also admits NK: NK is constant
    /// everywhere yet never produced a value.
    fn is_conclusive(&self) -> bool;
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

    fn is_conclusive(&self) -> bool {
        matches!(self, Nyes::Constant | Nyes::Independent)
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
    fn conclusive_states() {
        for &nyes in ALL_NYES {
            let expected = matches!(nyes, Nyes::Constant | Nyes::Independent);
            assert_eq!(nyes.is_conclusive(), expected, "{nyes:?}");
        }
    }

    #[test]
    fn preconstanic_states() {
        for &nyes in ALL_NYES {
            let expected = matches!(nyes, Nyes::Prembrionic | Nyes::Embryonic | Nyes::Braning);
            assert_eq!(nyes.is_preconstanic(), expected, "{nyes:?}");
        }
    }

    #[test]
    fn is_nye_is_alias_for_preconstanic() {
        for &nyes in ALL_NYES {
            assert_eq!(nyes.is_nye(), nyes.is_preconstanic(), "{nyes:?}");
        }
    }

    #[test]
    fn conclusive_is_subset_of_constantew() {
        for &nyes in ALL_NYES {
            if nyes.is_conclusive() {
                assert!(
                    nyes.is_constantew(),
                    "{nyes:?} is conclusive but not constantew"
                );
            }
        }
    }

    #[test]
    fn conclusive_and_constantew_differ_exactly_on_nk() {
        for &nyes in ALL_NYES {
            if nyes == Nyes::Nk {
                assert!(nyes.is_constantew() && !nyes.is_conclusive(), "{nyes:?}");
            } else {
                assert_eq!(
                    nyes.is_conclusive(),
                    nyes.is_constantew(),
                    "{nyes:?} should agree on conclusive vs constantew off of NK"
                );
            }
        }
    }

    #[test]
    fn preconstanic_is_complement_of_constanic() {
        for &nyes in ALL_NYES {
            assert_ne!(nyes.is_preconstanic(), nyes.is_constanic(), "{nyes:?}");
        }
    }
}
