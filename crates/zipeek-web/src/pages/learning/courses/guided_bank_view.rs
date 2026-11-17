use super::guided_bank::{GuidedCourse, GuidedKind, GuidedSession, accepts, reference_answer};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GuidedGrammarBankProps {
    #[prop_or_default]
    pub course: GuidedCourse,
}

#[function_component(GuidedGrammarBank)]
pub fn guided_grammar_bank(props: &GuidedGrammarBankProps) -> Html {
    let course = props.course;
    let activities = course.activities();
    let session = use_state(move || GuidedSession::for_course(course));
    let reference_visible = use_state(|| false);
    {
        let reference_visible = reference_visible.clone();
        use_effect_with(session.index, move |_| {
            reference_visible.set(false);
        });
    }
    if session.finished() {
        let restart = {
            let session = session.clone();
            Callback::from(move |_: MouseEvent| session.set(GuidedSession::for_course(course)))
        };
        return html! {
            <section data-guided-bank-review="" class="rounded-2xl border border-line bg-surface p-6 space-y-3" aria-live="polite">
                <h2 class="text-xl font-bold">{"Guided word-bank review"}</h2>
                <p>{format!("{} of {} teaching exercises correct", session.correct, activities.len())}</p>
                <p class="text-sm text-muted">{"This is guided practice with optional reference answers, not an independent assessment. No grammar acquisition or artwork reveal is awarded."}</p>
                <button type="button" onclick={restart} class="rounded-lg border border-line px-4 py-2">{"Practice again"}</button>
            </section>
        };
    }
    let activity = &activities[session.index];
    let add_tile = {
        let session = session.clone();
        Callback::from(move |id: usize| {
            let mut next = (*session).clone();
            next.select(id);
            session.set(next);
        })
    };
    let remove_tile = {
        let session = session.clone();
        Callback::from(move |position: usize| {
            let mut next = (*session).clone();
            next.remove(position);
            session.set(next);
        })
    };
    let undo = {
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*session).clone();
            next.undo();
            session.set(next);
        })
    };
    let reset = {
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*session).clone();
            next.reset_selection();
            session.set(next);
        })
    };
    let check = {
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*session).clone();
            next.check();
            session.set(next);
        })
    };
    let advance = {
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*session).clone();
            next.advance();
            session.set(next);
        })
    };
    let correct = accepts(activity, &session.selected);
    let reference = reference_answer(activity);
    let toggle_reference = {
        let reference_visible = reference_visible.clone();
        Callback::from(move |_: MouseEvent| reference_visible.set(!*reference_visible))
    };
    html! {
        <section data-guided-grammar-bank="" data-guided-activity={activity.id} class="space-y-4 rounded-2xl border border-line bg-surface p-5" aria-label="Guided grammar word bank">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <p class="text-xs font-semibold text-accent">{format!("Teaching activity {} of {}",session.index+1,activities.len())}</p>
                <p class="text-xs text-muted">{match activity.kind { GuidedKind::SentenceBuilder => "Sentence Builder", GuidedKind::SentenceTransformation => "Sentence Transformation" }}</p>
            </div>
            <h3 class="text-lg font-bold">{activity.title}</h3>
            <p>{activity.instruction}</p>
            <p class="text-sm"><strong>{"Starting sentence: "}</strong><span lang={if activity.kind == GuidedKind::SentenceBuilder { "en" } else { "zh-Hans" }}>{activity.source}</span></p>
            <p class="text-sm"><strong>{"Target pattern: "}</strong>{activity.target_pattern}</p>
            <div class="rounded-lg border border-line bg-raised px-4 py-3 space-y-2">
                <button type="button" onclick={toggle_reference} aria-expanded={(*reference_visible).to_string()} class="rounded-lg border border-line px-3 py-2 text-sm font-medium focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent">
                    {if *reference_visible {"Hide reference answer"} else {"Reveal reference answer"}}
                </button>
                if *reference_visible {
                    <p lang="zh-Hans" class="text-lg font-semibold">{reference.clone()}{"。"}</p>
                }
            </div>
            <div data-guided-assembled="" role="group" aria-label="Constructed Chinese sentence; select a tile to remove" lang="zh-Hans" class="flex min-h-16 flex-wrap items-center gap-2 rounded-lg border border-line p-3">
                {for session.selected.iter().enumerate().map(|(position, &id)|{
                    let remove_tile = remove_tile.clone();
                    let onclick = Callback::from(move |_: MouseEvent| remove_tile.emit(position));
                    html! { <button key={position.to_string()} type="button" onclick={onclick} disabled={session.checked} aria-label={format!("Remove {} from position {}",activity.bank[id],position+1)} class="rounded-md bg-raised px-3 py-2 disabled:opacity-70">{activity.bank[id]}</button> }
                })}
                if session.selected.is_empty() { <span class="text-sm text-muted">{"Select tiles to build the sentence"}</span> }
            </div>
            <div role="group" aria-label="Available Chinese tiles" lang="zh-Hans" class="flex flex-wrap gap-2">
                {for activity.presented_order.iter().map(|&id|{
                    let add_tile = add_tile.clone();
                    let onclick = Callback::from(move |_: MouseEvent| add_tile.emit(id));
                    html! { <button key={id.to_string()} type="button" onclick={onclick} disabled={session.checked || session.selected.contains(&id)} aria-label={format!("Add {}",activity.bank[id])} class="rounded-lg border border-line bg-raised px-4 py-3 focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-40">{activity.bank[id]}</button> }
                })}
            </div>
            if !session.checked {
                <div class="flex flex-wrap gap-2">
                    <button type="button" onclick={undo} disabled={session.selected.is_empty()} class="rounded-lg border border-line px-4 py-2 disabled:opacity-40">{"Undo"}</button>
                    <button type="button" onclick={reset} disabled={session.selected.is_empty()} class="rounded-lg border border-line px-4 py-2 disabled:opacity-40">{"Reset"}</button>
                    <button type="button" onclick={check} disabled={session.selected.is_empty()} data-guided-check="" class="rounded-lg bg-brand-action px-4 py-2 text-white disabled:opacity-40">{"Check answer"}</button>
                </div>
            } else {
                <div data-guided-feedback="" role="status" aria-live="polite" class="space-y-3 rounded-lg border border-line bg-raised p-4">
                    <p class="font-semibold">{if correct { "Correct" } else { "Not quite" }}</p>
                    <p class="text-sm">{if correct { activity.explanation } else { activity.incorrect_feedback }}</p>
                    <p lang="zh-Hans" class="text-base">{reference}{"。"}</p>
                    <button type="button" data-guided-next="" onclick={advance} class="rounded-lg bg-brand-action px-4 py-2 font-semibold text-white">{if session.index+1==activities.len() {"Finish guided practice"} else {"Next teaching activity"}}</button>
                </div>
            }
            <p class="text-xs text-muted">{"Reference answers are available on request. These activities do not establish independently verified mastery."}</p>
        </section>
    }
}
