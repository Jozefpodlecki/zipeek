use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstructionIcon {
    Find,
    Rewrite,
    Choose,
    Match,
    Listen,
    Arrange,
    Write,
    Speak,
}

#[derive(Properties, PartialEq)]
pub(crate) struct ExerciseInstructionProps {
    pub icon: InstructionIcon,
    pub text: AttrValue,
}

#[function_component(ExerciseInstruction)]
pub(crate) fn exercise_instruction(props: &ExerciseInstructionProps) -> Html {
    let icon = match props.icon {
        InstructionIcon::Find => IconData::LUCIDE_SEARCH,
        InstructionIcon::Rewrite => IconData::LUCIDE_PENCIL,
        InstructionIcon::Choose => IconData::LUCIDE_LIST,
        InstructionIcon::Match => IconData::LUCIDE_CHECK_CIRCLE,
        InstructionIcon::Listen => IconData::LUCIDE_HEADPHONES,
        InstructionIcon::Arrange => IconData::LUCIDE_ARROW_LEFT_RIGHT,
        InstructionIcon::Write => IconData::LUCIDE_PENCIL,
        InstructionIcon::Speak => IconData::LUCIDE_MIC,
    };
    html! {
        <div class="flex items-center gap-3" data-practice-instruction="">
            <span class="flex size-11 shrink-0 items-center justify-center rounded-xl bg-accent-soft text-accent" aria-hidden="true">
                <Icon data={icon} width="22" height="22" />
            </span>
            <p class="font-medium text-ink">{&props.text}</p>
        </div>
    }
}
