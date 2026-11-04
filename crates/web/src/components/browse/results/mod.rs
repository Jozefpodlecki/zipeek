mod pagination;
mod status;
mod word_card;

use yew::prelude::*;

use crate::{
    components::{WordSetsCommand, WordSetsContext},
    reducers::browse::{BrowseAction, BrowseState, SearchState, PAGE_SIZE},
};

use self::{
    pagination::BrowsePagination,
    status::{BrowseEmpty, BrowseError, BrowseLoading},
    word_card::BrowseWordCard,
};
use super::search_form::submit_search;

#[function_component(BrowseResults)]
pub(crate) fn browse_results() -> Html {
    let state = use_context::<UseReducerHandle<BrowseState>>().unwrap();
    let sets = use_context::<WordSetsContext>().unwrap();

    let previous = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(BrowseAction::Previous))
    };
    let next = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(BrowseAction::Next))
    };
    let retry = {
        let state = state.clone();
        let sets = sets.clone();
        Callback::from(move |_| submit_search(&state, &sets.state.sets, false))
    };
    let on_toggle = {
        let execute = sets.execute.clone();
        Callback::from(move |id: i32| execute.emit(WordSetsCommand::Toggle(id)))
    };
    let selected_set = sets
        .state
        .target
        .parse::<usize>()
        .ok()
        .and_then(|index| sets.state.sets.as_ref().ok().and_then(|saved| saved.get(index)));

    let status_component = match &state.status {
        SearchState::Loading => html! { <BrowseLoading /> },
        SearchState::Failed(message) => html! {
            <BrowseError
                message={message.clone()}
                on_retry={retry}
                can_retry={state.blocked.is_none()}
                show_regeneration_tip={state.blocked.is_none()}
            />
        },
        SearchState::Ready(words) if words.is_empty() => html! { <BrowseEmpty /> },
        SearchState::Ready(words) => {
            let has_next = words.len() > PAGE_SIZE as usize;
            html! {
                <div data-browse-results="" class="flex flex-col gap-3">
                    <p role="status" class="text-sm text-muted">
                        {format!("Showing {}–{}", state.offset + 1, state.offset + words.len().min(PAGE_SIZE as usize) as u32)}
                    </p>
                    <ul data-component="browse-results-list" class="overflow-hidden rounded-xl border border-line bg-surface">
                        {for words.iter().take(PAGE_SIZE as usize).map(|word| {
                            let included = selected_set.map(|set| {
                                set.word_ids.binary_search(&word.id.as_i32()).is_ok()
                            });
                            html! {
                                <BrowseWordCard
                                    key={word.id.as_i32().to_string()}
                                    word={word.clone()}
                                    included={included}
                                    on_toggle={on_toggle.clone()}
                                />
                            }
                        })}
                    </ul>
                    <BrowsePagination
                        offset={state.offset}
                        has_next={has_next}
                        on_previous={previous}
                        on_next={next}
                    />
                </div>
            }
        }
    };

    html! {
        <section data-component="browse-results" class="flex flex-col gap-3">
            {status_component}
        </section>
    }
}
