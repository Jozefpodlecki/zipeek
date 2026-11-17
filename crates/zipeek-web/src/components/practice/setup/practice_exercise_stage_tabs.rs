use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::reducers::practice_setup_flow::ExerciseStage;

#[derive(Properties, PartialEq)]
pub(crate) struct PracticeExerciseStageTabsProps {
    pub selected: ExerciseStage,
    pub has_exercises: bool,
    pub selected_count: usize,
    pub on_select: Callback<ExerciseStage>,
}

#[function_component(PracticeExerciseStageTabs)]
pub(crate) fn practice_exercise_stage_tabs(props: &PracticeExerciseStageTabsProps) -> Html {
    html! {
        <nav
            role="group"
            aria-label="Exercise setup"
            data-practice-exercise-stages=""
            class="flex gap-1 rounded-xl border border-line bg-surface p-1"
        >
            {for [
                (ExerciseStage::Choose, "Choose", IconData::LUCIDE_LAYOUT_GRID),
                (ExerciseStage::Configure, "Configure", IconData::LUCIDE_SETTINGS),
            ].into_iter().map(|(stage, label, icon)| {
                let selected = props.selected == stage;
                let disabled = stage == ExerciseStage::Configure && !props.has_exercises;
                let on_select = props.on_select.clone();
                let on_select_stage = Callback::from(move |_| on_select.emit(stage));
                let element_classes_1 = classes!(
                                "flex",
                                "min-h-11",
                                "items-center",
                                "justify-center",
                                "gap-2",
                                "rounded-lg",
                                "px-5",
                                "text-sm",
                                "font-semibold",
                                "transition-colors",
                                if selected {"bg-brand-action text-white"}
                            else {"text-muted hover:bg-raised hover:text-ink"},
                                if disabled {"cursor-not-allowed opacity-50"} else {""},
                            );
                html! {
                    <button
                        type="button"
                        key={label}
                        data-practice-exercise-stage={if stage == ExerciseStage::Choose {"choose"} else {"configure"}}
                        aria-pressed={selected.to_string()}
                        disabled={disabled}
                        class={element_classes_1}
                        onclick={on_select_stage}
                    >
                        <Icon
                            data={icon}
                            width="17px"
                            height="17px"
                        />
                        {label}
                    </button>
                }
            })}
            <span
                data-practice-selected-count=""
                role="status"
                class="ml-auto inline-flex min-h-11 items-center rounded-lg border border-brand-action/60 bg-brand-action/10 px-4 text-sm font-semibold text-ink"
            >
                <span aria-hidden="true" class="size-2.5 rounded-full bg-brand-action" />
                <span class="ml-2 tabular-nums">{props.selected_count}</span>
                <span class="ml-1 text-muted">{"selected"}</span>
            </span>
        </nav>
    }
}
