use std::{
    cell::Cell,
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use yew::prelude::*;
use yew_router::prelude::use_location;
use zipeek_core::{Word, WordId};

use crate::{
    contexts::database::Dictionary,
    hooks::{use_dictionary, use_word_sets},
    services::word_sets::UserWordSet,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum GameSource {
    Starter,
    Hsk { standard: u8, level: u8 },
    WordSet { id: String, word_ids: Vec<i32> },
}

impl GameSource {
    fn label(&self) -> String {
        match self {
            Self::Starter => "Starter questions".to_owned(),
            Self::Hsk { standard, level } => format!("HSK {standard}.0 · Level {level}"),
            Self::WordSet { .. } => "Selected Word Set".to_owned(),
        }
    }

    fn from_query(query: &HashMap<String, String>, sets: &[UserWordSet]) -> Result<Self, String> {
        match query.get("source").map(String::as_str).unwrap_or("starter") {
            "starter" => Ok(Self::Starter),
            "hsk" => {
                let standard = query
                    .get("standard")
                    .map(String::as_str)
                    .unwrap_or("3")
                    .parse::<u8>()
                    .map_err(|_| "Invalid HSK standard")?;
                let level = query
                    .get("level")
                    .map(String::as_str)
                    .unwrap_or("1")
                    .parse::<u8>()
                    .map_err(|_| "Invalid HSK level")?;
                if !matches!(standard, 2 | 3)
                    || level == 0
                    || level > (if standard == 2 { 6 } else { 7 })
                {
                    return Err("Choose an available HSK standard and level".to_owned());
                }
                Ok(Self::Hsk { standard, level })
            }
            "set" => {
                let id = query.get("id").ok_or("Select a Word Set")?;
                let set = sets
                    .iter()
                    .find(|set| set.id.as_str() == id.as_str())
                    .ok_or("Word Set no longer exists")?;
                Ok(Self::WordSet {
                    id: id.clone(),
                    word_ids: set.word_ids.clone(),
                })
            }
            _ => Err("Unknown vocabulary source".to_owned()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct LoadedWords {
    pub source: GameSource,
    pub result: Result<Vec<Word>, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum GameVocabulary {
    Starter,
    Loading,
    Ready(LoadedWords),
    Error(String),
}

impl GameVocabulary {
    pub fn label(&self) -> String {
        match self {
            Self::Starter => GameSource::Starter.label(),
            Self::Ready(data) => data.source.label(),
            Self::Loading => "Loading vocabulary".to_owned(),
            Self::Error(_) => "Invalid vocabulary source".to_owned(),
        }
    }
}

#[hook]
pub(super) fn use_game_vocabulary() -> (GameVocabulary, Vec<UserWordSet>) {
    let dictionary = use_dictionary();
    let sets = use_word_sets();
    let available_sets = sets.state.sets.as_ref().ok().cloned().unwrap_or_default();
    let location = use_location();
    let query = location
        .as_ref()
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .unwrap_or_default();
    let source = GameSource::from_query(&query, &available_sets);
    let loaded = use_state(|| None::<LoadedWords>);

    {
        let dictionary = dictionary.clone();
        let loaded = loaded.clone();
        let key = source.clone().ok();
        use_effect_with(key, move |source| {
            let live = Rc::new(Cell::new(true));
            let pending = live.clone();
            loaded.set(None);
            if let Some(source) = source.clone() {
                if source != GameSource::Starter {
                    wasm_bindgen_futures::spawn_local(async move {
                        let result = load_words(&dictionary, &source).await;
                        if pending.get() {
                            loaded.set(Some(LoadedWords { source, result }));
                        }
                    });
                }
            }
            move || live.set(false)
        });
    }
    let state = match source {
        Ok(GameSource::Starter) => GameVocabulary::Starter,
        Err(message) => GameVocabulary::Error(message),
        Ok(expected) => match &*loaded {
            Some(data) if data.source == expected => GameVocabulary::Ready(data.clone()),
            _ => GameVocabulary::Loading,
        },
    };
    (state, available_sets)
}

async fn load_words(dictionary: &Dictionary, source: &GameSource) -> Result<Vec<Word>, String> {
    let database = dictionary.database().map_err(|error| error.to_string())?;
    match source {
        GameSource::Starter => Ok(Vec::new()),
        GameSource::Hsk { standard, level } => database
            .game_words_for_hsk(*standard, *level)
            .await
            .map_err(|error| error.to_string()),
        GameSource::WordSet { word_ids, .. } => {
            if word_ids.is_empty() {
                return Ok(Vec::new());
            }
            let ids = word_ids
                .iter()
                .map(|id| WordId::try_from(i64::from(*id)).map_err(|error| error.to_string()))
                .collect::<Result<BTreeSet<_>, _>>()?;
            database
                .words_by_ids(&ids)
                .await
                .map_err(|error| error.to_string())
        }
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct VocabularyFallbackProps {
    pub state: GameVocabulary,
}

#[function_component(VocabularyFallback)]
pub(super) fn vocabulary_fallback(props: &VocabularyFallbackProps) -> Html {
    match &props.state {
        GameVocabulary::Loading => html! { <p role="status">{"Loading game vocabulary…"}</p> },
        GameVocabulary::Error(error) => {
            html! { <p role="alert">{format!("Could not load vocabulary: {error}")}</p> }
        }
        GameVocabulary::Ready(data) => match &data.result {
            Err(error) => {
                html! { <p role="alert">{format!("Could not load vocabulary: {error}")}</p> }
            }
            Ok(_) => Html::default(),
        },
        _ => Html::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(source: &str, standard: &str, level: &str) -> HashMap<String, String> {
        HashMap::from([
            ("source".to_owned(), source.to_owned()),
            ("standard".to_owned(), standard.to_owned()),
            ("level".to_owned(), level.to_owned()),
        ])
    }

    #[test]
    fn starter_mode_is_explicit_and_remains_the_default() {
        assert_eq!(
            GameSource::from_query(&HashMap::new(), &[]),
            Ok(GameSource::Starter)
        );
        assert_eq!(
            GameSource::from_query(&query("starter", "invalid", "invalid"), &[]),
            Ok(GameSource::Starter)
        );
    }

    #[test]
    fn hsk_sources_reject_invalid_levels_without_fallbacks() {
        assert_eq!(
            GameSource::from_query(&query("hsk", "2", "6"), &[]),
            Ok(GameSource::Hsk {
                standard: 2,
                level: 6
            })
        );
        assert_eq!(
            GameSource::from_query(&query("hsk", "3", "7"), &[]),
            Ok(GameSource::Hsk {
                standard: 3,
                level: 7
            })
        );
        assert_eq!(
            GameSource::Hsk {
                standard: 3,
                level: 7
            }
            .label(),
            "HSK 3.0 · Level 7"
        );
        assert!(GameSource::from_query(&query("hsk", "2", "7"), &[]).is_err());
        assert!(GameSource::from_query(&query("hsk", "3", "0"), &[]).is_err());
        assert!(GameSource::from_query(&query("hsk", "6", "1"), &[]).is_err());
    }

    #[test]
    fn deleted_and_unknown_word_sets_do_not_start_another_source() {
        let query = HashMap::from([
            ("source".to_owned(), "set".to_owned()),
            ("id".to_owned(), "missing".to_owned()),
        ]);
        assert!(GameSource::from_query(&query, &[]).is_err());
        assert!(
            GameSource::from_query(
                &HashMap::from([("source".to_owned(), "other".to_owned())]),
                &[]
            )
            .is_err()
        );
    }
}
