use super::state::CloudSetupState;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::cloud_match::{CloudPair, MIN_PAIRS, SpawnMode, eligible_pairs};

#[derive(Properties, PartialEq)]
pub(super) struct StartControlsProps {
    pub setup: UseReducerHandle<CloudSetupState>,
    pub pool: Option<Result<Vec<CloudPair>, String>>,
    pub setup_error: String,
    pub on_start: Callback<MouseEvent>,
}

#[function_component(StartControls)]
pub(super) fn start_controls(props: &StartControlsProps) -> Html {
    let playable_count = match &props.pool {
        Some(Ok(words)) => eligible_pairs(words.clone(), props.setup.mode).len(),
        _ => 0,
    };
    let required_count = props
        .setup
        .length
        .target()
        .unwrap_or(MIN_PAIRS)
        .max(MIN_PAIRS);
    let eligibility_message = match &props.pool {
        None => "Checking vocabulary…".to_owned(),
        Some(Err(error)) => format!("Could not load vocabulary: {error}"),
        Some(Ok(_)) if playable_count < MIN_PAIRS => format!(
            "Only {playable_count} distinct pairs available; at least {MIN_PAIRS} are required. Choose a different level or Word Set."
        ),
        Some(Ok(_)) if playable_count < required_count => {
            format!("Choose a smaller game length: {playable_count} distinct pairs are available.")
        }
        Some(Ok(_)) if props.setup.spawn == SpawnMode::Continuous => format!(
            "{playable_count} distinct playable pairs available. Continuous mode has no wave timer."
        ),
        Some(Ok(_)) => format!(
            "{playable_count} distinct playable pairs available. {}s per wave",
            props.setup.difficulty.seconds_per_wave()
        ),
    };
    let readiness = match &props.pool {
        None => "loading",
        Some(Err(_)) => "error",
        Some(Ok(_)) if playable_count < required_count => "insufficient",
        Some(Ok(_)) => "ready",
    };
    html! {
        <>
            <div role="status" aria-live="polite" data-cloud-match-eligibility={readiness} class="rounded-lg border border-line bg-raised p-4 text-sm">{eligibility_message}</div>
            <button type="button" data-cloud-match-start="" data-cloud-match-start-state={readiness} class="rounded-xl bg-brand-action px-5 py-4 font-bold text-white disabled:cursor-not-allowed disabled:opacity-40" disabled={playable_count < required_count} onclick={props.on_start.clone()}>
                <span class="inline-flex items-center justify-center gap-2"><Icon data={IconData::LUCIDE_PLAY} width="19" height="19"/>{"Start"}</span>
            </button>
            {if props.setup_error.is_empty() {Html::default()} else {html! {<p role="alert" data-cloud-match-setup-error="" class="text-sm text-rose-500">{props.setup_error.clone()}</p>}}}
        </>
    }
}
