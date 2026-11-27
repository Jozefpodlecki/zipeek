mod start;
mod step;
pub use self::step::PracticeStep;
pub(crate) use start::PracticeStartButton;

use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub struct PracticeFlowNavigationProps {
    pub current: PracticeStep,
    pub furthest: PracticeStep,
    pub on_select: Callback<PracticeStep>,
    pub can_continue: bool,
    pub on_back: Callback<()>,
    pub on_continue: Callback<()>,
    pub start_action: Html,
}

#[function_component(PracticeFlowNavigation)]
pub fn practice_flow_navigation(props: &PracticeFlowNavigationProps) -> Html {
    let fraction = match props.furthest.index() {
        0 => "0%",
        1 => "33.333%",
        2 => "66.667%",
        _ => "100%",
    };
    let on_back = {
        let action = props.on_back.clone();
        Callback::from(move |_: MouseEvent| action.emit(()))
    };
    let on_continue = {
        let action = props.on_continue.clone();
        Callback::from(move |_: MouseEvent| action.emit(()))
    };
    let back_label = if props.current == PracticeStep::Summary {
        "Back to vocabulary"
    } else {
        "Back"
    };
    let continue_label = match props.current {
        PracticeStep::Path => "Choose a practice path",
        PracticeStep::Exercises => "Continue to vocabulary",
        _ => "Continue to summary",
    };
    html! {
        <div
            class="shrink-0 space-y-4"
        >
            <nav
                aria-label="Practice setup progress"
                data-practice-progress=""
                class="flex items-center gap-2 rounded-xl border border-line bg-surface/95 px-2 py-3 sm:gap-4 sm:px-4"
            >
                <button type="button" data-practice-back=""
                    data-practice-summary-back={(props.current == PracticeStep::Summary).then_some("")}
                    aria-label={back_label} title={back_label}
                    disabled={props.current == PracticeStep::Path}
                    onclick={on_back}
                    class="flex size-11 shrink-0 items-center justify-center rounded-xl border border-line bg-raised text-ink hover:bg-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-40">
                    <span aria-hidden="true"><Icon data={IconData::LUCIDE_ARROW_LEFT} width="20px" height="20px" /></span>
                </button>
                <div class="relative min-w-0 flex-1">
                    <div
                        aria-hidden="true"
                        class="pointer-events-none absolute left-[12.5%] right-[12.5%] top-4 h-px bg-line"
                    >
                        <div
                            class="h-full bg-brand-action transition-all duration-200 motion-reduce:transition-none"
                            style={format!("width: {fraction}")}
                        />
                    </div>
                    <ol
                        class="relative grid grid-cols-4 gap-1 sm:gap-3"
                    >
                        {for PracticeStep::ALL.into_iter().map(|step| {
                            let active = step == props.current;
                            let completed = step.is_before(props.current);
                            let available = step.index() <= props.furthest.index();
                            let on_select = props.on_select.clone();
                            let on_choose_step = Callback::from(move |_: MouseEvent| on_select.emit(step));
                            let progress_step_marker = if active {
                                html! { <span class="size-2.5 rounded-full bg-white" /> }
                            } else if completed {
                                html! { <span class="size-3 rounded-full bg-white" /> }
                            } else {
                                html! { <span class="size-2 rounded-full bg-line" /> }
                            };
                            let step_description = match step {
                                PracticeStep::Path => "Choose path",
                                PracticeStep::Exercises => "Select & configure",
                                PracticeStep::Vocabulary => "Choose vocabulary",
                                PracticeStep::Summary => "Review & start",
                            };
                            let element_classes_1 = classes!(
                                                "group",
                                                "flex",
                                                "w-full",
                                                "min-w-0",
                                                "flex-col",
                                                "items-center",
                                                "gap-1",
                                                "rounded-lg",
                                                "px-1",
                                                "py-1",
                                                "text-center",
                                                "focus-visible:outline-2",
                                                "focus-visible:outline-offset-2",
                                                "focus-visible:outline-brand-action",
                                                if available {"cursor-pointer hover:text-ink"} else {"cursor-not-allowed"},
                                            );
                            let element_classes_2 = classes!(
                                                    "z-10",
                                                    "flex",
                                                    "size-6",
                                                    "items-center",
                                                    "justify-center",
                                                    "rounded-full",
                                                    "border-2",
                                                    "transition-colors",
                                                    if active {"border-brand-action bg-brand-action text-white ring-4 ring-brand-action/25"}
                                            else if completed {"border-brand-action bg-brand-action text-white"}
                                            else {"border-line bg-surface text-muted"},
                                                );
                            let element_classes_3 = classes!(
                                                    "text-[10px]", "sm:text-sm", "break-words",
                                                    "font-semibold",
                                                    if active {"text-ink"} else if available {"text-ink"} else {"text-muted"},
                                                );
                            html! {
                                <li
                                    key={step.id()}
                                    class="min-w-0"
                                >
                                    <button
                                        type="button"
                                        data-practice-step={step.id()}
                                        data-progress-state={if active {"current"} else if completed {"visited"} else {"upcoming"}}
                                        aria-current={if active {"step"} else {"false"}}
                                        aria-label={format!("{}{}", step.label(), if completed {" (visited)"} else {""})}
                                        disabled={!available}
                                        onclick={on_choose_step}
                                        class={element_classes_1}
                                    >
                                        <span
                                            class={element_classes_2}
                                        >
                                            {progress_step_marker}
                                        </span>
                                        <span
                                            class={element_classes_3}
                                        >
                                            {step.label()}
                                        </span>
                                        <span
                                            class="hidden text-xs leading-snug text-muted sm:block"
                                        >
                                            {step_description}
                                        </span>
                                    </button>
                                </li>
                            }
                        })}
                    </ol>
                </div>
                if props.current == PracticeStep::Summary {
                    {props.start_action.clone()}
                } else {
                    <button type="button"
                        data-practice-continue={(props.current != PracticeStep::Path).then_some("")}
                        aria-label={continue_label} title={continue_label}
                        disabled={props.current == PracticeStep::Path || !props.can_continue}
                        onclick={on_continue}
                        class="flex size-11 shrink-0 items-center justify-center rounded-xl bg-brand-action text-white hover:bg-brand-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-40">
                        <span aria-hidden="true"><Icon data={IconData::LUCIDE_ARROW_RIGHT} width="20px" height="20px" /></span>
                    </button>
                }
            </nav>
        </div>
    }
}
