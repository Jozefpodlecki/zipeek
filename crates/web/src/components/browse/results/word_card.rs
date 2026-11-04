use yew::prelude::*;
use crate::{components::ListenButton, contexts::database::Dictionary, services::database::{BrowseField, BrowseRelation, BrowseSearch}, reducers::browse::{BrowseAction, BrowseInput, BrowseState, PAGE_SIZE}};
use yew_icons::{Icon, IconData};
use zipeek_core::Word;

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
fn browse_word_relations(props: &BrowseWordRelationsProps) -> Html {
    let dictionary = use_context::<Dictionary>().expect("Dictionary provider is required");
    let browse = use_context::<UseReducerHandle<BrowseState>>().expect("Browse provider is required");
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
                    Ok(database) => database.word_relations(word_id).await.map_err(|error| error.to_string()),
                    Err(error) => Err(error.to_string()),
                };
                if pending.get() { result.set(Some(fetched)); }
            });
            move || active.set(false)
        });
    }
    match &*result {
        Some(Ok(relations)) if !relations.is_empty() => html! {
            <div data-browse-relations="" class="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted">
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
                        <span class="inline-flex items-center gap-1" title={relation.source_annotation.clone()}>
                            {relation_label(&relation.relation_type)}
                            <a class="font-medium text-accent underline decoration-accent/40 underline-offset-4 hover:decoration-accent"
                               lang="zh-Hans" href={link} onclick={on_reference}>{&relation.target_simplified}</a>
                        </span>
                    }
                })}
            </div>
        },
        Some(Err(_)) => html! { <span role="status" class="text-xs text-muted">{"References unavailable"}</span> },
        _ => Html::default(),
    }
}

#[derive(Properties, PartialEq)]
struct BrowseWordRelationsProps {
    word_id: zipeek_core::WordId,
}

#[derive(Properties, PartialEq)]
pub(super) struct BrowseWordCardProps {
    pub word: Word,
    pub included: Option<bool>,
    pub on_toggle: Callback<i32>,
}

#[function_component(BrowseWordCard)]
pub(super) fn browse_word_card(props: &BrowseWordCardProps) -> Html {
    let id = props.word.id.as_i32();
    let traditional = props
        .word
        .traditional
        .as_ref()
        .filter(|traditional| *traditional != &props.word.simplified)
        .map(|traditional| html! {
            <span class="text-xs text-muted" lang="zh-Hant">{traditional}</span>
        })
        .unwrap_or_default();
    let toggle_button = props
        .included
        .map(|included| {
            let on_toggle = props.on_toggle.clone();
            html! {
                <button
                    type="button"
                    data-toggle-saved-word=""
                    class="min-h-9 shrink-0 rounded-lg border border-line px-3 text-xs hover:border-accent"
                    onclick={Callback::from(move |_| on_toggle.emit(id))}
                >
                    <span class="inline-flex items-center gap-1"><Icon data={if included { IconData::LUCIDE_MINUS } else { IconData::LUCIDE_PLUS }} width="15px" height="15px" />{if included { "Remove" } else { "Add to set" }}</span>
                </button>
            }
        })
        .unwrap_or_default();

    html! {
        <li data-component="browse-result" class="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-2 border-b border-line px-3 py-2.5 transition-colors last:border-b-0 hover:bg-raised/50" data-browse-word-id={id.to_string()}>
            <div class="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-1 sm:flex-nowrap">
                <div class="flex min-w-0 basis-full flex-wrap items-baseline gap-x-2 gap-y-0.5 sm:basis-60 sm:shrink-0">
                    <span class="text-lg font-semibold leading-snug" lang="zh-Hans">{&props.word.simplified}</span>
                    <span class="text-sm text-muted">{&props.word.pinyin}</span>
                    {traditional}
                </div>
                <div class="min-w-0 flex-1">
                    <p class="text-sm leading-relaxed">
                        {props.word.english.as_deref().unwrap_or("Meaning unavailable").replace('|', " · ")}
                    </p>
                    <BrowseWordRelations word_id={props.word.id} />
                </div>
            </div>
            <div class="flex shrink-0 items-center gap-1">
                <ListenButton hanzi={props.word.simplified.clone()} compact={true} />
                {toggle_button}
            </div>
        </li>
    }
}
