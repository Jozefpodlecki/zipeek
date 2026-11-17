#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalIdentity {
    pub source: String,
    pub source_lexeme_key: String,
    pub reading_key: String,
    pub sense_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityError {
    MissingField,
    Ambiguous,
}

impl LexicalIdentity {
    pub fn validate(&self) -> Result<(), IdentityError> {
        if [
            &self.source,
            &self.source_lexeme_key,
            &self.reading_key,
            &self.sense_key,
        ]
        .into_iter()
        .any(|field| {
            field.trim().is_empty()
                || field.trim() != field.as_str()
                || field.chars().any(char::is_control)
        }) {
            return Err(IdentityError::MissingField);
        }
        Ok(())
    }

    pub fn stable_key(&self) -> Result<String, IdentityError> {
        self.validate()?;
        let fields = [
            &self.source,
            &self.source_lexeme_key,
            &self.reading_key,
            &self.sense_key,
        ];
        Ok(fields
            .iter()
            .map(|field| format!("{}:{}", field.len(), field))
            .collect::<Vec<_>>()
            .join("|"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_keys_cannot_collide_across_field_boundaries() {
        let a = LexicalIdentity {
            source: "a:b".into(),
            source_lexeme_key: "c".into(),
            reading_key: "d".into(),
            sense_key: "e".into(),
        };
        let b = LexicalIdentity {
            source: "a".into(),
            source_lexeme_key: "b:c".into(),
            reading_key: "d".into(),
            sense_key: "e".into(),
        };
        assert_ne!(a.stable_key().unwrap(), b.stable_key().unwrap());
        let mut empty = a;
        empty.sense_key = " ".into();
        assert_eq!(empty.stable_key(), Err(IdentityError::MissingField));
    }

    #[test]
    fn readings_and_senses_are_distinct() {
        let original = LexicalIdentity {
            source: "hsk".into(),
            source_lexeme_key: "行".into(),
            reading_key: "xing2".into(),
            sense_key: "1".into(),
        };
        let mut polyphone = original.clone();
        polyphone.reading_key = "hang2".into();
        assert_ne!(
            original.stable_key().unwrap(),
            polyphone.stable_key().unwrap()
        );
        let mut other_sense = original.clone();
        other_sense.sense_key = "2".into();
        assert_ne!(
            original.stable_key().unwrap(),
            other_sense.stable_key().unwrap()
        );
    }

    #[test]
    fn invisible_lexical_key_variants_are_rejected() {
        let base = LexicalIdentity {
            source: "dictionary".into(),
            source_lexeme_key: "行".into(),
            reading_key: "xing2".into(),
            sense_key: "sense1".into(),
        };
        for source in [" dictionary", "dictionary ", "dictionary\0"] {
            let mut changed = base.clone();
            changed.source = source.into();
            assert_eq!(changed.stable_key(), Err(IdentityError::MissingField));
        }
    }

    #[test]
    fn control_characters_cannot_create_hidden_lexical_identity_variants() {
        let base = LexicalIdentity {
            source: "dictionary".into(),
            source_lexeme_key: "学习".into(),
            reading_key: "xue2xi2".into(),
            sense_key: "one".into(),
        };
        for invalid in ["one\n", "one\t", "one\r", "one\0"] {
            let mut identity = base.clone();
            identity.sense_key = invalid.into();
            assert_eq!(identity.validate(), Err(IdentityError::MissingField));
        }
    }
}
