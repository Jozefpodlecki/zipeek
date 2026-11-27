use super::feedback::{ExerciseReview, ExerciseSubmitAction, FeedbackVerdict};
use super::instruction::InstructionIcon;
use super::surface::ExerciseSurface;
use crate::services::practice_session_store::{ExerciseAssessment, ExerciseVerdict};
use yew::prelude::*;
use zipeek_exercises::spot_mistake::{self, MistakePrompt};

#[derive(Properties, PartialEq)]
pub struct SpotMistakeProps {
    pub seed: u64,
    pub on_completed: Callback<()>,
    #[prop_or_default]
    pub on_assessed: Option<Callback<ExerciseAssessment>>,
}

#[function_component(SpotMistake)]
pub fn spot_mistake(props: &SpotMistakeProps) -> Html {
    let prompt: &'static MistakePrompt =
        &spot_mistake::PROMPTS[props.seed as usize % spot_mistake::PROMPTS.len()];
    let selected = use_state(|| None::<usize>);
    let replacement = use_state(String::new);
    let checked = use_state(|| false);
    let composing = use_state(|| false);
    {
        let selected = selected.clone();
        let replacement = replacement.clone();
        let checked = checked.clone();
        let composing = composing.clone();
        use_effect_with(props.seed, move |_| {
            selected.set(None);
            replacement.set(String::new());
            checked.set(false);
            composing.set(false);
        });
    }
    let verdict = spot_mistake::grade(prompt, *selected, &replacement);
    let on_input = {
        let replacement = replacement.clone();
        let composing = composing.clone();
        Callback::from(move |event: InputEvent| {
            composing.set(event.is_composing());
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            replacement.set(input.value());
        })
    };
    let on_keydown = Callback::from(move |event: KeyboardEvent| {
        if event.key() == "Enter" && (event.is_composing() || event.key_code() == 229) {
            event.prevent_default();
        }
    });
    let on_check = {
        let selected = selected.clone();
        let replacement = replacement.clone();
        let checked = checked.clone();
        let composing = composing.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            if selected.is_some() && !replacement.trim().is_empty() && !*composing && !*checked {
                checked.set(true);
            }
        })
    };
    let on_continue = {
        let on_completed = props.on_completed.clone();
        let on_assessed = props.on_assessed.clone();
        let identified = verdict.identified;
        let repaired = verdict.repaired;
        Callback::from(move |_: MouseEvent| {
            if let Some(on_assessed) = &on_assessed {
                on_assessed.emit(ExerciseAssessment {
                    verdict: match (identified, repaired) {
                        (true, true) => ExerciseVerdict::Correct,
                        (false, false) => ExerciseVerdict::Incorrect,
                        _ => ExerciseVerdict::PartiallyCorrect,
                    },
                    attempts: 1,
                    mistakes: u16::from(!identified) + u16::from(!repaired),
                    hints: 0,
                    assisted: false,
                    item_evidence: Vec::new(),
                });
            } else {
                on_completed.emit(());
            }
        })
    };
    html! {
        <div class="w-full" data-practice-spot-mistake="">
            <ExerciseSurface icon={InstructionIcon::Find} instruction="Find the mistake and enter its correction">
            <p>{prompt.meaning}</p>
            <div class="flex flex-wrap gap-2" role="group" aria-label="Select the incorrect Chinese phrase" lang="zh-CN">
                {for prompt.spans.iter().enumerate().map(|(index, span)| {
                    let selected_value = selected.clone();
                    let choose = Callback::from(move |_: MouseEvent| selected_value.set(Some(index)));
                    html! {
                        <button key={index.to_string()} type="button" onclick={choose}
                            disabled={*checked} aria-pressed={(*selected == Some(index)).to_string()}
                            class="rounded-lg border border-line bg-raised px-3 py-3 text-ink disabled:opacity-60">
                            {span.to_string()}
                        </button>
                    }
                })}
            </div>
            <form onsubmit={on_check} class="space-y-3">
                <label for="spot-mistake-repair" class="block font-medium">{"Replacement Chinese phrase"}</label>
                <input id="spot-mistake-repair" type="text" lang="zh-CN" autocomplete="off"
                    class="w-full rounded-lg border border-line bg-raised px-4 py-3 text-ink"
                    value={(*replacement).clone()} oninput={on_input} onkeydown={on_keydown}
                    disabled={*checked} />
                {if !*checked { html! {
                    <ExerciseSubmitAction label="Check answer" spot_mistake=true
                        disabled={selected.is_none() || replacement.trim().is_empty() || *composing} />
                }} else { Html::default() }}
            </form>
            {if *checked { html! {
                <ExerciseReview verdict={if verdict.identified && verdict.repaired { FeedbackVerdict::Correct } else if verdict.identified || verdict.repaired { FeedbackVerdict::Partial } else { FeedbackVerdict::Incorrect }} on_continue={on_continue} spot_mistake=true>
                    <p>{if verdict.identified { "Mistake identified correctly" } else { "Mistake not identified" }}</p>
                    <p>{if verdict.repaired { "Correction accepted" } else { "Correction not accepted" }}</p>
                    <p lang="zh-CN">{spot_mistake::corrected_sentence(prompt, prompt.accepted_replacements[0]).unwrap_or_default()}</p>
                    <p class="text-sm text-muted">{prompt.explanation}</p>
                </ExerciseReview>
            }} else { Html::default() }}
            </ExerciseSurface>
        </div>
    }
}
