use cc_cedict_parser_rs::{OwnedClassifier, OwnedSense, Reference, ReferenceKind, Sense};

use crate::*;

impl From<PartOfSpeech> for i32 {
    fn from(value: PartOfSpeech) -> Self {
        value as i32
    }
}

impl From<i32> for PartOfSpeech {
    fn from(value: i32) -> Self {
        match value {
            0 => PartOfSpeech::Noun,
            1 => PartOfSpeech::Verb,
            2 => PartOfSpeech::Adjective,
            3 => PartOfSpeech::Adverb,
            4 => PartOfSpeech::Pronoun,
            5 => PartOfSpeech::Numeral,
            6 => PartOfSpeech::MeasureWord,
            7 => PartOfSpeech::Conjunction,
            8 => PartOfSpeech::Preposition,
            9 => PartOfSpeech::Particle,
            10 => PartOfSpeech::Interjection,
            _ => PartOfSpeech::Unknown
        }
    }
}

impl From<LexicalVariant> for storage::LexicalVariant {
    fn from(value: LexicalVariant) -> Self {
        storage::LexicalVariant {
            traditional: value.traditional.into(),
            pinyin: value.pinyin.into_iter().map(Into::into).collect(),
            senses: value.senses.into_iter().map(Into::into).collect(),
            classifiers: value.classifiers.into_iter().map(Into::into).collect(),
            references: value.references.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<storage::LexicalVariant> for LexicalVariant {
    fn from(value: storage::LexicalVariant) -> Self {
        LexicalVariant {
            traditional: value.traditional.into(),
            pinyin: value.pinyin.into_iter().map(Into::into).collect(),
            senses: value.senses.into_iter().map(Into::into).collect(),
            classifiers: value.classifiers.into_iter().map(Into::into).collect(),
            references: value.references.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<HashToLexemeMap> for storage::HashToLexeme {
    fn from(map: HashToLexemeMap) -> Self {
        let entries = map.0.into_iter()
            .map(|(hash, lex_id)| storage::hash_to_lexeme::Entry {
                hash,
                lexeme_id: lex_id,
            })
            .collect();

        storage::HashToLexeme { entries }
    }
}

impl From<ChineseLexeme> for storage::Lexeme {
    fn from(value: ChineseLexeme) -> Self {
        storage::Lexeme {
            id: value.id,
            simplified: value.simplified.into(),
            variants: vec![],
            // traditional: value.traditional.into(),
            // pinyin: value.pinyin.into_iter().map(Into::into).collect(),
            // senses: value.senses.into_iter().map(Into::into).collect(),
            // classifiers: value.classifiers.into_iter().map(Into::into).collect(),
            // references: value.references.into_iter().map(Into::into).collect(),
            part_of_speech: value.part_of_speech.into_iter().map(Into::into).collect(),
            standards: value.standards.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<storage::Lexeme> for ChineseLexeme {
    fn from(value: storage::Lexeme) -> Self {
        ChineseLexeme {
            id: value.id,
            // traditional: value.traditional.into(),
            simplified: value.simplified.into(),
            variants: value.variants.into_iter().map(Into::into).collect(),
            // pinyin: value.pinyin.into_iter().map(Into::into).collect(),
            // senses: value.senses.into_iter().map(Into::into).collect(),
            // classifiers: value.classifiers.into_iter().map(Into::into).collect(),
            // references: value.references.into_iter().map(Into::into).collect(),
            part_of_speech: value.part_of_speech.into_iter().map(Into::into).collect(),
            standards: value.standards.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ChineseSense> for storage::Sense {
    fn from(value: ChineseSense) -> Self {
        storage::Sense {
            glosses: value.glosses.into_iter().map(Into::into).collect(),
            tags: value.tags.into_iter().map(Into::into).collect(),
            qualifier: value.qualifier.map(Into::into),
            part_of_speech: value.part_of_speech.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<storage::Sense> for ChineseSense {
    fn from(value: storage::Sense) -> Self {
        ChineseSense {
            glosses: value.glosses.into_iter().map(Into::into).collect(),
            tags: value.tags.into_iter().map(Into::into).collect(),
            qualifier: value.qualifier.map(Into::into),
            part_of_speech: value.part_of_speech.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> From<Sense<'a>> for ChineseSense {
    fn from(value: Sense<'a>) -> Self {
        value.to_owned().into()
    }
}

impl From<OwnedSense> for ChineseSense {
    fn from(value: OwnedSense) -> Self {
        ChineseSense {
            glosses: value.glosses.into_iter().map(Into::into).collect(),
            tags: value.tags.into_iter().map(Into::into).collect(),
            qualifier: value.qualifier.map(Into::into),
            part_of_speech: vec![]
        }
    }
}

impl From<OwnedClassifier> for storage::Classifier {
    fn from(value: OwnedClassifier) -> Self {
        storage::Classifier {
            traditional: value.traditional.into(),
            simplified: value.simplified.map(Into::into),
            pinyin: value.pinyin.into(),
        }
    }
}

impl From<storage::Classifier> for OwnedClassifier {
    fn from(value: storage::Classifier) -> Self {
        OwnedClassifier {
            traditional: value.traditional.into(),
            simplified: value.simplified.map(Into::into),
            pinyin: value.pinyin.into(),
        }
    }
}

impl From<Reference> for storage::Reference {
    fn from(value: Reference) -> Self {
        storage::Reference {
            kind: value.kind as i32, // prost enum
            traditional: value.traditional.into(),
            simplified: value.simplified.map(Into::into),
            pinyin: value.pinyin
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<storage::Reference> for Reference {
    fn from(value: storage::Reference) -> Self {
        Reference {
            kind: value.kind.into(),
            traditional: value.traditional.into(),
            simplified: value.simplified.map(Into::into),
            pinyin: if value.pinyin.is_empty() { None } else { Some(value.pinyin.into_iter().map(Into::into).collect()) }
        }
    }
}

impl From<storage::ReferenceStandard> for ReferenceStandard {
    fn from(value: storage::ReferenceStandard) -> Self {
        ReferenceStandard {
            name: value.name.into(),
            kind: value.kind.into(),
            value: value.value.into(),
        }
    }
}

impl From<ReferenceStandard> for storage::ReferenceStandard {
    fn from(value: ReferenceStandard) -> Self {
        storage::ReferenceStandard {
            name: value.name.into(),
            kind: value.kind.into(),
            value: value.value.into(),
        }
    }
}

impl From<ReferenceKind> for storage::ReferenceKind {
    fn from(value: ReferenceKind) -> Self {
        match value {
            ReferenceKind::See => storage::ReferenceKind::See,
            ReferenceKind::Variant => storage::ReferenceKind::Variant,
            ReferenceKind::Abbreviation => storage::ReferenceKind::Abbreviation,
            ReferenceKind::AlsoWritten => storage::ReferenceKind::Alsowritten,
        }
    }
}

impl From<Shard> for storage::Shard {
    fn from(value: Shard) -> Self {
        storage::Shard {
            shard_id: value.id,
            lexemes: value.entries.into_iter().map(|(_, lexeme)| lexeme.into()).collect(),
        }
    }
}

impl From<storage::Shard> for Shard {
    fn from(value: storage::Shard) -> Self {
        Shard {
            id: value.shard_id,
            entries: value.lexemes.into_iter().map(|pr| (pr.id, pr.into())).collect(),
        }
    }
}
