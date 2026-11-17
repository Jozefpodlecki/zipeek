use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CourseWorkspaceProps {
    pub mosaic: Html,
    pub details: Html,
    pub explorer: Html,
}

#[function_component(CourseWorkspace)]
pub fn course_workspace(props: &CourseWorkspaceProps) -> Html {
    html! {
        <div data-course-workspace="" class="grid min-h-0 min-w-0 gap-3 lg:flex-1 lg:grid-cols-[minmax(0,1fr)_minmax(280px,350px)] lg:overflow-hidden">
            <div class="flex min-h-0 min-w-0 flex-col gap-3 lg:h-full">
                <div data-course-artwork="" class="min-h-0 lg:flex-[1_1_auto] lg:overflow-hidden">
                    {props.mosaic.clone()}
                </div>
                <div data-course-explorer="" class="min-w-0 lg:h-[166px] lg:shrink-0">
                    {props.explorer.clone()}
                </div>
            </div>
            <div data-course-summary="" class="min-h-0 min-w-0 lg:h-full lg:overflow-hidden">
                {props.details.clone()}
            </div>
        </div>
    }
}
