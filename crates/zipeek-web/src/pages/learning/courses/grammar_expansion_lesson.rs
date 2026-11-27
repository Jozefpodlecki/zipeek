use super::grammar_ai_example::GrammarAiExample;
use super::grammar_expansion::expanded_course;
use super::grammar_learning_flow::{self, LearningPhase, PracticeAction, PracticeState};
use super::grammar_lesson_scaffold::GrammarLessonScaffold;
use super::progress;
use crate::components::ListenButton;
use crate::hooks::use_app_environment;
use crate::route::Route;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GrammarExpandedLessonProps {
    pub region: usize,
}

#[function_component(GrammarExpandedLessonPage)]
pub fn grammar_expanded_lesson_page(props: &GrammarExpandedLessonProps) -> Html {
    let Some(course) = expanded_course(props.region) else {
        return html! { <section role="alert" class="mx-auto max-w-3xl p-6">{"This grammar lesson is not yet authored."}</section> };
    };
    html! { <ExpandedLessonContent key={course.id} region={props.region} /> }
}

#[derive(Properties, PartialEq)]
struct ExpandedLessonContentProps {
    region: usize,
}

#[function_component(ExpandedLessonContent)]
fn expanded_lesson_content(props: &ExpandedLessonContentProps) -> Html {
    let Some(course) = expanded_course(props.region) else {
        return Html::default();
    };
    let environment = use_app_environment();
    let total_steps = grammar_learning_flow::step_count(course.points.len());
    let initial = progress::load()
        .ok()
        .and_then(|saved| saved.grammar_steps.get(course.id).copied())
        .unwrap_or(0)
        .min(total_steps - 1);
    let step = use_state(|| initial);
    let practice = use_reducer(|| PracticeState::new(course.points.len()));
    let save_error = use_state(|| None::<String>);
    let current = *step;
    {
        let window = environment.window.clone();
        use_effect_with(current, move |_| {
            window.scroll_to_with_x_and_y(0.0, 0.0);
        });
    }
    let previous = {
        let step = step.clone();
        let save_error = save_error.clone();
        let id = course.id.to_owned();
        Callback::from(move |_: MouseEvent| {
            let value = step.saturating_sub(1);
            match progress::update(|record| record.mark_grammar_step(&id, value, total_steps)) {
                Ok(()) => {
                    save_error.set(None);
                    step.set(value);
                }
                Err(error) => save_error.set(Some(error)),
            }
        })
    };
    let next = {
        let step = step.clone();
        let save_error = save_error.clone();
        let id = course.id.to_owned();
        Callback::from(move |_: MouseEvent| {
            let value = (*step + 1).min(total_steps - 1);
            match progress::update(|record| record.mark_grammar_step(&id, value, total_steps)) {
                Ok(()) => {
                    save_error.set(None);
                    step.set(value);
                }
                Err(error) => save_error.set(Some(error)),
            }
        })
    };
    let learning_step = grammar_learning_flow::step_at(current, course.points.len())
        .expect("A bounded grammar checkpoint always corresponds to a learning step");
    let question_index = learning_step.point_index;
    let point = &course.points[question_index];
    let example_pinyin =
        zipeek_pedagogy::grammar::pinyin::example_pinyin(props.region, question_index);
    let can_continue = grammar_learning_flow::can_advance(
        learning_step,
        practice.revealed[question_index],
        practice.answers[question_index],
    );
    let stage = match learning_step.phase {
        LearningPhase::Explain => {
            let point_id = format!("{}-{}", course.id, question_index + 1);
            html! {
                <section data-grammar-explanation={course.id} data-grammar-pattern-index={question_index.to_string()} class="space-y-5 rounded-2xl border border-line bg-surface p-6 sm:p-8">
                    <p class="text-xs font-semibold uppercase tracking-wide text-muted">{format!("Pattern {} of {}", question_index + 1, course.points.len())}</p>
                    <h2 class="text-2xl font-bold">{point.label}</h2>
                    <p class="leading-7">{point.explanation}</p>
                    <div class="rounded-xl border border-line bg-raised p-5 space-y-2">
                        <div class="flex items-center justify-between gap-3">
                            <p lang="zh-Hans" class="text-2xl font-semibold">{point.example}</p>
                            <ListenButton hanzi={point.example.to_owned()} compact=true />
                        </div>
                        if let Some(pinyin) = example_pinyin {
                            <p lang="zh-Latn" class="text-sm leading-6 text-muted">{pinyin}</p>
                        }
                        <p class="text-sm text-muted">{point.translation}</p>
                    </div>
                    <GrammarAiExample key={point_id.clone()} point_id={point_id.clone()} pattern={point.label.to_owned()}
                        explanation={point.explanation.to_owned()} example={point.example.to_owned()} />
                </section>
            }
        }
        LearningPhase::Practise => {
            let answer_is_revealed = practice.revealed[question_index];
            let selected = practice.answers[question_index];
            let practice_for_reveal = practice.clone();
            let reveal = Callback::from(move |_: MouseEvent| {
                practice_for_reveal.dispatch(PracticeAction::Reveal(question_index));
            });
            html! {
                <section data-grammar-expansion-practice="" data-grammar-course-id={course.id} data-grammar-pattern-index={question_index.to_string()} data-grammar-practice-phase={if answer_is_revealed {"recognition"} else {"recall"}} class="space-y-5 rounded-2xl border border-line bg-surface p-5 sm:p-7">
                    <p class="text-xs font-semibold uppercase tracking-wide text-muted">{format!("Guided practice {} of {}", question_index + 1, course.points.len())}</p>
                    <h2 class="text-xl font-bold">{point.label}</h2>
                    <div class="space-y-3 rounded-xl border border-line bg-raised p-4 sm:p-5">
                        <p class="text-sm font-semibold">{"Recall the Chinese sentence"}</p>
                        <p class="text-sm text-muted">{"Try to say or write the Chinese sentence for this meaning before revealing it. Returning to this step after a page reload starts the guided practice again."}</p>
                        <p class="text-lg font-medium">{point.translation}</p>
                        if answer_is_revealed {
                            <div class="flex items-center justify-between gap-3 border-t border-line pt-3">
                                <p lang="zh-Hans" class="text-xl font-semibold">{point.example}</p>
                                <ListenButton hanzi={point.example.to_owned()} compact=true />
                            </div>
                            if let Some(pinyin) = example_pinyin {
                                <p lang="zh-Latn" class="text-sm leading-6 text-muted">{pinyin}</p>
                            }
                            <p class="text-xs text-muted">{"Compare your answer with the authored example. Revealing an answer is practice, not a mastery result."}</p>
                        } else {
                            <button type="button" data-grammar-reveal="" onclick={reveal} class="min-h-11 rounded-lg border border-line bg-surface px-4 py-2 font-semibold focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent">{"Reveal sentence"}</button>
                        }
                    </div>
                    if answer_is_revealed {
                        <div class="space-y-3">
                            <h3 class="text-base font-semibold">{"Check the meaning"}</h3>
                            <p class="text-sm text-muted">{"Choose the Chinese sentence that matches the English meaning. These are the authored teaching examples, not independent assessment questions."}</p>
                            <div class="grid gap-2">
                                {for grammar_learning_flow::choice_indices(question_index, course.points.len()).into_iter().map(|choice| {
                                    let picked = selected == Some(choice);
                                    let correct_locked = selected == Some(question_index);
                                    let practice = practice.clone();
                                    let choose = Callback::from(move |_: MouseEvent| {
                                        practice.dispatch(PracticeAction::Choose { point: question_index, choice });
                                    });
                                    html! {
                                        <div data-grammar-choice-row={choice.to_string()} class={classes!("flex", "items-center", "gap-2", "rounded-xl", "border", "border-line", "bg-raised", "pr-2", if picked {"ring-2 ring-accent"} else {""})}>
                                            <button type="button" data-grammar-choice={choice.to_string()} data-grammar-choice-state={if picked {if correct_locked {"correct"} else {"incorrect"}} else {"unselected"}} lang="zh-Hans" aria-pressed={picked.to_string()} disabled={correct_locked} onclick={choose}
                                                class="min-h-12 min-w-0 flex-1 rounded-xl px-4 py-3 text-left text-lg font-semibold focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent disabled:cursor-default">
                                                <span class="block">{course.points[choice].example}</span>
                                                if let Some(pinyin) = zipeek_pedagogy::grammar::pinyin::example_pinyin(props.region, choice) {
                                                    <span lang="zh-Latn" class="mt-1 block text-sm font-normal text-muted">{pinyin}</span>
                                                }
                                            </button>
                                            <ListenButton hanzi={course.points[choice].example.to_owned()} compact=true />
                                        </div>
                                    }
                                })}
                            </div>
                            if let Some(chosen) = selected {
                                <p role="status" data-grammar-practice-feedback={if chosen == question_index {"correct"} else {"incorrect"}} class={classes!("rounded-lg", "border", "border-line", "p-3", "text-sm", if chosen == question_index {"text-ink"} else {"text-muted"})}>
                                    {if chosen == question_index {"Correct. You can continue."} else {"Not quite. Compare the meaning and try another sentence."}}
                                </p>
                            }
                        </div>
                    }
                </section>
            }
        }
    };
    html! {
        <GrammarLessonScaffold title={course.title} course_id={course.id} expanded=true current={current} total_steps={total_steps} subtitle={Some(AttrValue::from("Teaching and exploratory practice. This lesson does not award verified mastery."))}>
            {stage}
            if let Some(error) = &*save_error {
                <p role="alert" data-grammar-progress-error="" class="text-sm text-muted">{format!("Progress could not be saved: {error}")}</p>
            }
            <div class="flex items-center justify-between gap-3">
                <button type="button" data-grammar-previous="" aria-label="Previous step" title="Previous step" disabled={current == 0} onclick={previous} class="inline-flex size-11 items-center justify-center rounded-lg border border-line bg-surface disabled:opacity-40">
                    <Icon data={IconData::LUCIDE_ARROW_LEFT} width="19px" height="19px" />
                </button>
                if current + 1 < total_steps {
                    <button type="button" data-grammar-continue="" disabled={!can_continue} onclick={next} class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white disabled:opacity-40">{"Continue"}</button>
                } else {
                    if can_continue {
                        <Link<Route> to={Route::GrammarCourse} classes="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white no-underline">{"Return to Grammar"}</Link<Route>>
                    } else {
                        <button type="button" data-grammar-finish="" disabled=true class="rounded-lg bg-brand-action px-5 py-3 font-semibold text-white opacity-40">{"Finish review"}</button>
                    }
                }
            </div>
        </GrammarLessonScaffold>
    }
}
