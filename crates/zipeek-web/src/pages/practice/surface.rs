//! A shared presentation surface for Practice activities.
//!
//! Individual exercises own input, scoring and their Yew reducers. This component
//! owns the consistent responsive card, accessible action instruction and the
//! spacing around gameplay, feedback and action controls. It deliberately does
//! not introduce a second session progress indicator or replace CreatureProgress.
use yew::prelude::*;

use super::instruction::{ExerciseInstruction, InstructionIcon};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ExerciseSurfaceSize {
    #[default]
    Standard,
    Wide,
}

impl ExerciseSurfaceSize {
    fn width_class(self) -> &'static str {
        match self {
            Self::Standard => "max-w-3xl",
            Self::Wide => "max-w-4xl",
        }
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct ExerciseSurfaceProps {
    pub icon: InstructionIcon,
    pub instruction: AttrValue,
    #[prop_or_default]
    pub size: ExerciseSurfaceSize,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ExerciseSurface)]
pub(crate) fn exercise_surface(props: &ExerciseSurfaceProps) -> Html {
    html! {
        <section
            class={classes!(
                "mx-auto", "w-full", "min-w-0", "space-y-4", "rounded-2xl", "border",
                "border-line", "bg-surface", "p-4", "shadow-sm", "sm:p-6",
                props.size.width_class(),
            )}
            data-practice-exercise-surface=""
        >
            {if props.instruction.trim().is_empty() {
                Html::default()
            } else {
                html! {
                    <ExerciseInstruction icon={props.icon} text={props.instruction.clone()} />
                }
            }}
            {for props.children.iter()}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_sizes_remain_bounded_in_the_session_workspace() {
        assert_eq!(ExerciseSurfaceSize::Standard.width_class(), "max-w-3xl");
        assert_eq!(ExerciseSurfaceSize::Wide.width_class(), "max-w-4xl");
    }
}
