use std::collections::BTreeSet;

use wasm_bindgen_futures::spawn_local;
use zipeek_ai_protocol::{Evaluation, evaluation_messages_for_direction, parse_evaluation};
use zipeek_exercises::translation::TranslationDirection;

use crate::services::{ai, translation_instance::canonical_tile_positions};

use crate::services::practice_generated::GeneratedInstance;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GeneratedPracticeProps {
    pub raw: String,
    pub exercise_id: String,
    pub seed: u64,
    pub on_completed: Callback<()>,
    pub on_scored: Callback<bool>,
}

#[function_component(GeneratedPractice)]
pub fn generated_practice(props: &GeneratedPracticeProps) -> Html {
    let selected = use_state(Vec::<usize>::new);
    let checked = use_state(|| false);
    let evaluation = use_state(|| None::<Result<Evaluation, String>>);
    let evaluating = use_state(|| false);
    let evaluation_version = use_mut_ref(|| 0_u64);
    let image_loaded = use_state(|| false);
    let image_failed = use_state(|| false);
    {
        let selected = selected.clone();
        let checked = checked.clone();
        let image_loaded = image_loaded.clone();
        let image_failed = image_failed.clone();
        let evaluation = evaluation.clone();
        let evaluating = evaluating.clone();
        let evaluation_version = evaluation_version.clone();
        use_effect_with((props.exercise_id.clone(), props.raw.clone()), move |_| {
            selected.set(Vec::new());
            checked.set(false);
            image_loaded.set(false);
            image_failed.set(false);
            evaluation.set(None);
            evaluating.set(false);
            move || {
                let mut version = evaluation_version.borrow_mut();
                *version = version.wrapping_add(1);
            }
        });
    }
    let parsed = GeneratedInstance::decode(&props.raw, &props.exercise_id);
    let on_continue = {
        let completed = props.on_completed.clone();
        Callback::from(move |_: MouseEvent| completed.emit(()))
    };
    let reset = {
        let selected = selected.clone();
        let checked = checked.clone();
        Callback::from(move |_: MouseEvent| {
            selected.set(Vec::new());
            checked.set(false);
        })
    };
    let body = match parsed {
        Err(error) => {
            html! { <p role="alert">{format!("This generated exercise is invalid: {error}")}</p> }
        }
        Ok(GeneratedInstance::SentenceTranslation { sentence }) => {
            let source_is_chinese = sentence.direction == "zh-to-en";
            let source = if source_is_chinese {
                &sentence.chinese
            } else {
                &sentence.english
            };
            let target = if source_is_chinese {
                &sentence.english
            } else {
                &sentence.chinese
            };
            let complete = selected.len() == sentence.word_bank.len();
            let positions = canonical_tile_positions(&sentence.word_bank, &sentence.solution);
            let mut tile_order = (0..sentence.word_bank.len()).collect::<Vec<_>>();
            let mut state = props.seed | 1;
            for index in (1..tile_order.len()).rev() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                tile_order.swap(index, (state as usize) % (index + 1));
            }
            if tile_order
                .iter()
                .enumerate()
                .all(|(index, tile)| *tile == index)
            {
                tile_order.rotate_left(1);
            }
            let answer = selected
                .iter()
                .filter_map(|index| sentence.word_bank.get(*index))
                .map(String::as_str)
                .collect::<Vec<_>>();
            let joined = if source_is_chinese {
                answer.join(" ")
            } else {
                answer.concat()
            };
            let direction = TranslationDirection::from_code(&sentence.direction)
                .expect("validated generated translation must have a direction");
            let check = {
                let checked = checked.clone();
                let evaluating = evaluating.clone();
                let evaluation = evaluation.clone();
                let evaluation_version = evaluation_version.clone();
                let source = source.to_owned();
                let target = target.to_owned();
                Callback::from(move |_: MouseEvent| {
                    if *evaluating {
                        return;
                    }
                    let config = ai::configuration();
                    if config.model.trim().is_empty() {
                        evaluation.set(Some(Err(
                            "Configure an AI model in Settings to check this translation"
                                .to_owned(),
                        )));
                        return;
                    }
                    let request_id = {
                        let mut version = evaluation_version.borrow_mut();
                        *version = version.wrapping_add(1);
                        *version
                    };
                    let selected_answer = joined.clone();
                    let source = source.clone();
                    let target = target.clone();
                    let checked = checked.clone();
                    let evaluating = evaluating.clone();
                    let evaluation = evaluation.clone();
                    let evaluation_version = evaluation_version.clone();
                    evaluating.set(true);
                    evaluation.set(None);
                    spawn_local(async move {
                        let result = ai::complete(
                            &config.url,
                            &config.model,
                            evaluation_messages_for_direction(
                                direction,
                                &source,
                                &target,
                                &selected_answer,
                            ),
                        )
                        .await
                        .and_then(|raw| parse_evaluation(&raw));
                        if *evaluation_version.borrow() != request_id {
                            return;
                        }
                        checked.set(result.is_ok());
                        evaluation.set(Some(result));
                        evaluating.set(false);
                    });
                })
            };
            let on_continue_translation = {
                let on_scored = props.on_scored.clone();
                let correct = (*evaluation)
                    .as_ref()
                    .and_then(|result| result.as_ref().ok())
                    .is_some_and(|result| result.accurate);
                Callback::from(move |_: MouseEvent| on_scored.emit(correct))
            };
            html! {
                <div
                            class="space-y-4"
                            data-generated-translation="">
                    <p class="text-sm text-muted">{"Translate by arranging every word-bank tile."}</p>
                    <p class="rounded-xl border border-line p-4 text-xl"
                        data-generated-translation-source=""
                        lang={if source_is_chinese {"zh-Hans"} else {"en"}}>{source}</p>
                    <span class="sr-only" data-generated-translation-target="">
                        {if source_is_chinese {"English"} else {"Chinese"}}
                    </span>
                    <div
                            role="group"
                            aria-label="Chosen translation tiles"
                            class="flex min-h-14 flex-wrap gap-2 rounded-lg border border-line p-3">
                        {for selected.iter().enumerate().filter_map(|(position, index)| {
                            let tile = sentence.word_bank.get(*index)?;
                            let selection = selected.clone();
                            let position = position;
                            let remove = Callback::from(move |_: MouseEvent| {
                                let mut next = (*selection).clone();
                                next.remove(position);
                                selection.set(next);
                            });
                            Some(html! { <button
                            type="button"
                            class="rounded-lg bg-surface px-3 py-2"
                            disabled={*checked || *evaluating}
                            data-solution-order={positions[*index].map_or("-1".to_owned(), |position| position.to_string())}
                            onclick={remove}>{tile}</button> })
                        })}
                    </div>
                    <div
                            role="group"
                            aria-label="Available translation tiles"
                            class="flex flex-wrap gap-2">
                        {for tile_order.into_iter().map(|index| {
                            let tile = &sentence.word_bank[index];
                            let used = selected.contains(&index);
                            let selection = selected.clone();
                            let add = Callback::from(move |_: MouseEvent| {
                                let mut next = (*selection).clone();
                                if !next.contains(&index) { next.push(index); selection.set(next); }
                            });
                            html! { <button
                            key={index}
                            type="button"
                            class="rounded-lg border border-line px-3 py-2"
                            disabled={used || *checked || *evaluating}
                            data-bank-tile={index.to_string()}
                            data-solution-order={positions[index].map_or("-1".to_owned(), |position| position.to_string())}
                            data-solution-first={positions[index].is_some_and(|position| position == 0).to_string()}
                            data-solution-last={positions[index].is_some_and(|position| position + 1 == sentence.solution.len()).to_string()}
                            onclick={add}>{tile}</button> }
                        })}
                    </div>
                    {if let Some(Err(message)) = (*evaluation).as_ref() {
                        html! { <p role="alert" data-generated-translation-error="">{format!("Could not evaluate: {message}. Try again.")}</p> }
                    } else { Html::default() }}
                    {if *checked {
                        html! { <div
                            role="status"
                            class="space-y-3">
                            {if let Some(Ok(verdict)) = (*evaluation).as_ref() {
                                html! { <div data-generated-translation-feedback="">
                                    <p data-generated-translation-result={if verdict.accurate {"correct"} else {"incorrect"}}>
                                        {if verdict.accurate { "Correct translation." } else { "Review this translation." }}
                                    </p>
                                    <p>{&verdict.feedback}</p>
                                    <p>{format!("Suggested: {}", verdict.suggested_translation)}</p>
                                </div> }
                            } else { Html::default() }}
                            <p>{format!("Reference: {target}")}</p>
                            <button
                            type="button"
                            class="rounded-lg bg-brand-action px-5 py-3 text-white"
                            data-generated-translation-next=""
                            onclick={on_continue_translation}>{"Continue"}</button>
                        </div> }
                    } else {
                        html! { <div class="flex gap-3">
                            <button
                            type="button"
                            class="rounded-lg border border-line px-4 py-2"
                            disabled={*evaluating}
                            onclick={reset}>{"Clear"}</button>
                            <button
                            type="button"
                            class="rounded-lg bg-brand-action px-5 py-3 text-white"
                            data-generated-translation-check=""
                            disabled={!complete || *evaluating}
                            onclick={check}>{if *evaluating {"Evaluating…"} else {"Check answer"}}</button>
                        </div> }
                    }}
                </div>
            }
        }
        Ok(GeneratedInstance::ImageVocabulary {
            choices,
            correct_ids,
            image_url,
            scene,
        }) => {
            let on_image_load = {
                let image_loaded = image_loaded.clone();
                let image_failed = image_failed.clone();
                Callback::from(move |_: Event| {
                    image_loaded.set(true);
                    image_failed.set(false);
                })
            };
            let on_image_error = {
                let image_failed = image_failed.clone();
                let image_loaded = image_loaded.clone();
                Callback::from(move |_: Event| {
                    image_loaded.set(false);
                    image_failed.set(true);
                })
            };
            let toggle = {
                let selected = selected.clone();
                let selection_limit = correct_ids.len();
                Callback::from(move |index: usize| {
                    let mut next = (*selected).clone();
                    if let Some(position) = next.iter().position(|item| *item == index) {
                        next.remove(position);
                    } else if next.len() < selection_limit {
                        next.push(index);
                    }
                    selected.set(next);
                })
            };
            let check = {
                let checked = checked.clone();
                Callback::from(move |_: MouseEvent| checked.set(true))
            };
            let selected_ids = selected
                .iter()
                .filter_map(|index| choices.get(*index))
                .map(|choice| choice.id)
                .collect::<BTreeSet<_>>();
            let correct = selected_ids == correct_ids;
            html! {
                <div
                            class="space-y-4"
                            data-generated-image="">
                    <img
                        src={image_url}
                        alt="Generated illustration for Mandarin vocabulary practice"
                        class="mx-auto max-h-96 rounded-xl object-contain"
                        onload={on_image_load}
                        onerror={on_image_error}
                    />
                    {
                        if *image_failed {
                            html! { <p role="alert" class="rounded-lg border border-line bg-surface p-3">{"The illustration did not load. Return to Practice to regenerate it."}</p> }
                        } else if !*image_loaded {
                            html! { <p role="status">{"Loading illustration…"}</p> }
                        } else {
                            Html::default()
                        }
                    }
                    <div class="mx-auto w-full max-w-2xl space-y-3 rounded-xl border border-line bg-surface p-4 shadow-lg">
                        <div class="space-y-1">
                            <h2 class="text-base font-semibold">{"Which Chinese words match the picture?"}</h2>
                            <p class="text-sm text-muted">
                                {if correct_ids.len() == 1 {
                                    "Choose the one word depicted in the image."
                                } else {
                                    "Choose both words depicted in the image. The other words are distractors."
                                }}
                            </p>
                        </div>
                        <div role="group" aria-label="Choose Chinese words matching the picture"
                             class="grid grid-cols-2 gap-2 sm:grid-cols-3">
                            {for choices.iter().enumerate().map(|(index, choice)| {
                                let toggle = toggle.clone();
                                let choose = Callback::from(move |_: MouseEvent| toggle.emit(index));
                                let is_selected = selected.contains(&index);
                                html! { <button
                                    key={choice.id}
                                    type="button"
                                    class={classes!("min-h-14", "rounded-xl", "border", "px-3", "py-3", "text-lg", "font-medium", "transition-colors", "focus-visible:outline-2", "focus-visible:outline-offset-2",
                                        if is_selected {"border-brand-action bg-brand-action text-white"} else {"border-line bg-surface hover:border-brand-action"})}
                                    aria-pressed={is_selected.to_string()}
                                    disabled={*checked || !*image_loaded || *image_failed}
                                    onclick={choose}>{&choice.hanzi}</button> }
                            })}
                        </div>
                        {if *checked {
                            html! { <div role="status" class="space-y-3">
                                <p class="font-semibold">{if correct {"Correct!"} else {"Not quite — review the expected words."}}</p>
                                <p class="text-sm">{format!("Expected: {}", choices.iter()
                                    .filter(|choice| correct_ids.contains(&choice.id))
                                    .map(|choice| choice.hanzi.as_str()).collect::<Vec<_>>().join(", "))}</p>
                                <button type="button" class="rounded-lg bg-brand-action px-5 py-3 text-white"
                                    onclick={on_continue.clone()}>{"Continue"}</button>
                            </div> }
                        } else {
                            html! { <button type="button"
                                class="rounded-lg bg-brand-action px-5 py-3 text-white"
                                disabled={selected.is_empty() || !*image_loaded || *image_failed}
                                onclick={check}>{"Check answer"}</button> }
                        }}
                        <details class="text-sm text-muted">
                            <summary class="cursor-pointer">{"Image quality and generation details"}</summary>
                            <p class="mt-2">{"If the illustration does not match the intended subjects, return to Practice and generate another. An unsuitable image should not be treated as a knowledge assessment."}</p>
                            <p class="mt-2">{scene}</p>
                        </details>
                    </div>
                </div>
            }
        }
    };
    html! { <section class="space-y-4">{body}</section> }
}
