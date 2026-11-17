use super::construction_assessment::{ConstructionSession, item_for_region};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ConstructionAssessmentProps {
    pub region: usize,
}

#[function_component(ConstructionAssessment)]
pub fn construction_assessment(props: &ConstructionAssessmentProps) -> Html {
    let Some(item) = item_for_region(props.region) else {
        return Html::default();
    };
    let session =
        use_state(|| ConstructionSession::new(item).expect("validated construction item"));
    let add = {
        let session = session.clone();
        Callback::from(move |index: usize| {
            let mut next = (*session).clone();
            if next.add(index).is_ok() {
                session.set(next);
            }
        })
    };
    let remove = {
        let session = session.clone();
        Callback::from(move |position: usize| {
            let mut next = (*session).clone();
            if next.remove(position).is_ok() {
                session.set(next);
            }
        })
    };
    let undo = {
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*session).clone();
            if next.undo().is_ok() {
                session.set(next);
            }
        })
    };
    let reset = {
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*session).clone();
            if next.reset().is_ok() {
                session.set(next);
            }
        })
    };
    let submit = {
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*session).clone();
            if next.submit().is_ok() {
                session.set(next);
            }
        })
    };
    let restart = {
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            session.set(ConstructionSession::new(item).expect("validated construction item"))
        })
    };
    let submitted = session.is_submitted();
    let review = session.review(item);
    html! {
        <section data-construction-assessment={item.id} class="space-y-4 rounded-2xl border border-line bg-surface p-5" aria-label="Independent Chinese sentence construction">
            <div class="space-y-2">
                <p class="text-xs font-semibold text-accent">{"Independent sentence construction · Draft"}</p>
                <h2 class="text-xl font-bold">{"Build a Chinese sentence without hints"}</h2>
                <p>{item.instruction}</p>
                <p class="text-sm text-muted">{"Choose the Chinese tiles in order. Distractors are included; the answer and explanation appear only after submission."}</p>
            </div>
            <p role="status" aria-live="polite" class="text-sm text-muted" data-construction-progress="">
                {if submitted { "Sentence submitted. Read the review below.".to_string() }
                    else { format!("{} of {} Chinese tiles selected. Use the tile buttons to add or remove words.", session.selected().len(), item.expected_order.len()) }}
            </p>
            <div data-construction-selected="" role="group" aria-label="Your selected Chinese sentence; select a tile to remove it" lang="zh-Hans" class="flex min-h-16 flex-wrap gap-2 rounded-lg border border-line p-3">
                {for session.selected().iter().enumerate().map(|(position, &index)| {
                    let remove = remove.clone();
                    let onclick = Callback::from(move |_: MouseEvent| remove.emit(position));
                    html! { <button key={index.to_string()} type="button" disabled={submitted} onclick={onclick} aria-label={format!("Remove {} from position {}", item.bank[index], position + 1)} class="rounded-md bg-raised px-3 py-2 focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-70">{item.bank[index]}</button> }
                })}
                if session.selected().is_empty() { <span class="text-sm text-muted">{"Select tiles to build your answer."}</span> }
            </div>
            <div role="group" aria-label="Available Chinese answer tiles" lang="zh-Hans" class="flex flex-wrap gap-2">
                {for item.display_order.iter().map(|&index| {
                    let add = add.clone();
                    let onclick = Callback::from(move |_: MouseEvent| add.emit(index));
                    html! { <button key={index.to_string()} type="button" disabled={submitted || session.selected().contains(&index) || session.is_ready()} onclick={onclick} aria-label={format!("Add {} to sentence", item.bank[index])} class="rounded-lg border border-line bg-raised px-4 py-3 focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-40">{item.bank[index]}</button> }
                })}
            </div>
            if !submitted {
                <div class="flex flex-wrap gap-2">
                    <button type="button" onclick={undo} disabled={session.selected().is_empty()} class="rounded-lg border border-line px-4 py-2 disabled:opacity-40">{"Undo"}</button>
                    <button type="button" onclick={reset} disabled={session.selected().is_empty()} class="rounded-lg border border-line px-4 py-2 disabled:opacity-40">{"Reset"}</button>
                    <button type="button" data-construction-submit="" onclick={submit} disabled={!session.is_ready()} class="rounded-lg bg-brand-action px-4 py-2 text-white disabled:opacity-40">{"Submit construction"}</button>
                </div>
            } else if let Some(correct) = review {
                <div data-construction-review="" role="status" aria-live="polite" class="space-y-2 rounded-lg border border-line bg-raised p-4">
                    <p class="font-semibold">{if correct { "Correct sentence" } else { "Review needed" }}</p>
                    <p lang="zh-Hans" class="text-lg">{for item.expected_order.iter().map(|&index| item.bank[index])}{"。"}</p>
                    <p>{item.explanation}</p>
                    <p class="text-xs text-muted">{"Draft-only result. No proficiency or mosaic reveal is recorded."}</p>
                    <button type="button" onclick={restart} class="rounded-lg border border-line px-4 py-2">{"Try another construction"}</button>
                </div>
            } else {
                <p role="alert">{"Authored construction content changed; reload this challenge."}</p>
            }
        </section>
    }
}
