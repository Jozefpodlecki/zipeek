use super::catalog::PINYIN_LESSONS;
use super::pinyin_practice::{PINYIN_CHECKS, grade};
use super::progress;
use crate::components::ListenButton;
use crate::route::Route;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CourseLessonProps {
    pub lesson: String,
}

pub(super) fn chinese_characters(example: &str) -> String {
    example
        .chars()
        .filter(|character| ('\u{3400}'..='\u{9fff}').contains(character))
        .collect()
}

#[function_component(CourseLessonPage)]
pub fn course_lesson_page(props: &CourseLessonProps) -> Html {
    let Some((index, _)) = PINYIN_LESSONS
        .iter()
        .enumerate()
        .find(|(_, lesson)| lesson.slug == props.lesson)
    else {
        return html! {
            <section role="alert" class="mx-auto max-w-3xl p-6">
                <h1 class="text-2xl font-bold">{"Pinyin lesson not found"}</h1>
                <Link<Route> to={Route::PinyinCourse}>{"Return to Pinyin"}</Link<Route>>
            </section>
        };
    };
    html! { <PinyinLessonContent key={props.lesson.clone()} index={index} /> }
}

#[derive(Properties, PartialEq)]
struct PinyinLessonContentProps {
    index: usize,
}

#[function_component(PinyinLessonContent)]
fn pinyin_lesson_content(props: &PinyinLessonContentProps) -> Html {
    let index = props.index;
    {
        use_effect_with(index, move |_| {
            if let Some(window) = web_sys::window() {
                window.scroll_to_with_x_and_y(0.0, 0.0);
            }
        });
    }
    let lesson = &PINYIN_LESSONS[index];
    let answers = use_state(|| [None::<usize>; 2]);
    let checking = use_state(|| false);
    let submitted = use_state(|| false);
    let error = use_state(|| None::<String>);
    let completed = progress::load()
        .ok()
        .is_some_and(|saved| saved.pinyin_passed.contains(lesson.slug));
    let begin_check = {
        let checking = checking.clone();
        Callback::from(move |_: MouseEvent| checking.set(true))
    };
    let reset_check = {
        let answers = answers.clone();
        let submitted = submitted.clone();
        let error = error.clone();
        Callback::from(move |_: MouseEvent| {
            answers.set([None; 2]);
            submitted.set(false);
            error.set(None);
        })
    };
    let score = grade(index, &answers);
    let submit = {
        let submitted = submitted.clone();
        let error = error.clone();
        let slug = lesson.slug.to_owned();
        let answers = answers.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(score) = grade(index, &answers) {
                if score == 2 {
                    if let Err(message) = progress::update(|saved| saved.mark_pinyin_passed(&slug))
                    {
                        error.set(Some(message));
                    } else {
                        error.set(None);
                    }
                }
                submitted.set(true);
            }
        })
    };
    let questions = &PINYIN_CHECKS[index];
    html! {
        <article data-component="course-lesson" data-pinyin-module={lesson.slug} class="mx-auto flex w-full max-w-3xl flex-col gap-4 py-3">
            <nav class="flex items-center justify-between gap-3">
                <Link<Route> to={Route::PinyinCourse} classes="inline-flex size-10 items-center justify-center rounded-lg border border-line bg-surface text-ink no-underline"><span class="sr-only">{"Exit lesson"}</span><Icon data={IconData::LUCIDE_X} width="19px" height="19px" /></Link<Route>>
                <span class="text-sm font-semibold text-muted">{format!("Module {} of {}", index + 1, PINYIN_LESSONS.len())}</span>
            </nav>
            <div class="h-2 overflow-hidden rounded-full bg-raised" role="progressbar" aria-label="Pinyin course progress" aria-valuenow={(index+1).to_string()} aria-valuemin="0" aria-valuemax={PINYIN_LESSONS.len().to_string()}>
                <div class="h-full bg-brand-action" style={format!("width: {}%", (index+1)*100/PINYIN_LESSONS.len())}/>
            </div>
            <header class="space-y-2">
                <p class="text-xs font-semibold uppercase tracking-wide text-accent">{"Pinyin pronunciation"}</p>
                <h1 class="text-2xl font-bold sm:text-3xl">{lesson.title}</h1>
                <p class="text-sm text-muted">{lesson.summary}</p>
                if completed { <p class="text-sm text-accent" data-pinyin-completed="">{"Written check passed previously"}</p> }
            </header>
            if !*checking {
                <section class="space-y-3 rounded-2xl border border-line bg-surface p-5 sm:p-7" aria-label="Pinyin explanation">
                    <h2 class="text-lg font-bold">{"Learn the sounds"}</h2>
                    <p class="leading-7">{lesson.explanation}</p>
                    {if super::pinyin_reference::groups_for(index).is_empty() { Html::default() } else { html! {
                        <details class="rounded-xl border border-line bg-raised p-3" aria-label="Pinyin sound reference">
                            <summary class="cursor-pointer text-sm font-semibold">{"Sound reference"}</summary>
                            <div class="mt-3 grid gap-2 sm:grid-cols-2">
                            {for super::pinyin_reference::groups_for(index).iter().map(|group| html! {
                                <div class="rounded-lg border border-line bg-raised p-3">
                                    <h3 class="font-semibold">{group.label}</h3>
                                    <p class="mt-2 font-semibold text-accent">{group.symbols}</p>
                                    <p class="mt-2 text-sm text-muted">{group.note}</p>
                                </div>
                            })}
                            </div>
                        </details>
                    }}}
                    <p class="text-sm text-muted">{"Listen buttons use your browser's Mandarin speech voice when available. Audio quality depends on the installed voice."}</p>
                    <div class="grid gap-3 sm:grid-cols-2">
                        {for lesson.examples.iter().map(|(example, note)| {
                            let hanzi = chinese_characters(example);
                            html! {
                                <div class="rounded-xl border border-line bg-raised p-4">
                                    <div class="flex items-center justify-between gap-2">
                                        <strong class="text-lg leading-7">{*example}</strong>
                                        if !hanzi.is_empty() { <ListenButton hanzi={hanzi} compact=true /> }
                                    </div>
                                    <p class="mt-2 text-sm text-muted">{*note}</p>
                                </div>
                            }
                        })}
                    </div>
                    <button type="button" class="rounded-xl bg-brand-action px-5 py-3 font-semibold text-white" onclick={begin_check}>{"Practise what you learned"}</button>
                </section>
            } else {
                <section class="space-y-4 rounded-2xl border border-line bg-surface p-5 sm:p-7" data-pinyin-check="">
                    <h2 class="text-xl font-bold">{"Check your Pinyin"}</h2>
                    <p class="text-sm text-muted">{"These are written recognition checks. They do not assess speech production or listening accuracy."}</p>
                    {for questions.iter().enumerate().map(|(question_index, question)| {
                        let answer = answers[question_index];
                        html! {
                            <fieldset class="space-y-2 rounded-xl border border-line bg-raised p-4" key={question_index.to_string()}>
                                <legend class="font-semibold">{format!("{}. {}",question_index+1,question.prompt)}</legend>
                                <div class="grid gap-2 sm:grid-cols-3">
                                    {for question.options.iter().enumerate().map(|(choice, option)| {
                                        let answers = answers.clone();
                                        let choose = Callback::from(move |_: MouseEvent| {
                                            let mut updated = *answers;
                                            updated[question_index] = Some(choice);
                                            answers.set(updated);
                                        });
                                        html! {
                                            <button type="button" aria-pressed={(answer == Some(choice)).to_string()} disabled={*submitted}
                                                class={classes!("rounded-lg", "border", "border-line", "px-3", "py-3", "text-left", "focus-visible:outline", "focus-visible:outline-2", "focus-visible:outline-accent", if answer == Some(choice) {"ring-2 ring-accent"} else {""})}
                                                onclick={choose}>{*option}</button>
                                        }
                                    })}
                                </div>
                                if *submitted {
                                    <p class="text-sm" role="status">{if answer == Some(question.answer) {"Correct. "} else {"Review needed. "}}{question.explanation}</p>
                                }
                            </fieldset>
                        }
                    })}
                    if *submitted {
                        <div class="space-y-3" role="status">
                            <p class="font-semibold">{format!("{} of 2 correct",score.unwrap_or_default())}</p>
                            <p class="text-sm text-muted">{if score == Some(2) {"Written recognition check passed. This does not certify spoken pronunciation."} else {"Review the explanation and retry the two questions."}}</p>
                            <button type="button" class="rounded-lg border border-line px-4 py-2" onclick={reset_check}>{"Retry check"}</button>
                        </div>
                    } else {
                        <button type="button" disabled={score.is_none()} onclick={submit} class="rounded-xl bg-brand-action px-5 py-3 font-semibold text-white disabled:opacity-40">{"Check answers"}</button>
                    }
                </section>
            }
            if let Some(message) = &*error {
                <p role="alert" class="rounded-lg border border-line bg-surface p-3 text-sm">{format!("Progress could not be saved: {message}")}</p>
            }
            <nav class="flex justify-between gap-3 text-sm">
                if index > 0 {
                    <Link<Route> to={Route::CourseLesson { lesson: PINYIN_LESSONS[index-1].slug.to_owned() }} classes="inline-flex size-10 items-center justify-center rounded-lg border border-line bg-surface no-underline"><span class="sr-only">{"Previous module"}</span><Icon data={IconData::LUCIDE_ARROW_LEFT} width="19px" height="19px" /></Link<Route>>
                } else { <span/> }
                if index+1 < PINYIN_LESSONS.len() {
                    <Link<Route> to={Route::CourseLesson { lesson: PINYIN_LESSONS[index+1].slug.to_owned() }} classes="inline-flex size-10 items-center justify-center rounded-lg border border-line bg-surface no-underline"><span class="sr-only">{"Next module"}</span><Icon data={IconData::LUCIDE_ARROW_RIGHT} width="19px" height="19px" /></Link<Route>>
                } else {
                    <Link<Route> to={Route::PinyinCourse} classes="rounded-lg border border-line bg-surface px-4 py-2 no-underline">{"Return to Pinyin"}</Link<Route>>
                }
            </nav>
        </article>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracts_only_chinese_characters_for_audio() {
        assert_eq!(chinese_characters("mǎ 马 — horse"), "马");
        assert_eq!(chinese_characters("mā = m + ā"), "");
        assert_eq!(chinese_characters("Xi'an 西安"), "西安");
    }
}
