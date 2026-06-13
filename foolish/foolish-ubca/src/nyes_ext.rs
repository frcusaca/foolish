//! Nyes extension trait for UBCa — adds `is_settled()` to the core `Nyes` enum.

use foolish_core::fir::Nyes;

/// Extension trait adding UBCa-specific predicates to `Nyes`.
pub trait NyesExt {
    /// Returns true when the NYES should be popped from the task queue.
    /// All constanic states plus NK are settled — NK is terminal (produces no
    /// value) but does not block the queue.
    fn is_settled(&self) -> bool;
}

impl NyesExt for Nyes {
    fn is_settled(&self) -> bool {
        self.is_constanic() || *self == Nyes::Nk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settled_states() {
        // Constanic states are settled
        assert!(Nyes::Econstanic.is_settled());
        assert!(Nyes::Woconstanic.is_settled());
        assert!(Nyes::Constant.is_settled());
        assert!(Nyes::Independent.is_settled());
        // NK is settled (terminal, doesn't block queue)
        assert!(Nyes::Nk.is_settled());
    }

    #[test]
    fn pre_constanic_states_are_not_settled() {
        assert!(!Nyes::Prembrionic.is_settled());
        assert!(!Nyes::Embryonic.is_settled());
        assert!(!Nyes::Braning.is_settled());
    }

    #[test]
    fn is_settled_implies_terminal() {
        // Every settled state is either constanic or NK
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
                    nyes.is_constanic() || nyes == Nyes::Nk,
                    "{nyes} is settled but neither constanic nor Nk"
                );
            }
        }
    }
}
