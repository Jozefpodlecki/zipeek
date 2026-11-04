use yew::prelude::*;

#[function_component(BrowseLoading)]
pub(super) fn browse_loading() -> Html {
    html! {
        <p role="status" data-browse-loading="" data-component="browse-loading">
            {"Searching dictionary…"}
        </p>
    }
}

#[function_component(BrowseEmpty)]
pub(super) fn browse_empty() -> Html {
    html! {
        <p data-browse-empty="" data-component="browse-empty" role="status" class="rounded-lg border border-line p-5">
            {"No matching words. Try removing a filter or changing the search."}
        </p>
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct BrowseErrorProps {
    pub message: String,
    pub on_retry: Callback<MouseEvent>,
    pub can_retry: bool,
    pub show_regeneration_tip: bool,
}

#[function_component(BrowseError)]
pub(super) fn browse_error(props: &BrowseErrorProps) -> Html {
    let regeneration_tip = if !props.show_regeneration_tip {
        Html::default()
    } else {
        html! {
            <p class="text-sm text-muted">
                {"If the dictionary is missing Browse tables, run cargo make db-generate, db-install and db-browse-check, then reload."}
            </p>
        }
    };
    let retry_button = if !props.can_retry {
        Html::default()
    } else {
        html! {
            <button
                type="button"
                class="min-h-11 rounded border border-line px-4"
                onclick={props.on_retry.clone()}
            >
                {"Retry"}
            </button>
        }
    };

    html! {
        <div role="alert" data-browse-error="" data-component="browse-error" class="flex flex-col gap-3 rounded-lg border border-line p-4">
            <p>{format!("Search failed: {}", props.message)}</p>
            {regeneration_tip}
            {retry_button}
        </div>
    }
}
