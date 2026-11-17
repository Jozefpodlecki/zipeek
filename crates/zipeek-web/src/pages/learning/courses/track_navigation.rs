use crate::route::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CourseTrack {
    Vocabulary,
    Grammar,
    Pinyin,
}

#[derive(Properties, PartialEq)]
pub struct CourseTrackNavigationProps {
    pub active: CourseTrack,
}

#[function_component(CourseTrackNavigation)]
pub fn course_track_navigation(props: &CourseTrackNavigationProps) -> Html {
    let tracks = [
        (CourseTrack::Vocabulary, Route::VocabularyCourse, "Vocabulary"),
        (CourseTrack::Grammar, Route::GrammarCourse, "Grammar"),
        (CourseTrack::Pinyin, Route::PinyinCourse, "Pinyin"),
    ];
    html! {
        <nav data-course-tracks="" aria-label="Course tracks" class="grid w-full grid-cols-3 gap-1 rounded-2xl border border-line bg-surface p-1 shadow-sm">
            {for tracks.into_iter().map(|(track, route, label)| {
                let active = props.active == track;
                let classes = if active {
                    "rounded-xl bg-brand-action px-3 py-2.5 text-center text-sm font-bold text-white no-underline shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                } else {
                    "rounded-xl px-3 py-2.5 text-center text-sm font-semibold text-muted no-underline transition-colors hover:bg-raised hover:text-ink focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent motion-reduce:transition-none"
                };
                html! {
                    <Link<Route> key={label} to={route} classes={classes}>
                        <span aria-current={active.then_some("page")}>{label}</span>
                    </Link<Route>>
                }
            })}
        </nav>
    }
}
