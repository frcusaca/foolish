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

    #[test]
    fn constanic_states() {
        assert!(Nyes::Econstanic.is_constanic());
        assert!(Nyes::Woconstanic.is_constanic());
        // constantew ⊂ constanic
        assert!(Nyes::Constant.is_constanic());
        assert!(Nyes::Independent.is_constanic());
        assert!(Nyes::Nk.is_constanic());
        assert!(!Nyes::Prembrionic.is_constanic());
    }

    #[test]
    fn constantew_states() {
        assert!(Nyes::Constant.is_constantew());
        assert!(Nyes::Independent.is_constantew());
        assert!(Nyes::Nk.is_constantew());
        assert!(!Nyes::Econstanic.is_constantew());
        assert!(!Nyes::Woconstanic.is_constantew());
    }

    #[test]
    fn settled_states() {
        assert!(Nyes::Econstanic.is_settled());
        assert!(Nyes::Woconstanic.is_settled());
        assert!(Nyes::Constant.is_settled());
        assert!(Nyes::Independent.is_settled());
        assert!(Nyes::Nk.is_settled());
    }

    #[test]
    fn pre_constanic_states_are_not_settled() {
        assert!(!Nyes::Prembrionic.is_settled());
        assert!(!Nyes::Embryonic.is_settled());
        assert!(!Nyes::Braning.is_settled());
    }

    #[test]
    fn settled_is_either_constanic_or_constantew() {
        for nyes in [
            Nyes::Prembrionic,
            Nyes::Embryonic,
            Nyes::Braning,
            Nyes::Econstanic,
            Nyes::Woconstanic,
            Nyes::Constant,
            Nyes::Independent,
            Nyes::Nk,
        ] {
            if nyes.is_settled() {
                assert!(
                    nyes.is_constanic() || nyes.is_constantew(),
                    "{nyes} is settled but neither constanic nor constantew"
                );
            }
        }
    }

    #[test]
    fn nnk_constanic_excludes_nk() {
        assert!(Nyes::Econstanic.is_nnk_constanic());
        assert!(Nyes::Woconstanic.is_nnk_constanic());
        assert!(Nyes::Constant.is_nnk_constanic());
        assert!(Nyes::Independent.is_nnk_constanic());
        assert!(!Nyes::Nk.is_nnk_constanic());
        assert!(!Nyes::Prembrionic.is_nnk_constanic());
    }
}
