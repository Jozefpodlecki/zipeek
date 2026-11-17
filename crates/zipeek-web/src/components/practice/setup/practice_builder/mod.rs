mod exercise_selector;
use self::exercise_selector::ExerciseSelector;
use crate::reducers::practice_setup_flow::ExerciseStage;
use yew::prelude::*;
use zipeek_exercises::practice_draft::PracticeDraft;

#[derive(Properties, PartialEq)]
pub struct PracticeBuilderProps {
    pub draft: PracticeDraft,
    pub active_tab: &'static str,
    pub advanced: bool,
    pub on_toggle: Callback<&'static str>,
    pub on_tab: Callback<&'static str>,
    pub section: ExerciseStage,
}

#[function_component(PracticeBuilder)]
pub fn practice_builder(props: &PracticeBuilderProps) -> Html {
    if props.section == ExerciseStage::Choose {
        return html! { <ExerciseSelector
            draft={props.draft.clone()}
            advanced={props.advanced}
            on_toggle={props.on_toggle.clone()}
        /> };
    }

    if props.section == ExerciseStage::Configure {
        let tabs = props.draft.configuration_tabs();
        return html! {
            <section
                class="flex flex-col gap-3"
                data-component="practice-builder"
            >
                <h3 class="text-sm font-semibold text-ink">{"Exercise settings"}</h3>
                <div
                    role="tablist"
                    aria-label="Exercise settings"
                    data-practice-config-tabs=""
                    class="flex gap-2 overflow-x-auto rounded-xl border border-line bg-surface p-2"
                >
                    {for tabs.into_iter().filter(|tab| *tab != "general").map(|tab| {
                        let selected = props.active_tab == tab;
                        let on_tab = props.on_tab.clone();
                        let label = match tab {
                            "matching" => "Matching",
                            "multiple-choice" => "Multiple choice",
                            "translation" => "Sentence translation",
                            "speaking" => "Pronunciation",
                            "image" => "Picture vocabulary",
                            "hunt" => "Character Hunt",
                            "type-pinyin" => "Type pinyin",
                            "character-builder" => "Character builder",
                            "word-mosaics" => "Word mosaics",
                            "fill-blank" => "Fill in the Blank",
                            "audio-dictation" => "Audio Dictation",
                            "sentence-builder" => "Sentence Builder",
                            "sentence-transformation" => "Sentence Transformation",
                            "spot-mistake" => "Spot the Mistake",
                            "write-hanzi" => "Write Hanzi",
                            "listening" => "Listening",
                            _ => "Listening",
                        };
                        let on_select_tab = Callback::from(move |_| on_tab.emit(tab));
                        let element_classes_1 = classes!(
                                        "min-h-11",
                                        "shrink-0",
                                        "rounded-lg",
                                        "px-4",
                                        "text-sm",
                                        "font-semibold",
                                        if selected {"bg-brand-action text-white"} else {"text-muted hover:bg-raised hover:text-ink"},
                                    );
                        html! {
                            <button
                                key={tab}
                                type="button"
                                role="tab"
                                aria-selected={selected.to_string()}
                                data-practice-config-tab={tab}
                                class={element_classes_1}
                                onclick={on_select_tab}
                            >
                                {label}
                            </button>
                        }
                    })}
                </div>
            </section>
        };
    }

    Html::default()
}
