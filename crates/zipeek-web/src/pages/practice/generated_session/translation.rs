use super::super::{
    feedback::{ExercisePrimaryAction, ExerciseReview, FeedbackVerdict},
    instruction::InstructionIcon,
    surface::ExerciseSurface,
};
use super::order;
use crate::services::{ai, translation_instance::canonical_tile_positions};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use zipeek_ai_protocol::{
    Evaluation, Sentence, evaluation_messages_for_direction, parse_evaluation,
};
use zipeek_exercises::translation::TranslationDirection;

#[derive(Properties, PartialEq)]
pub(super) struct TranslationPracticeProps {
    pub sentence: Sentence,
    pub seed: u64,
    pub on_scored: Callback<bool>,
}

#[function_component(TranslationPractice)]
pub(super) fn translation_practice(props: &TranslationPracticeProps) -> Html {
    let selected = use_state(Vec::<usize>::new);
    let evaluation = use_state(|| None::<Result<Evaluation, String>>);
    let evaluating = use_state(|| false);
    let checked = (*evaluation).as_ref().is_some_and(Result::is_ok);
    let evaluation_version = use_mut_ref(|| 0_u64);
    {
        let evaluation_version = evaluation_version.clone();
        use_effect_with((), move |_| {
            move || {
                let mut version = evaluation_version.borrow_mut();
                *version = version.wrapping_add(1);
            }
        });
    }
    let reset = {
        let selected = selected.clone();
        let evaluation = evaluation.clone();
        let evaluating = evaluating.clone();
        let evaluation_version = evaluation_version.clone();
        Callback::from(move |_: MouseEvent| {
            let mut version = evaluation_version.borrow_mut();
            *version = version.wrapping_add(1);
            selected.set(Vec::new());
            evaluation.set(None);
            evaluating.set(false);
        })
    };
    let sentence = &props.sentence;
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
    let tile_order = order::shuffled_indices(sentence.word_bank.len(), props.seed);
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
                    "Configure an AI model in Settings to check this translation".to_owned(),
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
        <div class="w-full" data-generated-translation="">
            <ExerciseSurface icon={InstructionIcon::Arrange} instruction="Translate by arranging every word-bank tile.">
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
                    disabled={checked || *evaluating}
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
                    disabled={used || checked || *evaluating}
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
            {if checked {
                html! { <div class="space-y-3">
                    {if let Some(Ok(verdict)) = (*evaluation).as_ref() {
                        html! {
                            <div data-generated-translation-next="">
                                <ExerciseReview verdict={if verdict.accurate { FeedbackVerdict::Correct } else { FeedbackVerdict::Incorrect }} on_continue={on_continue_translation.clone()}>
                                    <div data-generated-translation-feedback="">
                                        <p data-generated-translation-result={if verdict.accurate {"correct"} else {"incorrect"}}>
                                            {if verdict.accurate { "Correct translation." } else { "Review this translation." }}
                                        </p>
                                        <p>{&verdict.feedback}</p>
                                        <p>{format!("Suggested: {}", verdict.suggested_translation)}</p>
                                    </div>
                                    <p>{format!("Reference: {target}")}</p>
                                </ExerciseReview>
                            </div>
                        }
                    } else { Html::default() }}
                </div> }
            } else {
                html! { <div class="flex gap-3">
                    <button
                    type="button"
                    class="rounded-lg border border-line px-4 py-2"
                    disabled={*evaluating}
                    onclick={reset}>{"Clear"}</button>
                    <ExercisePrimaryAction
                        label={if *evaluating {"Evaluating…"} else {"Check answer"}}
                        on_activate={check}
                        disabled={!complete || *evaluating}
                        translation=true
                    />
                </div> }
            }}
            </ExerciseSurface>
        </div>
    }
}
