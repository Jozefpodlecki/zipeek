use super::curriculum::{authored_grammar_region, authored_grammar_region_from_slug};
use super::curriculum_registry::{registry_course, registry_entry};
use super::guided_bank::GuidedCourse;
use super::guided_bank_view::GuidedGrammarBank;
use super::progress;
use super::grammar_lesson_scaffold::GrammarLessonScaffold;
use super::grammar_ai_example::GrammarAiExample;
use yew_icons::{Icon, IconData};
use crate::components::ListenButton;
use crate::route::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GrammarLessonProps {
    pub course: GuidedCourse,
}

#[function_component(GrammarLesson)]
fn grammar_lesson(props: &GrammarLessonProps) -> Html {
    let course_order = props.course.region();
    let (course, checks) = authored_grammar_region(course_order).expect("authored grammar course");
    let registry = registry_entry(course_order).expect("published grammar registry");
    let answers = use_state(|| vec![None::<usize>; checks.len()]);
    let teaching_count = course.points.len();
    let check_start = teaching_count;
    let bank_step = check_start + checks.len();
    let assessment_step = bank_step + 1;
    let total_steps = assessment_step + 1;
    let starting_step = progress::load().ok()
        .and_then(|saved| saved.grammar_steps.get(course.id).copied())
        .unwrap_or(0).min(assessment_step);
    let step = use_state(|| starting_step);
    let save_error = use_state(|| None::<String>);
    let current = *step;
    {
        use_effect_with(current, move |_| {
            if let Some(window) = web_sys::window() {
                window.scroll_to_with_x_and_y(0.0, 0.0);
            }
        });
    }
    let next = {
        let step = step.clone();
        let save_error = save_error.clone();
        let course_id = course.id.to_owned();
        Callback::from(move |_: MouseEvent| {
            let value = (*step + 1).min(assessment_step);
            step.set(value);
            save_error.set(progress::update(|saved| saved.mark_grammar_step(&course_id, value, total_steps)).err());
        })
    };
    let previous = {
        let step = step.clone();
        let save_error = save_error.clone();
        let course_id = course.id.to_owned();
        Callback::from(move |_: MouseEvent| {
            let value = step.saturating_sub(1);
            step.set(value);
            save_error.set(progress::update(|saved| saved.mark_grammar_step(&course_id, value, total_steps)).err());
        })
    };
    let next_allowed = if (check_start..bank_step).contains(&current) {
        answers[current - check_start].is_some()
    } else {
        true
    };
    let answered = answers.iter().filter(|item| item.is_some()).count();
    let correct = answers
        .iter()
        .enumerate()
        .filter(|(i, answer)| **answer == Some(checks[*i].correct))
        .count();
    let restart = {
        let answers = answers.clone();
        let step = step.clone();
        let save_error = save_error.clone();
        let course_id = course.id.to_owned();
        Callback::from(move |_: MouseEvent| {
            answers.set(vec![None; checks.len()]);
            step.set(check_start);
            save_error.set(progress::update(|saved| saved.mark_grammar_step(&course_id, check_start, total_steps)).err());
        })
    };
    let stage_content = if current < teaching_count {
        let point = &course.points[current];
        html! {
                        <section class="space-y-5 rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8" aria-label="Grammar explanation" data-grammar-point-id={registry.point_ids[current]}>
                            <p class="text-xs font-semibold uppercase tracking-wide text-muted">{format!("Grammar point {} of {}", current + 1, teaching_count)}</p>
                            <h2 class="text-2xl font-bold">{point.label}</h2>
                            <p class="leading-8">{point.explanation}</p>
                            <div class="space-y-2 rounded-xl border border-line bg-raised p-5">
                                <p lang="zh-Hans" class="text-2xl font-semibold leading-relaxed">{point.example}</p>
                                if let Some(pinyin) = zipeek_pedagogy::grammar::pinyin::example_pinyin(course_order, current) {
                                    <p lang="zh-Latn" class="text-sm leading-6 text-muted">{pinyin}</p>
                                }
                                <div class="flex items-center justify-between gap-3">
                                    <p class="text-sm text-muted">{point.translation}</p>
                                    <ListenButton hanzi={point.example.to_owned()} compact=true />
                                </div>
                            </div>
                            <GrammarAiExample key={registry.point_ids[current]} point_id={registry.point_ids[current].to_owned()}
                                pattern={point.label.to_owned()} explanation={point.explanation.to_owned()} example={point.example.to_owned()} />
                        </section>
        }
    } else if current < bank_step {
        let index = current - check_start;
        let question = &checks[index];
        let answer = answers[index];
        html! {
                        <fieldset class="space-y-4 rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8" data-grammar-question={index.to_string()}>
                            <legend class="text-lg font-bold">{format!("Check {} of {}: {}", index + 1, checks.len(), question.prompt)}</legend>
                            <div class="grid gap-3">
                                {for question.choices.iter().enumerate().map(|(choice_index, choice)| {
                                    let answers = answers.clone();
                                    let picked = answer == Some(choice_index);
                                    let onclick = Callback::from(move |_: MouseEvent| {
                                        if answers[index].is_none() {
                                            let mut updated = (*answers).clone();
                                            updated[index] = Some(choice_index);
                                            answers.set(updated);
                                        }
                                    });
                                    html! {
                                        <button type="button" lang="zh-Hans" disabled={answer.is_some()}
                                            aria-pressed={picked.to_string()}
                                            class="rounded-xl border border-line bg-raised px-4 py-3 text-left text-lg hover:border-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-80"
                                            onclick={onclick}>{*choice}</button>
                                    }
                                })}
                            </div>
                            if let Some(chosen) = answer {
                                <p role="status" class="rounded-lg bg-raised p-3 text-sm leading-6">
                                    {if chosen == question.correct {"Correct. "} else {"Not quite. "}}{question.rationale}
                                </p>
                            }
                        </fieldset>
        }
    } else if current == bank_step {
        html! { <section class="space-y-4 rounded-2xl border border-line bg-surface p-5 sm:p-7" aria-label="Guided sentence building and transformation">
                    <h2 class="text-xl font-bold">{"Build and transform sentences"}</h2>
                    <p class="text-sm text-muted">{"Rehearse the patterns with the word bank. Reference answers remain visible in this guided activity."}</p>
                    <GuidedGrammarBank course={props.course} />
                </section> }
    } else {
        html! { <section class="space-y-4 rounded-2xl border border-line bg-surface p-6 sm:p-8" aria-label="Independent grammar assessment">
                    <h2 class="text-2xl font-bold">{"Check your understanding"}</h2>
                    <p class="text-muted">{"Try the independent assessment without hints. Guided practice does not count as mastery."}</p>
                    <p data-grammar-review="" class="text-sm text-muted">{format!("Guided checks: {correct} of {answered} correct")}</p>
                    <div class="flex flex-wrap gap-3">
                        <Link<Route> to={Route::GrammarAssessment { course: course.id.to_owned() }} classes="inline-flex rounded-lg bg-brand-action px-5 py-3 font-semibold text-white no-underline">{"Start assessment"}</Link<Route>>
                        <button type="button" class="rounded-lg border border-line px-4 py-2" onclick={restart}>{"Retry guided checks"}</button>
                    </div>
                </section> }
    };
    html! {
        <GrammarLessonScaffold title={course.title} course_id={course.id} current={current} total_steps={total_steps}>
            {stage_content}
            if let Some(message) = &*save_error {
                <p role="alert" class="rounded-lg border border-line bg-surface p-3 text-sm">{format!("Lesson checkpoint could not be saved: {message}")}</p>
            }
            <div class="flex items-center justify-between gap-3">
                <button type="button" data-grammar-previous="" aria-label="Previous step" title="Previous step" disabled={current == 0} onclick={previous} class="inline-flex size-11 items-center justify-center rounded-lg border border-line bg-surface disabled:opacity-40"><Icon data={IconData::LUCIDE_ARROW_LEFT} width="20px" height="20px" /></button>
                if current < assessment_step {
                    <button type="button" data-grammar-next="" disabled={!next_allowed} onclick={next} class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white disabled:opacity-40">{"Continue"}</button>
                }
            </div>
        </GrammarLessonScaffold>
    }
}

