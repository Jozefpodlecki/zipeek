use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_core::practice::ExerciseVariant;
use zipeek_exercises::practice_draft::{MatchingColumnOrder, PracticeDraft};

#[derive(Properties, PartialEq)]
pub struct PracticeFormatSettingsProps {
    pub draft: PracticeDraft,
    pub family: String,
    pub on_toggle: Callback<ExerciseVariant>,
    pub on_column_order: Callback<MatchingColumnOrder>,
    pub pair_count: u8,
    pub on_pair_count: Callback<u8>,
}

#[derive(Properties, PartialEq)]
struct MatchingColumnOrderProps {
    formats: Vec<ExerciseVariant>,
    order: MatchingColumnOrder,
    on_change: Callback<MatchingColumnOrder>,
}

#[function_component(MatchingColumnOrderSettings)]
fn matching_column_order_settings(props: &MatchingColumnOrderProps) -> Html {
    let (standard_label, reversed_label) = match props.formats.as_slice() {
        [ExerciseVariant::MatchingHanziMeaning] => ("Hanzi / Meaning", "Meaning / Hanzi"),
        [ExerciseVariant::MatchingAudioHanzi] => ("Audio / Hanzi", "Audio / Hanzi"),
        [ExerciseVariant::MatchingAudioMeaning] => ("Audio / Meaning", "Audio / Meaning"),
        [ExerciseVariant::MatchingEnglishPinyin] => ("Meaning / Pinyin", "Pinyin / Meaning"),
        [ExerciseVariant::MatchingPinyinHanzi] => ("Pinyin / Hanzi", "Hanzi / Pinyin"),
        _ => ("Prompts / Answers", "Answers / Prompts"),
    };

    html! {
        <fieldset data-matching-column-order="" class="space-y-2 border-t border-line pt-4">
            <legend class="font-semibold">{"Column order"}</legend>
            <p class="text-sm text-muted">
                {"Reverse the columns for every selected Matching format."}
            </p>
            <div class="flex flex-wrap gap-2">
                {for [
                    (MatchingColumnOrder::Standard, standard_label),
                    (MatchingColumnOrder::Reversed, reversed_label),
                ].into_iter().map(|(choice, label)| {
                    let selected = choice == props.order;
                    let selected_class = if selected {
                        "border-brand-action bg-accent-soft text-ink ring-2 ring-brand-action/30"
                    } else {
                        "border-line bg-surface hover:bg-raised"
                    };
                    let on_change = props.on_change.clone();
                    let on_select_order = Callback::from(move |_: MouseEvent| on_change.emit(choice));
                    let element_classes_1 = classes!(
                                "min-h-11", "rounded-xl", "border-2", "px-4", "py-2", "transition-colors", "duration-200", "motion-reduce:transition-none",
                                "font-semibold", selected_class,
                            );
                    html! {
                        <button
                            type="button"
                            key={choice.query_value()}
                            data-matching-order={choice.query_value()}
                            aria-pressed={selected.to_string()}
                            class={element_classes_1}
                            onclick={on_select_order}
                        >
                            {label}
                        </button>
                    }
                })}
            </div>
        </fieldset>
    }
}

#[function_component(PracticeFormatSettings)]
pub fn practice_format_settings(props: &PracticeFormatSettingsProps) -> Html {
    let mcq = props.family == "multiple-choice";
    let selected_formats = props.draft.selected_formats();
    let format_settings_content = if mcq {
        Html::default()
    } else {
        html! {
            <>
                <div data-matching-pair-count="" class="space-y-2">
                    <p class="font-semibold">{"Pairs per round"}</p>
                    <div role="group" aria-label="Pairs per matching round" class="flex flex-wrap gap-2">
                        {for (2u8..=7).map(|count| {
                            let selected = count == props.pair_count;
                            let on_pair_count = props.on_pair_count.clone();
                            let choose = Callback::from(move |_: MouseEvent| on_pair_count.emit(count));
                            html! {
                                <button key={count} type="button" data-pair-count={count.to_string()}
                                    aria-pressed={selected.to_string()} onclick={choose}
                                    class={classes!("size-11", "rounded-xl", "border-2", "font-semibold", "transition-all", "duration-150", "motion-reduce:transition-none", "focus-visible:outline", "focus-visible:outline-2", "focus-visible:outline-offset-2", "focus-visible:outline-accent",
                                        if selected {"border-brand-action bg-brand-action text-white shadow-md"} else {"border-line bg-raised text-ink hover:border-brand-action"})}>
                                    {count}
                                </button>
                            }
                        })}
                    </div>
                </div>
                {if props.draft.matching_formats.iter().any(|format| !matches!(format, ExerciseVariant::MatchingAudioHanzi | ExerciseVariant::MatchingAudioMeaning)) {
                    html! { <MatchingColumnOrderSettings
                        formats={props.draft.matching_formats.clone()}
                        order={props.draft.matching_column_order}
                        on_change={props.on_column_order.clone()}
                    /> }
                } else { Html::default() }}
            </>
        }
    };
    let missing_format_warning = if props
        .draft
        .active_formats()
        .iter()
        .any(|variant| variant.is_mcq() == mcq)
    {
        Html::default()
    } else {
        html! {
            <p role="status" class="text-sm text-amber-400">
                {"Choose at least one format before starting this exercise."}
            </p>
        }
    };

    html! {
        <section
            role="tabpanel"
            data-practice-variant-settings={props.family.clone()}
            class="flex max-w-5xl flex-col gap-4 rounded-2xl border border-line bg-surface p-6"
        >
            <fieldset class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                <legend class="mb-2 font-semibold">{"Question formats"}</legend>
                {for ExerciseVariant::ALL.into_iter()
                    .filter(|variant| variant.is_mcq() == mcq)
                    .map(|variant| {
                        let selected = selected_formats.contains(&variant);
                        let selected_class = if selected {
                            "border-brand-action bg-accent-soft text-ink ring-2 ring-brand-action/30"
                        } else {
                            "border-line bg-surface text-ink hover:bg-raised"
                        };
                        let on_toggle = props.on_toggle.clone();
                        let on_select_format = Callback::from(move |_: MouseEvent| on_toggle.emit(variant));
                        let element_classes_2 = classes!(
                                    "flex", "min-h-12", "items-center", "gap-3",
                                    "rounded-xl", "border-2", "px-4", "py-3", "text-left",
                                    "font-semibold", "transition-colors",
                                    "motion-reduce:transition-none", selected_class,
                                );
                        html! {
                            <button
                                key={variant.seed_token()}
                                type="button"
                                data-setup-choice={variant.seed_token()}
                                aria-pressed={selected.to_string()}
                                class={element_classes_2}
                                onclick={on_select_format}
                            >
                                <Icon data={IconData::LUCIDE_ARROW_LEFT_RIGHT} width="20" height="20" />
                                <span class="flex-1">{variant.label()}</span>
                                {if selected {
                                    html! { <Icon data={IconData::LUCIDE_CHECK} width="18px" height="18px" /> }
                                } else {
                                    Html::default()
                                }}
                            </button>
                        }
                    })}
            </fieldset>
            {format_settings_content}
            {missing_format_warning}
        </section>
    }
}
