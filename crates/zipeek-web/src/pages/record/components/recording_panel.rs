use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::waveform::{Waveform, use_playback_progress};
use crate::pages::record::state::{Playback, RecordState, RecordingStage};
use zipeek_browser::local_speech;

#[derive(Properties, PartialEq)]
pub(super) struct RecordingPanelProps {
    pub state: UseReducerHandle<RecordState>,
    pub microphone_waveform: Vec<f32>,
    pub recorded_waveform: Vec<f32>,
    pub has_recording: bool,
    pub error: Option<AttrValue>,
    pub on_record: Callback<MouseEvent>,
    pub on_replay: Callback<MouseEvent>,
    pub on_cancel: Callback<MouseEvent>,
    pub on_download: Callback<MouseEvent>,
}

#[function_component(RecordingPanel)]
pub(super) fn recording_panel(props: &RecordingPanelProps) -> Html {
    let state = &props.state;
    let active = state.playback == Playback::Recording;
    let progress = use_playback_progress(active);
    let on_seek = Callback::from(|fraction: f64| {
        local_speech::seek_playback(fraction);
    });
    html! {
        <section class="flex min-w-0 flex-col items-center justify-center gap-4 rounded-2xl border border-line bg-surface p-4 shadow-sm sm:p-5" aria-label="Microphone recording">
            <div class="w-full">
                if state.stage == RecordingStage::Recording {
                    <Waveform peaks={props.microphone_waveform.clone()} label="Live microphone waveform" />
                } else {
                    <Waveform peaks={props.recorded_waveform.clone()} label="Your recorded audio waveform" progress={*progress} on_seek={if active {Some(on_seek)} else {None}} on_activate={if props.has_recording && state.stage == RecordingStage::Idle && state.playback == Playback::None {Some(props.on_replay.clone())} else {None}} />
                }
            </div>
            <div class="flex flex-wrap items-center justify-center gap-3">
                <button type="button" data-record-microphone="" disabled={state.target().is_none() || state.playback != Playback::None || state.stage == RecordingStage::Recognizing} onclick={props.on_record.clone()} aria-label={if state.stage == RecordingStage::Recording {"Stop recording and recognize"} else {"Start recording"}} title={if state.stage == RecordingStage::Recording {"Stop and recognize"} else {"Record"}} class="inline-flex h-12 w-12 items-center justify-center rounded-xl bg-brand-action text-white disabled:opacity-40">
                    <Icon data={if state.stage == RecordingStage::Recording {IconData::LUCIDE_SQUARE} else {IconData::LUCIDE_MIC}} width="20px" height="20px" />
                </button>
                if props.has_recording {
                    <button type="button" data-record-replay="" onclick={props.on_replay.clone()} disabled={state.stage != RecordingStage::Idle || state.playback == Playback::Reference} aria-label={if state.playback == Playback::Recording {"Stop your recording"} else {"Play your recording"}} title={if state.playback == Playback::Recording {"Stop your recording"} else {"Play your recording"}} class="inline-flex h-12 w-12 items-center justify-center rounded-lg border border-line disabled:opacity-40">
                        <Icon data={if state.playback == Playback::Recording {IconData::LUCIDE_SQUARE} else {IconData::LUCIDE_PLAY}} width="16px" height="16px" />
                    </button>
                }
                if props.has_recording {
                    <button type="button" onclick={props.on_download.clone()} disabled={state.stage != RecordingStage::Idle} aria-label="Download your recording as WAV" title="Download your recording (WAV)" class="inline-flex h-12 w-12 items-center justify-center rounded-lg border border-line disabled:opacity-40">
                        <Icon data={IconData::LUCIDE_DOWNLOAD} width="18px" height="18px" />
                    </button>
                }
                if state.playback != Playback::None || state.stage != RecordingStage::Idle {
                    <button type="button" onclick={props.on_cancel.clone()} class="min-h-10 rounded-lg border border-line px-4 text-sm font-semibold">{"Cancel"}</button>
                }
            </div>
            if let Some(message) = props.error.as_ref() {
                <p class="flex max-w-full items-center gap-2 text-sm text-amber-300" role="alert">
                    <Icon data={IconData::LUCIDE_ALERT_TRIANGLE} width="16px" height="16px" />
                    {message.clone()}
                </p>
            }
            if state.stage == RecordingStage::Recognizing {
                <p class="text-xs text-muted" role="status">{"Transcribing..."}</p>
            }
        </section>
    }
}
