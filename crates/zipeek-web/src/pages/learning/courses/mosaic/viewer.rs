use super::super::curriculum::authored_grammar_region;
use super::super::grammar_expansion::expanded_course;
use super::super::grammar_plan::planned_title;
use super::{
    details::CourseDetails,
    geometry,
};
use yew::prelude::*;
use super::super::workspace::CourseWorkspace;

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
                explorer={html! { <GrammarPatternPreview selected={*selected} /> }}
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
    let point = authored_grammar_region(props.selected)
        .map(|(course, _)| course)
        .or_else(|| expanded_course(props.selected))
        .and_then(|course| course.points.first());
    html! {
        <section data-grammar-preview="" class="h-[166px] overflow-hidden rounded-xl border border-line bg-surface p-3 shadow-sm" aria-label="Grammar pattern preview">
            <h3 class="text-sm font-bold text-ink">{"Grammar pattern"}</h3>
            if let Some(point) = point {
                <p class="mt-1 truncate text-xs font-semibold text-muted">{point.label}</p>
                <p class="mt-2 truncate text-xl font-semibold text-ink" lang="zh">{point.example}</p>
                <p class="mt-1 line-clamp-2 text-sm text-muted">{point.translation}</p>
            } else {
                <p class="mt-3 text-sm text-muted">{"Select a published course to preview a sentence pattern."}</p>
            }
        </section>
    }
}
