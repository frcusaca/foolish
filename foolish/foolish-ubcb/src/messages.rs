use foolish_core::Nyes;
use crate::luid::Luid;

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum UbcbMessage {
    FulfillSearch {
        source_luid: Luid,
        query: String,
    },
    RespondToSearch {
        target_luid: Luid,
        query: String,
        result: Option<foolish_core::FirRef>,
    },
    StateChange {
        source_luid: Luid,
        old_state: Nyes,
        new_state: Nyes,
    },
}
