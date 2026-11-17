use std::fmt;

use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct TabId(Uuid);

impl TabId {
    pub(super) fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub(super) fn parse(value: &str) -> Option<Self> {
        let id = Uuid::parse_str(value).ok()?;
        (id.get_version_num() == 7).then_some(Self(id))
    }
}

impl fmt::Display for TabId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
