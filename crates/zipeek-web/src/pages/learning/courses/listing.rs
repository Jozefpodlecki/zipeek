use super::{
    mosaic::CourseMosaic,
    track_navigation::{CourseTrack, CourseTrackNavigation},
};
use yew::prelude::*;

#[function_component(CoursesPage)]
pub fn courses_page() -> Html {
    html! {
        <section data-component="courses-page" class="mx-auto flex w-full max-w-[1500px] flex-col gap-3 py-1 lg:h-full lg:min-h-0 lg:overflow-hidden">
            <CourseTrackNavigation active={CourseTrack::Grammar} />
            <CourseMosaic />
        </section>
    }
}
