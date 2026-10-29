use crate::{
    components::{ListenButton, Loader, PracticeSummary, RoutePlaceholder},
    hooks::PracticeControls,
    reducers::practice::PracticeStatus,
};
use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, KeyboardEvent};
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_core::{
    practice::{ExerciseVariant, PracticeMode},
    round::RoundContent,
};

#[derive(Properties, PartialEq)]
pub(crate) struct McqPracticeProps {
    pub variant: ExerciseVariant,
    pub status: PracticeStatus,
    pub retry: Callback<()>,
    pub session: PracticeControls,
}

#[function_component(McqPractice)]
pub(crate) fn mcq_practice(props: &McqPracticeProps) -> Html {
    let session = props.session.mcq();
    let prompt_ref = use_node_ref();
    let question_key = session
        .state
        .question
        .as_ref()
        .map(|question| question.key.clone());
    {
        let prompt_ref = prompt_ref.clone();
        use_effect_with((question_key, session.ready), move |(_, ready)| {
            if *ready
                && let Some(element) = prompt_ref.cast::<HtmlElement>()
                && let Err(error) = element.focus()
            {
                log::warn!("Could not focus MCQ prompt: {error:?}");
            }
        });
    }
    {
        let select = session.select.clone();
        use_effect_with(
            (session.ready, session.state.selected),
            move |(ready, selected)| {
                let listener = if *ready && selected.is_none() {
                    web_sys::window().map(|window| EventListener::new(&window,"keydown",move |event| {
                    let Some(event) = event.dyn_ref::<KeyboardEvent>() else {return};
                    if event.repeat() || event.ctrl_key() || event.alt_key() || event.meta_key() || event.is_composing() {return;}
                    if event.target().and_then(|target|target.dyn_into::<Element>().ok()).and_then(|target|target.closest("input, textarea, select, [contenteditable], [data-listen]").ok().flatten()).is_some() {return;}
                    let index = match event.key().as_str() {"1"=>0,"2"=>1,"3"=>2,_=>return};
                    event.prevent_default();
                    select.emit(index);
                }))
                } else {
                    None
                };
                move || drop(listener)
            },
        );
    }
    let prepared = match &props.status {
        PracticeStatus::Loading => return html! {<Loader />},
        PracticeStatus::Failed(_) | PracticeStatus::InsufficientPool => {
            let retry = props.retry.clone();
            return html! {<RoutePlaceholder title="Practice unavailable" message="This round could not be prepared. Try again or choose another level."><button type="button" data-practice-retry="" class="mt-6 min-h-11 rounded px-3 py-2 text-accent hover:bg-raised" onclick={Callback::from(move |_|retry.emit(()))}>{"Retry"}</button></RoutePlaceholder>};
        }
        PracticeStatus::Ready(prepared) => prepared,
    };
    if !session.ready {
        return html! {<Loader />};
    }
    let RoundContent::Mcq {
        prompt,
        options,
        correct_id,
    } = &prepared.round.content
    else {
        log::error!("MCQ route received matching round content");
        return html! {<RoutePlaceholder title="Practice unavailable" message="Choose another activity from setup." />};
    };
    let prompt_hanzi = props.variant == ExerciseVariant::HanziToMeaning;
    let answer_hanzi = props.variant == ExerciseVariant::MeaningToHanzi;
    let answer_audio = props.variant != ExerciseVariant::HanziToMeaning;
    let short = options.iter().all(|card| card.label.chars().count() <= 24);
    let next = session.advance.clone();
    let total = match prepared.query.mode {
        PracticeMode::Fixed(rounds) => Some(rounds.count()),
        PracticeMode::Endless => None,
    };
    if session.state.finished_at.is_some()
        && let Some(_total) = total
    {
        return html! {<PracticeSummary correct={session.state.first_attempt as u32} total={session.required as u32} mistakes={session.state.mistakes} elapsed_ms={session.state.elapsed_ms()} highest_streak={session.state.highest_streak} restart={session.restart} back_to_setup={session.back_to_setup} navigation_failed={session.navigation_failed} />};
    }
    html! {
        <section data-mcq="" class="mx-auto flex w-full min-w-0 max-w-4xl flex-col gap-6 rounded-2xl border border-line bg-white p-4 shadow-xl shadow-slate-950/10 md:p-6 dark:bg-slate-950">
            <header class="flex flex-wrap items-center justify-between gap-3 text-sm text-muted">
                <h1 class="text-lg font-semibold text-ink">{crate::route::Route::practice(props.variant).title()}</h1>
                <div class="flex items-center gap-5">
                    <span data-mcq-streak={session.state.streak.to_string()} class="inline-flex items-center gap-1">
                        {if session.state.streak >= 3 {html!{<span data-mcq-flame="" class="text-accent" aria-hidden="true"><Icon data={IconData::LUCIDE_FLAME} width="18px" height="18px" /></span>}} else {Html::default()}}
                        {format!("Streak: {}",session.state.streak)}
                    </span>
                    <span data-mcq-correct={session.state.correct.to_string()}>{format!("Correct: {}",session.state.correct)}</span>
                </div>
            </header>
            {if let Some(total)=total {html!{<progress data-mcq-progress="" aria-label="Completed questions" value={session.state.progress.map_or(0,|progress|progress.completed()).to_string()} max={total.to_string()} class="h-3 w-full accent-blue-600" />}} else {Html::default()}}
            {if session.state.looped {html!{<p class="text-sm text-muted">{"pool exhausted, looping"}</p>}} else {Html::default()}}
            <div class="flex min-h-40 flex-col items-center justify-center gap-3 rounded-2xl border border-line bg-surface p-6 text-center">
                <h2 data-mcq-prompt="" tabindex="-1" ref={prompt_ref} class="max-w-full break-words text-4xl leading-relaxed font-semibold md:text-5xl">{&prompt.label}</h2>
                {if prompt_hanzi {prepared.words.get(&prompt.word_id).map(|word|html!{
                    <>
                        {word.traditional.as_ref().filter(|traditional|*traditional != &word.simplified).map(|traditional|html!{<p data-traditional="" class="text-2xl text-muted">{traditional}</p>}).unwrap_or_default()}
                        <p data-practice-pinyin="" class="text-lg text-muted">{&word.pinyin}</p>
                        <ListenButton hanzi={word.simplified.clone()} />
                    </>
                }).unwrap_or_default()} else {Html::default()}}
            </div>
            <div data-mcq-options="" class={classes!("grid","gap-3", if short {"lg:grid-cols-3"} else {"grid-cols-1"})}>
                {for options.iter().enumerate().map(|(index,card)| {
                    let selected = session.state.selected == Some(index);
                    let answered = session.state.selected.is_some();
                    let correct = card.word_id == *correct_id;
                    let select = session.select.clone();
                    let word = prepared.words.get(&card.word_id);
                    html!{
                        <div class="relative flex min-w-0 flex-col gap-2">
                            <button type="button" data-mcq-option={index.to_string()} data-feedback={if selected {if correct {"correct"} else {"wrong"}} else {"none"}}
                                disabled={answered} aria-pressed={selected.to_string()}
                                class={classes!("flex","min-h-24","w-full","min-w-0","items-center","gap-3","rounded-xl","border-2","p-4","text-left", if selected && correct {"border-emerald-600"} else if selected {"border-red-600"} else {"border-line"}, if selected && correct {"bg-emerald-100"} else if selected {"bg-red-100"} else {"bg-surface"}, if selected && correct {"text-emerald-950"} else if selected {"text-red-950"} else if answered {"text-muted"} else {"text-ink"}, if answered && !selected {"opacity-50"} else {"opacity-100"})}
                                onclick={Callback::from(move |_|select.emit(index))}>
                                <span class="grid size-7 shrink-0 place-items-center rounded border border-current text-sm" aria-hidden="true">{index+1}</span>
                                <span class="min-w-0 flex-1 break-words text-xl">
                                    <span>{&card.label}</span>
                                    {if answer_hanzi {word.and_then(|word|word.traditional.as_ref().filter(|traditional|*traditional!=&word.simplified)).map(|traditional|html!{<span data-traditional="" class="mt-1 block text-base text-muted">{traditional}</span>}).unwrap_or_default()} else {Html::default()}}
                                </span>
                                {if selected {html!{<span class="shrink-0 text-2xl" aria-label={if correct {"Correct"} else {"Incorrect"}}>{if correct {"✓"} else {"✗"}}</span>}} else {Html::default()}}
                            </button>
                            {if answer_audio {word.map(|word|html!{<div class="flex justify-end"><ListenButton hanzi={word.simplified.clone()} /></div>}).unwrap_or_default()} else {Html::default()}}
                        </div>
                    }
                })}
            </div>
            <div class="sticky bottom-0 z-10 -mx-4 -mb-4 mt-1 flex flex-col items-stretch gap-2 rounded-b-2xl border-t border-line bg-white px-4 py-3 sm:items-end md:-mx-6 md:-mb-6 md:px-6 dark:bg-slate-950">
                {if session.navigation_failed {html!{<p role="alert" class="text-sm text-muted">{"Couldn't continue. Try again."}</p>}} else {Html::default()}}
                <button type="button" data-mcq-continue="" disabled={session.state.selected.is_none()} onclick={Callback::from(move |_|next.emit(()))} class="min-h-12 w-full rounded-xl sm:w-auto sm:min-w-44 bg-blue-600 px-5 py-3 font-semibold text-white hover:bg-blue-700 disabled:cursor-default disabled:opacity-50">{"Continue"}</button>
            </div>
        </section>
    }
}
