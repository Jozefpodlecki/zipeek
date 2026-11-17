use yew::prelude::*;
use zipeek_core::{Word, normalize_for_comparison};

use crate::models::{AnswerPolicy, TypePinyinOptions};

use super::{
    answer::{AnswerAction, AnswerPhase, AnswerState},
    vocabulary::meaning,
};

#[derive(Properties, PartialEq)]
pub(super) struct TypePinyinProps {
    pub word: Word,
    pub options: TypePinyinOptions,
    pub on_completed: Callback<u32>,
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
        Callback::from(move |_| on_completed.emit(answer.mistakes))
    };

    html! {
        <div class="space-y-6 text-center" data-exercise="type-pinyin">
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
                    <div role="status" class="rounded-xl border border-line bg-raised p-4">
                        <strong>{if correct { "Correct" } else { "Incorrect" }}</strong>
                        {if correct { Html::default() } else { html! { <p>{format!("Correct answer: {}", props.word.pinyin)}</p> } }}
                    </div>
                }
            } else { Html::default() }}
            <div class="flex flex-wrap items-center justify-between gap-3">
                {for props.children.iter()}
                {if reviewed {
                    html! { <button type="button" data-exercise-continue="" onclick={on_continue} class="rounded-xl bg-brand-action px-6 py-4 font-bold text-white">{"Continue"}</button> }
                } else {
                    html! { <button type="button" data-exercise-check="" onclick={on_check} disabled={input.trim().is_empty()} class="rounded-xl bg-brand-action px-6 py-4 font-bold text-white disabled:opacity-40">{"Check answer"}</button> }
                }}
            </div>
        </div>
    }
}
