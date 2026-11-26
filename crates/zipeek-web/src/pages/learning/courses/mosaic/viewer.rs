use super::super::curriculum::authored_grammar_region;
use super::super::grammar_expansion::expanded_course;
use super::super::grammar_plan::planned_title;
use super::super::workspace::CourseWorkspace;
use super::{details::CourseDetails, geometry};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[function_component(CourseMosaic)]
pub fn course_mosaic() -> Html {
    let regions = use_state(geometry::regions);
    let selected = use_state(|| 1usize);
    html! {
        <section data-course-mosaic="" class="flex min-h-0 min-w-0 flex-1 flex-col">
            <CourseWorkspace
                mosaic={html! {
                    <div class="aspect-[5/3] h-full w-full overflow-hidden rounded-xl border border-line bg-surface shadow-sm lg:aspect-auto" data-mosaic-map="">
                        <svg
                            viewBox="0 0 1000 600"
                            preserveAspectRatio="none" class="group/mosaic block h-full w-full"
                            role="group"
                            aria-label="Grammar course mosaic with 100 selectable regions"
                        >
                            <image href="courses/grammar-autumn-bridge.png" x="0" y="0" width="1000" height="600" preserveAspectRatio="xMidYMid slice" class="pointer-events-none" />
                        {for regions.iter().map(|region| {
                            let id = region.id.clone();
                            let is_selected = region.order == *selected;
                            let label = match authored_grammar_region(region.order) {
                                Some((course, _)) => format!("Grammar course {}: {}", region.order, course.title),
                                None => match expanded_course(region.order) {
                                    Some(course) => format!("Grammar course {}: {}", region.order, course.title),
                                    None => format!("Planned grammar topic {}: {}", region.order, planned_title(region.order).unwrap_or("Upcoming")),
                                },
                            };
                            let order = region.order;
                            let selected = selected.clone();
                            let choose = Callback::from(move |_: ()| selected.set(order));
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
                            html! {
                                <path
                                    key={id.clone()}
                                    d={region.path.clone()}
                                    fill="var(--mosaic-cover)"
                                    fill-opacity={if is_selected {"0.27"} else {"0.58"}}
                                    stroke={if is_selected {"var(--mosaic-selected)"} else {"var(--mosaic-edge)"}}
                                    stroke-width={if is_selected {"2.8"} else {"1.05"}}
                                    role="button"
                                    tabindex="0"
                                    aria-label={label.clone()}
                                    aria-pressed={is_selected.to_string()}
                                    data-mosaic-region={id}
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
                details={html! { <CourseDetails key={(*selected).to_string()} selected={*selected} /> }}
                explorer={html! { <GrammarPatternPreview key={(*selected).to_string()} selected={*selected} /> }}
            />
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct GrammarPatternPreviewProps {
    selected: usize,
}

#[function_component(GrammarPatternPreview)]
fn grammar_pattern_preview(props: &GrammarPatternPreviewProps) -> Html {
    let current = use_state(|| 0usize);
    let course = authored_grammar_region(props.selected)
        .map(|(course, _)| course)
        .or_else(|| expanded_course(props.selected));
    let points = course.map(|course| course.points).unwrap_or(&[]);
    let count = points.len();
    let index = (*current).min(count.saturating_sub(1));
    let point = points.get(index);
    let pinyin = zipeek_pedagogy::grammar::pinyin::example_pinyin(props.selected, index);
    let previous = {
        let current = current.clone();
        Callback::from(move |_: MouseEvent| current.set(index.saturating_sub(1)))
    };
    let next = {
        let current = current.clone();
        Callback::from(move |_: MouseEvent| current.set((index + 1).min(count.saturating_sub(1))))
    };
    html! {
        <section data-grammar-preview="" class="flex h-[166px] min-w-0 flex-col rounded-xl border border-line bg-surface px-4 py-3 shadow-sm" aria-label="Grammar examples">
            if let Some(point) = point {
                <div class="flex items-center justify-between gap-2">
                    <span class="truncate text-xs font-semibold text-muted">{point.label}</span>
                    <div class="flex shrink-0 items-center gap-2">
                        <span class="text-xs tabular-nums text-muted">{format!("{} / {}", index + 1, count)}</span>
                        <button type="button" aria-label="Previous grammar example" title="Previous example" disabled={index == 0} onclick={previous}
                            class="inline-flex size-8 items-center justify-center rounded-lg border border-line hover:bg-raised disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-accent">
                            <Icon data={IconData::LUCIDE_CHEVRON_LEFT} width="16px" height="16px" />
                        </button>
                        <button type="button" aria-label="Next grammar example" title="Next example" disabled={index + 1 >= count} onclick={next}
                            class="inline-flex size-8 items-center justify-center rounded-lg border border-line hover:bg-raised disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-accent">
                            <Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="16px" height="16px" />
                        </button>
                    </div>
                </div>
                <p class="mt-1 truncate text-xl font-semibold text-ink" lang="zh-Hans">{point.example}</p>
                if let Some(pinyin) = pinyin {
                    <p class="truncate text-sm text-accent" lang="zh-Latn">{pinyin}</p>
                }
                <p class="mt-1 line-clamp-2 text-sm text-muted">{point.translation}</p>
            } else {
                <p class="text-sm text-muted">{"Select an available course to view examples."}</p>
            }
        </section>
    }
}
