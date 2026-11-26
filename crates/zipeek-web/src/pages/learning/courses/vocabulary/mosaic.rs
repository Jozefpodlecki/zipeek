use super::vocabulary_geometry;
use yew::prelude::*;
use zipeek_pedagogy::curriculum::VOCABULARY_REGIONS as COURSES;

#[derive(Properties, PartialEq)]
pub struct MosaicProps {
    pub selected: usize,
    pub on_select: Callback<usize>,
}

#[function_component(VocabularyMosaic)]
pub fn vocabulary_mosaic(props: &MosaicProps) -> Html {
    let paths = use_memo((), |_| {
        vocabulary_geometry::region_paths(&COURSES.map(|region| region.readings.len()))
    });
    html! {
                <div data-vocabulary-mosaic="" class="min-w-0 overflow-hidden rounded-2xl lg:h-full border border-line bg-surface shadow-lg">
                    <svg viewBox="0 0 1000 600" preserveAspectRatio="none" class="block aspect-[5/3] h-auto w-full lg:aspect-auto lg:h-full" role="group" aria-label="Vocabulary mosaic with 100 selectable regions">
                        <image href="courses/vocabulary-peony-garden.png" x="0" y="0" width="1000" height="600" preserveAspectRatio="xMidYMid slice" />
                        {for (0..COURSES.len()).map(|index| {
                            let title = COURSES.get(index).map(|course| course.title).unwrap_or("Collection unavailable");
                            let active = index == props.selected;
                            let select = props.on_select.clone();
                            let choose = Callback::from(move |_: ()| select.emit(index));
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
                            let reading_count = COURSES[index].readings.len();
                            let label = format!("Vocabulary region {}: {title}, {reading_count} readings", index + 1);
                            html! {
                                <path key={index} data-vocabulary-region={format!("vocabulary-{}", index+1)}
                                    d={paths[index].clone()} data-vocabulary-readings={reading_count.to_string()} role="button" tabindex="0" aria-label={label.clone()}
                                    aria-pressed={active.to_string()}
                                    data-vocabulary-authored="true"
                                    fill="var(--mosaic-cover)" fill-opacity={if active {"0.28"} else {"0.57"}}
                                    stroke={if active {"var(--mosaic-selected)"} else {"var(--mosaic-edge)"}}
                                    stroke-width={if active {"2.8"} else {"1.05"}}
                                    class="cursor-pointer transition-[fill-opacity,stroke] duration-200 focus:outline-none focus:stroke-[var(--mosaic-selected)] hover:stroke-[var(--mosaic-selected)] hover:fill-opacity-40"
                                    {onclick} {onkeydown}>
                                    <title>{label}</title>
                                </path>
                            }
                        })}
                    </svg>
                </div>
    }
}
