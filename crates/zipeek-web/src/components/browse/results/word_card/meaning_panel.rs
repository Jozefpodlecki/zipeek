use crate::components::home::word_spotlight::MeaningNavigator;
use crate::hooks::use_dictionary;
use crate::services::database::DictionaryMeaning;
use yew::prelude::*;
use zipeek_core::WordId;

#[derive(Properties, PartialEq)]
pub(super) struct BrowseMeaningPanelProps {
    pub word_id: WordId,
}

#[function_component(BrowseMeaningPanel)]
pub(super) fn browse_meaning_panel(props: &BrowseMeaningPanelProps) -> Html {
    let dictionary = use_dictionary();
    let result = use_state(|| None::<Result<Vec<DictionaryMeaning>, String>>);
    let selected = use_state(|| 0_usize);
    {
        let dictionary = dictionary.clone();
        let result = result.clone();
        let selected = selected.clone();
        use_effect_with(props.word_id, move |word_id| {
            let word_id = *word_id;
            let alive = std::rc::Rc::new(std::cell::Cell::new(true));
            let pending = alive.clone();
            result.set(None);
            selected.set(0);
            wasm_bindgen_futures::spawn_local(async move {
                let response = match dictionary.database() {
                    Ok(database) => database
                        .word_meanings(word_id)
                        .await
                        .map_err(|error| error.to_string()),
                    Err(error) => Err(error.to_string()),
                };
                if pending.get() {
                    result.set(Some(response));
                }
            });
            move || alive.set(false)
        });
    }
    let on_select = {
        let selected = selected.clone();
        Callback::from(move |index| selected.set(index))
    };
    html! {
        <div
            data-browse-sense-panel=""
            class="mt-3 min-w-0"
        >
            {
                match &*result {
                    None => html! { <p role="status" class="text-xs text-muted">{"Loading meanings"}</p> },
                    Some(Err(error)) => html! { <p role="alert" class="text-xs text-red-500">{error}</p> },
                    Some(Ok(meanings)) if meanings.is_empty() => html! { <p class="text-xs text-muted">{"No individual meanings available"}</p> },
                    Some(Ok(meanings)) => html! {
                        <MeaningNavigator
                            meanings={meanings.clone()}
                            selected={*selected}
                            on_select={on_select}
                        />
                    },
                }
            }
        </div>
    }
}
