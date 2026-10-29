use crate::{
    components::{Loader, MatchCard, PracticeSummary, RoutePlaceholder},
    hooks::PracticeControls,
    reducers::{
        matching::{CardChoice, Side},
        practice::PracticeStatus,
    },
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
pub(crate) struct MatchingPracticeProps {
    pub variant: ExerciseVariant,
    pub status: PracticeStatus,
    pub retry: Callback<()>,
    pub session: PracticeControls,
}

#[function_component(MatchingPractice)]
pub(crate) fn matching_practice(props: &MatchingPracticeProps) -> Html {
    let session = props.session.matching();
    let heading = use_node_ref();
    {
        let heading = heading.clone();
        use_effect_with(
            (session.state.epoch, session.ready),
            move |(epoch, ready)| {
                if *ready
                    && *epoch > 1
                    && let Some(element) = heading.cast::<HtmlElement>()
                    && let Err(error) = element.focus()
                {
                    log::warn!("Could not focus matching heading: {error:?}");
                }
            },
        );
    }
    {
        let escape = session.escape.clone();
        use_effect_with((session.state.epoch, session.ready), move |(_, ready)| {
            let listener = if *ready {
                web_sys::window().map(|window| {
                    EventListener::new(&window, "keydown", move |event| {
                        let Some(event) = event.dyn_ref::<KeyboardEvent>() else {
                            return;
                        };
                        if event.key() != "Escape" || event.is_composing() {
                            return;
                        }
                        if event
                            .target()
                            .and_then(|target| target.dyn_into::<Element>().ok())
                            .and_then(|target| {
                                target
                                    .closest("input, textarea, select, [contenteditable]")
                                    .ok()
                                    .flatten()
                            })
                            .is_some()
                        {
                            return;
                        }
                        event.prevent_default();
                        escape.emit(());
                    })
                })
            } else {
                None
            };
            move || drop(listener)
        });
    }
    let prepared = match &props.status {
        PracticeStatus::Loading => return html! {<Loader />},
        PracticeStatus::InsufficientPool => {
            let back = session.back_to_setup;
            return html! {<RoutePlaceholder title="Not enough distinct words" message="Not enough distinct words for this filter. Try a different HSK level or reset exclusions."><button type="button" data-match-setup="" class="mt-6 min-h-11 rounded-lg bg-blue-600 px-5 py-2 font-semibold text-white" onclick={Callback::from(move|_|back.emit(()))}>{"Back to setup"}</button></RoutePlaceholder>};
        }
        PracticeStatus::Failed(_) => {
            let retry = props.retry.clone();
            return html! {<RoutePlaceholder title="Practice unavailable" message="This round could not be prepared. Try again or choose another level."><button type="button" data-practice-retry="" class="mt-6 min-h-11 rounded px-3 py-2 text-accent hover:bg-raised" onclick={Callback::from(move|_|retry.emit(()))}>{"Retry"}</button></RoutePlaceholder>};
        }
        PracticeStatus::Ready(prepared) => prepared,
    };
    if !session.ready {
        return html! {<Loader />};
    }
    let RoundContent::Matching { left, right } = &prepared.round.content else {
        log::error!("Matching route received MCQ content");
        return html! {<RoutePlaceholder title="Practice unavailable" message="Choose another activity from setup." />};
    };
    let total = match prepared.query.mode {
        PracticeMode::Fixed(rounds) => Some(rounds.count()),
        PracticeMode::Endless => None,
    };
    if session.state.finished_at.is_some()
        && let Some(_total) = total
    {
        return html! {<PracticeSummary correct={session.state.first_attempt as u32} total={session.required as u32} mistakes={session.state.mistakes} elapsed_ms={session.state.elapsed_ms()} highest_streak={session.state.highest_streak} restart={session.restart} back_to_setup={session.back_to_setup} navigation_failed={session.navigation_failed} />};
    }
    let next = session.advance.clone();
    html! {
        <section data-matching="" class="mx-auto flex w-full min-w-0 max-w-4xl flex-col gap-4 rounded-2xl border border-line bg-white p-4 shadow-xl shadow-slate-950/10 dark:bg-slate-950">
            <header class="flex flex-col gap-3 border-b border-line pb-3 sm:flex-row sm:items-center sm:justify-between">
                <h1 ref={heading} tabindex="-1" class="text-lg font-semibold tracking-tight">{crate::route::Route::practice(props.variant).title()}</h1>
                <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-muted">
                    {total.map(|total|html!{<span data-match-progress={session.state.progress.map_or(0,|progress|progress.completed()).to_string()}>{format!("Round {} of {}",session.state.progress.map_or(0,|progress|progress.completed())+1,total)}</span>}).unwrap_or_default()}
                    <span data-match-streak={session.state.streak.to_string()} class="inline-flex items-center gap-1">
                        {if session.state.streak>=3 {html!{<span data-match-flame="" class="text-accent" aria-hidden="true"><Icon data={IconData::LUCIDE_FLAME} width="18px" height="18px" /></span>}} else {Html::default()}}
                        {format!("Streak: {}",session.state.streak)}
                    </span>
                    <span data-match-correct={session.state.correct.to_string()}>{format!("Correct: {}",session.state.correct)}</span>
                    <span data-match-count={session.state.matched.len().to_string()}>{format!("{} / 4",session.state.matched.len())}</span>
                </div>
            </header>
            {if session.state.looped {html!{<p class="text-sm text-muted">{"pool exhausted, looping"}</p>}} else {Html::default()}}
            <div class="grid min-w-0 grid-cols-2 items-stretch gap-3">
                {for left.iter().zip(right.iter()).enumerate().flat_map(|(index,(left,right))|[(Side::Left,index,left),(Side::Right,index,right)]).map(|(side,index,card)| {
                    let choice=CardChoice {side,index};
                    let word=prepared.words.get(&card.word_id);
                    let hanzi=matches!((props.variant,side),(ExerciseVariant::HanziEng,Side::Left)|(ExerciseVariant::PinyinHanzi,Side::Right));
                    let speak=hanzi || matches!((props.variant,side),(ExerciseVariant::EngPinyin,Side::Right)|(ExerciseVariant::PinyinHanzi,Side::Left));
                    let traditional=if hanzi {word.and_then(|word|word.traditional.as_ref().filter(|traditional|*traditional!=&word.simplified)).map(|value|AttrValue::from(value.clone()))} else {None};
                    let wrong=session.state.wrong.as_ref().is_some_and(|wrong|wrong.first==choice || wrong.second==choice);
                    html!{<MatchCard key={format!("{side:?}-{index}")} {choice} label={card.label.clone()} {traditional} {hanzi} pinyin={if props.variant==ExerciseVariant::HanziEng && side==Side::Left {word.map(|word|AttrValue::from(word.pinyin.clone()))} else {None}} speak={if speak {word.map(|word|AttrValue::from(word.simplified.clone()))} else {None}} selected={session.state.selected==Some(choice)} {wrong} matched={session.state.matched.contains(&card.word_id)} locked={session.state.wrong.is_some()} on_select={session.select.clone()} />}
                })}
            </div>
            <div class="sticky bottom-0 z-10 -mx-4 -mb-4 mt-1 flex flex-col items-stretch gap-2 rounded-b-2xl border-t border-line bg-white px-4 py-2 sm:items-end dark:bg-slate-950">
                {if session.navigation_failed {html!{<p role="alert" class="text-sm text-muted">{"Couldn't continue. Try again."}</p>}} else {Html::default()}}
                <button type="button" data-match-next="" disabled={!session.state.complete()} class="min-h-11 w-full rounded-xl sm:w-auto sm:min-w-44 bg-blue-600 px-5 py-2 font-semibold text-white hover:bg-blue-700 disabled:cursor-default disabled:opacity-50" onclick={Callback::from(move|_|next.emit(()))}>{"Next round"}</button>
            </div>
        </section>
    }
}
