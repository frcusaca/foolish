use std::collections::HashMap;

use crate::luid::Luid;
use crate::messages::UbcbMessage;

pub struct MessageChannel {
    mailboxes: HashMap<Luid, Vec<UbcbMessage>>,
}

impl MessageChannel {
    pub fn new() -> Self {
        Self {
            mailboxes: HashMap::new(),
        }
    }

    pub fn send(&mut self, target_luid: Luid, msg: UbcbMessage) {
        self.mailboxes.entry(target_luid).or_default().push(msg);
    }

    pub fn drain(&mut self, luid: Luid) -> Vec<UbcbMessage> {
        self.mailboxes.entry(luid).or_default().drain(..).collect()
    }

    pub fn pending_count(&self, luid: Luid) -> usize {
        self.mailboxes.get(&luid).map_or(0, |mb| mb.len())
    }

    pub fn total_pending(&self) -> usize {
        self.mailboxes.values().map(|mb| mb.len()).sum()
    }
}

impl Default for MessageChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(variant: &str) -> UbcbMessage {
        match variant {
            "search" => UbcbMessage::FulfillSearch {
                source_luid: 0,
                query: "x".into(),
            },
            "response" => UbcbMessage::RespondToSearch {
                target_luid: 0,
                query: "x".into(),
                result: None,
            },
            _ => UbcbMessage::StateChange {
                source_luid: 0,
                old_state: foolish_core::Nyes::Embryonic,
                new_state: foolish_core::Nyes::Braning,
            },
        }
    }

    #[test]
    fn send_and_drain() {
        let mut ch = MessageChannel::new();
        ch.send(1, make_msg("search"));
        ch.send(1, make_msg("update"));
        ch.send(2, make_msg("response"));

        let drained = ch.drain(1);
        assert_eq!(drained.len(), 2);
        assert_eq!(ch.drain(1).len(), 0);
    }

    #[test]
    fn drain_unknown_luid_returns_empty() {
        let mut ch = MessageChannel::new();
        let drained = ch.drain(999);
        assert!(drained.is_empty());
    }

    #[test]
    fn pending_count_per_luid() {
        let mut ch = MessageChannel::new();
        ch.send(1, make_msg("search"));
        ch.send(1, make_msg("update"));
        ch.send(2, make_msg("response"));

        assert_eq!(ch.pending_count(1), 2);
        assert_eq!(ch.pending_count(2), 1);
        assert_eq!(ch.pending_count(999), 0);
    }

    #[test]
    fn total_pending_across_all() {
        let mut ch = MessageChannel::new();
        ch.send(1, make_msg("search"));
        ch.send(1, make_msg("update"));
        ch.send(2, make_msg("response"));
        ch.send(3, make_msg("search"));

        assert_eq!(ch.total_pending(), 4);
    }

    #[test]
    fn total_pending_after_drain() {
        let mut ch = MessageChannel::new();
        ch.send(1, make_msg("search"));
        ch.send(2, make_msg("update"));
        ch.send(3, make_msg("response"));

        assert_eq!(ch.total_pending(), 3);
        ch.drain(2);
        assert_eq!(ch.total_pending(), 2);
        ch.drain(1);
        assert_eq!(ch.total_pending(), 1);
        ch.drain(3);
        assert_eq!(ch.total_pending(), 0);
    }

    #[test]
    fn default_impl_is_empty() {
        let ch = <MessageChannel as Default>::default();
        assert_eq!(ch.total_pending(), 0);
    }
}