#[derive(Properties, PartialEq)]
pub struct AuthoredGrammarLessonProps {
    pub slug: String,
}

#[function_component(AuthoredGrammarLessonPage)]
pub fn authored_grammar_lesson_page(props: &AuthoredGrammarLessonProps) -> Html {
    let Some(course) = authored_course_from_slug(&props.slug) else {
        return html! { <section role="alert" class="mx-auto max-w-3xl p-6">{"Grammar lesson not found."}</section> };
    };
    html! { <GrammarLesson key={props.slug.clone()} course={course} /> }
}

fn authored_course_from_slug(slug: &str) -> Option<GuidedCourse> {
    match authored_grammar_region_from_slug(slug)? {
        1 => Some(GuidedCourse::Intro),
        2 => Some(GuidedCourse::TimePlace),
        3 => Some(GuidedCourse::PossessionQuantity),
        4 => Some(GuidedCourse::Descriptions),
        5 => Some(GuidedCourse::WantsAbilities),
        _ => None,
    }
}

#[cfg(test)]
mod route_tests {
    use super::*;

    #[test]
    fn only_published_authored_slugs_are_accepted() {
        for slug in super::super::curriculum::AUTHORED_GRAMMAR_SLUGS {
            assert!(authored_course_from_slug(slug).is_some());
        }
        for slug in ["", "unknown", "../intro", "topic", "assessment", "INTRO"] {
            assert!(authored_course_from_slug(slug).is_none());
        }
    }
}
