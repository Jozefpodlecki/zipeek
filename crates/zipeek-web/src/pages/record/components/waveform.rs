use gloo::timers::callback::Interval;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_browser::local_speech;

#[derive(Properties, PartialEq)]
pub(super) struct WaveformProps {
    pub peaks: Vec<f32>,
    pub label: AttrValue,
    #[prop_or_default]
    pub progress: f64,
    #[prop_or_default]
    pub on_seek: Option<Callback<f64>>,
    #[prop_or_default]
    pub on_activate: Option<Callback<MouseEvent>>,
}

#[hook]
pub(super) fn use_playback_progress(active: bool) -> UseStateHandle<f64> {
    let progress = use_state(|| 0.0_f64);
    {
        let progress = progress.clone();
        use_effect_with(active, move |active| {
            let interval = if *active {
                let progress = progress.clone();
                Some(Interval::new(60, move || {
                    if let Some(value) = local_speech::playback_progress() {
                        if (*progress - value).abs() >= 0.001 {
                            progress.set(value);
                        }
                    }
                }))
            } else {
                progress.set(0.0);
                None
            };
            move || drop(interval)
        });
    }
    progress
}

#[function_component(Waveform)]
pub(super) fn waveform(props: &WaveformProps) -> Html {
    let progress = props.progress.clamp(0.0, 1.0);
    let on_seek = props.on_seek.as_ref().map(|seek| {
        let seek = seek.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target().and_then(|target| target.dyn_into::<HtmlInputElement>().ok()) {
                if let Ok(value) = target.value().parse::<f64>() {
                    seek.emit((value / 1000.0).clamp(0.0, 1.0));
                }
            }
        })
    });
    html! {
        <div class="group relative flex h-10 min-w-0 items-center justify-center" role="group" aria-label={props.label.clone()}>
            <div class="flex w-full items-center justify-center gap-0.5 overflow-hidden" aria-hidden="true">
                {for props.peaks.iter().enumerate().map(|(index, peak)| {
                    let height = 3.0 + peak.clamp(0.0, 1.0) * 34.0;
                    let played = (index as f64 + 0.5) / props.peaks.len().max(1) as f64;
                    html! { <span key={index} class={classes!("min-w-0", "w-1", "rounded-full", "transition-colors", if played <= progress {"bg-violet-300"} else {"bg-brand-action/45"})} style={format!("height:{height:.1}px")} /> }
                })}
            </div>
            if let Some(on_activate) = props.on_activate.as_ref() {
                <button type="button" onclick={on_activate.clone()} aria-label={format!("Play {}", props.label)} title="Play audio" class="absolute inset-0 flex h-full w-full cursor-pointer items-center justify-center rounded-lg text-sm font-medium text-muted focus-visible:outline focus-visible:outline-2 focus-visible:outline-violet-300">
                    {if props.peaks.is_empty() {
                        html! { <span class="text-xs">{"Play reference"}</span> }
                    } else {
                        html! { <span class="rounded-full bg-surface/75 p-2 opacity-40 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100" aria-hidden="true"><Icon data={IconData::LUCIDE_PLAY} width="17px" height="17px" /></span> }
                    }}
                </button>
            }
            if let Some(on_seek) = on_seek {
                if !props.peaks.is_empty() {
                    <input type="range" min="0" max="1000" step="1" value={(progress * 1000.0).round().to_string()} oninput={on_seek} aria-label={format!("Seek {}", props.label)} class="absolute inset-0 h-full w-full cursor-pointer opacity-0 focus-visible:opacity-30" />
                }
            }
        </div>
    }
}
