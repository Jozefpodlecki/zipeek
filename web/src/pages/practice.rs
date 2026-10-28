use yew::prelude::*;
use yew_router::prelude::use_location;
use zipeek_core::{practice::PracticeMode, round::RoundContent};

use crate::{
    components::{Loader, RoutePlaceholder},
    hooks::use_prepared_practice,
    reducers::practice::PracticeStatus,
};

pub use zipeek_core::ExerciseVariant as Variant;

#[derive(Properties, PartialEq)]
pub struct PracticeProps {
    pub variant: Variant,
}

#[function_component(Practice)]
pub fn practice(props: &PracticeProps) -> Html {
    let location = use_location();
    let parsed = location
        .as_ref()
        .map(crate::services::practice_url::read_query)
        .unwrap_or_default();
    let (status, retry) = use_prepared_practice(props.variant, parsed.clone());
    let query = match &status {
        PracticeStatus::Ready(prepared) => Some(&prepared.query),
        _ => None,
    };
    let warnings = query.map_or_else(|| parsed.warnings.clone(), |query| query.warnings.clone());
    use_effect_with(warnings, |warnings| {
        if cfg!(debug_assertions) {
            for warning in warnings {
                log::warn!("Invalid practice {}: {}", warning.field, warning.value);
            }
        }
    });
    let mode = match parsed.mode {
        PracticeMode::Endless => "endless",
        PracticeMode::Fixed(_) => "fixed",
    };
    let (load_state, content) = match &status {
        PracticeStatus::Loading => ("loading", html! { <Loader /> }),
        PracticeStatus::Ready(_) => (
            "ready",
            html! { <RoutePlaceholder title="Practice" message="Practice coming soon." /> },
        ),
        PracticeStatus::Failed(_) => (
            "failed",
            html! { <RoutePlaceholder title="Practice unavailable" message="This round could not be prepared. Try again or choose another level."><button type="button" class="mt-6 rounded px-3 py-2 text-ink hover:bg-raised" data-practice-retry="" onclick={Callback::from(move |_| retry.emit(()))}>{"Retry"}</button></RoutePlaceholder> },
        ),
    };
    let debug_round = match &status {
        PracticeStatus::Ready(prepared) if cfg!(debug_assertions) => {
            let cards = match &prepared.round.content {
                RoundContent::Matching { left, right } => {
                    left.iter().chain(right.iter()).collect::<Vec<_>>()
                }
                RoundContent::Mcq {
                    prompt, options, ..
                } => std::iter::once(prompt).chain(options.iter()).collect(),
            };
            serde_json::to_string(
                &cards
                    .iter()
                    .map(|card| (card.word_id.as_i32(), card.label.as_str()))
                    .collect::<Vec<_>>(),
            )
            .map_err(|error| log::error!("Could not encode round diagnostics: {error}"))
            .ok()
        }
        _ => None,
    };
    html! {
        <div data-practice-variant={props.variant.seed_token()}
            data-practice-state={load_state}
            data-practice-mode={mode}
            data-practice-level={query.map_or(parsed.filter.level.as_i32(), |query| query.filter.level.as_i32()).to_string()}
            data-practice-offset={query.map_or(parsed.offset, |query| query.offset).to_string()}
            data-practice-pos={parsed.filter.pos_tokens()}
            data-practice-seed={query.and_then(|query| query.seed.original_id()).map(|id| id.to_string())}
            data-practice-exclusions={query.map(|query| query.exclusions.iter().map(ToString::to_string).collect::<Vec<_>>().join(","))}
            data-practice-round={debug_round}>
            {content}
        </div>
    }
}
