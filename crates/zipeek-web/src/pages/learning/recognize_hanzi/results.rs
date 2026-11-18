use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::{
    components::ListenButton,
    components::providers::word_sets_provider::WordSetsCommand,
    hooks::use_word_sets,
};

use super::{RecognitionHit, DictionaryMatch};

#[derive(Properties, PartialEq)]
pub(super) struct RecognitionResultsProps {
    pub hits: Vec<RecognitionHit>,
    pub panel_ref: NodeRef,
    pub busy: bool,
}

#[derive(Clone, PartialEq)]
struct SaveAction {
    callback: Callback<MouseEvent>,
    title: String,
    saved: bool,
}

#[derive(Properties, PartialEq)]
struct CandidateProps {
    hit: RecognitionHit,
    save_action: Option<SaveAction>,
}

#[function_component(CandidateRow)]
fn candidate_row(props: &CandidateProps) -> Html {
    let hit = &props.hit;
    let dictionary = hit.dictionary.as_ref();
    let search = format!("#/browse?query={}", js_sys::encode_uri_component(&hit.candidate));
    let save = props.save_action.as_ref();
    html! {
        <article class="flex min-w-0 items-center gap-3 border-b border-line px-2 py-2 last:border-b-0" data-recognition-candidate="">
            <span class="flex size-12 shrink-0 items-center justify-center rounded-lg bg-raised text-2xl text-ink" lang="zh-CN">{hit.candidate.clone()}</span>
            <div class="min-w-0 flex-1">
                {if let Some(word) = dictionary {
                    html! {
                        <>
                            <p class="truncate text-sm font-semibold text-ink">{word.pinyin.clone()}</p>
                            <p class="line-clamp-2 text-xs leading-snug text-muted" title={word.english.clone()}>{word.english.clone()}</p>
                        </>
                    }
                } else {
                    html! {<p class="text-xs text-muted">{"Not found in dictionary"}</p>}
                }}
            </div>
            <div class="flex shrink-0 items-center gap-1">
                if dictionary.is_some() {
                    <ListenButton hanzi={hit.candidate.clone()} compact={true} />
                }
                <a href={search} title="Open dictionary" aria-label={format!("Open dictionary for {}", hit.candidate)} class="inline-flex size-11 items-center justify-center rounded-lg text-ink hover:bg-raised focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand-action">
                    <Icon data={IconData::LUCIDE_BOOK_OPEN} width="18px" height="18px" />
                </a>
                if let Some(DictionaryMatch { id, .. }) = dictionary {
                    <a href={format!("#/write-hanzi?word_id={id}")} title="Practise writing" aria-label={format!("Practise writing {}", hit.candidate)} class="inline-flex size-11 items-center justify-center rounded-lg text-ink hover:bg-raised focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand-action">
                        <Icon data={IconData::LUCIDE_PENCIL} width="18px" height="18px" />
                    </a>
                }
                if let Some(action) = save {
                    <button type="button" onclick={action.callback.clone()} title={action.title.clone()} aria-label={action.title.clone()} aria-pressed={action.saved.to_string()} class="inline-flex size-11 items-center justify-center rounded-lg text-ink hover:bg-raised focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand-action">
                        <Icon data={if action.saved { IconData::LUCIDE_BOOKMARK } else { IconData::LUCIDE_BOOKMARK_PLUS }} width="18px" height="18px" />
                    </button>
                }
            </div>
        </article>
    }
}

#[function_component(RecognitionResults)]
pub(super) fn recognition_results(props: &RecognitionResultsProps) -> Html {
    let sets = use_word_sets();
    let selected_set = sets.state.target.as_ref().and_then(|target| {
        sets.state.sets.as_ref().ok().and_then(|items| crate::services::word_sets::find(items, target))
    }).cloned();
    let on_add_word = {
        let sets = sets.clone();
        Callback::from(move |id: i32| sets.execute.emit(WordSetsCommand::Toggle(id)))
    };
    html! {
        <section ref={props.panel_ref.clone()} class="flex min-w-0 min-h-0 flex-col scroll-mt-4 rounded-2xl border border-line bg-surface p-3 shadow-sm lg:h-full lg:max-h-[calc(100dvh-10rem)]" aria-label="Recognition candidates" aria-busy={props.busy.to_string()} data-recognition-results="">
            <div class="flex items-center justify-between gap-2 pb-2">
                <h2 class="text-base font-semibold text-ink">{"Suggestions"}</h2>
                if props.busy {
                    <span role="status" aria-label="Recognizing handwriting" class="inline-flex size-8 items-center justify-center text-brand-action">
                        <Icon data={IconData::LUCIDE_REFRESH_CW} width="20px" height="20px" class="animate-spin motion-reduce:animate-none" />
                        <span class="sr-only">{"Recognizing handwriting"}</span>
                    </span>
                }
            </div>
            <div class="min-h-0 flex-1 lg:overflow-y-auto" data-recognition-candidates="">
                {for props.hits.iter().cloned().map(|hit| {
                    let save_action = hit.dictionary.as_ref().and_then(|word| selected_set.as_ref().map(|set| {
                        let word_id = word.id;
                        let saved = set.word_ids.contains(&word_id);
                        let title = if saved { format!("Remove from {}", set.name) } else { format!("Save to {}", set.name) };
                        SaveAction { callback: on_add_word.reform(move |_: MouseEvent| word_id), title, saved }
                    }));
                    let candidate_key = hit.candidate.clone();
                    html! { <CandidateRow key={candidate_key} hit={hit} save_action={save_action} /> }
                })}
            </div>
            if !props.hits.is_empty() {
                <p class="sr-only" role="status">{format!("{} recognition suggestions available", props.hits.len())}</p>
            }
        </section>
    }
}
