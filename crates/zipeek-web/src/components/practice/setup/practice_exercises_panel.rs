use yew::prelude::*;

use crate::reducers::practice_setup_flow::{ExerciseStage, PracticeSetupAction, PracticeSetupFlow};

use super::{
    practice_builder::PracticeBuilder, practice_exercise_settings::PracticeExerciseSettings,
    practice_exercise_stage_tabs::PracticeExerciseStageTabs,
};

#[derive(Properties, PartialEq)]
pub struct PracticeExercisesPanelProps {
    pub flow: UseReducerHandle<PracticeSetupFlow>,
}

#[function_component(PracticeExercisesPanel)]
pub fn practice_exercises_panel(props: &PracticeExercisesPanelProps) -> Html {
    let flow = &props.flow;
    let on_select_stage = {
        let flow = flow.clone();
        Callback::from(move |stage| {
            flow.dispatch(PracticeSetupAction::SelectExerciseStage(stage));
        })
    };
    let on_toggle = {
        let flow = flow.clone();
        Callback::from(move |id| {
            flow.dispatch(PracticeSetupAction::ToggleExercise(id));
        })
    };
    let on_tab = {
        let flow = flow.clone();
        Callback::from(move |tab| {
            flow.dispatch(PracticeSetupAction::SelectTab(tab));
        })
    };
    html! {
        <div data-practice-step-panel="exercises" class="flex min-h-0 flex-col gap-3 md:h-0 md:flex-1 md:overflow-hidden">
            <PracticeExerciseStageTabs
                selected={flow.exercise_stage}
                has_exercises={!flow.draft.exercise_ids.is_empty()}
                selected_count={flow.draft.exercise_ids.len()}
                on_select={on_select_stage}
            />
            <div data-practice-exercise-scroll="" tabindex="0" role="region" aria-label="Exercise types and settings" class="min-h-0 pr-2 md:flex-1 md:overflow-y-scroll md:overscroll-contain md:[scrollbar-gutter:stable]">
                if flow.exercise_stage == ExerciseStage::Choose {
                    <PracticeBuilder
                        section={ExerciseStage::Choose}
                        draft={flow.draft.clone()}
                        advanced={flow.advanced}
                        active_tab={flow.active_tab}
                        on_toggle={on_toggle.clone()}
                        on_tab={on_tab.clone()}
                    />
                } else {
                    <>
                        <PracticeBuilder
                            section={ExerciseStage::Configure}
                            draft={flow.draft.clone()}
                            advanced={flow.advanced}
                            active_tab={flow.active_tab}
                            on_toggle={on_toggle}
                            on_tab={on_tab}
                        />
                        <PracticeExerciseSettings flow={flow.clone()} />
                    </>
                }
            </div>
        </div>
    }
}
