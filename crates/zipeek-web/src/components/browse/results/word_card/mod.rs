mod meaning_panel;
mod relations;
use self::meaning_panel::BrowseMeaningPanel;
use self::relations::BrowseWordRelations;
use crate::{components::ListenButton, route::Route};
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;
use zipeek_core::Word;

#[derive(Properties, PartialEq)]
pub(super) struct BrowseWordCardProps {
    pub word: Word,
    pub included: Option<bool>,
    pub on_toggle: Callback<i32>,
}

#[function_component(BrowseWordCard)]
pub(super) fn browse_word_card(props: &BrowseWordCardProps) -> Html {
    let id = props.word.id.as_i32();
    let expanded = use_state(|| false);
    let definitions = props
        .word
        .english
        .as_deref()
        .unwrap_or("Meaning unavailable")
        .split('|')
        .map(str::trim)
        .filter(|definition| !definition.is_empty())
        .collect::<Vec<_>>();
    let has_more = !definitions.is_empty();
    let displayed_meanings = if definitions.is_empty() {
        "Meaning unavailable".to_owned()
    } else {
        definitions
            .iter()
            .take(2)
            .copied()
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let on_toggle_meanings = {
        let expanded = expanded.clone();
        Callback::from(move |_| expanded.set(!*expanded))
    };
    let traditional = props
        .word
        .traditional
        .as_ref()
        .filter(|traditional| *traditional != &props.word.simplified)
        .map(|traditional| {
            html! {
                <span
                    class="text-xs text-muted"
                    lang="zh-Hant"
                >
                    {traditional}
                </span>
            }
        })
        .unwrap_or_default();
    let toggle_button = if let Some(included) = props.included {
        let on_toggle = props.on_toggle.clone();
        let on_toggle_saved_word = Callback::from(move |_| on_toggle.emit(id));
        let label = if included {
            "Remove from Word Set"
        } else {
            "Add to Word Set"
        };
        html! {
            <button
                type="button"
                data-toggle-saved-word=""
                aria-label={label}
                title={label}
                aria-pressed={included.to_string()}
                class="grid size-10 shrink-0 place-items-center rounded-lg border border-line text-accent hover:border-accent hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent"
                onclick={on_toggle_saved_word}
            >
                <Icon
                    data={if included { IconData::LUCIDE_BOOKMARK_MINUS } else { IconData::LUCIDE_BOOKMARK_PLUS }}
                    width="18px"
                    height="18px"
                />
            </button>
        }
    } else {
        html! {
            <span data-choose-word-set="" title="Choose a Word Set to save this word">
            <Link<Route>
                to={Route::WordSets}
                classes="grid size-10 shrink-0 place-items-center rounded-lg border border-line text-muted hover:border-accent hover:text-accent focus-visible:outline-2 focus-visible:outline-accent"
            >
                <span class="sr-only">{"Choose a Word Set to save this word"}</span>
                <Icon data={IconData::LUCIDE_BOOKMARK_PLUS} width="18px" height="18px" />
            </Link<Route>>
            </span>
        }
    };

    html! {
        <li
            data-component="browse-result"
            class="flex min-w-0 flex-wrap items-start gap-x-4 gap-y-2 border-b border-line px-4 py-3 transition-colors duration-150 last:border-b-0 hover:bg-raised/60 motion-reduce:transition-none"
            data-browse-word-id={id.to_string()}
        >
            <div
                class="grid min-w-0 flex-1 grid-cols-1 items-start gap-x-5 gap-y-2 sm:grid-cols-[minmax(0,14rem)_minmax(0,1fr)]"
            >
                <div
                    class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1"
                >
                    <span
                        class="text-2xl font-semibold leading-tight sm:text-[1.65rem]"
                        data-browse-headword=""
                        lang="zh-Hans"
                    >
                        {&props.word.simplified}
                    </span>
                    <span
                        class="text-sm text-muted"
                    >
                        {&props.word.pinyin}
                    </span>
                    <ListenButton
                        hanzi={props.word.simplified.clone()}
                        compact={true}
                    />
                    {traditional}
                </div>
                <div
                    class="min-w-0"
                >
                    <p
                        id={format!("browse-meanings-{id}")}
                        class="text-sm leading-relaxed"
                    >
                        {displayed_meanings}
                    </p>
                    {
                        if has_more {
                            html! {
                                <button
                                    type="button"
                                    aria-expanded={(*expanded).to_string()}
                                    aria-controls={format!("browse-meanings-{id}")}
                                    data-toggle-definitions=""
                                    class="mt-1 min-h-8 rounded-md px-1 text-left text-xs font-semibold text-accent transition-colors hover:bg-accent-soft focus-visible:outline-2 focus-visible:outline-accent motion-reduce:transition-none"
                                    onclick={on_toggle_meanings}
                                >
                                    {if *expanded { "Hide meanings" } else { "Explore meanings" }}
                                </button>
                            }
                        } else {
                            Html::default()
                        }
                    }
                    if *expanded {
                        <BrowseMeaningPanel
                            word_id={props.word.id}
                        />
                    }
                    <BrowseWordRelations
                        word_id={props.word.id}
                    />
                </div>
            </div>
            <div
                class="flex shrink-0 items-center gap-1"
            >
                {toggle_button}
            </div>
        </li>
    }
}
