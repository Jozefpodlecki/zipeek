use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct SelectionCheckboxProps {
    pub label: String,
    pub checked: bool,
    #[prop_or_default]
    pub disabled: bool,
    pub on_toggle: Callback<()>,
}

#[function_component(SelectionCheckbox)]
pub(super) fn selection_checkbox(props: &SelectionCheckboxProps) -> Html {
    let on_toggle = props.on_toggle.clone();
    let onclick = Callback::from(move |_: MouseEvent| on_toggle.emit(()));
    html! {
        <button type="button" role="checkbox" aria-label={props.label.clone()}
            aria-checked={props.checked.to_string()} disabled={props.disabled} onclick={onclick}
            class={classes!("inline-flex", "size-9", "shrink-0", "items-center", "justify-center", "rounded-md", "border", "transition-colors", "focus-visible:outline-2", "focus-visible:outline-accent", "disabled:opacity-40", if props.checked { "border-accent bg-brand-action text-white" } else { "border-line bg-raised text-ink hover:border-accent" })}>
            if props.checked { <Icon data={IconData::LUCIDE_CHECK} width="17px" height="17px" /> }
        </button>
    }
}
