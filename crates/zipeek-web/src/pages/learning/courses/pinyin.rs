use super::{catalog::PINYIN_LESSONS, progress, track_navigation::CourseTrackNavigation};
use crate::route::Route;
use crate::components::ListenButton;
use super::workspace::CourseWorkspace;
use yew::prelude::*;
use yew_router::prelude::*;

const REGIONS: [&str; 6] = [
    "M 0 0 H 330 C 370 80 355 180 380 290 C 300 305 220 315 0 300 Z",
    "M 330 0 H 670 C 640 95 650 185 680 285 C 600 300 470 305 380 290 C 355 180 370 80 330 0 Z",
    "M 670 0 H 1000 V 300 C 790 315 740 300 680 285 C 650 185 640 95 670 0 Z",
    "M 0 300 C 220 315 300 305 380 290 C 395 370 365 460 390 600 H 0 Z",
    "M 380 290 C 470 305 600 300 680 285 C 700 370 660 470 675 600 H 390 C 365 460 395 370 380 290 Z",
    "M 680 285 C 740 300 790 315 1000 300 V 600 H 675 C 660 470 700 370 680 285 Z",
];

#[function_component(PinyinCoursePage)]
pub fn pinyin_course_page() -> Html {
    let selected = use_state(|| 0usize);
    let module_start = *selected * 2;
    let lessons = &PINYIN_LESSONS[module_start..module_start + 2];
    let lesson = &lessons[0];
    let saved = progress::load();
    let completed = |slug: &str| saved.as_ref().is_ok_and(|state| state.pinyin_passed.contains(slug));
    let next_module = lessons.iter().find(|item| !completed(item.slug)).unwrap_or(lesson);

    html! {
        <section data-component="pinyin-course" class="mx-auto flex w-full max-w-[1500px] flex-col gap-3 py-1 lg:h-full lg:min-h-0 lg:overflow-hidden">
            <CourseTrackNavigation active={super::track_navigation::CourseTrack::Pinyin} />
            <CourseWorkspace
                mosaic={html! {
                    <div data-pinyin-mosaic="" class="aspect-[5/3] h-full w-full overflow-hidden rounded-xl border border-line bg-surface shadow-sm lg:aspect-auto">
                        <svg viewBox="0 0 1000 600" preserveAspectRatio="none"
                            class="block h-full w-full" role="group"
                            aria-label="Pinyin course mosaic with six selectable stages">
                            <image href="courses/pinyin-landscape.png" x="0" y="0" width="1000" height="600" preserveAspectRatio="xMidYMid slice" />
                            {for REGIONS.iter().enumerate().map(|(index, path)| {
                                let active = *selected == index;
                                let select = selected.clone();
                                let choose = Callback::from(move |_: ()| select.set(index));
                                let onclick = {
                                    let choose = choose.clone();
                                    Callback::from(move |_: MouseEvent| choose.emit(()))
                                };
                                let onkeydown = Callback::from(move |event: KeyboardEvent| {
                                    if event.key() == "Enter" || event.key() == " " {
                                        event.prevent_default();
                                        choose.emit(());
                                    }
                                });
                                let label = format!("Pinyin stage {}: {} and {}", index + 1, PINYIN_LESSONS[index*2].title, PINYIN_LESSONS[index*2+1].title);
                                html! {
                                    <path
                                        key={PINYIN_LESSONS[index*2].slug}
                                        d={*path}
                                        role="button"
                                        tabindex="0"
                                        aria-label={label.clone()}
                                        aria-pressed={active.to_string()}
                                        data-pinyin-region={PINYIN_LESSONS[index*2].slug}
                                        fill="var(--mosaic-cover)"
                                        fill-opacity={if active {"0.35"} else {"0.52"}}
                                        stroke={if active {"var(--mosaic-selected)"} else {"var(--mosaic-edge)"}}
                                        stroke-width={if active {"2.75"} else {"1.25"}}
                                        class="cursor-pointer focus:outline-none focus:stroke-[var(--mosaic-selected)] hover:stroke-[var(--mosaic-selected)]"
                                        onclick={onclick}
                                        onkeydown={onkeydown}
                                    >
                                        <title>{label}</title>
                                    </path>
                                }
                            })}
                        </svg>
                    </div>
                }}
                explorer={html! {
                    <section data-pinyin-preview="" class="h-[166px] overflow-hidden rounded-xl border border-line bg-surface p-3 shadow-sm" aria-label="Pronunciation examples">
                        <h3 class="text-sm font-bold text-ink">{"Listen and notice"}</h3>
                        <div class="mt-2 grid grid-cols-2 gap-2 sm:grid-cols-3">
                            {for lesson.examples.iter().take(3).map(|(example, detail)| {
                                let chinese = super::lesson::chinese_characters(example);
                                html! {
                                    <div class="min-w-0 rounded-lg border border-line bg-raised p-2">
                                        <div class="flex items-center justify-between gap-1">
                                            <p class="truncate text-sm font-semibold text-ink" title={*example}>{*example}</p>
                                            if !chinese.is_empty() { <ListenButton hanzi={chinese} compact=true /> }
                                        </div>
                                        <p class="mt-1 line-clamp-2 text-xs text-muted" title={*detail}>{*detail}</p>
                                    </div>
                                }
                            })}
                        </div>
                    </section>
                }}
                details={html! {
                    <aside data-pinyin-details="" aria-live="polite" class="flex h-full min-h-0 flex-col gap-3 rounded-xl border border-line bg-surface p-4 shadow-sm">
                        <p class="text-xs font-semibold uppercase tracking-wide text-muted">{format!("Stage {} of 6", *selected + 1)}</p>
                        <h2 class="text-xl font-bold text-ink">{format!("{} & {}", lessons[0].title, lessons[1].title)}</h2>
                        <p class="text-sm leading-6 text-muted">{"Work through both modules, listen to the examples and check your understanding."}</p>
                        <div class="space-y-2" aria-label="Stage modules">
                            {for lessons.iter().enumerate().map(|(offset, module)| html! {
                                <Link<Route> to={Route::CourseLesson { lesson: module.slug.to_owned() }}
                                    classes="flex items-center justify-between gap-3 rounded-lg border border-line bg-raised px-3 py-3 text-sm font-semibold no-underline">
                                    <span>{format!("{}. {}", module_start + offset + 1, module.title)}</span>
                                    <span class="shrink-0 text-xs text-muted">{if completed(module.slug) {"Check passed"} else {"Not checked"}}</span>
                                </Link<Route>>
                            })}
                        </div>
                        if let Err(message) = &saved {
                            <p role="alert" class="text-xs text-muted">{format!("Progress unavailable: {message}")}</p>
                        }
                        <div class="mt-auto">
                            <Link<Route> to={Route::CourseLesson { lesson: next_module.slug.to_owned() }}
                                classes="inline-flex rounded-xl bg-brand-action px-5 py-3 font-semibold text-white no-underline hover:bg-brand-hover focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent">
                                {"Continue learning"}
                            </Link<Route>>
                        </div>
                    </aside>
                }}
            />
        </section>
    }
}
