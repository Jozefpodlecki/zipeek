use web_sys::KeyboardEvent;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::super::state::TranslationDirection;

#[derive(Properties, PartialEq)]
pub(super) struct TranslationSentenceInputProps {
    pub direction: TranslationDirection,
    pub source: String,
    pub on_source: Callback<InputEvent>,
    pub on_keydown: Callback<KeyboardEvent>,
    pub on_clear: Callback<MouseEvent>,
}

#[function_component(TranslationSentenceInput)]
pub(super) fn translation_sentence_input(props: &TranslationSentenceInputProps) -> Html {
    let count = props.source.chars().count();
    let chinese = props.direction == TranslationDirection::ChineseToEnglish;

    let clear_input_button = {
        if props.source.is_empty() {
            Html::default()
        } else {
            html! {
                <button
                    type="button"
                    aria-label="Clear sentence"
                    title="Clear sentence"
                    onclick={props.on_clear.clone()}
                    class="inline-flex h-9 w-9 items-center justify-center rounded-lg text-muted hover:bg-raised hover:text-ink"
                >
                    <Icon
                        data={IconData::LUCIDE_X}
                        width="17px"
                        height="17px"
                    />
                </button>
            }
        }
    };

    html! {
        <>
            <div
                class="mb-3 flex items-center justify-between gap-3"
            >
                <span
                    class="text-sm text-muted"
                >
                    {"Enter a sentence to translate and analyse"}
                </span>
                <div
                    class="flex shrink-0 items-center gap-1"
                >
                    <span
                        class="w-20 text-right text-xs tabular-nums text-muted"
                    >
                        {format!("{count} / 4,000")}
                    </span>
                    <span
                        class="inline-flex h-9 w-9 items-center justify-center"
                    >
                        {clear_input_button}
                    </span>
                </div>
            </div>
            <textarea
                id="translate-sentence-input"
                aria-label={if chinese { "Chinese source sentence" } else { "English source sentence" }}
                data-translation-source=""
                rows="4"
                maxlength="4000"
                value={props.source.clone()}
                oninput={props.on_source.clone()}
                onkeydown={props.on_keydown.clone()}
                placeholder={if chinese { "请输入中文句子…" } else { "Enter an English sentence…" }}
                class="w-full min-w-0 resize-y rounded-xl border border-line bg-raised p-4 text-base text-ink focus-visible:outline-1 focus-visible:outline-brand-action"
            />
        </>
    }
}
