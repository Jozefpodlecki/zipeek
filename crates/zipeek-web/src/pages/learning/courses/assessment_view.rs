use super::assessment::items_for_region;
use super::assessment_session::AssessmentSession;
use super::construction_assessment_view::ConstructionAssessment;
use super::curriculum::authored_grammar_region;
use super::progress;
use crate::route::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GrammarAssessmentProps {
    pub course: String,
}

#[function_component(GrammarAssessmentPage)]
pub fn grammar_assessment_page(props: &GrammarAssessmentProps) -> Html {
    let region = props
        .course
        .strip_prefix("grammar-")
        .and_then(|id| id.parse::<usize>().ok());
    let authored = region.and_then(authored_grammar_region);
    let items = region.map(items_for_region).unwrap_or_default();
    let count = items.len();

    let initial_items = items.clone();
    let storage_error = use_state(|| None::<String>);
    let previous_record = progress::load()
        .ok()
        .and_then(|saved| saved.grammar_assessments.get(&props.course).cloned());
    let session = use_state(move || AssessmentSession::for_items(&initial_items));

    {
        let session = session.clone();
        use_effect_with(props.course.clone(), move |course| {
            let region = course
                .strip_prefix("grammar-")
                .and_then(|id| id.parse::<usize>().ok());
            let next_items = region.map(items_for_region).unwrap_or_default();
            session.set(AssessmentSession::for_items(&next_items));
        });
    }
    let Some((course, _)) = authored else {
        return html! { <p role="alert">{"This grammar assessment has not been authored."}</p> };
    };
    let submitted = session.is_submitted();
    let answered = session.answered_count();
    let review = session.review(&items);
    let onsubmit = {
        let session = session.clone();
        let storage_error = storage_error.clone();
        let current_items = items.clone();
        let course_id = props.course.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let mut updated = (*session).clone();
            if updated.submit().is_ok() {
                if let Some(report) = updated.review(&current_items) {
                    storage_error.set(
                        progress::update(|saved| {
                            saved.record_grammar_assessment(
                                &course_id,
                                report.correct,
                                report.total,
                            )
                        })
                        .err(),
                    );
                }
                session.set(updated);
            }
        })
    };
    let restart = {
        let session = session.clone();
        let restart_items = items.clone();
        Callback::from(move |_: MouseEvent| {
            session.set(AssessmentSession::for_items(&restart_items))
        })
    };
    html! {
        <article class="mx-auto flex w-full max-w-3xl flex-col gap-6 py-4" data-grammar-assessment={course.id}>
            <Link<Route> to={Route::GrammarCourse} classes="text-sm font-semibold text-accent">{"← Courses mosaic"}</Link<Route>>
            <header class="space-y-2">
                <p class="text-sm font-semibold text-accent">{"Independent grammar check · Draft"}</p>
                <h1 class="text-3xl font-bold">{course.title}{" — Assessment"}</h1>
                <p class="text-muted">{"Answer without hints. Explanations appear after you submit every question."}</p>
                <p class="text-sm text-muted">{"Your best assessment score is saved in this browser. These draft questions are practice evidence, not verified mastery and do not unlock mosaic artwork."}</p>
                if let Some(best) = &previous_record {
                    <p data-grammar-best-score="" class="text-sm font-semibold text-accent">{format!("Previous best: {} of {}", best.correct, best.total)}</p>
                }
                if let Some(message) = &*storage_error {
                    <p role="alert" class="text-sm text-muted">{format!("Could not save assessment: {message}")}</p>
                }
            </header>
            <form onsubmit={onsubmit} class="space-y-5" aria-label="Independent grammar questions">
                <p role="status" aria-live="polite" class="text-sm text-muted" data-assessment-announcement="">
                    {if submitted { "Assessment submitted. Review the results and each grammar point below.".to_string() }
                        else { format!("{answered} of {count} questions answered. Explanations are hidden until submission.") }}
                </p>
                {for items.iter().enumerate().map(|(index, item)| {
                    let answer = session.response(index);
                    html! {
                        <fieldset id={format!("assessment-question-{}", item.id)} class="rounded-xl border border-line bg-surface p-5 space-y-3 scroll-mt-24" data-assessment-item={item.id} key={item.id}>
                            <legend class="font-semibold">{format!("{}. {}",index+1,item.prompt)}</legend>
                            <div class="grid gap-2">
                                {for item.choices.iter().enumerate().map(|(choice, text)| {
                                    let session = session.clone();
                                    let onchoose = Callback::from(move |_: MouseEvent| {
                                        let mut updated = (*session).clone();
                                        if updated.answer(index, choice).is_ok() { session.set(updated); }
                                    });
                                    html! {
                                        <button type="button" lang="zh-Hans" disabled={submitted} aria-pressed={(answer==Some(choice)).to_string()}
                                            class={classes!("rounded-lg", "border", "border-line", "bg-raised", "px-4", "py-3", "text-left", "text-lg", "hover:border-accent", "focus-visible:outline", "focus-visible:outline-2", "focus-visible:outline-accent", if answer==Some(choice) {"ring-2"} else {""})}
                                            onclick={onchoose}>{*text}</button>
                                    }
                                })}
                            </div>
                            if submitted && review.is_some() {
                                <div role="status" class="text-sm space-y-1">
                                    <p class="font-semibold">{if answer==Some(item.correct) {"Correct."} else {"Review needed."}}</p>
                                    <p>{format!("Correct answer: {}",item.choices[item.correct])}</p>
                                    <p>{item.rationale}</p>
                                </div>
                            }
                        </fieldset>
                    }
                })}
                if submitted && review.is_none() {
                    <p role="alert" class="text-sm text-muted" data-assessment-invalid-review="">{"Assessment content changed. Restart this attempt to review the current questions."}</p>
                    <button type="button" class="rounded-lg border border-line px-4 py-2" onclick={restart.clone()}> {"Restart assessment"} </button>
                } else if let Some(report) = &review {
                    <section class="rounded-xl border border-line bg-raised p-5 space-y-3" data-assessment-review="">
                        <h2 class="text-xl font-bold">{format!("Draft score: {} / {}",report.correct,report.total)}</h2>
                        <div class="space-y-2" data-assessment-point-review="">
                                <h3 class="font-semibold">{"Review by grammar point"}</h3>
                                <ul class="space-y-1">
                                    {for report.points.iter().map(|point| {
                                        let item_id = point.item_id;
                                        let on_jump = Callback::from(move |_: MouseEvent| {
                                            if let Some(element) = web_sys::window()
                                                .and_then(|window| window.document())
                                                .and_then(|document| document.get_element_by_id(&format!("assessment-question-{item_id}")))
                                            {
                                                element.scroll_into_view();
                                            }
                                        });
                                        html! {
                                            <li key={point.item_id} class="flex flex-wrap items-center justify-between gap-2 text-sm">
                                                <button type="button" onclick={on_jump} class="rounded text-accent underline-offset-4 hover:underline focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent" aria-label={format!("Go to grammar point {} question; {}", point.point_id, if point.correct { "correct" } else { "review needed" })}>
                                                    <code>{point.point_id}</code>
                                                </button>
                                                <span>{if point.correct {"Correct"} else {"Review needed"}}</span>
                                            </li>
                                        }
                                    })}
                                </ul>
                                <p class="text-sm text-muted">{"This attempt contributes to your saved best practice score, not verified mastery."}</p>
                        </div>
                        <p class="text-sm text-muted" data-assessment-review-needed="">
                            {format!("{} grammar points need review. Select any point above to jump to its explanation.", report.total - report.correct)}
                        </p>
                        <p class="text-sm text-muted">{"This practice assessment does not unlock Grammar mosaic artwork."}</p>
                        <button type="button" class="rounded-lg border border-line px-4 py-2" onclick={restart}>{"Try another attempt"}</button>
                    </section>
                } else {
                    <p class="text-sm text-muted" data-assessment-progress="">{format!("Answered {answered} of {count}")}</p>
                    <button type="submit" disabled={answered!=count} class="rounded-xl bg-brand-action px-5 py-3 font-semibold text-white disabled:opacity-50">{"Submit assessment"}</button>
                }
            </form>
            <ConstructionAssessment key={props.course.clone()} region={region.unwrap_or_default()} />
        </article>
    }
}
