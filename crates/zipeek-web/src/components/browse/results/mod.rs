mod pagination;
mod status;
mod word_card;

use crate::hooks::use_browse_state;
use crate::hooks::use_word_sets;
use yew::prelude::*;

use crate::{
    components::WordSetsCommand,
    reducers::browse::{BrowseAction, PAGE_SIZE, SearchState},
};

use self::{
    pagination::BrowsePagination,
    status::{BrowseEmpty, BrowseError, BrowseLoading},
    word_card::BrowseWordCard,
};
use super::search_form::submit_search;

#[function_component(BrowseResults)]
pub(crate) fn browse_results() -> Html {
    let state = use_browse_state();
    let sets = use_word_sets();

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
    let selected_set = sets.state.target.as_ref().and_then(|id| {
        sets.state
            .sets
            .as_ref()
            .ok()
            .and_then(|saved| crate::services::word_sets::find(saved, id))
    });

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
                <div
                    data-browse-results=""
                    class="flex flex-col gap-3"
                >
                    <ul
                        data-component="browse-results-list"
                        aria-label="Dictionary search results"
                        class="overflow-hidden rounded-xl border border-line bg-surface"
                    >
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
                        visible_count={words.len().min(PAGE_SIZE as usize) as u32}
                        on_previous={previous}
                        on_next={next}
                    />
                </div>
            }
        }
    };

    html! {
        <section
            data-component="browse-results"
            class="flex flex-col gap-3"
        >
            {status_component}
        </section>
    }
}
