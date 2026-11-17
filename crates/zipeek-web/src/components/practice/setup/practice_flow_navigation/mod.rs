mod step;
pub use self::step::PracticeStep;

use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub struct PracticeFlowNavigationProps {
    pub current: PracticeStep,
    pub furthest: PracticeStep,
    pub on_select: Callback<PracticeStep>,
}

#[function_component(PracticeFlowNavigation)]
pub fn practice_flow_navigation(props: &PracticeFlowNavigationProps) -> Html {
    let fraction = match props.furthest.index() {
        0 => "0%",
        1 => "33.333%",
        2 => "66.667%",
        _ => "100%",
    };
    html! {
        <div
            class="shrink-0 space-y-4"
        >
            <nav
                aria-label="Practice setup progress"
                data-practice-progress=""
                class="relative rounded-xl border border-line bg-surface/95 px-3 pb-3 pt-4 sm:px-6"
            >
                <div
                    aria-hidden="true"
                    class="pointer-events-none absolute left-[12.5%] right-[12.5%] top-[32px] h-px bg-line"
                >
                    <div
                        class="h-full bg-brand-action transition-all duration-200"
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
                                                "text-sm",
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
            </nav>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct PracticeStepFooterProps {
    pub current: PracticeStep,
    pub exercise_stage: crate::reducers::practice_setup_flow::ExerciseStage,
    pub can_continue: bool,
    #[prop_or_default]
    pub embedded: bool,
    pub on_back: Callback<()>,
    pub on_continue: Callback<()>,
}

#[function_component(PracticeStepFooter)]
pub fn practice_step_footer(props: &PracticeStepFooterProps) -> Html {
    let can_go_back = props.current != PracticeStep::Path;
    let can_go_forward = !matches!(props.current, PracticeStep::Path | PracticeStep::Summary);
    let on_back = {
        let action = props.on_back.clone();
        Callback::from(move |_: MouseEvent| action.emit(()))
    };
    let on_continue = {
        let action = props.on_continue.clone();
        Callback::from(move |_: MouseEvent| action.emit(()))
    };
    let forward_navigation_action = {
        if can_go_forward {
            html! {
                <button
                    type="button"
                    data-practice-continue=""
                    disabled={!props.can_continue}
                    class="inline-flex min-h-11 items-center gap-2 rounded-xl bg-brand-action px-5 font-semibold text-white hover:bg-brand-hover disabled:cursor-not-allowed disabled:opacity-50"
                    onclick={on_continue}
                >
                    {"Continue"}
                    <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="17px" height="17px" />
                </button>
            }
        } else {
            Html::default()
        }
    };

    let back_navigation_action = if can_go_back {
        html! {
            <button
                type="button"
                class="min-h-11 rounded-xl border border-line bg-raised px-5 font-semibold"
                onclick={on_back}
            >
                {"Back"}
            </button>
        }
    } else {
        Html::default()
    };

    html! {
        <footer
            class={if props.embedded {
                "flex min-h-16 shrink-0 items-center justify-between gap-3 border-b border-line bg-transparent px-3 py-2 sm:px-4"
            } else {
                "flex min-h-16 shrink-0 items-center justify-between gap-3 rounded-xl border border-line bg-surface px-3 py-2 sm:px-4"
            }}
            data-practice-step-navigation=""
        >
            {back_navigation_action}
            <span
                class="flex-1"
            />
            {forward_navigation_action}
        </footer>
    }
}
