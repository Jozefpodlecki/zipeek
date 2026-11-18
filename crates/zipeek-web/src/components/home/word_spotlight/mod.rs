mod data;
mod meaning_navigator;

use crate::hooks::use_dictionary;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use yew_icons::{Icon, IconData};
use zipeek_core::{HskLevel, Word};

use crate::{
    components::{ListenButton, ToastBus},
    services::ai,
};

mod relations;
pub(crate) use meaning_navigator::MeaningNavigator;
use relations::relation_label;


#[derive(Properties, PartialEq)]
pub struct WordSpotlightProps {
    #[prop_or_default]
    pub daily: bool,
}

#[function_component(WordSpotlight)]
pub fn word_spotlight(props: &WordSpotlightProps) -> Html {
    let dictionary = use_dictionary();
    let day = chrono::Utc::now().timestamp().div_euclid(86_400) as u32;
    let refresh = use_state(|| 0_u32);
    let result = use_state(|| {
        None::<
            Result<
                (
                    Word,
                    Vec<crate::services::database::DictionaryMeaning>,
                    Vec<crate::services::database::BrowseRelation>,
                ),
                String,
            >,
        >
    });
    let meaning_index = use_state(|| 0_usize);
    {
        let dictionary = dictionary.clone();
        let meaning_index = meaning_index.clone();
        let result = result.clone();
        let day = if props.daily { Some(day) } else { None };
        use_effect_with((day, *refresh), move |(day, _)| {
            let day = *day;
            let active = std::rc::Rc::new(std::cell::Cell::new(true));
            let pending = active.clone();
            result.set(None);
            meaning_index.set(0);
            wasm_bindgen_futures::spawn_local(async move {
                let loaded = data::load(&dictionary, day).await;
                if pending.get() {
                    result.set(Some(loaded));
                }
            });
            move || active.set(false)
        });
    }
    let toast = use_context::<ToastBus>();
    let on_refresh = {
        let refresh = refresh.clone();
        let meaning_index = meaning_index.clone();
        Callback::from(move |_| {
            meaning_index.set(0);
            refresh.set((*refresh).wrapping_add(1));
        })
    };
    let spotlight_content = {
        match &*result {
            None => html! {
            <p
                role="status"
                class="py-16 text-center text-sm text-muted"
            >
                {"Loading vocabulary…"}
            </p>},
            Some(Err(error)) => html! {
            <p
                role="alert"
                class="py-16 text-center text-sm text-red-500"
            >
                {error}
            </p>},
            Some(Ok((word, meanings, relations))) => {
                let link = format!(
                    "#/browse?query={}",
                    js_sys::encode_uri_component(&word.simplified)
                );
                let writing_link = format!("#/write-hanzi?word_id={}", word.id.as_i32());
                let on_copy = {
                    let hanzi = word.simplified.clone();
                    let toast = toast.clone();
                    Callback::from(move |_| {
                        let hanzi = hanzi.clone();
                        let toast = toast.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match zipeek_browser::clipboard::copy_hanzi(&hanzi).await {
                                Ok(()) => {
                                    if let Some(toast) = toast {
                                        toast.success("Hanzi copied to clipboard");
                                    }
                                }
                                Err(_) => {
                                    if let Some(toast) = toast {
                                        toast.error("Could not copy. Check clipboard permissions.");
                                    }
                                }
                            }
                        });
                    })
                };
                let meaning = meanings.get((*meaning_index).min(meanings.len().saturating_sub(1)));
                let on_select_meaning = {
                    let meaning_index = meaning_index.clone();
                    Callback::from(move |index: usize| {
                        meaning_index.set(index);
                    })
                };
                let ai_configured = !ai::configuration().model.trim().is_empty();
                let sentences_link = format!(
                    "#/sentences?word={}&pinyin={}&meaning={}",
                    js_sys::encode_uri_component(&word.simplified),
                    js_sys::encode_uri_component(&word.pinyin),
                    js_sys::encode_uri_component(
                        &meaning
                            .map(|sense| sense.definition.as_str())
                            .unwrap_or_default()
                    ),
                );
                let cross_reference_links = {
                    if !relations.is_empty() {
                        html! {
                            <div
                                data-word-cross-reference=""
                                class="flex flex-wrap items-center justify-center gap-x-4 gap-y-1 text-sm text-muted"
                            >
                                {for relations.iter().map(|reference| {
                                    let reference_link = format!("#/browse?query={}", js_sys::encode_uri_component(&reference.target_simplified));
                                    html! {
                                        <span
                                            class="inline-flex items-center gap-1"
                                            title={reference.source_annotation.clone()}
                                        >
                                            {relation_label(&reference.relation_type)}{" "}
                                            <a
                                                href={reference_link}
                                                lang="zh-Hans"
                                                class="font-medium text-accent underline decoration-accent/40 underline-offset-4 hover:decoration-accent"
                                                title="Look up this expression in Dictionary"
                                            >
                                                {&reference.target_simplified}
                                            </a>
                                        </span>
                                    }
                                })}
                            </div>
                        }
                    } else {
                        Html::default()
                    }
                };
                let spotlight_refresh_action = if !props.daily {
                    html! {
                        <button
                            type="button"
                            data-next-random-word=""
                            onclick={on_refresh}
                            class="inline-flex min-h-10 items-center justify-center gap-2 rounded-lg bg-brand-action px-4 text-sm font-semibold text-white transition-colors hover:bg-brand-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                        >
                            <Icon
                                data={IconData::LUCIDE_DICE_5}
                                width="18px"
                                height="18px"
                            />{"Surprise me"}
                        </button>
                    }
                } else {
                    Html::default()
                };
                let hsk2_word_level = {
                    if let Some(level) = word.hsk2_level {
                        html! {
                        <span
                            class="rounded-full border border-line bg-raised px-2.5 py-1 text-xs font-medium"
                        >
                            {HskLevel::Hsk2(level).badge_label()}
                        </span>}
                    } else {
                        Html::default()
                    }
                };
                let hsk3_word_level = {
                    if let Some(level) = word.hsk3_level {
                        html! {
                        <span
                            class="rounded-full border border-line bg-raised px-2.5 py-1 text-xs font-medium"
                        >
                            {HskLevel::Hsk3(level).badge_label()}
                        </span>}
                    } else {
                        Html::default()
                    }
                };
                let word_display_meaning = html! {
                    <MeaningNavigator
                        meanings={meanings.clone()}
                        selected={*meaning_index}
                        on_select={on_select_meaning}
                    />
                };
                let traditional_word_variant = {
                    if let Some(traditional) = word
                        .traditional
                        .as_ref()
                        .filter(|value| *value != &word.simplified)
                    {
                        html! {
                        <span
                            class="text-sm text-muted"
                            lang="zh-Hant"
                        >
                            {traditional}
                        </span>}
                    } else {
                        Html::default()
                    }
                };
                let action_grid_classes = classes!(
                    "mt-2",
                    "grid",
                    "w-full",
                    "grid-cols-1",
                    "gap-2",
                    if props.daily {
                        "sm:grid-cols-3"
                    } else {
                        "sm:grid-cols-4"
                    },
                );
                html! {
                    <div
                        class="mx-auto flex w-full max-w-2xl flex-col items-center gap-3 text-center"
                    >
                        <div
                            class="flex items-center justify-center gap-4"
                        >
                            <strong
                                class="text-5xl font-semibold tracking-wide sm:text-6xl"
                                lang="zh-Hans"
                            >
                                {&word.simplified}
                            </strong>
                            <div
                                class="inline-flex items-center gap-1"
                                role="group"
                                aria-label="Word actions"
                            >
                                <ListenButton
                                    hanzi={word.simplified.clone()}
                                />
                                <button
                                    type="button"
                                    data-copy-hanzi=""
                                    onclick={on_copy}
                                    title="Copy Chinese characters"
                                    aria-label="Copy Chinese characters"
                                    class="inline-flex min-h-10 min-w-10 items-center justify-center rounded-lg border border-line bg-raised text-accent hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                                >
                                    <Icon
                                        data={IconData::LUCIDE_COPY}
                                        width="19px"
                                        height="19px"
                                    />
                                </button>
                            </div>
                        </div>
                        {traditional_word_variant}
                        <span
                            class="text-lg font-semibold text-accent"
                        >
                            {&word.pinyin}
                        </span>
                        {word_display_meaning}
                        {cross_reference_links}
                        <div
                            class="mt-1 flex flex-wrap items-center justify-center gap-1.5"
                            data-word-metadata=""
                        >
                            {hsk3_word_level}
                            {hsk2_word_level}
                            {for word.part_of_speech.iter().map(|pos| html! {
                                <span
                                    class="rounded-full border border-line bg-raised px-2.5 py-1 text-xs font-medium"
                                >
                                    {pos.as_str().replace('_', " ")}
                                </span>})}
                        </div>
                        <div
                            class={action_grid_classes}
                            data-spotlight-actions=""
                        >
                            {spotlight_refresh_action}
                            <a
                                class="inline-flex min-h-10 items-center justify-center gap-2 rounded-lg border border-line bg-raised px-4 text-sm font-semibold text-ink transition-colors hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                                href={link}
                            >
                                <Icon
                                    data={IconData::LUCIDE_BOOK_OPEN}
                                    width="18px"
                                    height="18px"
                                />{"Dictionary"}
                            </a>
                            <a
                                data-write-hanzi-action=""
                                class="inline-flex min-h-10 items-center justify-center gap-2 rounded-lg border border-line bg-raised px-4 text-sm font-semibold text-ink transition-colors hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                                href={writing_link}
                            >
                                <Icon
                                    data={IconData::LUCIDE_FILE_TEXT}
                                    width="18px"
                                    height="18px"
                                />{"Write Hanzi"}
                            </a>
                            if ai_configured {
                                <a
                                    data-example-sentence=""
                                    href={sentences_link}
                                    class="inline-flex min-h-10 items-center justify-center gap-2 rounded-lg border border-line bg-raised px-4 text-sm font-semibold text-ink no-underline hover:border-accent focus-visible:outline-2 focus-visible:outline-accent"
                                >
                                    <Icon data={IconData::LUCIDE_FILE_TEXT} width="16px" height="16px" />
                                    {"Sentences"}
                                </a>
                            } else {
                                <span
                                    data-sentences-unavailable=""
                                    title="Choose an AI model in Settings to generate sentences"
                                    role="note"
                                    aria-label="Sentences unavailable until an AI model is configured"
                                    class="inline-flex min-h-10 cursor-not-allowed items-center justify-center gap-2 rounded-lg border border-line bg-raised px-4 text-sm font-semibold text-muted opacity-50"
                                >
                                    <Icon data={IconData::LUCIDE_FILE_TEXT} width="16px" height="16px" />
                                    {"Sentences"}
                                </span>
                            }
                        </div>
                    </div>
                }
            }
        }
    };
    html! {
        <section
            data-component="word-spotlight"
            data-daily-word={if props.daily {"available"} else {"random"}}
            class="relative w-full rounded-2xl border border-line bg-surface/95 px-4 py-5 shadow-lg sm:px-7 sm:py-6"
        >
            {spotlight_content}
        </section>
    }
}
