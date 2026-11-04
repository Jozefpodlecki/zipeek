use crate::{
    components::{ActivityPicker, HskPicker, SetupChoice},
    hooks::{SavedExclusions, use_saved_exclusions},
    reducers::setup::{SetupAction, SetupState},
    route::Route,
};
use yew::prelude::*;
use web_sys::HtmlSelectElement;
use zipeek_ai::STARTER_SETS;
use yew_router::prelude::use_navigator;
use zipeek_core::practice::FixedRounds;

#[function_component(PracticeSetup)]
pub fn practice_setup() -> Html {
    let state = use_reducer_eq(SetupState::default);
    let saved = use_saved_exclusions();
    let navigator = use_navigator();
    let start_failed = use_state(|| false);
    let translation_set = use_state(|| "travel".to_owned());
    let translation_navigator = use_navigator();
    let change_translation_set = {
        let translation_set = translation_set.clone();
        Callback::from(move |event: Event| {
            translation_set.set(event.target_unchecked_into::<HtmlSelectElement>().value());
        })
    };
    let open_translation = {
        let translation_set = translation_set.clone();
        Callback::from(move |_| {
            if let Some(navigator) = &translation_navigator {
                if let Err(error) = navigator.push_with_query(
                    &Route::SentenceTranslation,
                    &[("set", translation_set.as_str())],
                ) {
                    log::error!("Could not open translation: {error}");
                }
            }
        })
    };
    let ready = matches!(saved.status, SavedExclusions::Ready(_)) && navigator.is_some();
    let on_start = {
        let query = match &saved.status {
            SavedExclusions::Ready(ids) => Some(crate::services::exercise_types::query_pairs(
                &state.query(ids.clone()),
                &state.types,
            )),
            _ => None,
        };
        let types = state.types.clone();
        let start_failed = start_failed.clone();
        Callback::from(move |_| {
            if let (Some(navigator), Some(query)) = (&navigator, &query) {
                let variant = match crate::services::exercise_types::random_type(&types, None) {
                    Ok(variant) => variant,
                    Err(error) => {
                        log::error!("Could not choose starting type: {error}");
                        start_failed.set(true);
                        return;
                    }
                };
                match navigator.push_with_query(&Route::practice(variant), query) {
                    Ok(()) => start_failed.set(false),
                    Err(error) => {
                        log::error!("Could not start practice: {error}");
                        start_failed.set(true);
                    }
                }
            } else {
                log::error!("Practice Start requires ready exclusions and router context");
            }
        })
    };
    let select_activity = {
        let dispatch = state.dispatcher();
        Callback::from(move |variant| dispatch.dispatch(SetupAction::Activity(variant)))
    };
    let select_system = {
        let dispatch = state.dispatcher();
        Callback::from(move |system| dispatch.dispatch(SetupAction::System(system)))
    };
    let select_level = {
        let dispatch = state.dispatcher();
        Callback::from(move |level| dispatch.dispatch(SetupAction::Level(level)))
    };
    html! {
        <section data-practice-setup="" class="flex w-full flex-col gap-8">
            <h1 class="text-3xl font-bold tracking-tight">{"Practice"}</h1>
            <div class="rounded-xl border border-line p-4" data-translation-setup="">
                <h2 class="text-lg font-semibold">{"Sentence translation (experimental)"}</h2>
                <p class="my-2 text-sm text-muted">{"Practise Chinese-to-English translation with predefined Word Sets and a local LM Studio model."}</p>
                <label class="mb-3 flex flex-col gap-2 text-sm font-semibold">{"Predefined Word Set"}
                    <select data-translation-word-set="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*translation_set).clone()} onchange={change_translation_set}>
                        {for STARTER_SETS.iter().map(|set| html! { <option value={set.id}>{set.name}</option> })}
                    </select>
                </label>
                <button type="button" data-translation-setup-start="" onclick={open_translation} class="inline-flex min-h-11 items-center rounded-lg bg-blue-600 px-4 font-semibold text-white">{"Configure sentence translation"}</button>
            </div>
            <ActivityPicker selected={state.types.clone()} on_select={select_activity} />
            <HskPicker level={state.level} on_system={select_system} on_level={select_level} />
            <fieldset class="min-w-0" data-mode-picker="">
                <legend class="mb-3 text-lg font-semibold">{"Mode"}</legend>
                <div class="flex gap-2">
                    {for [(false,"Endless"),(true,"Fixed")].into_iter().map(|(fixed,label)| {
                        let dispatch=state.dispatcher();
                        html! { <SetupChoice label={label} value={label} selected={state.fixed==fixed} on_select={Callback::from(move |_| dispatch.dispatch(SetupAction::Fixed(fixed)))} /> }
                    })}
                </div>
                {if state.fixed {html! {
                    <fieldset class="mt-4" data-round-counts="">
                        <legend class="mb-2 text-sm text-muted">{"Rounds"}</legend>
                        <div class="flex gap-2">
                            {for [FixedRounds::Five,FixedRounds::Ten,FixedRounds::Twenty].into_iter().map(|rounds| {
                                let dispatch=state.dispatcher();
                                html! { <SetupChoice label={rounds.count().to_string()} value={format!("rounds-{}",rounds.count())} selected={state.rounds==rounds} on_select={Callback::from(move |_| dispatch.dispatch(SetupAction::Rounds(rounds)))} /> }
                            })}
                        </div>
                    </fieldset>
                }} else {Html::default()}}
            </fieldset>
            <div class="flex flex-col gap-3">
                {match &saved.status {
                    SavedExclusions::Loading => html! { <p role="status" class="text-sm text-muted">{"Loading excluded words"}</p> },
                    SavedExclusions::Failed => { let retry=saved.retry.clone(); html! { <p class="text-sm text-muted">{"Couldn't load excluded words. "}<button type="button" data-exclusions-retry="" class="min-h-11 rounded px-3 text-accent hover:bg-raised" onclick={Callback::from(move |_| retry.emit(()))}>{"Retry"}</button></p> } },
                    SavedExclusions::Ready(ids) if !ids.is_empty() => { let reset=saved.reset.clone(); html! { <p class="text-sm text-muted" data-exclusions-count={ids.len().to_string()}>{format!("Excluded words: {} · ",ids.len())}<button type="button" data-exclusions-reset="" class="min-h-11 rounded px-3 text-accent hover:bg-raised" onclick={Callback::from(move |_| reset.emit(()))}>{"Reset"}</button></p> } },
                    _ => Html::default(),
                }}
                {if saved.reset_failed {html!{<p class="text-sm text-muted" role="alert">{"Couldn't reset exclusions. Try again."}</p>}} else {Html::default()}}
                {if *start_failed {html!{<p class="text-sm text-muted" role="alert">{"Couldn't start practice. Try again."}</p>}} else {Html::default()}}
                <button type="button" data-practice-start="" disabled={!ready} onclick={on_start} class="min-h-12 w-full rounded-xl bg-blue-600 px-5 py-3 font-semibold text-white transition-colors hover:bg-blue-700 disabled:cursor-wait disabled:opacity-50 motion-reduce:transition-none">{"Start"}</button>
            </div>
        </section>
    }
}
