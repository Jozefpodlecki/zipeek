use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct PracticeCheckboxProps {
    pub checked: bool,
    pub on_change: Callback<bool>,
    pub label: AttrValue,
    #[prop_or_default]
    pub description: Option<AttrValue>,
    #[prop_or_default]
    pub test_id: AttrValue,
}

#[function_component(PracticeCheckbox)]
pub(crate) fn practice_checkbox(props: &PracticeCheckboxProps) -> Html {
    let on_change = {
        let callback = props.on_change.clone();
        Callback::from(move |event: Event| {
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            callback.emit(input.checked());
        })
    };
    html! {
        <label class="group flex cursor-pointer items-start gap-3 rounded-lg p-2 hover:bg-raised/60">
            <span class="relative mt-0.5 size-5 shrink-0">
                <input
                    type="checkbox"
                    class="peer absolute inset-0 size-5 cursor-pointer appearance-none rounded-md border border-line bg-raised transition-colors checked:border-accent checked:bg-brand-action focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                    checked={props.checked}
                    onchange={on_change}
                    data-practice-option={props.test_id.clone()}
                    data-practice-retry-incorrect={(props.test_id.as_str() == "retry").then_some("")}
                    data-practice-show-audio={(props.test_id.as_str() == "audio").then_some("")}
                    data-practice-pinyin-hint={(props.test_id.as_str() == "pinyin-hint").then_some("")}
                    data-practice-auto-check={(props.test_id.as_str() == "auto-check").then_some("")}
                />
                <svg aria-hidden="true" class="pointer-events-none absolute inset-0 m-auto opacity-0 text-white peer-checked:opacity-100" viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12l4 4L19 6" /></svg>
            </span>
            <span class="flex flex-col gap-1">
                <span class="text-sm font-medium text-ink">{props.label.clone()}</span>
                {if let Some(description) = &props.description {
                    html! { <span class="text-xs text-muted">{description.clone()}</span> }
                } else { Html::default() }}
            </span>
        </label>
    }
}
