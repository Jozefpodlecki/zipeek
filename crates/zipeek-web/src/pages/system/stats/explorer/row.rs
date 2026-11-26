use super::super::{controls::SelectionCheckbox, data::WordRecord, fixture::EvidenceStatus};
use yew::prelude::*;

fn evidence_icon(status: EvidenceStatus) -> Html {
    let (symbol, class) = match status {
        EvidenceStatus::Observed => ("✓", "text-accent"),
        EvidenceStatus::NeedsReview => ("!", "text-amber-400"),
        EvidenceStatus::NeedsEvidence => ("–", "text-muted"),
    };
    html! { <span title={status.label()} aria-label={status.label()} class={classes!("inline-flex", "min-w-7", "justify-center", "font-semibold", class)}>{symbol}</span> }
}

#[derive(Properties, PartialEq)]
pub(super) struct VocabularyRowProps {
    pub record: WordRecord,
    pub selected: bool,
    pub on_toggle: Callback<String>,
}

#[function_component(VocabularyRow)]
pub(super) fn vocabulary_row(props: &VocabularyRowProps) -> Html {
    let record = &props.record;
    let toggle = {
        let id = record.id.clone();
        let on_toggle = props.on_toggle.clone();
        Callback::from(move |_: ()| on_toggle.emit(id.clone()))
    };
    html! {
        <tr class="h-14 border-t border-line hover:bg-raised/60">
            <td class="w-14 min-w-14 px-2"><SelectionCheckbox label={format!("Select {}", record.hanzi)} checked={props.selected} on_toggle={toggle} /></td>
            <th scope="row" class="px-2"><span lang="zh-Hans" class="font-semibold">{record.hanzi.clone()}</span><span class="block truncate text-xs font-normal text-muted">{record.pinyin.clone()}{" · "}{record.meaning.clone()}</span></th>
            <td class="px-2 tabular-nums">{record.encounters}</td>
            <td class="px-2 tabular-nums">{record.days}</td>
            <td class="px-2">{evidence_icon(record.recognition)}</td>
            <td class="px-2">{evidence_icon(record.listening)}</td>
            <td class="px-2">{evidence_icon(record.speaking)}</td>
            <td class="px-2">{evidence_icon(record.writing)}</td>
            <td class="px-2">{evidence_icon(record.sentence)}</td>
        </tr>
    }
}
