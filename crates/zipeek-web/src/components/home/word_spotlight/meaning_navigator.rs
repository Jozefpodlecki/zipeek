use crate::services::database::DictionaryMeaning;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub struct MeaningNavigatorProps {
    pub meanings: Vec<DictionaryMeaning>,
    pub selected: usize,
    pub on_select: Callback<usize>,
}

#[function_component(MeaningNavigator)]
pub fn meaning_navigator(props: &MeaningNavigatorProps) -> Html {
    let count = props.meanings.len();
    let selected = props.selected.min(count.saturating_sub(1));
    let previous = {
        let on_select = props.on_select.clone();
        Callback::from(move |_| on_select.emit(selected.saturating_sub(1)))
    };
    let next = {
        let on_select = props.on_select.clone();
        Callback::from(move |_| on_select.emit((selected + 1).min(count.saturating_sub(1))))
    };
    let Some(meaning) = props.meanings.get(selected) else {
        return html! { <p data-spotlight-meaning="" class="text-sm text-muted">{"No definition available"}</p> };
    };
    html! {
        <section data-meaning-navigator="" class={if count > 1 {"inline-flex max-w-full flex-col items-center gap-2 rounded-xl border border-line bg-raised/50 px-2 py-1.5"} else {"inline-flex max-w-full flex-col items-center gap-2 py-1"}} aria-label="Word meanings">
            <div class="flex max-w-full items-center justify-center gap-2">
                {if count > 1 { html! { <button type="button" data-previous-meaning="" aria-label="Previous meaning" title="Previous meaning" disabled={selected == 0} onclick={previous}
                    class="inline-flex min-h-9 min-w-9 shrink-0 items-center justify-center rounded-lg border border-line bg-raised text-accent transition-[border-color,background-color] duration-200 hover:border-accent motion-reduce:transition-none disabled:cursor-not-allowed disabled:opacity-35 focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_CHEVRON_LEFT} width="16px" height="16px" />
                </button> } } else { Html::default() }}
                <div class="min-w-0 max-w-64" aria-live="polite" aria-atomic="true">
                    <p data-spotlight-meaning="" class="break-words text-center text-base font-semibold leading-snug">{&meaning.definition}</p>
                </div>
                {if count > 1 { html! { <button type="button" data-next-meaning="" aria-label="Next meaning" title="Next meaning" disabled={selected + 1 >= count} onclick={next}
                    class="inline-flex min-h-9 min-w-9 shrink-0 items-center justify-center rounded-lg border border-line bg-raised text-accent transition-[border-color,background-color] duration-200 hover:border-accent motion-reduce:transition-none disabled:cursor-not-allowed disabled:opacity-35 focus-visible:outline-2 focus-visible:outline-accent">
                    <Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="16px" height="16px" />
                </button> } } else { Html::default() }}
            </div>
            if !meaning.categories.is_empty() || !meaning.qualifiers.is_empty() {
                <div
                    data-meaning-tags=""
                    aria-label="Labels for this meaning"
                    class="flex flex-wrap items-center justify-center gap-2"
                >
                    {for meaning.categories.iter().map(|category| html! {
                        <span class="rounded-full border border-accent/40 bg-accent-soft px-3 py-1 text-xs font-medium text-accent">{category}</span>
                    })}
                    {for meaning.qualifiers.iter().map(|qualifier| html! {
                        <span title={qualifier.annotation.clone()} class="rounded-full border border-line bg-surface px-3 py-1 text-xs font-medium text-muted">{qualifier.label()}</span>
                    })}
                </div>
            }
        </section>
    }
}
