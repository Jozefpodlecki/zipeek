use std::{cell::{Cell, RefCell}, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_browser::local_speech::{self, live::{LiveConnection, LiveEvent}};

#[derive(Clone, PartialEq, Eq)]
enum LivePhase {
    Idle,
    Connecting,
    Listening,
    Finishing,
    Completed,
    Failed(String),
}

#[function_component(LivePanel)]
pub(super) fn live_panel() -> Html {
    let phase = use_state(|| LivePhase::Idle);
    let transcript = use_state(String::new);
    let waveform = use_state(Vec::<f32>::new);
    let session = use_mut_ref(|| None::<LiveConnection>);
    let generation = use_mut_ref(|| 0_u64);
    let mounted = use_mut_ref(|| true);
    {
        let session = session.clone();
        let generation = generation.clone();
        let mounted = mounted.clone();
        use_effect_with((), move |_| {
            *mounted.borrow_mut() = true;
            move || {
                *mounted.borrow_mut() = false;
                *generation.borrow_mut() += 1;
                session.borrow_mut().take();
            }
        });
    }
    let on_start = {
        let phase = phase.clone();
        let transcript = transcript.clone();
        let waveform = waveform.clone();
        let session = session.clone();
        let generation = generation.clone();
        let mounted = mounted.clone();
        Callback::from(move |_: MouseEvent| {
            if matches!(*phase, LivePhase::Connecting | LivePhase::Listening | LivePhase::Finishing) { return; }
            session.borrow_mut().take();
            if !local_speech::allowed() || (!local_speech::enabled() && local_speech::select(true).is_err()) {
                phase.set(LivePhase::Failed("Live speech requires the trusted localhost development app".into()));
                return;
            }
            transcript.set(String::new());
            waveform.set(Vec::new());
            phase.set(LivePhase::Connecting);
            *generation.borrow_mut() += 1;
            let request = *generation.borrow();
            let generation = generation.clone();
            let mounted = mounted.clone();
            let phase = phase.clone();
            let transcript = transcript.clone();
            let waveform = waveform.clone();
            let session = session.clone();
            spawn_local(async move {
                let event_generation = generation.clone();
                let event_mounted = mounted.clone();
                let event_phase = phase.clone();
                let event_transcript = transcript.clone();
                let event_waveform = waveform.clone();
                let on_event: Rc<dyn Fn(LiveEvent)> = Rc::new(move |event| {
                    if !*event_mounted.borrow() || *event_generation.borrow() != request { return; }
                    match event {
                        LiveEvent::Connected => {},
                        LiveEvent::Waveform(peaks) => event_waveform.set(peaks),
                        LiveEvent::Ready => event_phase.set(LivePhase::Listening),
                        LiveEvent::Partial(text) => event_transcript.set(text),
                        LiveEvent::Final(text) => {
                            event_transcript.set(text);
                            event_phase.set(LivePhase::Completed);
                        }
                        LiveEvent::Error(message) => event_phase.set(LivePhase::Failed(message)),
                        LiveEvent::Closed => {
                            if matches!(*event_phase, LivePhase::Listening | LivePhase::Finishing | LivePhase::Connecting) {
                                event_phase.set(LivePhase::Failed("Live session disconnected".into()));
                            }
                        }
                    }
                });
                match LiveConnection::connect(on_event).await {
                    Ok(connection) if *mounted.borrow() && *generation.borrow() == request => {
                        session.borrow_mut().replace(connection);
                    }
                    Ok(_) => {},
                    Err(error) if *mounted.borrow() && *generation.borrow() == request => {
                        web_sys::console::error_1(&error);
                        phase.set(LivePhase::Failed("Could not start microphone streaming".into()));
                    }
                    Err(_) => {},
                }
            });
        })
    };
    let on_finish = {
        let session = session.clone();
        let phase = phase.clone();
        Callback::from(move |_: MouseEvent| {
            if *phase != LivePhase::Listening { return; }
            let result = session.borrow().as_ref().map(LiveConnection::finish);
            match result {
                Some(Ok(())) => phase.set(LivePhase::Finishing),
                _ => phase.set(LivePhase::Failed("Could not finish the live recording".into())),
            }
        })
    };
    let on_cancel = {
        let session = session.clone();
        let generation = generation.clone();
        let phase = phase.clone();
        Callback::from(move |_: MouseEvent| {
            *generation.borrow_mut() += 1;
            session.borrow_mut().take();
            phase.set(LivePhase::Idle);
        })
    };
    html! {
        <section data-record-live="" class="rounded-2xl border border-line bg-surface p-5 shadow-sm space-y-4" aria-label="Experimental live recognition">
            <div class="flex items-center justify-between gap-2">
                <h2 class="text-lg font-semibold">{"Live recognition"}</h2>
                <span class="rounded-lg border border-line px-2 py-1 text-xs text-muted">{"Experimental · Qwen snapshots"}</span>
            </div>
            <div class="min-h-24 rounded-xl border border-line bg-raised p-4" aria-live="polite" aria-atomic="false">
                if !transcript.is_empty() {
                    <p class="text-xl leading-relaxed text-ink">{(*transcript).clone()}</p>
                } else if matches!(*phase, LivePhase::Connecting | LivePhase::Listening | LivePhase::Finishing) {
                    <p class="text-sm text-muted">{"Waiting for speech..."}</p>
                }
            </div>
            if !waveform.is_empty() && matches!(&*phase, LivePhase::Listening | LivePhase::Finishing) {
                <div class="flex h-14 items-center justify-center gap-0.5" role="img" aria-label="Live microphone sound levels">
                    {for waveform.iter().enumerate().map(|(index, level)| {
                        let height = (level * 100.0).clamp(3.0, 100.0);
                        html! { <span key={index.to_string()} class="w-1 flex-1 max-w-2 rounded-full bg-brand-action" style={format!("height: {height:.1}%")} /> }
                    })}
                </div>
            }
            <div class="flex flex-wrap items-center gap-3">
                if matches!(*phase, LivePhase::Idle | LivePhase::Completed | LivePhase::Failed(_)) {
                    <button type="button" onclick={on_start} aria-label="Start live recognition" class="inline-flex h-11 w-11 items-center justify-center rounded-xl bg-brand-action text-white">
                        <Icon data={IconData::LUCIDE_MIC} width="20px" height="20px" />
                    </button>
                }
                if *phase == LivePhase::Listening {
                    <button type="button" onclick={on_finish} aria-label="Finish live recognition" title="Finish" class="inline-flex h-11 w-11 items-center justify-center rounded-xl bg-brand-action text-white">
                        <Icon data={IconData::LUCIDE_SQUARE} width="18px" height="18px" />
                    </button>
                }
                if matches!(*phase, LivePhase::Connecting | LivePhase::Listening | LivePhase::Finishing) {
                    <button type="button" onclick={on_cancel} class="rounded-lg border border-line px-3 py-2 text-sm">{"Cancel"}</button>
                }
                if *phase == LivePhase::Connecting { <span role="status" class="text-sm text-muted">{"Connecting..."}</span> }
                if *phase == LivePhase::Listening { <span role="status" class="text-sm text-muted">{"Streaming microphone"}</span> }
                if *phase == LivePhase::Finishing { <span role="status" class="text-sm text-muted">{"Finalizing..."}</span> }
                if let LivePhase::Failed(error) = &*phase { <span role="alert" class="text-sm text-amber-300">{error.clone()}</span> }
            </div>
        </section>
    }
}
