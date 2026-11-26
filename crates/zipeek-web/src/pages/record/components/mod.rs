mod recognition_result;
mod live_panel;
mod recording_panel;
mod reference_panel;
mod target_selector;
mod waveform;

use yew::prelude::*;
use yew_icons::{Icon, IconData};

use super::controller::RecordController;
use recognition_result::RecognitionResult;
use recording_panel::RecordingPanel;
use reference_panel::ReferencePanel;
use target_selector::TargetSelector;
use live_panel::LivePanel;
use super::state::TargetMode;

#[derive(Properties, PartialEq)]
pub(super) struct RecordViewProps {
    pub controller: RecordController,
}

#[function_component(RecordView)]
pub(super) fn record_view(props: &RecordViewProps) -> Html {
    let controller = &props.controller;
    let state = &controller.state;
    html! {
        <main data-record-page="" class="mx-auto w-full max-w-6xl space-y-3 px-1 py-3 sm:py-4">
            <header class="flex items-center gap-2">
                <Icon data={IconData::LUCIDE_MIC} width="26px" height="26px" />
                <h1 class="text-2xl font-bold tracking-tight">{"Record"}</h1>
            </header>
            <TargetSelector state={state.clone()} on_mode={controller.on_mode.clone()} on_sentence={controller.on_sentence.clone()} on_bank={controller.on_bank.clone()} on_random={controller.on_random.clone()} />
            if state.mode == TargetMode::Live {
                <LivePanel />
            } else {
            if let Some(message) = state.error.as_ref().filter(|message| !message.starts_with("No speech was recognized")) {
                <p class="flex items-center gap-2 text-sm text-amber-300" role="alert">
                    <Icon data={IconData::LUCIDE_ALERT_TRIANGLE} width="16px" height="16px" />
                    {message}
                </p>
            }
            <div class="grid gap-3 lg:grid-cols-2">
                <ReferencePanel state={state.clone()} waveform={controller.reference_waveform.clone()} on_play={controller.on_play.clone()} on_retry={controller.on_retry.clone()} status={controller.reference_status} readings={controller.reading_glyphs.clone()} />
                <RecordingPanel state={state.clone()} microphone_waveform={controller.microphone_waveform.clone()} recorded_waveform={controller.recorded_waveform.clone()} has_recording={controller.has_recording} error={state.error.as_ref().filter(|message| message.starts_with("No speech was recognized")).map(|_| AttrValue::from("No speech detected. Try again."))} on_record={controller.on_record.clone()} on_replay={controller.on_replay.clone()} on_cancel={controller.on_cancel.clone()} on_download={controller.on_download.clone()} />
            </div>
            <RecognitionResult state={state.clone()} />
            }
        </main>
    }
}
