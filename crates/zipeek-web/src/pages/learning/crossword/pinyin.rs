use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::crossword::PinyinTile;

use super::state::{CrosswordAction, CrosswordState};

pub(super) struct PinyinController {
    pub query: UseStateHandle<String>,
    pub input_ref: NodeRef,
    pub on_input: Callback<InputEvent>,
    pub on_key: Callback<KeyboardEvent>,
    pub matches: Vec<PinyinTile>,
}

#[hook]
pub(super) fn use_crossword_pinyin(
    session: UseReducerHandle<CrosswordState>,
    document: web_sys::Document,
) -> PinyinController {
    let pinyin_input = use_state(String::new);
    let pinyin_ref = use_node_ref();
    {
        let pinyin_input = pinyin_input.clone();
        let pinyin_ref = pinyin_ref.clone();
        let can_type = session.game.as_ref().is_some_and(|game| !game.finished());
        let document = document.clone();
        use_effect_with(can_type, move |can_type| {
            let listener = if *can_type {
                Some(EventListener::new(&document, "keydown", move |raw| {
                    let Some(event) = raw.dyn_ref::<KeyboardEvent>() else {
                        return;
                    };
                    if event.repeat()
                        || event.is_composing()
                        || event.ctrl_key()
                        || event.meta_key()
                        || event.alt_key()
                    {
                        return;
                    }
                    if event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                        .and_then(|element| {
                            element
                                .closest("input, textarea, select, [contenteditable]")
                                .ok()
                                .flatten()
                        })
                        .is_some()
                    {
                        return;
                    }
                    let key = event.key();
                    if key.len() != 1 || !key.chars().all(|ch| ch.is_ascii_alphabetic()) {
                        return;
                    }
                    event.prevent_default();
                    pinyin_input.set(format!("{}{}", *pinyin_input, key.to_ascii_lowercase()));
                    let pinyin_ref = pinyin_ref.clone();
                    gloo::timers::callback::Timeout::new(0, move || {
                        if let Some(input) = pinyin_ref.cast::<web_sys::HtmlInputElement>() {
                            let _ = input.focus();
                        }
                    })
                    .forget();
                }))
            } else {
                None
            };
            move || drop(listener)
        });
    }
    let on_pinyin_input = {
        let pinyin_input = pinyin_input.clone();
        Callback::from(move |event: InputEvent| {
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            pinyin_input.set(input.value());
        })
    };
    let on_pinyin_key = {
        let session = session.clone();
        let pinyin_input = pinyin_input.clone();
        Callback::from(move |event: KeyboardEvent| {
            let Some(game) = session.game.as_ref() else {
                return;
            };
            let (base, typed_tone) = zipeek_exercises::crossword::normalize_pinyin(&pinyin_input);
            let explicit_tone = pinyin_input
                .chars()
                .last()
                .is_some_and(|ch| ('1'..='5').contains(&ch));
            if event.key() == "Escape" {
                event.prevent_default();
                pinyin_input.set(String::new());
                return;
            }
            if base.is_empty() {
                return;
            }
            let matches: Vec<_> = game
                .pinyin_tiles()
                .into_iter()
                .filter(|tile| tile.syllable == base)
                .collect();
            let requested_tone = if event.key().len() == 1 {
                event
                    .key()
                    .parse::<u8>()
                    .ok()
                    .filter(|tone| (1..=5).contains(tone))
            } else {
                None
            };
            if event.key() != "Enter" && requested_tone.is_none() {
                return;
            }
            let candidates: Vec<_> = matches
                .into_iter()
                .filter(|tile| {
                    requested_tone
                        .or(explicit_tone.then_some(typed_tone))
                        .is_none_or(|tone| tile.tone == tone)
                })
                .collect();
            let distinct: std::collections::BTreeSet<_> = candidates
                .iter()
                .map(|tile| (tile.character, tile.tone))
                .collect();
            if distinct.len() == 1 {
                event.prevent_default();
                if let Some(tile) = candidates.first() {
                    session.dispatch(CrosswordAction::SelectTile(tile.bank_index));
                    pinyin_input.set(String::new());
                }
            } else if requested_tone.is_some() && !candidates.is_empty() {
                event.prevent_default();
            }
        })
    };
    let pinyin_matches = session
        .game
        .as_ref()
        .map(|game| {
            let (base, _) = zipeek_exercises::crossword::normalize_pinyin(&pinyin_input);
            let mut found = game
                .pinyin_tiles()
                .into_iter()
                .filter(|tile| !base.is_empty() && tile.syllable.starts_with(&base))
                .collect::<Vec<_>>();
            found.sort_by_key(|tile| (tile.syllable.clone(), tile.tone, tile.character));
            found.dedup_by_key(|tile| (tile.syllable.clone(), tile.tone, tile.character));
            found
        })
        .unwrap_or_default();

    PinyinController {
        query: pinyin_input,
        input_ref: pinyin_ref,
        on_input: on_pinyin_input,
        on_key: on_pinyin_key,
        matches: pinyin_matches,
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct PinyinPopoverProps {
    pub query: String,
    pub input_ref: NodeRef,
    pub on_input: Callback<InputEvent>,
    pub on_key: Callback<KeyboardEvent>,
    pub on_select: Callback<usize>,
    pub on_close: Callback<()>,
    pub matches: Vec<PinyinTile>,
}

#[function_component(PinyinPopover)]
pub(super) fn pinyin_popover(props: &PinyinPopoverProps) -> Html {
    if props.query.is_empty() {
        return Html::default();
    }
    let on_close = {
        let close = props.on_close.clone();
        Callback::from(move |_| close.emit(()))
    };
    let no_matching_hanzi_notice = {
        if props.matches.is_empty() {
            html! { <span
                class="text-sm text-muted"
            >
                {"No matching Hanzi"}
            </span> }
        } else {
            Html::default()
        }
    };
    html! {
        <aside
            class="crossword-pinyin-popover fixed bottom-28 left-1/2 z-50 w-[min(26rem,calc(100vw-1.5rem))] -translate-x-1/2 rounded-2xl border border-line bg-raised p-3 shadow-2xl"
            role="group"
            aria-label="Pinyin character suggestions"
        >
            <div
                class="flex items-center gap-2"
            >
                <Icon
                    data={IconData::LUCIDE_SEARCH}
                    width="18"
                    height="18"
                />
                <input
                    id="crossword-pinyin"
                    ref={props.input_ref.clone()}
                    type="text"
                    autocomplete="off"
                    spellcheck="false"
                    aria-label="Type Pinyin"
                    value={props.query.clone()}
                    oninput={props.on_input.clone()}
                    onkeydown={props.on_key.clone()}
                    class="min-w-0 flex-1 bg-transparent text-base text-ink outline-none"
                />
                <button
                    type="button"
                    aria-label="Close Pinyin suggestions"
                    class="rounded-lg border border-line p-2 text-muted hover:text-ink"
                    onclick={on_close}
                >
                    <Icon
                        data={IconData::LUCIDE_X}
                        width="16"
                        height="16"
                    />
                </button>
            </div>
            <div
                class="mt-3 flex max-h-40 flex-wrap gap-2 overflow-y-auto"
                role="group"
                aria-label="Matching Hanzi"
            >
                {for props.matches.iter().take(12).map(|tile| {
                    let index = tile.bank_index;
                    let on_select = props.on_select.clone();
                    let on_close = props.on_close.clone();
                    let on_select_pinyin = Callback::from(move |_| { on_select.emit(index); on_close.emit(()); });
                    html! { <button
                        type="button"
                        class="rounded-lg border border-line bg-surface px-3 py-2 text-sm hover:border-brand-action focus-visible:outline-2 focus-visible:outline-brand-action"
                        aria-label={format!("Select {}, tone {}", tile.character, tile.tone)}
                        onclick={on_select_pinyin}
                    >
                        <span
                            class="block text-xl font-semibold"
                        >
                            {tile.character}
                        </span>
                        <span
                            class="text-xs text-muted"
                        >
                            {format!("{}{}", tile.syllable, tile.tone)}
                        </span>
                    </button> }
                })}
                {no_matching_hanzi_notice}
            </div>
        </aside>
    }
}
