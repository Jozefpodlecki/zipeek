use web_sys::{HtmlElement, KeyboardEvent};
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
            class="group inline-flex size-9 shrink-0 items-center justify-center rounded-lg border border-transparent transition-colors hover:bg-raised focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent disabled:opacity-40">
            <span aria-hidden="true"
                class={classes!("inline-flex", "size-[18px]", "shrink-0", "items-center", "justify-center", "rounded-[5px]", "border", "transition-colors", if props.checked { "border-accent bg-brand-action text-white" } else { "border-line bg-surface group-hover:border-accent" })}>
                if props.checked { <Icon data={IconData::LUCIDE_CHECK} width="13px" height="13px" /> }
            </span>
        </button>
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct ChoiceOption {
    pub value: String,
    pub label: String,
}

#[derive(Properties, PartialEq)]
struct ChoiceTriggerProps {
    label: String,
    current_label: String,
    list_id: String,
    open: bool,
    focused_index: usize,
    trigger_ref: NodeRef,
    on_toggle: Callback<MouseEvent>,
}

#[function_component(ChoiceTrigger)]
fn choice_trigger(props: &ChoiceTriggerProps) -> Html {
    html! {
        <>
            <span class="mb-1 block text-xs text-muted">{props.label.clone()}</span>
            <button type="button" ref={props.trigger_ref.clone()} aria-label={props.label.clone()}
                aria-haspopup="listbox" aria-expanded={props.open.to_string()}
                aria-controls={props.list_id.clone()}
                aria-activedescendant={if props.open { Some(format!("{}-{}", props.list_id, props.focused_index)) } else { None }}
                onclick={props.on_toggle.clone()}
                class="relative z-40 inline-flex min-h-10 w-full min-w-24 items-center justify-between gap-3 rounded-lg border border-line bg-surface px-3 text-left text-sm text-ink hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent">
                <span class="truncate">{props.current_label.clone()}</span>
                <Icon data={IconData::LUCIDE_CHEVRON_DOWN} width="15px" />
            </button>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct ChoiceOptionRowProps {
    option: ChoiceOption,
    index: usize,
    id: String,
    selected: bool,
    focused: bool,
    on_select: Callback<usize>,
}

#[function_component(ChoiceOptionRow)]
fn choice_option_row(props: &ChoiceOptionRowProps) -> Html {
    let on_select = props.on_select.clone();
    let index = props.index;
    let select = Callback::from(move |_: MouseEvent| on_select.emit(index));
    html! {
        <button id={props.id.clone()} type="button" role="option"
            aria-selected={props.selected.to_string()} onclick={select}
            class={classes!("flex", "min-h-9", "w-full", "items-center", "rounded-md", "px-3", "text-left", "text-sm", "focus-visible:outline-2", "focus-visible:outline-accent", if props.focused { "bg-raised" } else { "hover:bg-raised" })}>
            {props.option.label.clone()}
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct ChoiceSelectProps {
    pub label: String,
    pub value: String,
    pub options: Vec<ChoiceOption>,
    pub on_change: Callback<String>,
}

#[function_component(ChoiceSelect)]
pub(super) fn choice_select(props: &ChoiceSelectProps) -> Html {
    let open = use_state(|| false);
    let focused = use_state(|| 0_usize);
    let trigger_ref = use_node_ref();
    let list_id = format!("stats-choice-{}", props.label.replace(' ', "-"));
    let current_index = props
        .options
        .iter()
        .position(|option| option.value == props.value)
        .unwrap_or(0);
    let current_label = props
        .options
        .get(current_index)
        .map_or("Choose", |option| option.label.as_str());
    let toggle = {
        let open = open.clone();
        let focused = focused.clone();
        Callback::from(move |_: MouseEvent| {
            focused.set(current_index);
            open.set(!*open);
        })
    };
    let on_keydown = {
        let open = open.clone();
        let focused = focused.clone();
        let options = props.options.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |event: KeyboardEvent| {
            if options.is_empty() {
                return;
            }
            match event.key().as_str() {
                "Escape" | "Tab" => open.set(false),
                "ArrowDown" => {
                    event.prevent_default();
                    if !*open {
                        focused.set(current_index);
                        open.set(true);
                    } else {
                        focused.set((*focused + 1) % options.len());
                    }
                }
                "ArrowUp" => {
                    event.prevent_default();
                    if !*open {
                        focused.set(current_index);
                        open.set(true);
                    } else {
                        focused.set((*focused + options.len() - 1) % options.len());
                    }
                }
                "Home" if *open => {
                    event.prevent_default();
                    focused.set(0);
                }
                "End" if *open => {
                    event.prevent_default();
                    focused.set(options.len() - 1);
                }
                "Enter" | " " => {
                    event.prevent_default();
                    if *open {
                        if let Some(option) = options.get(*focused) {
                            on_change.emit(option.value.clone());
                        }
                        open.set(false);
                    } else {
                        focused.set(current_index);
                        open.set(true);
                    }
                }
                _ => {}
            }
        })
    };
    let dismiss = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(false))
    };
    let on_select = {
        let open = open.clone();
        let focused = focused.clone();
        let options = props.options.clone();
        let on_change = props.on_change.clone();
        let trigger_ref = trigger_ref.clone();
        Callback::from(move |index: usize| {
            if let Some(option) = options.get(index) {
                focused.set(index);
                on_change.emit(option.value.clone());
                open.set(false);
                if let Some(trigger) = trigger_ref.cast::<HtmlElement>() {
                    let _ = trigger.focus();
                }
            }
        })
    };
    html! {
        <div class={classes!("relative", "min-w-0", if *open { "z-50" } else { "" })} onkeydown={on_keydown}>
            if *open { <div class="fixed inset-0 z-30" aria-hidden="true" onclick={dismiss} /> }
            <ChoiceTrigger label={props.label.clone()} current_label={current_label.to_owned()}
                list_id={list_id.clone()} open={*open} focused_index={*focused}
                trigger_ref={trigger_ref.clone()} on_toggle={toggle} />
            if *open {
                <div id={list_id.clone()} role="listbox" aria-label={props.label.clone()}
                    class="absolute left-0 top-full z-40 mt-1 max-h-64 min-w-full overflow-y-auto rounded-lg border border-line bg-surface p-1 shadow-xl">
                    {for props.options.iter().enumerate().map(|(index, option)| html! {
                        <ChoiceOptionRow key={option.value.clone()} option={option.clone()}
                            id={format!("{list_id}-{index}")} index={index}
                            selected={option.value == props.value} focused={*focused == index}
                            on_select={on_select.clone()} />
                    })}
                </div>
            }
        </div>
    }
}
