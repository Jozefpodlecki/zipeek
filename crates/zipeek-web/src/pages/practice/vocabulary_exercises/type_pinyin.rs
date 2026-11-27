use yew::prelude::*;
use zipeek_core::{Word, normalize_for_comparison};

use super::super::{
    feedback::{ExercisePrimaryAction, ExerciseReview, FeedbackVerdict},
    instruction::InstructionIcon,
    surface::ExerciseSurface,
};
use crate::{
    models::{AnswerPolicy, TypePinyinOptions},
    services::practice_session_store::{AssessmentCapability, AssessmentEvidence, ExerciseVerdict},
};

use super::{
    answer::{AnswerAction, AnswerPhase, AnswerState},
    progress::VocabularyRoundResult,
    vocabulary::meaning,
};

#[derive(Properties, PartialEq)]
pub(super) struct TypePinyinProps {
    pub word: Word,
    pub options: TypePinyinOptions,
    pub on_completed: Callback<VocabularyRoundResult>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(TypePinyin)]
pub(super) fn type_pinyin(props: &TypePinyinProps) -> Html {
    let answer = use_reducer(AnswerState::default);
    let input = use_state(String::new);
    let reviewed = matches!(answer.phase, AnswerPhase::Reviewed { .. });
    let supplied = normalize_for_comparison(input.trim());
    let correct = supplied == normalize_for_comparison(&props.word.pinyin)
        || props
            .word
            .pinyin_numeric
            .as_deref()
            .is_some_and(|numeric| supplied == normalize_for_comparison(numeric));
    let retry = props.options.answer_policy == AnswerPolicy::RetryUntilCorrect;
    let placeholder = if props.options.show_hint {
        props.word.pinyin.clone()
    } else {
        "Type pinyin with tone marks or tone numbers".to_owned()
    };

    let on_input = {
        let input = input.clone();
        let answer = answer.clone();
        Callback::from(move |event: InputEvent| {
            let field: web_sys::HtmlInputElement = event.target_unchecked_into();
            input.set(field.value());
            answer.dispatch(AnswerAction::Edit);
        })
    };
    let on_check = {
        let answer = answer.clone();
        let input = input.clone();
        Callback::from(move |_| {
            answer.dispatch(AnswerAction::Submit { correct, retry });
            if !correct && retry {
                input.set(String::new());
            }
        })
    };
    let on_keydown = {
        let answer = answer.clone();
        let input = input.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.key() != "Enter" || event.is_composing() || reviewed || input.trim().is_empty()
            {
                return;
            }
            event.prevent_default();
            answer.dispatch(AnswerAction::Submit { correct, retry });
            if !correct && retry {
                input.set(String::new());
            }
        })
    };
    let on_continue = {
        let on_completed = props.on_completed.clone();
        let answer = answer.clone();
        let target = props.word.simplified.clone();
        let hint_visible = props.options.show_hint;
        Callback::from(move |_| {
            if let AnswerPhase::Reviewed { correct } = answer.phase {
                let attempts = answer.attempts.clamp(1, u16::MAX as u32) as u16;
                let mistakes = answer.mistakes.min(u16::MAX as u32) as u16;
                let assisted = hint_visible;
                let verdict = if !correct {
                    ExerciseVerdict::Incorrect
                } else if mistakes > 0 || assisted {
                    ExerciseVerdict::PartiallyCorrect
                } else {
                    ExerciseVerdict::Correct
                };
                on_completed.emit(VocabularyRoundResult {
                    correct,
                    attempts: answer.attempts,
                    mistakes: answer.mistakes,
                    item_evidence: vec![AssessmentEvidence {
                        capability: AssessmentCapability::PinyinReading,
                        target: target.clone(),
                        verdict,
                        attempts,
                        mistakes,
                        hints: u16::from(hint_visible),
                        assisted,
                    }],
                });
            }
        })
    };

    html! {
        <div class="w-full text-center" data-exercise="type-pinyin">
            <ExerciseSurface icon={InstructionIcon::Write} instruction="Type the pinyin reading for the displayed word.">
            <p class="text-6xl font-semibold" lang="zh-Hans">{&props.word.simplified}</p>
            <p class="text-xl text-muted">{meaning(&props.word).unwrap_or_default()}</p>
            <label for="pinyin-answer" class="block text-left text-sm font-semibold">{"Pinyin reading"}</label>
            <input
                id="pinyin-answer"
                aria-label="Pinyin reading"
                autocomplete="off"
                spellcheck="false"
                class="w-full rounded-xl border border-line bg-raised p-4 text-xl text-ink"
                placeholder={placeholder}
                value={(*input).clone()}
                disabled={reviewed}
                oninput={on_input}
                onkeydown={on_keydown}
            />
            {if answer.retry_notice { html! { <p role="status" data-practice-retry-notice="">{"Try again"}</p> } } else { Html::default() }}
            {if let AnswerPhase::Reviewed { correct } = answer.phase {
                html! {
                    <div data-exercise-continue=""><ExerciseReview verdict={if correct { FeedbackVerdict::Correct } else { FeedbackVerdict::Incorrect }} on_continue={on_continue}>
                        {if correct { Html::default() } else { html! { <p>{format!("Correct answer: {}", props.word.pinyin)}</p> } }}
                    </ExerciseReview></div>
                }
            } else { Html::default() }}
            <div class="flex flex-wrap items-center justify-between gap-3">
                {for props.children.iter()}
                {if !reviewed {
                    html! { <ExercisePrimaryAction label="Check answer" on_activate={on_check} disabled={input.trim().is_empty()} vocabulary=true /> }
                } else { Html::default() }}
            </div>
            </ExerciseSurface>
        </div>
    }
}
