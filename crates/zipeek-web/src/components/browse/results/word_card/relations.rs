use crate::hooks::use_browse_state;
use crate::hooks::use_dictionary;
use crate::{
    reducers::browse::{BrowseAction, BrowseInput, PAGE_SIZE},
    services::database::{BrowseField, BrowseRelation, BrowseSearch},
};
use yew::prelude::*;

fn relation_label(relation_type: &str) -> &str {
    match relation_type {
        "abbreviation_of" => "Abbreviation of",
        "short_for" => "Short for",
        "variant_of" => "Variant of",
        "also_written_as" => "Also written as",
        "see_also" => "See also",
        _ => "Reference",
    }
}

#[function_component(BrowseWordRelations)]
pub(super) fn browse_word_relations(props: &BrowseWordRelationsProps) -> Html {
    let dictionary = use_dictionary();
    let browse = use_browse_state();
    let result = use_state(|| None::<Result<Vec<BrowseRelation>, String>>);
    {
        let dictionary = dictionary.clone();
        let result = result.clone();
        let word_id = props.word_id;
        use_effect_with(word_id, move |word_id| {
            let word_id = *word_id;
            let active = std::rc::Rc::new(std::cell::Cell::new(true));
            let pending = active.clone();
            result.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let fetched = match dictionary.database() {
                    Ok(database) => database
                        .word_relations(word_id)
                        .await
                        .map_err(|error| error.to_string()),
                    Err(error) => Err(error.to_string()),
                };
                if pending.get() {
                    result.set(Some(fetched));
                }
            });
            move || active.set(false)
        });
    }
    match &*result {
        Some(Ok(relations)) if !relations.is_empty() => html! {
            <div
                data-browse-relations=""
                class="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted"
            >
                {for relations.iter().map(|relation| {
                    let link = format!("#/browse?query={}", js_sys::encode_uri_component(&relation.target_simplified));
                    let browse = browse.clone();
                    let target = relation.target_simplified.clone();
                    let on_reference = Callback::from(move |_| {
                        browse.dispatch(BrowseAction::ClearFilters);
                        browse.dispatch(BrowseAction::Input(BrowseInput::QueryText(target.clone())));
                        browse.dispatch(BrowseAction::Search(BrowseSearch {
                            text: target.clone(), field: BrowseField::Hanzi,
                            limit: PAGE_SIZE, ..Default::default()
                        }));
                    });
                    html! {
                        <span
                            class="inline-flex items-center gap-1"
                            title={relation.source_annotation.clone()}
                        >
                            {relation_label(&relation.relation_type)}
                            <a
                                class="font-medium text-accent underline decoration-accent/40 underline-offset-4 hover:decoration-accent"
                                lang="zh-Hans"
                                href={link}
                                onclick={on_reference}
                            >
                                {&relation.target_simplified}
                            </a>
                        </span>
                    }
                })}
            </div>
        },
        Some(Err(_)) => {
            html! { <span
                role="status"
                class="text-xs text-muted"
            >
                {"References unavailable"}
            </span> }
        }
        _ => Html::default(),
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct BrowseWordRelationsProps {
    pub(super) word_id: zipeek_core::WordId,
}
