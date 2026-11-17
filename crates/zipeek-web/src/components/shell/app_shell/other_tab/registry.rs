use std::collections::BTreeMap;

use super::{identity::TabId, protocol::TabMessage};

const PEER_TIMEOUT_MS: i64 = 7_000;
const MAX_PEERS: usize = 32;

#[derive(Default)]
pub(super) struct PeerRegistry {
    peers: BTreeMap<TabId, i64>,
}

impl PeerRegistry {
    pub(super) fn observe(&mut self, message: TabMessage, now_ms: i64) {
        match message {
            TabMessage::Hello(id) | TabMessage::Alive(id) => {
                if self.peers.contains_key(&id) || self.peers.len() < MAX_PEERS {
                    self.peers.insert(id, now_ms);
                }
            }
            TabMessage::Goodbye(id) => {
                self.peers.remove(&id);
            }
        }
    }

    pub(super) fn expire(&mut self, now_ms: i64) {
        self.peers.retain(|_, last_seen| {
            now_ms.saturating_sub(*last_seen) < PEER_TIMEOUT_MS && *last_seen <= now_ms
        });
    }

    pub(super) fn has_peers(&self) -> bool {
        !self.peers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_heartbeat_goodbye_and_expiry() {
        let id = TabId::new();
        let mut peers = PeerRegistry::default();
        peers.observe(TabMessage::Hello(id), 1_000);
        assert!(peers.has_peers());
        peers.observe(TabMessage::Alive(id), 3_000);
        peers.expire(9_999);
        assert!(peers.has_peers());
        peers.expire(10_000);
        assert!(!peers.has_peers());
        peers.observe(TabMessage::Hello(id), 10_000);
        peers.observe(TabMessage::Goodbye(id), 10_001);
        assert!(!peers.has_peers());
    }

    #[test]
    fn future_timestamps_and_peer_flood_are_bounded() {
        let mut peers = PeerRegistry::default();
        for _ in 0..64 {
            peers.observe(TabMessage::Hello(TabId::new()), 5_000);
        }
        assert_eq!(peers.peers.len(), MAX_PEERS);
        peers.expire(4_999);
        assert!(!peers.has_peers());
    }
}
