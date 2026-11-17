use super::identity::TabId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TabMessage {
    Hello(TabId),
    Alive(TabId),
    Goodbye(TabId),
}

impl TabMessage {
    pub(super) fn encode(self) -> String {
        match self {
            Self::Hello(id) => format!("hello:{id}"),
            Self::Alive(id) => format!("alive:{id}"),
            Self::Goodbye(id) => format!("bye:{id}"),
        }
    }

    pub(super) fn decode(value: &str) -> Option<Self> {
        let (kind, value) = value.split_once(':')?;
        let id = TabId::parse(value)?;
        match kind {
            "hello" => Some(Self::Hello(id)),
            "alive" => Some(Self::Alive(id)),
            "bye" => Some(Self::Goodbye(id)),
            _ => None,
        }
    }

    pub(super) fn sender(self) -> TabId {
        match self {
            Self::Hello(id) | Self::Alive(id) | Self::Goodbye(id) => id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_round_trip() {
        let id = TabId::new();
        for message in [
            TabMessage::Hello(id),
            TabMessage::Alive(id),
            TabMessage::Goodbye(id),
        ] {
            assert_eq!(TabMessage::decode(&message.encode()), Some(message));
        }
    }

    #[test]
    fn malformed_messages_are_rejected() {
        for message in [
            "",
            "hello",
            "hello:",
            "hello:123",
            "alive:123:extra",
            "unknown:123",
            "hello:00000000-0000-4000-8000-000000000000",
        ] {
            assert!(TabMessage::decode(message).is_none());
        }
    }
}
