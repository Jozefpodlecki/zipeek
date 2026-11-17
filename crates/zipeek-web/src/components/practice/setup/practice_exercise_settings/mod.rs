mod answer;
mod hunt;
mod listening;
mod matching;
mod translation;
mod transformation;
mod writing;

use yew::prelude::*;

use crate::reducers::practice_setup_flow::{PracticeSetupAction, PracticeSetupFlow};

use self::{
    answer::AnswerSettings, hunt::HuntSettings, listening::ListeningSettings, matching::MatchingSettings,
    translation::TranslationSettings, transformation::TransformationSettings, writing::WritingSettings,
};

#[derive(Properties, PartialEq)]
pub(crate) struct PracticeExerciseSettingsProps {
    pub flow: UseReducerHandle<PracticeSetupFlow>,
}

#[function_component(PracticeExerciseSettings)]
pub(crate) fn practice_exercise_settings(props: &PracticeExerciseSettingsProps) -> Html {
    let flow = &props.flow;
    let description = match flow.active_tab {
        "listening" if flow.draft.has_exercise("listening-choice") => Some((
            "Listening practice",
            "Play browser-generated Mandarin and select the matching Hanzi. Requires a saved Word Set with three distinct words.",
        )),
        "sentence-builder" if flow.draft.has_exercise("sentence-builder") => Some((
            "Sentence Builder",
            "Reorder locally authored Chinese phrase tiles. Multiple natural orders may be accepted. This activity does not measure independent Hanzi typing.",
        )),
        "spot-mistake" if flow.draft.has_exercise("spot-mistake") => Some((
            "Spot the Mistake",
            "Select the incorrect phrase, then type a replacement. Identification and repair are reviewed separately.",
        )),
        "image" if flow.draft.has_exercise("image-vocabulary") => Some((
            "Picture vocabulary",
            "Generate an image from a saved Word Set and approve it before answering. Requires the configured image server.",
        )),
        _ => None,
    };
    let on_action = {
        let flow = flow.clone();
        Callback::from(move |action| flow.dispatch(action))
    };
    match flow.active_tab {
        "matching" | "multiple-choice" => html! {
            <MatchingSettings draft={flow.draft.clone()} family={flow.active_tab}
                pair_count={flow.exercise_options.matching_pairs} on_action={on_action} />
        },
        "character-builder" | "type-pinyin" => {
            flow.exercise_options.for_exercise(flow.active_tab)
                .map(|options| html! { <AnswerSettings options={options} on_action={on_action} /> })
                .unwrap_or_default()
        }
        "hunt" if flow.draft.has_exercise("character-hunt") => html! {
            <HuntSettings selected={flow.draft.hunt_difficulty} on_action={on_action} />
        },
        "translation" if flow.draft.has_exercise("sentence-translation") => html! {
            <TranslationSettings direction={flow.draft.translation.direction} on_action={on_action} />
        },
        "sentence-transformation" if flow.draft.has_exercise("sentence-transformation") => html! {
            <TransformationSettings options={flow.exercise_options.sentence_transformation} on_action={on_action} />
        },
        "listening" if flow.draft.has_exercise("listening-choice") => html! {
            <ListeningSettings options={flow.exercise_options.listening_choice} on_action={on_action} />
        },
        "write-hanzi" if flow.draft.has_exercise("write-hanzi") => html! {
            <WritingSettings options={flow.exercise_options.write_hanzi} on_action={on_action} />
        },
        "speaking" if flow.draft.has_exercise("speaking") => html! {
            <section role="tabpanel" data-speaking-setup="" class="rounded-2xl border border-line bg-surface p-6">
                <h3 class="text-lg font-semibold">{"Pronunciation"}</h3>
                <p class="mt-2 text-sm text-muted">{"Record and replay your voice. Practice only, not scored."}</p>
            </section>
        },
        _ => description.map(|(title, detail)| html! {
            <section role="tabpanel" data-standalone-exercise="" class="rounded-2xl border border-line bg-surface p-6">
                <h3 class="text-lg font-semibold">{title}</h3>
                <p class="mt-2 text-sm text-muted">{detail}</p>
            </section>
        }).unwrap_or_default(),
    }
}
