use crate::{
    reducers::browse::{BrowseAction, BrowseInput, BrowseState},
    services::database::BrowseField,
};
use yew::prelude::*;
use zipeek_core::{HskLevel, HskSystem};

pub(super) fn selected_hsk(value: &str) -> Option<HskLevel> {
    let (system, number) = value.split_once(':')?;
    let system = HskSystem::from_code(system)?;
    let number = number.parse::<i32>().ok()?;
    HskLevel::try_from((system, number)).ok()
}

pub(super) fn selected_field(value: &str) -> BrowseField {
    match value {
        "hanzi" => BrowseField::Hanzi,
        "pinyin" => BrowseField::Pinyin,
        "english" => BrowseField::English,
        "english_fts" => BrowseField::EnglishFts,
        _ => BrowseField::All,
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct FilterChoice {
    pub(super) field: &'static str,
    pub(super) value: String,
    pub(super) label: String,
}

pub(super) fn filter_choices() -> Vec<FilterChoice> {
    let mut choices = Vec::new();
    for (value, label) in [
        ("all", "All fields"),
        ("hanzi", "Search Hanzi"),
        ("pinyin", "Search Pinyin"),
        ("english", "Search English meanings"),
        ("english_fts", "English relevance (FTS / BM25)"),
    ] {
        choices.push(FilterChoice {
            field: "field",
            value: value.to_owned(),
            label: label.to_owned(),
        });
    }
    for system in HskSystem::ALL {
        let prefix = system.code();
        for level in system.levels() {
            choices.push(FilterChoice {
                field: "hsk",
                value: format!("{prefix}:{}", level.as_i32()),
                label: level.badge_label(),
            });
        }
    }
    for (value, label) in [
        ("noun", "Noun"),
        ("verb", "Verb"),
        ("adjective", "Adjective"),
        ("adverb", "Adverb"),
        ("pronoun", "Pronoun"),
        ("classifier", "Classifier"),
        ("preposition", "Preposition"),
        ("conjunction", "Conjunction"),
    ] {
        choices.push(FilterChoice {
            field: "pos",
            value: value.to_owned(),
            label: format!("Part of speech: {label}"),
        });
    }
    for (value, label) in [
        ("travel", "Travel"),
        ("transport", "Transport"),
        ("food", "Food and drink"),
        ("shopping", "Shopping"),
        ("family", "Family"),
        ("education", "Education"),
        ("work", "Work"),
        ("health", "Health"),
        ("nature", "Nature"),
        ("time", "Time"),
        ("home", "Home"),
        ("communication", "Communication"),
    ] {
        choices.push(FilterChoice {
            field: "category",
            value: value.to_owned(),
            label: format!("Topic: {label}"),
        });
    }
    choices
}

pub(super) fn apply_choice(state: &UseReducerHandle<BrowseState>, choice: &FilterChoice) {
    let action = match choice.field {
        "field" => BrowseInput::Field(choice.value.clone()),
        "hsk" => BrowseInput::Hsk(choice.value.clone()),
        "pos" => BrowseInput::PartOfSpeech(choice.value.clone()),
        "category" => BrowseInput::Category(choice.value.clone()),
        _ => return,
    };
    state.dispatch(BrowseAction::Input(action));
}
