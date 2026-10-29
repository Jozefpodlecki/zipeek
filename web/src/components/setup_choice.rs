use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SetupChoiceProps {
    pub selected: bool,
    pub on_select: Callback<()>,
    pub label: AttrValue,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub value: AttrValue,
}

#[function_component(SetupChoice)]
pub fn setup_choice(props: &SetupChoiceProps) -> Html {
    let select = props.on_select.clone();
    html! {
        <button type="button" aria-pressed={props.selected.to_string()} data-setup-choice={props.value.clone()} onclick={Callback::from(move |_| select.emit(()))}
            class={classes!("flex", "min-h-11", "min-w-0", "flex-col", "items-start", "gap-1", "rounded-xl", "border", "px-4", "py-3", "text-left", "transition-colors", "motion-reduce:transition-none", if props.selected {"border-accent"} else {"border-line"}, if props.selected {"bg-raised"} else {"bg-surface"}, if props.selected {"text-accent"} else {"text-ink"}, (!props.selected).then_some("hover:bg-raised"))}>
            <span class="flex items-center gap-2 font-medium">{props.label.clone()}</span>
            {props.children.clone()}
        </button>
    }
}
