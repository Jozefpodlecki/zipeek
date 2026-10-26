use std::{error::Error, fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartOfSpeech {
    Adjective,
    AdjectiveAdverbial,
    AdjectiveMorpheme,
    AdjectiveNominal,
    NonPredicateAdjective,
    Conjunction,
    Adverb,
    AdverbMorpheme,
    Interjection,
    DirectionalLocality,
    Morpheme,
    Prefix,
    Idiom,
    Abbreviation,
    Suffix,
    FixedExpression,
    Numeral,
    NumericMorpheme,
    Noun,
    NounMorpheme,
    PersonalName,
    PlaceName,
    OrganizationName,
    NominalString,
    OtherProperNoun,
    Onomatopoeia,
    Preposition,
    Classifier,
    Pronoun,
    PronounMorpheme,
    SpaceWord,
    TimeWord,
    TimeWordMorpheme,
    Auxiliary,
    Verb,
    VerbAdverbial,
    VerbMorpheme,
    VerbNominal,
    SymbolPunctuation,
    Unclassified,
    ModalParticle,
    Descriptive,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartOfSpeechError {
    pub value: String,
}
impl fmt::Display for PartOfSpeechError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown part of speech: {:?}", self.value)
    }
}
impl Error for PartOfSpeechError {}
impl PartOfSpeech {
    pub fn from_source_code(code: &str) -> Result<Self, PartOfSpeechError> {
        match code.to_ascii_lowercase().as_str() {
            "a" => Ok(Self::Adjective),
            "ad" => Ok(Self::AdjectiveAdverbial),
            "ag" => Ok(Self::AdjectiveMorpheme),
            "an" => Ok(Self::AdjectiveNominal),
            "b" => Ok(Self::NonPredicateAdjective),
            "c" => Ok(Self::Conjunction),
            "d" => Ok(Self::Adverb),
            "dg" => Ok(Self::AdverbMorpheme),
            "e" => Ok(Self::Interjection),
            "f" => Ok(Self::DirectionalLocality),
            "g" => Ok(Self::Morpheme),
            "h" => Ok(Self::Prefix),
            "i" => Ok(Self::Idiom),
            "j" => Ok(Self::Abbreviation),
            "k" => Ok(Self::Suffix),
            "l" => Ok(Self::FixedExpression),
            "m" => Ok(Self::Numeral),
            "mg" => Ok(Self::NumericMorpheme),
            "n" => Ok(Self::Noun),
            "ng" => Ok(Self::NounMorpheme),
            "nr" => Ok(Self::PersonalName),
            "ns" => Ok(Self::PlaceName),
            "nt" => Ok(Self::OrganizationName),
            "nx" => Ok(Self::NominalString),
            "nz" => Ok(Self::OtherProperNoun),
            "o" => Ok(Self::Onomatopoeia),
            "p" => Ok(Self::Preposition),
            "q" => Ok(Self::Classifier),
            "r" => Ok(Self::Pronoun),
            "rg" => Ok(Self::PronounMorpheme),
            "s" => Ok(Self::SpaceWord),
            "t" => Ok(Self::TimeWord),
            "tg" => Ok(Self::TimeWordMorpheme),
            "u" => Ok(Self::Auxiliary),
            "v" => Ok(Self::Verb),
            "vd" => Ok(Self::VerbAdverbial),
            "vg" => Ok(Self::VerbMorpheme),
            "vn" => Ok(Self::VerbNominal),
            "w" => Ok(Self::SymbolPunctuation),
            "x" => Ok(Self::Unclassified),
            "y" => Ok(Self::ModalParticle),
            "z" => Ok(Self::Descriptive),
            _ => Err(PartOfSpeechError {
                value: code.to_owned(),
            }),
        }
    }
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Adjective => "adjective",
            Self::AdjectiveAdverbial => "adjective_adverbial",
            Self::AdjectiveMorpheme => "adjective_morpheme",
            Self::AdjectiveNominal => "adjective_nominal",
            Self::NonPredicateAdjective => "non_predicate_adjective",
            Self::Conjunction => "conjunction",
            Self::Adverb => "adverb",
            Self::AdverbMorpheme => "adverb_morpheme",
            Self::Interjection => "interjection",
            Self::DirectionalLocality => "directional_locality",
            Self::Morpheme => "morpheme",
            Self::Prefix => "prefix",
            Self::Idiom => "idiom",
            Self::Abbreviation => "abbreviation",
            Self::Suffix => "suffix",
            Self::FixedExpression => "fixed_expression",
            Self::Numeral => "numeral",
            Self::NumericMorpheme => "numeric_morpheme",
            Self::Noun => "noun",
            Self::NounMorpheme => "noun_morpheme",
            Self::PersonalName => "personal_name",
            Self::PlaceName => "place_name",
            Self::OrganizationName => "organization_name",
            Self::NominalString => "nominal_string",
            Self::OtherProperNoun => "other_proper_noun",
            Self::Onomatopoeia => "onomatopoeia",
            Self::Preposition => "preposition",
            Self::Classifier => "classifier",
            Self::Pronoun => "pronoun",
            Self::PronounMorpheme => "pronoun_morpheme",
            Self::SpaceWord => "space_word",
            Self::TimeWord => "time_word",
            Self::TimeWordMorpheme => "time_word_morpheme",
            Self::Auxiliary => "auxiliary",
            Self::Verb => "verb",
            Self::VerbAdverbial => "verb_adverbial",
            Self::VerbMorpheme => "verb_morpheme",
            Self::VerbNominal => "verb_nominal",
            Self::SymbolPunctuation => "symbol_punctuation",
            Self::Unclassified => "unclassified",
            Self::ModalParticle => "modal_particle",
            Self::Descriptive => "descriptive",
        }
    }
    pub fn from_source_codes<'a>(
        codes: impl IntoIterator<Item = &'a str>,
    ) -> Result<Vec<Self>, PartOfSpeechError> {
        let mut values = codes
            .into_iter()
            .map(Self::from_source_code)
            .collect::<Result<Vec<_>, _>>()?;
        values.sort_unstable_by_key(|value| value.as_str());
        values.dedup();
        Ok(values)
    }
}
impl FromStr for PartOfSpeech {
    type Err = PartOfSpeechError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "adjective" => Ok(Self::Adjective),
            "adjective_adverbial" => Ok(Self::AdjectiveAdverbial),
            "adjective_morpheme" => Ok(Self::AdjectiveMorpheme),
            "adjective_nominal" => Ok(Self::AdjectiveNominal),
            "non_predicate_adjective" => Ok(Self::NonPredicateAdjective),
            "conjunction" => Ok(Self::Conjunction),
            "adverb" => Ok(Self::Adverb),
            "adverb_morpheme" => Ok(Self::AdverbMorpheme),
            "interjection" => Ok(Self::Interjection),
            "directional_locality" => Ok(Self::DirectionalLocality),
            "morpheme" => Ok(Self::Morpheme),
            "prefix" => Ok(Self::Prefix),
            "idiom" => Ok(Self::Idiom),
            "abbreviation" => Ok(Self::Abbreviation),
            "suffix" => Ok(Self::Suffix),
            "fixed_expression" => Ok(Self::FixedExpression),
            "numeral" => Ok(Self::Numeral),
            "numeric_morpheme" => Ok(Self::NumericMorpheme),
            "noun" => Ok(Self::Noun),
            "noun_morpheme" => Ok(Self::NounMorpheme),
            "personal_name" => Ok(Self::PersonalName),
            "place_name" => Ok(Self::PlaceName),
            "organization_name" => Ok(Self::OrganizationName),
            "nominal_string" => Ok(Self::NominalString),
            "other_proper_noun" => Ok(Self::OtherProperNoun),
            "onomatopoeia" => Ok(Self::Onomatopoeia),
            "preposition" => Ok(Self::Preposition),
            "classifier" => Ok(Self::Classifier),
            "pronoun" => Ok(Self::Pronoun),
            "pronoun_morpheme" => Ok(Self::PronounMorpheme),
            "space_word" => Ok(Self::SpaceWord),
            "time_word" => Ok(Self::TimeWord),
            "time_word_morpheme" => Ok(Self::TimeWordMorpheme),
            "auxiliary" => Ok(Self::Auxiliary),
            "verb" => Ok(Self::Verb),
            "verb_adverbial" => Ok(Self::VerbAdverbial),
            "verb_morpheme" => Ok(Self::VerbMorpheme),
            "verb_nominal" => Ok(Self::VerbNominal),
            "symbol_punctuation" => Ok(Self::SymbolPunctuation),
            "unclassified" => Ok(Self::Unclassified),
            "modal_particle" => Ok(Self::ModalParticle),
            "descriptive" => Ok(Self::Descriptive),
            _ => Err(PartOfSpeechError {
                value: value.to_owned(),
            }),
        }
    }
}
impl fmt::Display for PartOfSpeech {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_42_codes_accept_case_aliases_and_roundtrip_canonical_tokens() {
        let mut tokens = std::collections::BTreeSet::new();
        for line in include_str!("../tests/fixtures/part_of_speech.tsv").lines() {
            let (code, token) = line.split_once('\t').unwrap();
            let pos = PartOfSpeech::from_source_code(code).unwrap();
            assert_eq!(
                PartOfSpeech::from_source_code(&code.to_ascii_uppercase()),
                Ok(pos)
            );
            assert_eq!(pos.as_str(), token);
            assert_eq!(token.parse::<PartOfSpeech>(), Ok(pos));
            assert!(tokens.insert(token));
        }
        assert_eq!(tokens.len(), 42);
    }
    #[test]
    fn unknown_source_code_rejects_the_whole_list() {
        for code in ["cc", "mq", "qt", "qv", "", " n", "noun"] {
            let error = PartOfSpeech::from_source_codes(["n", code, "v"]).unwrap_err();
            assert_eq!(error.value, code);
        }
    }
    #[test]
    fn missing_and_repeated_codes_have_deterministic_results() {
        assert!(PartOfSpeech::from_source_codes([]).unwrap().is_empty());
        assert_eq!(
            PartOfSpeech::from_source_codes(["V", "n", "v"]).unwrap(),
            vec![PartOfSpeech::Noun, PartOfSpeech::Verb]
        );
        assert!("NOUN".parse::<PartOfSpeech>().is_err());
        assert_eq!(
            PartOfSpeech::from_source_code("Mg"),
            Ok(PartOfSpeech::NumericMorpheme)
        );
        assert_eq!(
            PartOfSpeech::from_source_code("Rg"),
            Ok(PartOfSpeech::PronounMorpheme)
        );
    }
}
