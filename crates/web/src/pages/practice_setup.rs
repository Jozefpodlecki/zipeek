use crate::{
    components::{PracticeBuilder, PracticeVocabularySource, PoolPreview, SetupChoice},
    hooks::{SavedExclusions, use_saved_exclusions},
    reducers::setup::{SetupAction, SetupState},
    route::Route,
    services::word_sets,
};
use yew::prelude::*;
use web_sys::HtmlSelectElement;
use zipeek_exercises::practice_draft::{PracticeDraft, SessionLength, VocabularySource};
use zipeek_exercises::planner::SessionPlan;
use yew_router::prelude::{use_location, use_navigator};
use std::collections::HashMap;
use zipeek_core::practice::{ExerciseVariant, FixedRounds, PracticeMode};
use zipeek_core::{Hsk2Level, Hsk3Level, HskLevel};

#[function_component(PracticeSetup)]
pub fn practice_setup() -> Html {
    let state = use_reducer_eq(SetupState::default);
    let saved = use_saved_exclusions();
    let navigator = use_navigator();
    let start_failed = use_state(|| false);
    let checked_pool = use_state(|| None::<(VocabularySource, Option<std::collections::BTreeSet<zipeek_core::WordId>>, Option<Result<PoolPreview, String>>)>);
    let personal_sets = use_state(word_sets::read);
    let location = use_location();
    let preselected_source = location
        .as_ref()
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .and_then(|query| query.get("source").cloned())
        .filter(|value| value.starts_with("saved:"))
        .unwrap_or_default();
    let draft = use_state(move || {
        let mut draft = PracticeDraft::default();
        draft.exercise_ids.insert("matching".to_owned());
        draft.length = SessionLength::Endless;
        if let Some(reference) = preselected_source.strip_prefix("saved:") {
            if reference.parse::<usize>().is_ok() {
                draft.vocabulary = VocabularySource::PersonalSet { reference: reference.to_owned() };
            }
        }
        draft
    });
    let tab = use_state(|| "general".to_owned());
    let step = use_state(|| "exercises".to_owned());
    let on_toggle_exercise = {
        let draft = draft.clone();
        let tab = tab.clone();
        Callback::from(move |id: &'static str| {
            let mut next = (*draft).clone();
            if let Err(error) = next.toggle_exercise(id) {
                log::error!("Could not toggle practice definition: {error:?}");
                return;
            }
            if !next.configuration_tabs().contains(&tab.as_str()) {
                tab.set("general".to_owned());
            }
            draft.set(next);
        })
    };
    let on_select_tab = {
        let tab = tab.clone();
        Callback::from(move |value: String| tab.set(value))
    };
    let translation_navigator = use_navigator();
    let on_vocabulary = {
        let draft = draft.clone();
        let dispatch = state.dispatcher();
        let checked_pool = checked_pool.clone();
        Callback::from(move |source: VocabularySource| {
            checked_pool.set(None);
            if let VocabularySource::Dictionary(filters) = &source {
                if let Some(band) = filters.hsk.iter().next() {
                    let level = match band.system {
                        2 => Hsk2Level::try_from(i32::from(band.level)).ok().map(HskLevel::Hsk2),
                        3 => Hsk3Level::try_from(i32::from(band.level)).ok().map(HskLevel::Hsk3),
                        _ => None,
                    };
                    if let Some(level) = level {
                        dispatch.dispatch(SetupAction::SelectHsk(level));
                    }
                }
            }
            let mut next = (*draft).clone();
            next.vocabulary = source;
            draft.set(next);
        })
    };
    let open_translation = {
        let draft = draft.clone();
        Callback::from(move |_| {
            let source = match &draft.vocabulary {
                VocabularySource::PersonalSet { reference } => format!("saved:{reference}"),
                VocabularySource::Dictionary(filters) => {
                    crate::services::practice_filter_source::encode(filters)
                }
            };
            if let Some(navigator) = &translation_navigator {
                if let Err(error) = navigator.push_with_query(
                    &Route::SentenceTranslation,
                    &[("source", source.as_str()), ("count", if draft.translation_count == 0 { "0" } else if draft.translation_count == 10 { "10" } else { "5" })],
                ) {
                    log::error!("Could not open translation: {error}");
                }
            }
        })
    };
    let on_pool_count = {
        let checked_pool = checked_pool.clone();
        Callback::from(move |update: (VocabularySource, Option<std::collections::BTreeSet<zipeek_core::WordId>>, Option<Result<PoolPreview, String>>)| checked_pool.set(Some(update)))
    };
    let current_exclusions = match &saved.status {
        SavedExclusions::Ready(ids) => Some(ids.clone()),
        _ => None,
    };
    let dictionary_pool_ready = match &draft.vocabulary {
        VocabularySource::PersonalSet { .. } => true,
        source @ VocabularySource::Dictionary(_) => matches!(&*checked_pool,
            Some((checked_source, checked_exclusions, Some(Ok(preview))))
                if checked_source == source && checked_exclusions == &current_exclusions && preview.matching_rows > 0),
    };
    let selected_variants = draft.active_formats();
    let planned_session = SessionPlan::from_draft(&draft).ok();
    let dictionary_round_ready = match &draft.vocabulary {
        VocabularySource::PersonalSet { .. } => true,
        source @ VocabularySource::Dictionary(_) => matches!(&*checked_pool,
            Some((checked_source, checked_exclusions, Some(Ok(preview))))
                if checked_source == source && checked_exclusions == &current_exclusions
                    && !selected_variants.is_empty() && selected_variants.iter().all(|variant| preview.supports(*variant))),
    };
    let personal_reference = match &draft.vocabulary {
        VocabularySource::PersonalSet { reference } => reference.as_str(),
        VocabularySource::Dictionary(_) => "",
    };
    let snapshot_error = match &*personal_sets {
        Ok(sets) => word_sets::practice_snapshot(sets, personal_reference).err(),
        Err(error) if !personal_reference.is_empty() => Some(format!("Word Sets unavailable: {error}")),
        Err(_) => None,
    };
    let ready = matches!(saved.status, SavedExclusions::Ready(_)) && navigator.is_some() && snapshot_error.is_none();
    let dictionary_filters_valid = match &draft.vocabulary {
        VocabularySource::Dictionary(filters) => !filters.hsk.is_empty() || !filters.categories.is_empty() || !filters.parts_of_speech.is_empty(),
        _ => true,
    };

    let on_start = {
        let query = match &saved.status {
            SavedExclusions::Ready(ids) => {
                let mut normalized = state.query(ids.clone());
                legacy_round_mode(&draft.length).map(|mode| {
                    normalized.mode = mode;
                    crate::services::exercise_types::query_pairs(
                        &normalized,
                        &draft.active_formats(),
                    )
                })
            }
            _ => None,
        };
        let types = draft.active_formats();
        let draft = draft.clone();
        let personal_sets = personal_sets.clone();
        let start_failed = start_failed.clone();
        let dictionary_pool_ready = dictionary_round_ready;
        Callback::from(move |_| {
            if !dictionary_pool_ready {
                start_failed.set(true);
                return;
            }
            if let (Some(navigator), Some(query)) = (&navigator, &query) {
                let allowed = types.iter().copied().filter(|variant| {
                    if variant.is_mcq() { draft.has_exercise("multiple-choice") }
                    else { draft.has_exercise("matching") }
                }).collect::<Vec<_>>();
                let variant = match crate::services::exercise_types::random_type(&allowed, None) {
                    Ok(variant) => variant,
                    Err(error) => {
                        log::error!("Could not choose starting type: {error}");
                        start_failed.set(true);
                        return;
                    }
                };
                let mut query = query.clone();
                if allowed.len() > 1 {
                    query.insert("types".to_owned(), allowed.iter().map(|variant| variant.seed_token()).collect::<Vec<_>>().join(","));
                } else {
                    query.remove("types");
                }
                if let VocabularySource::Dictionary(filters) = &draft.vocabulary {
                    if filters.hsk.len() != 1 || !filters.categories.is_empty() || !filters.parts_of_speech.is_empty() {
                        query.insert("source".to_owned(), crate::services::practice_filter_source::encode(filters));
                    }
                }
                let reference = match &draft.vocabulary {
                    VocabularySource::PersonalSet { reference } => reference.as_str(),
                    VocabularySource::Dictionary(_) => "",
                };
                let snapshot = match &*personal_sets {
                    Ok(sets) => word_sets::practice_snapshot(sets, reference),
                    Err(_) if reference.is_empty() => Ok(None),
                    Err(error) => Err(format!("Word Sets unavailable: {error}")),
                };
                match snapshot {
                    Ok(Some(ids)) => {
                        query.insert("set_ids".to_owned(), ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(","));
                    }
                    Ok(None) => {}
                    Err(error) => {
                        log::error!("Could not start practice with Word Set: {error}");
                        start_failed.set(true);
                        return;
                    }
                }
                match navigator.push_with_query(&Route::practice(variant), &query) {
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
    html! {
        <section data-component="practice-setup-page" data-practice-setup="" class="flex w-full flex-col gap-8">
            <div class="space-y-2">
                <h1 class="text-3xl font-bold tracking-tight">{"Practice"}</h1>
                <p class="text-sm text-muted">{"Choose exercises, then configure the available settings before starting."}</p>
            </div>
            <nav role="tablist" aria-label="Practice builder steps" data-practice-step-tabs="" class="grid grid-cols-2 gap-2 rounded-2xl border border-line bg-surface p-2 sm:grid-cols-4">
                {for [("exercises", "1 · Exercise Types"), ("vocabulary", "2 · Vocabulary"), ("configure", "3 · Configure"), ("summary", "4 · Summary")].into_iter().map(|(id, label)| {
                    let selected = *step == id;
                    let step = step.clone();
                    html! {<button type="button" role="tab" key={id} data-practice-step={id} aria-selected={selected.to_string()} onclick={Callback::from(move |_| step.set(id.to_owned()))}
                        class={classes!("min-h-12", "rounded-xl", "px-3", "py-2", "text-sm", "font-semibold", "transition-colors", if selected {"bg-blue-600 text-white"} else {"text-muted hover:bg-raised hover:text-ink"})}>{label}</button>}
                })}
            </nav>
            {if *step == "exercises" {html! {<PracticeBuilder section={"exercises".to_owned()} draft={(*draft).clone()} active_tab={(*tab).clone()} on_toggle={on_toggle_exercise.clone()} on_tab={on_select_tab.clone()} />}} else {Html::default()}}
            {if *step == "configure" {html! {<PracticeBuilder section={"configure".to_owned()} draft={(*draft).clone()} active_tab={(*tab).clone()} on_toggle={on_toggle_exercise.clone()} on_tab={on_select_tab.clone()} />}} else {Html::default()}}
            {if *step == "vocabulary" {html! {<div role="tabpanel" data-practice-step-panel="vocabulary" class="flex flex-col gap-5">
                <div><h2 class="text-xl font-semibold">{"Vocabulary"}</h2><p class="text-sm text-muted">{"Choose one shared vocabulary source for your selected exercises."}</p></div>
                <PracticeVocabularySource
                    source={draft.vocabulary.clone()}
                    personal_sets={match &*personal_sets { Ok(sets) => sets.clone(), Err(_) => Vec::new() }}
                    on_change={on_vocabulary.clone()}
                    on_pool_count={on_pool_count.clone()}
                    exclusions={current_exclusions.clone()}
                />
            </div>}} else {Html::default()}}
            {if *step == "summary" {html! {<div role="tabpanel" data-practice-step-panel="summary" class="rounded-2xl border border-line bg-surface p-6">
                <h2 class="text-xl font-semibold">{"Session summary"}</h2>
                <p class="mt-1 text-sm text-muted">{"Review your selections and launch a supported exercise. Unified multi-exercise progression is still being integrated."}</p>
                <div class="mt-5 grid gap-4 md:grid-cols-2">
                    <div class="rounded-xl border border-line bg-raised p-4"><h3 class="font-semibold">{"Selected exercises"}</h3>
                        <ul class="mt-2 space-y-2 text-sm">{for draft.exercise_ids.iter().map(|id| html! {<li key={id.clone()}>{match id.as_str() {"matching"=>"Matching", "multiple-choice"=>"Multiple choice", "sentence-translation"=>"Sentence translation", "speaking"=>"Speaking recorder", _=>id.as_str()}}</li>})}</ul>
                    </div>
                    <div class="rounded-xl border border-line bg-raised p-4" data-practice-summary-configuration="">
                        <h3 class="font-semibold">{"Exercise configuration"}</h3>
                        {if draft.has_exercise("matching") {html! {
                            <div class="mt-3 text-sm" data-summary-family="matching">
                                <p class="font-semibold">{"Matching"}</p>
                                {if selected_variants.iter().any(|variant| !variant.is_mcq()) {
                                    html! {<ul class="mt-1 list-inside list-disc text-muted">{for selected_variants.iter().filter(|variant| !variant.is_mcq()).map(|variant| html!{<li key={variant.seed_token()}>{match variant {
                                        ExerciseVariant::EngPinyin => "English → Pinyin",
                                        ExerciseVariant::PinyinHanzi => "Pinyin → Hanzi",
                                        _ => "Hanzi → English",
                                    }}</li>})}</ul>}
                                } else {html!{<p role="alert" class="mt-1 text-amber-400">{"Choose a Matching question format in Configure."}</p>}}}
                            </div>
                        }} else {Html::default()}}
                        {if draft.has_exercise("multiple-choice") {html! {
                            <div class="mt-3 text-sm" data-summary-family="multiple-choice">
                                <p class="font-semibold">{"Multiple choice"}</p>
                                {if selected_variants.iter().any(|variant| variant.is_mcq()) {
                                    html! {<ul class="mt-1 list-inside list-disc text-muted">{for selected_variants.iter().filter(|variant| variant.is_mcq()).map(|variant| html!{<li key={variant.seed_token()}>{match variant {
                                        ExerciseVariant::HanziToMeaning => "Hanzi → Meaning",
                                        ExerciseVariant::MeaningToHanzi => "Meaning → Hanzi",
                                        _ => "Meaning → Pinyin",
                                    }}</li>})}</ul>}
                                } else {html!{<p role="alert" class="mt-1 text-amber-400">{"Choose a Multiple choice question format in Configure."}</p>}}}
                            </div>
                        }} else {Html::default()}}
                        {if draft.has_exercise("sentence-translation") {html! {
                            <p class="mt-3 text-sm" data-summary-family="translation">{format!("Sentence translation · {}", if draft.translation_count == 0 {"Endless".to_owned()} else {format!("{} exercises", draft.translation_count)})}</p>
                        }} else {Html::default()}}
                        {if draft.has_exercise("speaking") {html! {<p class="mt-3 text-sm" data-summary-family="speaking">{"Speaking · Unscored recording preview"}</p>}} else {Html::default()}}
                        {if draft.has_exercise("matching") || draft.has_exercise("multiple-choice") {html! {
                            <p class="mt-3 text-sm text-muted">{match &draft.length { SessionLength::Fixed(count) => format!("Fixed · {} rounds", count), SessionLength::Endless => "Endless mode".to_owned() }}</p>
                        }} else {Html::default()}}
                    </div>
                    <div class="rounded-xl border border-line bg-raised p-4"><h3 class="font-semibold">{"Vocabulary source"}</h3>
                        <p class="mt-2 text-sm text-muted">{match &draft.vocabulary {
                            VocabularySource::PersonalSet { reference } => format!("Personal Word Set: {reference}"),
                            VocabularySource::Dictionary(filters) => format!("Dictionary · {} HSK bands · {} categories · {} parts of speech", filters.hsk.len(), filters.categories.len(), filters.parts_of_speech.len()),
                        }}</p>
                        <p class="mt-3 text-sm text-muted">{match &*checked_pool {
                            Some((source, excluded, Some(Ok(preview)))) if source == &draft.vocabulary && excluded == &current_exclusions => format!("{} matching dictionary words", preview.matching_rows),
                            _ => "Vocabulary count pending or unavailable".to_owned(),
                        }}</p>
                    </div>
                </div>
                <div class="mt-4 rounded-xl border border-line bg-raised p-4" data-practice-plan-preview="">
                    <h3 class="font-semibold">{"Planned session sequence"}</h3>
                    {if let Some(plan) = &planned_session {
                        html! {
                            <div>
                                <p class="mt-1 text-sm text-muted">{format!("{} exercises · deterministic order · preview only", plan.steps.len())}</p>
                                <ol class="mt-3 grid gap-2 sm:grid-cols-2" data-practice-planned-steps="">
                                    {for plan.steps.iter().take(8).map(|item| html! {
                                        <li key={item.instance_id.clone()} class="flex items-center gap-3 rounded-lg border border-line px-3 py-2 text-sm">
                                            <span class="font-semibold text-accent">{format!("{:02}", item.ordinal + 1)}</span>
                                            {match item.definition_id {
                                                "matching" => "Matching",
                                                "multiple-choice" => "Multiple Choice",
                                                "sentence-translation" => "Sentence Translation",
                                                other => other,
                                            }}
                                        </li>
                                    })}
                                </ol>
                                {if plan.steps.len() > 8 {html!{<p class="mt-2 text-sm text-muted">{format!("+ {} more exercises", plan.steps.len() - 8)}</p>}} else {Html::default()}}
                            </div>
                        }
                    } else {
                        html!{<p class="mt-1 text-sm text-muted">{"A finite scored session plan is available after choosing compatible exercises and a Fixed session length. Endless and unscored exercises have no shared plan yet."}</p>}
                    }}
                </div>
            </div>}} else {Html::default()}}

            {if *step == "summary" {html! {<div class="flex flex-col gap-3 rounded-2xl border border-line bg-surface p-5" data-practice-summary-actions="">
                <h3 class="font-semibold">{"Available launch actions"}</h3>
                <p class="text-sm text-muted">{"The unified Start Session action will be enabled after the shared session planner is connected to exercise result acknowledgement. Until then, choose one supported launch path."}</p>
                {if draft.has_exercise("matching") || draft.has_exercise("multiple-choice") {html! {<button type="button" data-practice-start="" disabled={!ready || !dictionary_filters_valid || !dictionary_round_ready || selected_variants.is_empty()
                    || (draft.has_exercise("matching") && !selected_variants.iter().any(|variant| !variant.is_mcq()))
                    || (draft.has_exercise("multiple-choice") && !selected_variants.iter().any(|variant| variant.is_mcq()))} onclick={on_start} class="min-h-12 rounded-xl bg-blue-600 px-5 py-3 font-semibold text-white hover:bg-blue-700 disabled:opacity-50">{"Start Matching / MCQ"}</button>}} else {Html::default()}}
                {if draft.has_exercise("sentence-translation") {html! {<button type="button" data-translation-setup-start="" disabled={snapshot_error.is_some() || !dictionary_filters_valid || !dictionary_pool_ready} onclick={open_translation} class="min-h-12 rounded-xl border border-line bg-raised px-5 py-3 font-semibold hover:bg-blue-900 disabled:opacity-50">{"Start translation only"}</button>}} else {Html::default()}}
                {if draft.has_exercise("speaking") {html! {<a href="#/practice/speaking" class="rounded-xl border border-line bg-raised px-5 py-3 text-center font-semibold">{"Open unscored speaking preview"}</a>}} else {Html::default()}}
            </div>}} else {Html::default()}}
            {if *step == "summary" && !dictionary_filters_valid {html! {<p role="alert" class="text-sm text-red-500">{"Select an HSK level or category before starting."}</p>} } else {Html::default()}}
            {if *step == "summary" && dictionary_filters_valid && !dictionary_pool_ready && matches!(&draft.vocabulary, VocabularySource::Dictionary(_)) {
                html! { <p data-practice-pool-gate="" role="status" class="text-sm text-muted">{"Practice launch is unavailable until the selected vocabulary has been checked and contains words."}</p> }
            } else { Html::default() }}
            {if *step == "summary" && dictionary_pool_ready && !dictionary_round_ready && matches!(&draft.vocabulary, VocabularySource::Dictionary(_)) && !selected_variants.is_empty() {
                html! {<p data-practice-playable-gate="" role="status" class="text-sm text-amber-400">{"Not enough distinct playable answers for one or more selected question formats. Change filters, exclusions, or question formats."}</p>}
            } else {Html::default()}}
            {if *step == "summary" { if let Some(message) = &snapshot_error { html! { <p role="alert" class="text-sm text-red-500">{message}</p> } } else { Html::default() } } else { Html::default() }}
            {if *step == "configure" && (*tab == "matching" || *tab == "multiple-choice") {html! {
                <section role="tabpanel" data-practice-variant-settings={(*tab).clone()} class="flex max-w-5xl flex-col gap-4 rounded-2xl border border-line bg-surface p-6 shadow-sm">
                    <h3 class="text-lg font-semibold">{if *tab == "matching" {"Matching formats"} else {"Multiple choice formats"}}</h3>
                    <p class="text-sm text-muted">{"Select one or more question formats. Only formats from the selected exercise family are used when launching that family."}</p>
                    <fieldset class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                        <legend class="sr-only">{"Question formats"}</legend>
                        {for [
                            (ExerciseVariant::EngPinyin, "English → Pinyin"),
                            (ExerciseVariant::PinyinHanzi, "Pinyin → Hanzi"),
                            (ExerciseVariant::HanziEng, "Hanzi → English"),
                            (ExerciseVariant::HanziToMeaning, "Hanzi → Meaning"),
                            (ExerciseVariant::MeaningToHanzi, "Meaning → Hanzi"),
                            (ExerciseVariant::MeaningToPinyin, "Meaning → Pinyin"),
                        ].into_iter().filter(|(variant, _)| variant.is_mcq() == (*tab == "multiple-choice")).map(|(variant, label)| {
                            html! {<SetupChoice label={label} value={variant.seed_token()} selected={draft.selected_formats().contains(&variant)} on_select={{
                                let draft = draft.clone();
                                Callback::from(move |_| {
                                    let mut next = (*draft).clone();
                                    let enabled = !next.selected_formats().contains(&variant);
                                    if let Err(error) = next.set_format(variant, enabled) {
                                        log::error!("Could not set format: {error:?}");
                                    } else { draft.set(next); }
                                })
                            }} />}
                        })}
                    </fieldset>
                    {if !draft.active_formats().iter().any(|variant| variant.is_mcq() == (*tab == "multiple-choice")) {
                        html! {<p role="status" class="text-sm text-amber-400">{"Choose at least one format here before launching this exercise family."}</p>}
                    } else {Html::default()}}
                </section>
            }} else {Html::default()}}
            {if *step == "configure" && *tab == "translation" && draft.has_exercise("sentence-translation") { html! {
                <section role="tabpanel" data-translation-setup="" class="flex max-w-3xl flex-col gap-5 rounded-2xl border border-line bg-surface p-6 shadow-sm">
                    <div class="space-y-2">
                        <h2 class="text-xl font-semibold">{"Sentence translation"}</h2>
                        <p class="text-sm text-muted">{"Generate a sentence using an HSK level or a personal Word Set, then translate it with a word bank. Requires a running local LM Studio model."}</p>
                    </div>
                    <p class="text-sm text-muted">{"Both Translation and Matching/MCQ use the vocabulary source selected above."}</p>
                    <label class="flex flex-col gap-2 text-sm font-semibold">{"Session length"}
                        <select data-translation-setup-count="" class="min-h-11 rounded-lg border border-line bg-raised px-3" value={draft.translation_count.to_string()}
                            onchange={{
                                let draft = draft.clone();
                                Callback::from(move |event: Event| {
                                    let value = event.target_unchecked_into::<HtmlSelectElement>().value();
                                    if let Ok(count) = value.parse::<u16>() {
                                        let mut next = (*draft).clone();
                                        next.translation_count = count;
                                        draft.set(next);
                                    }
                                })
                            }}>
                            <option value="5">{"5 exercises"}</option>
                            <option value="10">{"10 exercises"}</option>
                            <option value="0">{"Endless"}</option>
                        </select>
                    </label>

                </section>
            }} else {Html::default()}}
            {if *step == "configure" && *tab == "speaking" && draft.has_exercise("speaking") { html! {
                <section role="tabpanel" data-speaking-setup="" class="flex max-w-3xl flex-col gap-4 rounded-2xl border border-line bg-surface p-6 shadow-sm">
                    <h2 class="text-xl font-semibold">{"Speaking"}</h2>
                    <p class="text-sm text-muted">{"Read a Mandarin sentence aloud, record your voice, and review the live waveform and playback. This exercise type is currently a local, unscored preview."}</p>

                </section>
            }} else {Html::default()}}
            {if *step == "configure" && *tab == "general" && (draft.has_exercise("matching") || draft.has_exercise("multiple-choice")) { html! { <div role="tabpanel" class="flex max-w-5xl flex-col gap-6 rounded-2xl border border-line bg-surface p-6 shadow-sm">
            <div class="space-y-1"><h3 class="text-lg font-semibold">{"General session settings"}</h3><p class="text-sm text-muted">{"Shared round mode and exclusions. Question formats belong to the Matching and Multiple choice tabs."}</p></div>
            <fieldset class="min-w-0" data-mode-picker="">
                <legend class="mb-3 text-lg font-semibold">{"Mode"}</legend>
                <div class="flex gap-2">
                    {for [(false,"Endless"),(true,"Fixed")].into_iter().map(|(fixed,label)| {
                        let draft = draft.clone();
                        html! { <SetupChoice label={label} value={label} selected={matches!(&draft.length, SessionLength::Fixed(_)) == fixed} on_select={Callback::from(move |_| {
                            let mut next = (*draft).clone();
                            next.length = if fixed { SessionLength::Fixed(10) } else { SessionLength::Endless };
                            draft.set(next);
                        })} /> }
                    })}
                </div>
                {if matches!(&draft.length, SessionLength::Fixed(_)) {html! {
                    <fieldset class="mt-4" data-round-counts="">
                        <legend class="mb-2 text-sm text-muted">{"Rounds"}</legend>
                        <div class="flex gap-2">
                            {for [FixedRounds::Five,FixedRounds::Ten,FixedRounds::Twenty].into_iter().map(|rounds| {
                                let draft = draft.clone();
                                html! { <SetupChoice label={rounds.count().to_string()} value={format!("rounds-{}",rounds.count())} selected={draft.length == SessionLength::Fixed(u16::from(rounds.count()))} on_select={Callback::from(move |_| {
                                    let mut next = (*draft).clone();
                                    next.length = SessionLength::Fixed(u16::from(rounds.count()));
                                    draft.set(next);
                                })} /> }
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
                {if *start_failed {html!{<p class="text-sm text-muted" role="alert">{"Couldn't start practice. Check your selection and try again."}</p>}} else {Html::default()}}

            </div>
            </div> }} else { Html::default() }}
            {if *step == "configure" && *tab == "general" && !draft.has_exercise("matching") && !draft.has_exercise("multiple-choice") {html! {
                <section role="tabpanel" data-practice-general-settings="" class="rounded-2xl border border-line bg-surface p-6">
                    <h3 class="text-lg font-semibold">{"General session settings"}</h3>
                    <p class="mt-2 text-sm text-muted">{"Shared session scheduling will be available when unified sessions are connected. Configure Translation or Speaking in their individual tabs; no Matching/MCQ round settings apply to this selection."}</p>
                </section>
            }} else {Html::default()}}
            <div class="flex items-center justify-between gap-3" data-practice-step-navigation="">
                {if *step != "exercises" {let step=step.clone(); html! {<button type="button" class="min-h-11 rounded-xl border border-line bg-raised px-5 font-semibold" onclick={Callback::from(move |_| step.set(match (*step).as_str() {"vocabulary"=>"exercises", "configure"=>"vocabulary", _=>"configure"}.to_owned()))}>{"Back"}</button>}} else {Html::default()}}
                <span class="text-sm text-muted">{match (*step).as_str() {"exercises"=>"Step 1 of 4", "vocabulary"=>"Step 2 of 4", "configure"=>"Step 3 of 4", _=>"Step 4 of 4"}}</span>
                {if *step != "summary" {let step=step.clone(); html! {<button type="button" class="min-h-11 rounded-xl bg-blue-600 px-5 font-semibold text-white hover:bg-blue-700" onclick={Callback::from(move |_| step.set(match (*step).as_str() {"exercises"=>"vocabulary", "vocabulary"=>"configure", _=>"summary"}.to_owned()))}>{"Continue"}</button>}} else {Html::default()}}
            </div>
        </section>
    }
}

fn legacy_round_mode(length: &SessionLength) -> Option<PracticeMode> {
    match length {
        SessionLength::Endless => Some(PracticeMode::Endless),
        SessionLength::Fixed(5) => Some(PracticeMode::Fixed(FixedRounds::Five)),
        SessionLength::Fixed(10) => Some(PracticeMode::Fixed(FixedRounds::Ten)),
        SessionLength::Fixed(20) => Some(PracticeMode::Fixed(FixedRounds::Twenty)),
        SessionLength::Fixed(_) => None,
    }
}

#[cfg(test)]
mod r104_tests {
    use super::*;

    #[test]
    fn draft_round_length_maps_to_existing_legacy_modes_without_replay_changes() {
        assert_eq!(legacy_round_mode(&SessionLength::Endless), Some(PracticeMode::Endless));
        assert_eq!(legacy_round_mode(&SessionLength::Fixed(5)), Some(PracticeMode::Fixed(FixedRounds::Five)));
        assert_eq!(legacy_round_mode(&SessionLength::Fixed(10)), Some(PracticeMode::Fixed(FixedRounds::Ten)));
        assert_eq!(legacy_round_mode(&SessionLength::Fixed(20)), Some(PracticeMode::Fixed(FixedRounds::Twenty)));
        assert_eq!(legacy_round_mode(&SessionLength::Fixed(7)), None);
    }
}
