use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::waveform::{Waveform, use_playback_progress};
use zipeek_browser::local_speech;
use crate::pages::record::controller::ReferenceStatus;
use crate::pages::record::readings::ReadingGlyph;
use crate::pages::record::state::{Playback, RecordState, RecordingStage, TargetMode};

#[derive(Properties, PartialEq)]
pub(super) struct ReferencePanelProps {
    pub state: UseReducerHandle<RecordState>,
    pub waveform: Vec<f32>,
    pub status: ReferenceStatus,
    pub readings: Vec<ReadingGlyph>,
    pub on_play: Callback<MouseEvent>,
    pub on_retry: Callback<MouseEvent>,
}

#[function_component(ReferencePanel)]
pub(super) fn reference_panel(props: &ReferencePanelProps) -> Html {
    let state = &props.state;
    let active = state.playback == Playback::Reference;
    let progress = use_playback_progress(active);
    let on_seek = Callback::from(|fraction: f64| { local_speech::seek_playback(fraction); });
    let render_text = |text: &str, class: &'static str| {
        let characters: Vec<char> = text.chars().collect();
        let highlighted = if active { (characters.len() as f64 * *progress).ceil() as usize } else { 0 };
        let aligned = props.readings.len() == characters.len() && props.readings.iter().zip(&characters).all(|(glyph, character)| glyph.character == *character);
        html! {
            <p class={classes!(class, "flex", "flex-wrap", "gap-x-0.5", "gap-y-2")} lang="zh-CN">
                {for characters.into_iter().enumerate().map(|(index, character)| {
                    let pinyin = if aligned {props.readings[index].pinyin.as_deref()} else {None};
                    html! {
                        <span key={index} class="inline-flex min-w-0 flex-col items-center justify-end align-bottom">
                            <span class="min-h-4 text-xs font-normal leading-4 text-muted" lang="zh-Latn">{pinyin.unwrap_or("\u{00a0}")}</span>
                            <span class={if index < highlighted {"text-violet-300"} else {"text-ink"}}>{character}</span>
                        </span>
                    }
                })}
            </p>
        }
    };
    html! {
        <section class="flex min-w-0 flex-col justify-between gap-3 rounded-2xl border border-line bg-surface p-4 shadow-sm sm:p-5" aria-label="Reference text">
            <div class="min-h-32 space-y-1">
                if state.mode == TargetMode::Sentence {
                    {render_text(if state.sentence.trim().is_empty() {"Enter a sentence above"} else {state.sentence.as_str()}, "break-words text-2xl font-semibold leading-relaxed")}
                } else if state.loading {
                    <p role="status">{"Loading words..."}</p>
                } else if let Some(word) = state.selected.as_ref() {
                    {render_text(&word.simplified, "text-5xl font-semibold leading-tight sm:text-6xl")}
                    if let Some(meaning) = word.english.as_ref() {
                        <p class="line-clamp-2 text-sm text-muted">{meaning.split('|').next().unwrap_or(meaning)}</p>
                    }
                }
            </div>
            <div class="relative rounded-xl border border-line bg-raised px-4 py-2">
                if active {
                    <button type="button" data-record-play="" aria-label="Stop reference audio" title="Stop audio" onclick={props.on_play.clone()} class="absolute right-2 top-1/2 z-10 inline-flex h-8 w-8 -translate-y-1/2 items-center justify-center rounded-lg bg-brand-action text-white shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-violet-300">
                        <Icon data={IconData::LUCIDE_SQUARE} width="16px" height="16px" />
                    </button>
                }
                if props.status == ReferenceStatus::Loading {
                    <div class="flex h-10 items-center justify-center gap-2 text-xs text-muted" role="status">
                        <Icon data={IconData::LUCIDE_REFRESH_CW} width="16px" height="16px" class="animate-spin motion-reduce:animate-none" />
                        {"Preparing audio"}
                    </div>
                } else if props.status == ReferenceStatus::Failed {
                    <button type="button" onclick={props.on_retry.clone()} aria-label="Retry generating reference audio" class="flex h-10 w-full items-center justify-center gap-2 text-sm text-muted hover:text-ink">
                        <Icon data={IconData::LUCIDE_REFRESH_CW} width="16px" height="16px" />{"Retry audio"}
                    </button>
                } else {
                    <Waveform peaks={props.waveform.clone()} label="reference audio" progress={*progress} on_seek={if active {Some(on_seek)} else {None}} on_activate={if props.status == ReferenceStatus::Ready && !active && state.target().is_some() && state.stage == RecordingStage::Idle && state.playback == Playback::None {Some(props.on_play.clone())} else {None}} />
                }
            </div>
        </section>
    }
}
