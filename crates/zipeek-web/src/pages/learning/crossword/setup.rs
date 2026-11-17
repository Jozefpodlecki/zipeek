use std::collections::BTreeSet;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::practice_draft::HskBand;

#[derive(Properties, PartialEq)]
pub(super) struct CrosswordSetupProps {
    pub selected_bands: BTreeSet<HskBand>,
    pub loading: bool,
    pub error: String,
    pub notice: Option<&'static str>,
    pub on_toggle_band: Callback<HskBand>,
    pub on_start: Callback<MouseEvent>,
}

#[function_component(CrosswordSetup)]
pub(super) fn crossword_setup(props: &CrosswordSetupProps) -> Html {
    let start_button_label = {
        if props.loading {
            html! { <span>
                {"Preparing crossword…"}
            </span> }
        } else {
            html! { <span
                class="inline-flex items-center gap-2"
            >
                <Icon
                    data={IconData::LUCIDE_PLAY}
                    width="18px"
                    height="18px"
                />{"Start"}
                </span> }
        }
    };

    let crossword_setup_error = if !props.error.is_empty() {
        html! { <p
            role="alert"
            class="mt-4 text-sm text-red-400"
        >
            {props.error.clone()}
        </p> }
    } else {
        Html::default()
    };
    let crossword_notice = {
        if let Some(notice) = props.notice {
            html! { <p
                role="alert"
                class="mt-4 text-sm text-red-400"
            >
                {notice}
            </p> }
        } else {
            Html::default()
        }
    };
    html! {
        <main
            class="flex min-h-0 flex-1 items-center justify-center overflow-y-auto p-4"
            aria-label="Crossword setup"
            data-crossword-setup=""
        >
            <div
                class="w-full max-w-3xl rounded-2xl border border-line bg-raised p-5 sm:p-7"
            >
                <h2
                    class="text-2xl font-bold"
                >
                    {"Crossword setup"}
                </h2>
                <p
                    class="mt-2 text-sm text-muted"
                >
                    {"Choose HSK levels to build a connected puzzle. Select more bands for greater variety."}
                </p>
                <div
                    class="mt-5 grid gap-4 md:grid-cols-2"
                >
                {for [2u8, 3u8].into_iter().map(|system| {
                    let max_level = if system == 2 { 6 } else { 7 };
                    html! {
                        <fieldset
                            class="min-w-0 rounded-xl border border-line p-4"
                        >
                            <legend
                                class="mb-3 font-semibold"
                            >
                                {format!("HSK {}.0", system)}
                            </legend>
                            <div
                                class="flex flex-wrap gap-2"
                            >
                                {for (1..=max_level).map(|level| {
                                    let band = HskBand::new(system, level).expect("static HSK level is valid");
                                    let selected = props.selected_bands.contains(&band);
                                    let on_toggle = props.on_toggle_band.clone();
                                    let on_toggle_band = Callback::from(move |_| on_toggle.emit(band.clone()));
                                    let element_classes_1 = classes!(
                                                    "bg-surface", "hover:bg-raised", "hover:border-accent",
                                                    "inline-flex",
                                                    "min-h-12",
                                                    "items-center",
                                                    "gap-2",
                                                    "rounded-xl",
                                                    "border",
                                                    "px-3",
                                                    "py-2",
                                                    "text-sm",
                                                    "font-semibold",
                                                    "transition-colors",
                                                    "focus-visible:outline-2",
                                                    "focus-visible:outline-offset-2",
                                                    "focus-visible:outline-brand-action",
                                                    if selected { classes!("border-brand-action", "bg-[color-mix(in_srgb,var(--brand-action)_12%,var(--surface))]") } else { classes!("border-line") },
                                                );
                                    let element_classes_2 = classes!(
                                                        "crossword-band-indicator",
                                                        "grid",
                                                        "size-5",
                                                        "shrink-0",
                                                        "place-items-center",
                                                        "rounded-md",
                                                        "border",
                                                        if selected { classes!("border-brand-action", "bg-brand-action", "text-white") } else { classes!("border-line") },
                                                    );
                                    html! {
                                        <button
                                            key={format!("{system}-{level}")}
                                            type="button"
                                            role="checkbox"
                                            aria-checked={selected.to_string()}
                                            aria-label={format!("HSK {}.0, level {}", system, level)}
                                            data-crossword-band={format!("{system}-{level}")}
                                            class={element_classes_1}
                                            onclick={on_toggle_band}
                                        >
                                            <span
                                                aria-hidden="true"
                                                class={element_classes_2}
                                            >
                                                {if selected { html! { <Icon
                                                    data={IconData::LUCIDE_CHECK}
                                                    width="14px"
                                                    height="14px"
                                                /> } } else { Html::default() }}
                                            </span>
                                            {format!("Level {}",level)}
                                        </button>
                                    }
                                })}
                            </div>
                        </fieldset>
                    }
                })}
                </div>
                {crossword_setup_error}
                {crossword_notice}
                <button
                    data-testid="crossword-start"
                    type="button"
                    disabled={props.loading || props.selected_bands.is_empty()}
                    class="mt-5 rounded-xl bg-brand-action px-6 py-3 font-semibold text-white disabled:cursor-not-allowed disabled:opacity-50"
                    onclick={props.on_start.clone()}
                >
                    {start_button_label}
                </button>
            </div>
        </main>
    }
}
