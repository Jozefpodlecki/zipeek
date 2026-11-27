use std::rc::Rc;

use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinishPrompt {
    Hidden,
    Confirming,
}

enum FinishPromptAction {
    Show,
    Cancel,
}

impl Reducible for FinishPrompt {
    type Action = FinishPromptAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            FinishPromptAction::Show => Rc::new(Self::Confirming),
            FinishPromptAction::Cancel => Rc::new(Self::Hidden),
        }
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct ExerciseFrameProps {
    pub position: usize,
    pub total: usize,
    pub endless: bool,
    #[prop_or_default]
    pub on_finish_endless: Option<Callback<MouseEvent>>,
    pub children: Children,
}

#[function_component(ExerciseFrame)]
pub(super) fn exercise_frame(props: &ExerciseFrameProps) -> Html {
    let finish_prompt = use_reducer_eq(|| FinishPrompt::Hidden);
    {
        let finish_prompt = finish_prompt.clone();
        use_effect_with(props.position, move |_| {
            finish_prompt.dispatch(FinishPromptAction::Cancel);
        });
    }
    let position = if props.endless {
        format!("Exercise {}", props.position)
    } else {
        format!("Exercise {} of {}", props.position, props.total)
    };
    let on_show_finish = {
        let finish_prompt = finish_prompt.clone();
        Callback::from(move |_: MouseEvent| finish_prompt.dispatch(FinishPromptAction::Show))
    };
    let on_cancel_finish = {
        let finish_prompt = finish_prompt.clone();
        Callback::from(move |_: MouseEvent| finish_prompt.dispatch(FinishPromptAction::Cancel))
    };
    html! {
        <section class="mx-auto flex w-full flex-col items-center gap-3 sm:gap-4" aria-label={position.clone()} data-practice-exercise-frame="">
            <div class="w-full" data-practice-exercise-body="">
                {for props.children.iter()}
            </div>
            {if props.endless {
                if let Some(on_finish) = props.on_finish_endless.as_ref() {
                    if *finish_prompt == FinishPrompt::Confirming {
                        html! {
                            <div class="flex w-full max-w-lg flex-col items-center gap-3 rounded-xl border border-line bg-surface/95 p-4 text-center" role="group" aria-label="Finish session confirmation" data-practice-finish-confirmation="">
                                <p class="text-sm text-ink">{"Finish this session and save your completed exercises?"}</p>
                                <div class="flex flex-wrap items-center justify-center gap-3">
                                    <button type="button" class="rounded-lg border border-line px-4 py-2 text-sm text-ink" onclick={on_cancel_finish}> {"Keep practicing"} </button>
                                    <button type="button" class="rounded-lg bg-brand-action px-4 py-2 text-sm font-medium text-white" onclick={on_finish.clone()} data-practice-finish-endless=""> {"Finish session"} </button>
                                </div>
                            </div>
                        }
                    } else {
                        html! {
                            <button type="button" class="rounded-lg border border-line bg-surface/95 px-4 py-2 text-sm text-ink hover:border-brand-action focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent" onclick={on_show_finish} data-practice-request-finish="">
                                {"Finish session"}
                            </button>
                        }
                    }
                } else {
                    Html::default()
                }
            } else {
                Html::default()
            }}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finish_confirmation_reducer_is_reversible() {
        let state = Rc::new(FinishPrompt::Hidden);
        let state = state.reduce(FinishPromptAction::Show);
        assert_eq!(*state, FinishPrompt::Confirming);
        let state = state.reduce(FinishPromptAction::Cancel);
        assert_eq!(*state, FinishPrompt::Hidden);
    }
}
