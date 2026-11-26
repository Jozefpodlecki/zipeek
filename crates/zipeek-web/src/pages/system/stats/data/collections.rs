use std::{ops::Deref, rc::Rc};

use super::{GrammarRecord, SessionRecord, WordRecord};

macro_rules! shared_records {
    ($name:ident, $record:ty) => {
        #[derive(Clone, PartialEq, Eq)]
        pub(in super::super) struct $name(Rc<[$record]>);

        impl $name {
            pub fn new(records: Vec<$record>) -> Self {
                Self(Rc::from(records.into_boxed_slice()))
            }

            pub fn as_slice(&self) -> &[$record] {
                &self.0
            }

            pub fn ptr_eq(&self, other: &Self) -> bool {
                Rc::ptr_eq(&self.0, &other.0)
            }
        }

        impl From<Vec<$record>> for $name {
            fn from(records: Vec<$record>) -> Self {
                Self::new(records)
            }
        }

        impl Deref for $name {
            type Target = [$record];

            fn deref(&self) -> &Self::Target {
                self.as_slice()
            }
        }

        impl AsRef<[$record]> for $name {
            fn as_ref(&self) -> &[$record] {
                self.as_slice()
            }
        }
    };
}

shared_records!(WordRecords, WordRecord);
shared_records!(GrammarRecords, GrammarRecord);
shared_records!(SessionRecords, SessionRecord);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_snapshot_clones_reuse_the_same_allocation() {
        let original = WordRecords::new(Vec::new());
        let cloned = original.clone();
        assert!(original.ptr_eq(&cloned));
        assert!(original.is_empty());
        assert_eq!(original.as_slice().len(), 0);
    }

    #[test]
    fn independently_created_empty_snapshots_are_not_the_same_allocation() {
        let first = GrammarRecords::new(Vec::new());
        let second = GrammarRecords::new(Vec::new());
        assert!(first == second);
        assert!(!first.ptr_eq(&second));
        let sessions = SessionRecords::new(Vec::new());
        assert!(sessions.is_empty());
    }
}
