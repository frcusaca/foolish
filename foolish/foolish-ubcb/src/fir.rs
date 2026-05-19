use foolish_core::{FirRef, Nyes};

use crate::luid::Luid;
use crate::messages::UbcbMessage;

#[derive(Debug)]
pub struct UbcbFir {
    pub luid: Luid,
    pub fir: FirRef,
    pub inbox: Vec<UbcbMessage>,
}

impl UbcbFir {
    pub fn new(luid: Luid, fir: FirRef) -> Self {
        Self { luid, fir, inbox: Vec::new() }
    }

    pub fn state(&self) -> Nyes {
        self.fir.borrow().state()
    }

    pub fn set_state(&self, state: Nyes) {
        self.fir.borrow_mut().set_state(state);
    }

    pub fn replace_fir(&mut self, new_fir: foolish_core::Fir) {
        self.fir = foolish_core::fir_to_ref(new_fir);
    }

    pub fn receive(&mut self, msg: UbcbMessage) {
        self.inbox.push(msg);
    }

    pub fn drain_inbox(&mut self) -> Vec<UbcbMessage> {
        std::mem::take(&mut self.inbox)
    }

    pub fn fir_variant(&self) -> &'static str {
        self.fir.borrow().fir_variant()
    }
}

/// Builder for UbcbFir.
pub struct UbcbFirBuilder {
    luid: Luid,
    fir: FirRef,
}

impl UbcbFirBuilder {
    pub fn new(luid: Luid, fir: foolish_core::Fir) -> Self {
        Self { luid, fir: foolish_core::fir_to_ref(fir) }
    }
    pub fn with_ref(luid: Luid, fir: FirRef) -> Self {
        Self { luid, fir }
    }
    pub fn build(self) -> UbcbFir {
        UbcbFir { luid: self.luid, fir: self.fir, inbox: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foolish_core::{Compiler, clone_steppable, fir_to_ref};

    fn make_stub_fir() -> FirRef {
        let firs = Compiler::compile("{42}").expect("compile stub brane");
        fir_to_ref(firs[0].clone())
    }

    fn make_empty_brane_fir() -> foolish_core::Fir {
        let firs = Compiler::compile("{}").expect("compile empty brane");
        clone_steppable(&fir_to_ref(firs[0].clone()))
    }

    #[test]
    fn new_stores_luid_and_empty_inbox() {
        let fir = make_stub_fir();
        let ubcb = UbcbFir::new(7, fir);
        assert_eq!(ubcb.luid, 7);
        assert_eq!(ubcb.inbox.len(), 0);
    }

    #[test]
    fn state_reflects_inner_fir() {
        let fir = make_stub_fir();
        let ubcb = UbcbFir::new(0, fir);
        assert_eq!(ubcb.state(), Nyes::Embryonic);
    }

    #[test]
    fn set_state_changes_inner_fir() {
        let fir = make_stub_fir();
        let ubcb = UbcbFir::new(0, fir);
        ubcb.set_state(Nyes::Independent);
        assert_eq!(ubcb.state(), Nyes::Independent);
        ubcb.set_state(Nyes::Constant);
        assert_eq!(ubcb.state(), Nyes::Constant);
    }

    #[test]
    fn receive_adds_to_inbox() {
        let fir = make_stub_fir();
        let mut ubcb = UbcbFir::new(0, fir);
        assert!(ubcb.inbox.is_empty());
        ubcb.receive(UbcbMessage::StateChange {
            source_luid: 5,
            old_state: Nyes::Embryonic,
            new_state: Nyes::Braning,
        });
        assert_eq!(ubcb.inbox.len(), 1);
        ubcb.receive(UbcbMessage::FulfillSearch {
            source_luid: 5,
            query: "x".into(),
        });
        assert_eq!(ubcb.inbox.len(), 2);
    }

    #[test]
    fn drain_inbox_returns_and_clears() {
        let fir = make_stub_fir();
        let mut ubcb = UbcbFir::new(0, fir);
        ubcb.receive(UbcbMessage::StateChange {
            source_luid: 5,
            old_state: Nyes::Embryonic,
            new_state: Nyes::Braning,
        });
        ubcb.receive(UbcbMessage::FulfillSearch {
            source_luid: 5,
            query: "x".into(),
        });

        let drained = ubcb.drain_inbox();
        assert_eq!(drained.len(), 2);
        assert!(ubcb.inbox.is_empty());
        assert!(ubcb.drain_inbox().is_empty());
    }

    #[test]
    fn replace_fir_changes_inner() {
        let fir = make_stub_fir();
        let mut ubcb = UbcbFir::new(0, fir);

        let replaced = make_stub_fir();
        ubcb.replace_fir(clone_steppable(&replaced));
        assert_eq!(ubcb.fir_variant(), "NormalBrane");
    }

    #[test]
    fn fir_variant_for_normal_brane() {
        let fir = make_stub_fir();
        let ubcb = UbcbFir::new(0, fir);
        assert_eq!(ubcb.fir_variant(), "NormalBrane");
    }

    // ── UbcbFirBuilder tests ─────────────────────────────────────────────

    #[test]
    fn ubcb_builder_new_stores_luid_and_fir() {
        let firs = Compiler::compile("{42}").expect("compile");
        let fir_val = firs[0].clone();
        let ubcb = UbcbFirBuilder::new(99, fir_val).build();
        assert_eq!(ubcb.luid, 99);
        assert!(ubcb.inbox.is_empty());
    }

    #[test]
    fn ubcb_builder_with_ref() {
        let fir = make_stub_fir();
        let ubcb = UbcbFirBuilder::with_ref(42, fir).build();
        assert_eq!(ubcb.luid, 42);
        assert_eq!(ubcb.fir_variant(), "NormalBrane");
    }

    #[test]
    fn ubcb_builder_state_reflects_inner() {
        let firs = Compiler::compile("{42}").expect("compile");
        let ubcb = UbcbFirBuilder::new(0, firs[0].clone()).build();
        assert_eq!(ubcb.state(), Nyes::Embryonic);
    }
}
