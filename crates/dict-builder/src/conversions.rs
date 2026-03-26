use cc_cedict_parser_rs::{OwnedClassifier, OwnedSense, Reference, ReferenceKind};

use crate::{dict, ChineseLexeme, HashToLexemeMap, PartOfSpeech, Shard};

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

impl From<HashToLexemeMap> for dict::HashToLexeme {
    fn from(map: HashToLexemeMap) -> Self {
        let entries = map.0.into_iter()
            .map(|(hash, lex_id)| dict::hash_to_lexeme::Entry {
                hash,
                lexeme_id: lex_id,
            })
            .collect();

        dict::HashToLexeme { entries }
    }
}

impl From<ChineseLexeme> for dict::Lexeme {
    fn from(value: ChineseLexeme) -> Self {
        dict::Lexeme {
            id: value.id,
            traditional: value.traditional.into(),
            simplified: value.simplified.into(),
            pinyin: value.pinyin.into_iter().map(Into::into).collect(),
            senses: value.senses.into_iter().map(Into::into).collect(),
            classifiers: value.classifiers.into_iter().map(Into::into).collect(),
            references: value.references.into_iter().map(Into::into).collect(),
            part_of_speech: value.part_of_speech.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<dict::Lexeme> for ChineseLexeme {
    fn from(value: dict::Lexeme) -> Self {
        ChineseLexeme {
            id: value.id,
            traditional: value.traditional.into(),
            simplified: value.simplified.into(),
            pinyin: value.pinyin.into_iter().map(Into::into).collect(),
            senses: value.senses.into_iter().map(Into::into).collect(),
            classifiers: value.classifiers.into_iter().map(Into::into).collect(),
            references: value.references.into_iter().map(Into::into).collect(),
            part_of_speech: value.part_of_speech.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<OwnedSense> for dict::Sense {
    fn from(value: OwnedSense) -> Self {
        dict::Sense {
            glosses: value.glosses.into_iter().map(Into::into).collect(),
            tags: value.tags.into_iter().map(Into::into).collect(),
            qualifier: value.qualifier.map(Into::into),
        }
    }
}

impl From<dict::Sense> for OwnedSense {
    fn from(value: dict::Sense) -> Self {
        OwnedSense {
            glosses: value.glosses.into_iter().map(Into::into).collect(),
            tags: value.tags.into_iter().map(Into::into).collect(),
            qualifier: value.qualifier.map(Into::into),
        }
    }
}

impl From<OwnedClassifier> for dict::Classifier {
    fn from(value: OwnedClassifier) -> Self {
        dict::Classifier {
            traditional: value.traditional.into(),
            simplified: value.simplified.map(Into::into),
            pinyin: value.pinyin.into(),
        }
    }
}

impl From<dict::Classifier> for OwnedClassifier {
    fn from(value: dict::Classifier) -> Self {
        OwnedClassifier {
            traditional: value.traditional.into(),
            simplified: value.simplified.map(Into::into),
            pinyin: value.pinyin.into(),
        }
    }
}

impl From<Reference> for dict::Reference {
    fn from(value: Reference) -> Self {
        dict::Reference {
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

impl From<dict::Reference> for Reference {
    fn from(value: dict::Reference) -> Self {
        Reference {
            kind: value.kind.into(),
            traditional: value.traditional.into(),
            simplified: value.simplified.map(Into::into),
            pinyin: if value.pinyin.is_empty() { None } else { Some(value.pinyin.into_iter().map(Into::into).collect()) }
        }
    }
}

impl From<ReferenceKind> for dict::ReferenceKind {
    fn from(value: ReferenceKind) -> Self {
        match value {
            ReferenceKind::See => dict::ReferenceKind::See,
            ReferenceKind::Variant => dict::ReferenceKind::Variant,
            ReferenceKind::Abbreviation => dict::ReferenceKind::Abbreviation,
            ReferenceKind::AlsoWritten => dict::ReferenceKind::Alsowritten,
        }
    }
}

impl From<Shard> for dict::Shard {
    fn from(value: Shard) -> Self {
        dict::Shard {
            shard_id: value.id,
            lexemes: value.entries.into_iter().map(|(_, lexeme)| lexeme.into()).collect(),
        }
    }
}

impl From<dict::Shard> for Shard {
    fn from(value: dict::Shard) -> Self {
        Shard {
            id: value.shard_id,
            entries: value.lexemes.into_iter().map(|pr| (pr.id, pr.into())).collect(),
        }
    }
}
