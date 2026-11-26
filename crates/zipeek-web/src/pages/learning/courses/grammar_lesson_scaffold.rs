use crate::route::Route;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GrammarLessonScaffoldProps {
    pub title: AttrValue,
    pub course_id: AttrValue,
    #[prop_or_default]
    pub expanded: bool,
    pub current: usize,
    pub total_steps: usize,
    #[prop_or_default]
    pub subtitle: Option<AttrValue>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(GrammarLessonScaffold)]
pub fn grammar_lesson_scaffold(props: &GrammarLessonScaffoldProps) -> Html {
    let current = props.current.min(props.total_steps.saturating_sub(1));
    let progress = if props.total_steps == 0 {
        0
    } else {
        (current + 1) * 100 / props.total_steps
    };
    html! {
        <article data-grammar-expanded={props.expanded.then_some(props.course_id.clone())} data-grammar-lesson={(!props.expanded).then_some(props.course_id.clone())} data-grammar-step={props.current.to_string()} class="mx-auto flex w-full max-w-3xl flex-col gap-4 py-3">
            <nav class="flex items-center justify-between gap-3 text-sm">
                <Link<Route> to={Route::GrammarCourse} classes="inline-flex size-10 items-center justify-center rounded-lg border border-line bg-surface text-ink no-underline focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent">
                    <span class="sr-only">{"Exit course"}</span>
                    <Icon data={IconData::LUCIDE_X} width="19px" height="19px" />
                </Link<Route>>
                <span class="text-right font-semibold text-muted">{format!("Step {} of {}", current + 1, props.total_steps)}</span>
            </nav>
            <div role="progressbar" aria-label="Grammar lesson progress" aria-valuenow={(current + 1).to_string()} aria-valuemin="0" aria-valuemax={props.total_steps.to_string()} class="h-2 overflow-hidden rounded-full bg-raised">
                <div class="h-full rounded-full bg-brand-action transition-all motion-reduce:transition-none" style={format!("width: {progress}%")} />
            </div>
            <header class="space-y-1">
                <h1 class="text-lg font-semibold text-ink">{&props.title}</h1>
                if let Some(subtitle) = &props.subtitle {
                    <p class="text-xs text-muted">{subtitle}</p>
                }
            </header>
            {for props.children.iter()}
        </article>
    }
}
