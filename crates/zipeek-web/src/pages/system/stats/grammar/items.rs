use super::super::{controls::SelectionCheckbox, data::GrammarRecord};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct GrammarCardProps {
    pub record: GrammarRecord,
    pub selected: bool,
    pub on_toggle: Callback<String>,
    pub on_inspect: Callback<String>,
}

#[function_component(GrammarCard)]
pub(super) fn grammar_card(props: &GrammarCardProps) -> Html {
    let pattern = &props.record;
    let toggle = {
        let id = pattern.id.clone();
        let action = props.on_toggle.clone();
        Callback::from(move |_: ()| action.emit(id.clone()))
    };
    let inspect = {
        let id = pattern.id.clone();
        let action = props.on_inspect.clone();
        Callback::from(move |_: MouseEvent| action.emit(id.clone()))
    };
    html! {
        <article class="flex min-h-36 flex-col gap-2 rounded-xl border border-line bg-surface p-3">
            <div class="flex items-start justify-between gap-2">
                <button type="button" onclick={inspect.clone()} class="min-h-10 flex-1 text-left font-semibold hover:text-accent focus-visible:outline-2 focus-visible:outline-accent">{pattern.pattern.clone()}</button>
                <SelectionCheckbox label={format!("Select {}", pattern.pattern)} checked={props.selected} on_toggle={toggle} />
            </div>
            <p class="line-clamp-2 text-xs text-muted">{pattern.description.clone()}</p>
            <p class="line-clamp-2 flex-1 text-sm" lang="zh-Hans">{pattern.example.clone()}</p>
            <div class="flex items-center justify-between gap-2 border-t border-line pt-2 text-xs text-muted">
                <span class="truncate">{pattern.course.clone()}</span>
                <button type="button" onclick={inspect} class="inline-flex min-h-9 items-center gap-1 rounded-md px-2 text-ink hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent" aria-label={format!("Inspect {}", pattern.pattern)} title="Details"><Icon data={IconData::LUCIDE_ARROW_UP_RIGHT} width="16px" /></button>
            </div>
        </article>
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct GrammarTableRowProps {
    pub record: GrammarRecord,
    pub selected: bool,
    pub detail_open: bool,
    pub on_toggle: Callback<String>,
    pub on_inspect: Callback<String>,
}

#[function_component(GrammarTableRow)]
pub(super) fn grammar_table_row(props: &GrammarTableRowProps) -> Html {
    let pattern = &props.record;
    let toggle = {
        let id = pattern.id.clone();
        let action = props.on_toggle.clone();
        Callback::from(move |_: ()| action.emit(id.clone()))
    };
    let inspect = {
        let id = pattern.id.clone();
        let action = props.on_inspect.clone();
        Callback::from(move |_: MouseEvent| action.emit(id.clone()))
    };
    html! {
        <tr class="h-14 border-t border-line hover:bg-raised/60">
            <td class="w-14 min-w-14 px-2"><SelectionCheckbox label={format!("Select {}", pattern.pattern)} checked={props.selected} on_toggle={toggle} /></td>
            <th scope="row" class="px-2">
                <button type="button" aria-expanded={props.detail_open.to_string()} aria-label={format!("Inspect grammar pattern {}", pattern.pattern)} onclick={inspect} class="block min-h-10 w-full rounded-md text-left focus-visible:outline-2 focus-visible:outline-accent">
                    <span class="font-semibold underline-offset-4 hover:underline">{pattern.pattern.clone()}</span>
                    if pattern.description != "Grammar pattern" { <span class="block text-xs font-normal text-muted">{pattern.description.clone()}</span> }
                </button>
            </th>
            <td class="px-2 text-muted">{pattern.course.clone()}</td>
            <td class="px-2" lang="zh-Hans">{pattern.example.clone()}</td>
            <td class="px-2 tabular-nums">{pattern.attempts}</td>
        </tr>
    }
}
