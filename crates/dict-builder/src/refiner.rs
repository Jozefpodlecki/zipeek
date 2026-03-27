use std::fs;

use cc_cedict_parser_rs::*;
use hashbrown::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use zipseek_core::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefinedLexeme {
    id: u64,
    friendly_name: Box<str>,
    action: RefinedLexemeAction
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub enum RefinedLexemeAction {
    #[default]
    None,
    Update(Vec<RefinedLexemeActionUpdate>),
    Replace(RefinedLexemeActionReplace)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RefinedLexemeActionUpdate {
    PartOfSpeech(Vec<PartOfSpeech>),
    ReferenceStandard(Vec<ReferenceStandard>),
    Senses(Vec<ChineseSense>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefinedLexemeActionReplace {
    pub traditional: Box<str>,
    pub simplified: Box<str>,
    pub pinyin: Vec<Box<str>>,
    pub senses: Vec<ChineseSense>,
    pub classifiers: Vec<OwnedClassifier>,
    pub references: Vec<Reference>,
    pub part_of_speech: Vec<PartOfSpeech>,
    pub standards: Vec<ReferenceStandard>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefinedLexemes(pub HashMap<u64, RefinedLexeme>);

pub struct Refiner {
    lexemes: RefinedLexemes
}

impl Refiner {

    pub fn generate_blank<'a, I>(lexemes: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a ChineseLexeme>,
    {
        let map: HashMap<u64, RefinedLexeme> = lexemes
            .into_iter()
            .map(|lexeme| {
                (
                    lexeme.id,
                    RefinedLexeme {
                        id: lexeme.id,
                        friendly_name: lexeme.simplified.clone(),
                        action: Default::default(),
                    },
                )
            })
            .collect();

        let path = r#"C:\repos\zipeek\crates\dict-builder\src\refined.json"#;
        let buffer = serde_json::to_vec_pretty(&map)?;
        fs::write(path, buffer)?;

        Ok(())
    }

    pub fn new() -> Result<Self> {
        let path = r#"C:\repos\zipeek\crates\dict-builder\src\refined.json"#;
        let buffer = fs::read(path)?;
        let lexemes = serde_json::from_slice(&buffer)?;

        Ok(Self {
            lexemes
        })
    }

    pub fn refine(&mut self, lexeme: &mut ChineseLexeme) {
        if let Some(entry) = self.lexemes.0.remove(&lexeme.id) {
            match entry.action {
                RefinedLexemeAction::None => {},
                RefinedLexemeAction::Update(actions) => {
                    for action in actions {
                        match action {
                            RefinedLexemeActionUpdate::PartOfSpeech(value) => {
                                lexeme.part_of_speech = value;
                            },
                            RefinedLexemeActionUpdate::ReferenceStandard(value) => {
                                lexeme.standards = value;
                            },
                            RefinedLexemeActionUpdate::Senses(value) => {
                                // lexeme.senses = value;
                            }
                        }
                    }
                },
                RefinedLexemeAction::Replace(action) => {
                    // *lexeme = ChineseLexeme {
                    //     id: lexeme.id,
                    //     traditional: action.traditional,
                    //     simplified: action.simplified,
                    //     pinyin: action.pinyin,
                    //     senses: action.senses,
                    //     classifiers: action.classifiers,
                    //     references: action.references,
                    //     part_of_speech: action.part_of_speech,
                    //     standards: action.standards,
                    // }
                },
            }
        }
    }
}


mod tests {
    use anyhow::Result;
    use super::*;

    #[test]
    fn should_() {
        let json = serde_json::to_string_pretty(&RefinedLexemeAction::Update(vec![RefinedLexemeActionUpdate::PartOfSpeech(vec![])])).unwrap();
        println!("{json}");

        let json = serde_json::to_string_pretty(&RefinedLexemeAction::Update(vec![RefinedLexemeActionUpdate::ReferenceStandard(vec![
            ReferenceStandard { name: "".into(), kind: "".into(), value: "".into() }
        ])])).unwrap();
        println!("{json}");

        let json = serde_json::to_string_pretty(&RefinedLexemeAction::Update(vec![RefinedLexemeActionUpdate::Senses(vec![

        ])])).unwrap();
        println!("{json}");

         let json = serde_json::to_string_pretty(&RefinedLexemeAction::Replace(RefinedLexemeActionReplace {
            traditional: "".into(),
            simplified: "".into(),
            pinyin: vec![],
            senses: vec![],
            classifiers: vec![],
            references: vec![],
            part_of_speech: vec![],
            standards: vec![],
        })).unwrap();
        println!("{json}");
    }
   
}