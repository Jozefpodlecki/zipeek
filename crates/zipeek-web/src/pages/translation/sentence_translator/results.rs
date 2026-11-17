use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_ai_protocol::sentence_analysis::SentenceAnalysis;

use super::state::TranslationDirection;

#[derive(Properties, PartialEq)]
pub(super) struct TranslationResultsProps {
    pub direction: TranslationDirection,
    pub result: String,
    pub analysis: Option<SentenceAnalysis>,
    pub busy: bool,
    pub copy_status: String,
    pub on_copy: Callback<String>,
}

#[function_component(TranslationResults)]
pub(super) fn translation_results(props: &TranslationResultsProps) -> Html {
    if props.result.is_empty() && !props.busy {
        return Html::default();
    }

    let heading = if props.direction == TranslationDirection::EnglishToChinese {
        "Chinese translations"
    } else {
        "English translations"
    };
    let alternatives = props
        .analysis
        .as_ref()
        .map_or(&[][..], |analysis| analysis.alternatives.as_slice());
    let variants = std::iter::once(props.result.as_str())
        .filter(|text| !text.is_empty())
        .chain(alternatives.iter().map(String::as_str));

    let translation_progress_or_result = {
        if props.busy {
            html! { <p
                role="status"
                class="text-sm text-muted"
            >
                {"Translating and analysing…"}
            </p> }
        } else {
            html! {
                <ol
                    class="space-y-2"
                >
                    {for variants.enumerate().map(|(index, text)| {
                        let copy = props.on_copy.clone();
                        let value = text.to_owned();
                        let on_copy_translation = Callback::from(move |_| copy.emit(value.clone()));
                        let element_classes_1 = classes!(
                                        "flex",
                                        "items-start",
                                        "gap-3",
                                        "rounded-xl",
                                        "border",
                                        "p-3",
                                        if index == 0 { "border-brand-action bg-raised" } else { "border-line" },
                                    );
                        html! {
                            <li
                                key={index}
                                class={element_classes_1}
                            >
                                <span
                                    class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-surface text-xs font-semibold"
                                >
                                    {index + 1}
                                </span>
                                <p
                                    class="min-w-0 flex-1 whitespace-pre-wrap break-words text-sm leading-6"
                                >
                                    {text.to_owned()}
                                </p>
                                <button
                                    type="button"
                                    aria-label={format!("Copy translation {}", index + 1)}
                                    title="Copy translation"
                                    onclick={on_copy_translation}
                                    class="rounded-lg p-2 text-muted hover:bg-raised hover:text-ink"
                                >
                                    <Icon
                                        data={IconData::LUCIDE_COPY}
                                        width="17px"
                                        height="17px"
                                    />
                                </button>
                            </li>
                        }
                    })}
                </ol>
            }
        }
    };

    let alternative_translations = {
        if !alternatives.is_empty() {
            html! {
                <span
                    class="text-xs text-muted"
                >
                    {format!("{} alternatives", alternatives.len())}
                </span>
            }
        } else {
            Html::default()
        }
    };
    html! {
        <section
            data-translation-result=""
            aria-live="polite"
            aria-busy={props.busy.to_string()}
            class="min-w-0 rounded-2xl border border-line bg-surface p-4 sm:p-5"
        >
            <div
                class="mb-4 flex items-center justify-between gap-2"
            >
                <h2
                    class="font-semibold"
                >
                    {heading}
                </h2>
                {alternative_translations}
            </div>
            {translation_progress_or_result}
            <p
                data-translation-copy-status=""
                role="status"
                aria-live="polite"
                class="mt-2 text-xs text-muted"
            >
                {props.copy_status.clone()}
            </p>
        </section>
    }
}
