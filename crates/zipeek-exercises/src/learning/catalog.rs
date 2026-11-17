use std::collections::BTreeMap;

use super::assessment::AssessmentPolicy;
use super::identity::LexicalIdentity;
use super::model::Skill;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    InvalidIdentity,
    DuplicatePolicy,
    InvalidPolicy,
    UnresolvedIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionStatus {
    Unresolved,
    Verified,
}

#[derive(Debug, Default)]
pub struct AssessmentCatalog {
    identities: BTreeMap<String, ResolutionStatus>,
    policies: BTreeMap<(String, Skill), AssessmentPolicy>,
}

impl AssessmentCatalog {
    pub fn register_identity(
        &mut self,
        identity: &LexicalIdentity,
        status: ResolutionStatus,
    ) -> Result<String, CatalogError> {
        let key = identity
            .stable_key()
            .map_err(|_| CatalogError::InvalidIdentity)?;
        if status != ResolutionStatus::Verified {
            self.policies.retain(|(identity, _), _| identity != &key);
        }
        self.identities.insert(key.clone(), status);
        Ok(key)
    }

    pub fn set_policy(
        &mut self,
        key: &str,
        skill: Skill,
        policy: AssessmentPolicy,
    ) -> Result<(), CatalogError> {
        if self.identities.get(key) != Some(&ResolutionStatus::Verified) {
            return Err(CatalogError::UnresolvedIdentity);
        }
        if policy.policy_version.trim().is_empty()
            || policy.policy_version.chars().any(char::is_control)
        {
            return Err(CatalogError::InvalidPolicy);
        }
        let entry = (key.to_owned(), skill);
        if self.policies.contains_key(&entry) {
            return Err(CatalogError::DuplicatePolicy);
        }
        self.policies.insert(entry, policy);
        Ok(())
    }

    pub fn policy(&self, key: &str, skill: Skill) -> Option<AssessmentPolicy> {
        self.policies.get(&(key.to_owned(), skill)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::{ContextRequirement, MasteryPolicy};

    #[test]
    fn identity_downgrade_revokes_existing_policies() {
        let mut catalog = AssessmentCatalog::default();
        let identity = LexicalIdentity {
            source: "hsk".into(),
            source_lexeme_key: "行".into(),
            reading_key: "xing2".into(),
            sense_key: "1".into(),
        };
        let key = catalog
            .register_identity(&identity, ResolutionStatus::Verified)
            .unwrap();
        let policy = AssessmentPolicy {
            context: ContextRequirement::PhraseRequired,
            policy_version: "assessment-v1",
        };
        catalog
            .set_policy(&key, Skill::Recognition, policy)
            .unwrap();
        catalog
            .register_identity(&identity, ResolutionStatus::Unresolved)
            .unwrap();
        assert_eq!(catalog.policy(&key, Skill::Recognition), None);
        assert_eq!(
            catalog.set_policy(&key, Skill::Recognition, policy),
            Err(CatalogError::UnresolvedIdentity)
        );
    }

    #[test]
    fn provisional_identity_cannot_support_validated_assessment_policy() {
        let mut catalog = AssessmentCatalog::default();
        let identity = LexicalIdentity {
            source: "hsk".into(),
            source_lexeme_key: "lexeme".into(),
            reading_key: "xing2".into(),
            sense_key: "s1".into(),
        };
        let key = catalog
            .register_identity(&identity, ResolutionStatus::Unresolved)
            .unwrap();
        let policy = AssessmentPolicy {
            context: ContextRequirement::PhraseRequired,
            policy_version: "policy-1",
        };
        assert_eq!(
            catalog.set_policy(&key, Skill::Recognition, policy),
            Err(CatalogError::UnresolvedIdentity)
        );
        catalog
            .register_identity(&identity, ResolutionStatus::Verified)
            .unwrap();
        assert_eq!(catalog.set_policy(&key, Skill::Recognition, policy), Ok(()));
        assert_eq!(catalog.policy(&key, Skill::Recognition), Some(policy));
        assert!(MasteryPolicy::default().validate());
    }

    #[test]
    fn unusable_assessment_policies_cannot_enter_catalog() {
        let identity = LexicalIdentity {
            source: "hsk".into(),
            source_lexeme_key: "行".into(),
            reading_key: "xing2".into(),
            sense_key: "1".into(),
        };
        let mut catalog = AssessmentCatalog::default();
        let key = catalog
            .register_identity(&identity, ResolutionStatus::Verified)
            .unwrap();
        let invalid = AssessmentPolicy {
            context: ContextRequirement::PhraseRequired,
            policy_version: " ",
        };
        assert_eq!(
            catalog.set_policy(&key, Skill::Recognition, invalid),
            Err(CatalogError::InvalidPolicy)
        );
        assert_eq!(catalog.policy(&key, Skill::Recognition), None);
    }

    #[test]
    fn invisible_policy_revisions_cannot_be_registered() {
        let mut catalog = AssessmentCatalog::default();
        let identity = LexicalIdentity {
            source: "dictionary".into(),
            source_lexeme_key: "学习".into(),
            reading_key: "xue2xi2".into(),
            sense_key: "1".into(),
        };
        let key = catalog
            .register_identity(&identity, ResolutionStatus::Verified)
            .unwrap();
        let policy = AssessmentPolicy {
            context: ContextRequirement::IsolatedAllowed,
            policy_version: "v1\0hidden",
        };
        assert_eq!(
            catalog.set_policy(&key, Skill::Recognition, policy),
            Err(CatalogError::InvalidPolicy)
        );
        assert_eq!(catalog.policy(&key, Skill::Recognition), None);
    }
}
