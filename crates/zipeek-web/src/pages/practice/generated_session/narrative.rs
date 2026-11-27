use crate::pages::practice::{
    feedback::ExercisePrimaryAction, instruction::InstructionIcon, surface::ExerciseSurface,
};
use std::rc::Rc;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::ai_narrative::{ConversationPractice, ReadingComprehension, StoryBuilder};

#[derive(Properties, PartialEq)]
pub(super) struct StoryProps {
    pub content: StoryBuilder,
    pub on_scored: Callback<bool>,
}

#[function_component(StoryPractice)]
pub(super) fn story_practice(props: &StoryProps) -> Html {
    let selected = use_state(|| vec![None::<usize>; props.content.gaps.len()]);
    let completed = selected.iter().all(Option::is_some);
    let on_check = {
        let selected = selected.clone();
        let content = props.content.clone();
        let on_scored = props.on_scored.clone();
        Callback::from(move |_: MouseEvent| {
            let all_correct = content.gaps.iter().enumerate().all(|(i, gap)| {
                selected
                    .get(i)
                    .and_then(|index| *index)
                    .and_then(|index| content.word_bank.get(index))
                    == Some(&gap.answer)
            });
            on_scored.emit(all_correct);
        })
    };
    html! {
        <div class="w-full" data-story-builder="">
            <ExerciseSurface icon={InstructionIcon::Arrange} instruction="Fill the gaps using the word bank">
            <div class="space-y-3 rounded-xl border border-line bg-surface p-4" lang="zh-Hans">
                {for props.content.sentences.iter().enumerate().map(|(sentence_index, sentence)| {
                    let gap = props.content.gaps.iter().enumerate()
                        .find(|(_, gap)| usize::from(gap.sentence_index) == sentence_index);
                    if let Some((gap_index, _)) = gap {
                        let parts = sentence.split_once("____").expect("validated story gap");
                        let current = selected.get(gap_index).and_then(|selection| *selection)
                            .and_then(|index| props.content.word_bank.get(index));
                        let on_slot = {
                            let selected = selected.clone();
                            Callback::from(move |_: MouseEvent| {
                                let mut updated = (*selected).clone();
                                updated[gap_index] = None;
                                selected.set(updated);
                            })
                        };
                        html! { <p class="text-lg leading-relaxed">{parts.0}
                            <button type="button" class="mx-1 min-w-16 rounded-md border-b-2 border-accent px-2 py-1 text-accent focus-visible:outline focus-visible:outline-2" onclick={on_slot}
                                aria-label={format!("Clear gap {}", gap_index + 1)}>
                                {current.cloned().unwrap_or_else(|| "____".to_owned())}
                            </button>{parts.1}</p> }
                    } else {
                        html! { <p class="text-lg leading-relaxed">{sentence}</p> }
                    }
                })}
            </div>
            <div role="group" aria-label="Word bank" class="flex flex-wrap gap-2 rounded-xl border border-line p-3">
                {for props.content.word_bank.iter().enumerate().map(|(index, word)| {
                    let already_used = selected.iter().any(|item| *item == Some(index));
                    let selected = selected.clone();
                    let onclick = Callback::from(move |_: MouseEvent| {
                        let mut updated = (*selected).clone();
                        if let Some(position) = updated.iter().position(Option::is_none) {
                            updated[position] = Some(index);
                            selected.set(updated);
                        }
                    });
                    html! { <button type="button" class="rounded-lg border border-line bg-surface px-3 py-2 text-base hover:border-accent disabled:opacity-40"
                        disabled={already_used} {onclick}>{word}</button> }
                })}
            </div>
            <ExercisePrimaryAction label="Check answers" on_activate={on_check} disabled={!completed}>
                <Icon data={IconData::LUCIDE_CHECK} width="18px" height="18px"/>
            </ExercisePrimaryAction>
            </ExerciseSurface>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct ReadingProps {
    pub content: ReadingComprehension,
    pub on_scored: Callback<bool>,
}

#[function_component(ReadingPractice)]
pub(super) fn reading_practice(props: &ReadingProps) -> Html {
    let selected = use_state(|| None::<u8>);
    let on_check = {
        let selected = selected.clone();
        let on_scored = props.on_scored.clone();
        let correct = props.content.correct_index;
        Callback::from(move |_: MouseEvent| {
            if let Some(index) = *selected {
                on_scored.emit(index == correct);
            }
        })
    };
    html! {
        <div class="w-full" data-reading-comprehension="">
            <ExerciseSurface icon={InstructionIcon::Choose} instruction="Read the passage and choose an answer">
            <div class="space-y-3 rounded-xl border border-line bg-surface p-4" lang="zh-Hans">
                {for props.content.sentences.iter().map(|sentence| html! { <p class="text-lg leading-relaxed">{sentence}</p> })}
            </div>
            <h3 class="font-semibold text-lg" lang="zh-Hans">{&props.content.question}</h3>
            <div role="radiogroup" aria-label="Reading answers" class="space-y-2">
                {for props.content.choices.iter().enumerate().map(|(index, choice)| {
                    let selected_state = selected.clone();
                    let onclick = Callback::from(move |_: MouseEvent| selected_state.set(Some(index as u8)));
                    html! { <button type="button" role="radio" aria-checked={(*selected == Some(index as u8)).to_string()}
                        class={classes!("block", "w-full", "rounded-lg", "border", "p-3", "text-left", "hover:border-accent", if *selected == Some(index as u8) { "border-accent bg-surface" } else { "border-line" })}
                        {onclick}>{choice}</button> }
                })}
            </div>
            <ExercisePrimaryAction label="Check answer" on_activate={on_check} disabled={selected.is_none()}>
                <Icon data={IconData::LUCIDE_CHECK} width="18px" height="18px"/>
            </ExercisePrimaryAction>
            </ExerciseSurface>
        </div>
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ConversationState {
    turn_index: usize,
    correct_count: usize,
    selection: Option<usize>,
    submitted: bool,
}

enum ConversationAction {
    Select(usize),
    Advance {
        correct_index: usize,
        total_turns: usize,
    },
}

impl Reducible for ConversationState {
    type Action = ConversationAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            ConversationAction::Select(answer) if !self.submitted => next.selection = Some(answer),
            ConversationAction::Select(_) => {}
            ConversationAction::Advance {
                correct_index,
                total_turns,
            } => {
                if !self.submitted && self.turn_index < total_turns {
                    if let Some(answer) = self.selection {
                        next.correct_count += usize::from(answer == correct_index);
                        next.turn_index += 1;
                        next.selection = None;
                        next.submitted = next.turn_index == total_turns;
                    }
                }
            }
        }
        Rc::new(next)
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct ConversationProps {
    pub content: ConversationPractice,
    pub on_scored: Callback<bool>,
}

#[function_component(ConversationPracticeView)]
pub(super) fn conversation_practice(props: &ConversationProps) -> Html {
    let state = use_reducer_eq(ConversationState::default);
    let Some(turn) = props.content.turns.get(state.turn_index) else {
        return html! {};
    };
    let on_next = {
        let state = state.clone();
        let on_scored = props.on_scored.clone();
        let total_turns = props.content.turns.len();
        let correct_index = usize::from(turn.correct_index);
        Callback::from(move |_: MouseEvent| {
            let Some(answer) = state.selection else {
                return;
            };
            if state.submitted {
                return;
            }
            let next_correct = state.correct_count + usize::from(answer == correct_index);
            state.dispatch(ConversationAction::Advance {
                correct_index,
                total_turns,
            });
            if state.turn_index + 1 == total_turns {
                on_scored.emit(next_correct == total_turns);
            }
        })
    };
    html! {
        <div class="w-full" data-conversation-practice="">
            <ExerciseSurface icon={InstructionIcon::Choose} instruction="Choose how to reply">
            <p class="text-sm text-muted">{&props.content.scenario} {format!("· Turn {} of {}", state.turn_index + 1, props.content.turns.len())}</p>
            <p class="rounded-2xl border border-line bg-surface p-4 text-lg" lang="zh-Hans">{&turn.speaker_text}</p>
            <div role="radiogroup" aria-label="Choose your reply" class="space-y-2">
                {for turn.choices.iter().enumerate().map(|(answer, text)| {
                    let state_for_selection = state.clone();
                    let onclick = Callback::from(move |_: MouseEvent| state_for_selection.dispatch(ConversationAction::Select(answer)));
                    html! { <button type="button" role="radio" aria-checked={(state.selection == Some(answer)).to_string()}
                        class={classes!("block", "w-full", "rounded-lg", "border", "p-3", "text-left", "hover:border-accent", if state.selection == Some(answer) { "border-accent bg-surface" } else { "border-line" })}
                        {onclick}>{text}</button> }
                })}
            </div>
            <ExercisePrimaryAction label={if state.turn_index + 1 == props.content.turns.len() { "Finish" } else { "Reply" }} on_activate={on_next} disabled={state.selection.is_none()}>
                <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="18px" height="18px"/>
            </ExercisePrimaryAction>
            </ExerciseSurface>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_advancement_is_atomic_and_counts_correct_answers() {
        let initial = Rc::new(ConversationState::default());
        let selected = initial.reduce(ConversationAction::Select(2));
        let advanced = selected.reduce(ConversationAction::Advance {
            correct_index: 2,
            total_turns: 2,
        });
        assert_eq!(advanced.turn_index, 1);
        assert_eq!(advanced.correct_count, 1);
        assert_eq!(advanced.selection, None);
        assert!(!advanced.submitted);
        let last =
            advanced
                .reduce(ConversationAction::Select(0))
                .reduce(ConversationAction::Advance {
                    correct_index: 1,
                    total_turns: 2,
                });
        assert_eq!(last.correct_count, 1);
        assert!(last.submitted);
        let ignored = last.clone().reduce(ConversationAction::Advance {
            correct_index: 0,
            total_turns: 2,
        });
        assert_eq!(ignored.as_ref(), last.as_ref());
    }
}
