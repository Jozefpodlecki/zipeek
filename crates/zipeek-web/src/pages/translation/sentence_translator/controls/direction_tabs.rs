use yew::prelude::*;

use super::super::state::TranslationDirection;

#[derive(Properties, PartialEq)]
pub(super) struct TranslationDirectionTabsProps {
    pub direction: TranslationDirection,
    pub on_direction: Callback<TranslationDirection>,
}

#[function_component(TranslationDirectionTabs)]
pub(super) fn translation_direction_tabs(props: &TranslationDirectionTabsProps) -> Html {
    html! {
        <div
            data-translation-direction-switch=""
            role="tablist"
            aria-label="Translation direction"
            class="grid grid-cols-2 rounded-xl border border-line bg-surface p-1"
        >
            {for [
                (TranslationDirection::EnglishToChinese, "English → 中文"),
                (TranslationDirection::ChineseToEnglish, "中文 → English"),
            ].into_iter().map(|(direction, label)| {
                let selected = props.direction == direction;
                let on_direction = props.on_direction.clone();
                let on_change_direction = {
                    let selected_direction = direction.clone();
                    Callback::from(move |_| on_direction.emit(selected_direction.clone()))
                };
                let element_classes_1 = classes!(
                                "min-h-11",
                                "rounded-lg",
                                "px-3",
                                "font-semibold",
                                "text-sm",
                                "focus-visible:outline-1",
                                "focus-visible:outline-brand-action",
                                if selected {
                                "bg-brand-action text-white"
                            } else {
                                "text-muted hover:bg-raised hover:text-ink"
                            },
                            );
                html! {
                    <button
                        key={direction.code()}
                        type="button"
                        role="tab"
                        data-translation-direction={direction.code()}
                        aria-selected={selected.to_string()}
                        class={element_classes_1}
                        onclick={on_change_direction}
                    >
                        {label}
                    </button>
                }
            })}
        </div>
    }
}
