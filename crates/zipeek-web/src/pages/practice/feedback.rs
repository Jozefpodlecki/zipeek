use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FeedbackVerdict {
    Correct,
    Partial,
    Incorrect,
}

impl FeedbackVerdict {
    fn label(self) -> &'static str {
        match self {
            Self::Correct => "Correct",
            Self::Partial => "Partially correct",
            Self::Incorrect => "Not quite",
        }
    }

    fn visual_class(self) -> &'static str {
        match self {
            Self::Correct => "border-green-500/40",
            Self::Partial => "border-amber-500/40",
            Self::Incorrect => "border-red-500/40",
        }
    }

    fn status(self) -> &'static str {
        match self {
            Self::Correct => "correct",
            Self::Partial => "partial",
            Self::Incorrect => "incorrect",
        }
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct ExerciseFeedbackProps {
    pub verdict: FeedbackVerdict,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ExerciseFeedback)]
pub(super) fn exercise_feedback(props: &ExerciseFeedbackProps) -> Html {
    html! {
        <section
            class={classes!("space-y-3", "rounded-xl", "border", "bg-raised/50", "p-4", props.verdict.visual_class())}
            role="status"
            aria-live="polite"
            aria-atomic="true"
            data-practice-feedback=""
            data-practice-feedback-verdict={props.verdict.status()}
        >
            <p class="font-semibold text-ink">{props.verdict.label()}</p>
            {for props.children.iter()}
        </section>
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct ExercisePrimaryActionProps {
    pub label: AttrValue,
    #[prop_or_default]
    pub translation: bool,
    #[prop_or_default]
    pub sentence_transformation: bool,
    #[prop_or_default]
    pub vocabulary: bool,
    pub on_activate: Callback<MouseEvent>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ExercisePrimaryAction)]
pub(super) fn exercise_primary_action(props: &ExercisePrimaryActionProps) -> Html {
    html! {
        <button
            type="button"
            class="inline-flex min-h-11 items-center gap-2 rounded-lg bg-brand-action px-5 py-3 font-semibold text-white hover:bg-brand-hover disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            onclick={props.on_activate.clone()}
            disabled={props.disabled}
            data-practice-primary-action=""
            data-generated-translation-check={props.translation.then_some("")}
            data-sentence-transformation-check={props.sentence_transformation.then_some("")}
            data-exercise-check={props.vocabulary.then_some("")}
        >
            {for props.children.iter()}
            {&props.label}
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct ExerciseSubmitActionProps {
    pub label: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub spot_mistake: bool,
}

#[function_component(ExerciseSubmitAction)]
pub(super) fn exercise_submit_action(props: &ExerciseSubmitActionProps) -> Html {
    html! {
        <button
            type="submit"
            class="inline-flex min-h-11 items-center rounded-lg bg-brand-action px-5 py-3 font-semibold text-white hover:bg-brand-hover disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            disabled={props.disabled}
            data-practice-primary-action=""
            data-spot-mistake-check={props.spot_mistake.then_some("")}
        >
            {&props.label}
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct ContinueExerciseProps {
    pub on_continue: Callback<MouseEvent>,
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub local_quiz: bool,
    #[prop_or_default]
    pub sentence_builder: bool,
    #[prop_or_default]
    pub word_mosaics: bool,
    #[prop_or_default]
    pub character_builder: bool,
    #[prop_or_default]
    pub listening_choice: bool,
    #[prop_or_default]
    pub speaking: bool,
    #[prop_or_default]
    pub spot_mistake: bool,
    #[prop_or_default]
    pub sentence_transformation: bool,
}

#[function_component(ContinueExercise)]
pub(crate) fn continue_exercise(props: &ContinueExerciseProps) -> Html {
    html! {
        <div class="flex flex-wrap items-center gap-3 pt-1" data-practice-actions="">
            <button
                type="button"
                class="inline-flex min-h-11 items-center rounded-lg bg-brand-action px-5 py-3 font-semibold text-white hover:bg-brand-hover disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                onclick={props.on_continue.clone()}
                disabled={props.disabled}
                data-local-quiz-continue={props.local_quiz.then_some("")}
                data-sentence-builder-continue={props.sentence_builder.then_some("")}
                data-practice-continue=""
                data-mosaic-continue={props.word_mosaics.then_some("")}
                data-exercise-continue={props.character_builder.then_some("")}
                data-listening-continue={props.listening_choice.then_some("")}
                data-speaking-session-continue={props.speaking.then_some("")}
                data-spot-mistake-continue={props.spot_mistake.then_some("")}
                data-sentence-transformation-continue={props.sentence_transformation.then_some("")}
            >
                {props.label.clone().unwrap_or_else(|| "Continue session".into())}
            </button>
        </div>
    }
}

/// A single reviewed exercise lifecycle boundary: verdict, explanation, and the
/// primary Continue action remain together. Evaluation belongs to each exercise;
/// this component does not infer correctness or attempt evidence.
#[derive(Properties, PartialEq)]
pub(super) struct ExerciseReviewProps {
    pub verdict: FeedbackVerdict,
    pub on_continue: Callback<MouseEvent>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub local_quiz: bool,
    #[prop_or_default]
    pub sentence_builder: bool,
    #[prop_or_default]
    pub word_mosaics: bool,
    #[prop_or_default]
    pub character_builder: bool,
    #[prop_or_default]
    pub listening_choice: bool,
    #[prop_or_default]
    pub speaking: bool,
    #[prop_or_default]
    pub spot_mistake: bool,
    #[prop_or_default]
    pub sentence_transformation: bool,
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ExerciseReview)]
pub(super) fn exercise_review(props: &ExerciseReviewProps) -> Html {
    html! {
        <ExerciseFeedback verdict={props.verdict}>
            {for props.children.iter()}
            <ContinueExercise
                on_continue={props.on_continue.clone()}
                disabled={props.disabled}
                label={props.label.clone()}
                local_quiz={props.local_quiz}
                sentence_builder={props.sentence_builder}
                word_mosaics={props.word_mosaics}
                character_builder={props.character_builder}
                listening_choice={props.listening_choice}
                speaking={props.speaking}
                spot_mistake={props.spot_mistake}
                sentence_transformation={props.sentence_transformation}
            />
        </ExerciseFeedback>
    }
}

#[cfg(test)]
mod tests {
    use super::FeedbackVerdict;

    #[test]
    fn feedback_verdicts_have_distinct_accessible_labels_and_statuses() {
        let values = [
            (FeedbackVerdict::Correct, "Correct", "correct"),
            (FeedbackVerdict::Partial, "Partially correct", "partial"),
            (FeedbackVerdict::Incorrect, "Not quite", "incorrect"),
        ];
        for (verdict, expected_label, expected_status) in values {
            assert_eq!(verdict.label(), expected_label);
            assert_eq!(verdict.status(), expected_status);
        }
    }
}
