use super::super::curriculum::{authored_grammar_region, authored_grammar_slug};
use super::super::curriculum_registry::{registry_course, registry_entry};
use super::super::progress;
use super::super::grammar_expansion::expanded_course;
use super::super::grammar_plan::planned_title;
use crate::route::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CourseDetailsProps {
    pub selected: usize,
}

#[function_component(CourseDetails)]
pub fn course_details(props: &CourseDetailsProps) -> Html {
    let authored = authored_grammar_region(props.selected);
    let expanded = expanded_course(props.selected);
    let display_course = authored.map(|(course, _)| course).or(expanded);
    let registered = registry_entry(props.selected);
    let saved = progress::load().ok();
    html! {
        <aside data-course-details="" class="min-w-0 rounded-xl border border-line bg-surface p-4 shadow-sm lg:h-full lg:overflow-y-auto" aria-live="polite">
                <div class="space-y-2">
                <h2 class="text-xl font-bold text-ink">{display_course.map(|course| course.title.to_owned()).or_else(|| planned_title(props.selected).map(str::to_owned)).unwrap_or_else(|| "More grammar courses coming".to_owned())}</h2>
                <p class="text-sm leading-5 text-muted">
                    {display_course.map(|course| course.description).unwrap_or("This topic is planned. Teaching examples and independent assessments have not been authored yet.")}
                </p>
                if let Some(course) = display_course {
                    <section class="space-y-2" aria-label="Grammar points">
                        <h3 class="text-sm font-bold">{"What you will learn"}</h3>
                        <ul class="grid gap-1.5 text-sm">
                            {for course.points.iter().map(|point| html! {
                                <li class="rounded-lg bg-raised px-3 py-1.5">{point.label}</li>
                            })}
                        </ul>
                    </section>
                    if let Some(registry) = registered {
                        <section class="space-y-1" aria-label="Curriculum sequence">
                            <p class="text-sm text-muted">{registry.teaching_stage_label()}</p>
                            <p class="text-xs text-muted" data-syllabus-status="unverified">{registry.syllabus_status_label()}</p>
                            if !registry.prerequisites.is_empty() {
                                <p class="text-sm text-muted">{"Recommended prerequisite: "}{registry.prerequisites.iter().filter_map(|id| registry_course(id)).map(|item| item.title).collect::<Vec<_>>().join(", ")}</p>
                            }
                        </section>
                    }
                    if let Some(score) = saved.as_ref().and_then(|record| record.grammar_assessments.get(course.id)) {
                        <p class="text-sm text-muted" data-grammar-best-score="">{format!("Best independent practice: {} of {}", score.correct, score.total)}</p>
                    }
                    if let Some(checkpoint) = saved.as_ref().and_then(|record| record.grammar_steps.get(course.id)) {
                        if *checkpoint > 0 {
                            <p class="text-sm text-accent">{"Lesson checkpoint saved"}</p>
                        }
                    }
                    <div class="pt-2">
                        <Link<Route> to={authored_grammar_slug(props.selected).map_or_else(|| Route::GrammarExpanded { region: props.selected }, |slug| Route::GrammarLesson { slug: slug.to_owned() })} classes="inline-flex rounded-xl bg-brand-action px-5 py-3 font-semibold text-white no-underline hover:bg-brand-hover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent">
                            {if saved.as_ref().and_then(|record| record.grammar_steps.get(course.id)).is_some_and(|step| *step > 0) {"Continue course"} else {"Start course"}}
                        </Link<Route>>
                    </div>
                } else {
                    <p data-course-unpublished="" class="rounded-lg border border-line bg-raised p-3 text-sm text-muted">
                        {"Choose a highlighted published course to start learning."}
                    </p>
                }
                </div>
        </aside>
    }
}
