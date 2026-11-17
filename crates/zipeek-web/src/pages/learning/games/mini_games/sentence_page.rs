use std::rc::Rc;
use yew::prelude::*;
use zipeek_exercises::mini_games::{
    SENTENCE_QUESTIONS, SentenceAction, SentenceState, sentence_rounds,
};

use super::{
    sentence::SentenceRound,
    setup::{GameFlow, GameFlowAction, GamePhase, GameSetup},
    shell::GameShell,
    summary::GameSummary,
    vocabulary::{GameVocabulary, VocabularyFallback, use_game_vocabulary},
};

#[derive(Clone, Debug, Default, PartialEq)]
struct SentenceController(SentenceState);

impl Reducible for SentenceController {
    type Action = SentenceAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = self.0.clone();
        next.apply(action);
        Rc::new(Self(next))
    }
}

#[derive(Properties, PartialEq)]
struct GameSessionProps {
    rounds: Vec<usize>,
    source: String,
}

#[function_component(SentenceRelayPage)]
pub fn sentence_page() -> Html {
    let (vocabulary, sets) = use_game_vocabulary();
    let rounds = match &vocabulary {
        GameVocabulary::Starter => Some((0..SENTENCE_QUESTIONS.len()).collect::<Vec<_>>()),
        GameVocabulary::Ready(data) => data
            .result
            .as_ref()
            .ok()
            .map(|words| sentence_rounds(words.iter().map(|word| word.simplified.as_str()))),
        _ => None,
    };
    let source_label = vocabulary.label();
    let session_key = match &vocabulary {
        GameVocabulary::Starter => "starter".to_owned(),
        GameVocabulary::Ready(data) => format!("{:?}:{rounds:?}", data.source),
        _ => String::new(),
    };
    let flow = use_reducer(GameFlow::default);
    {
        let flow = flow.clone();
        use_effect_with(session_key.clone(), move |_| {
            flow.dispatch(GameFlowAction::ChangeSetup);
        });
    }
    let on_start = {
        let flow = flow.clone();
        Callback::from(move |_: MouseEvent| flow.dispatch(GameFlowAction::Start))
    };
    let on_change_setup = {
        let flow = flow.clone();
        Callback::from(move |_: MouseEvent| flow.dispatch(GameFlowAction::ChangeSetup))
    };
    html! {
        <div class="mx-auto flex w-full max-w-3xl flex-col gap-5 px-4 py-8">
            if let Some(rounds) = rounds {
                if rounds.is_empty() {
                    <GameSetup title="Sentence Relay" description="Reorder words into meaningful Mandarin sentences." path="/games/sentence-relay" sets={sets} source={source_label} available={0} on_start={on_start} />
                } else if flow.phase == GamePhase::Setup {
                    <GameSetup title="Sentence Relay" description="Reorder words into meaningful Mandarin sentences." path="/games/sentence-relay" sets={sets} source={source_label} available={rounds.len()} on_start={on_start} />
                } else {
                    <button type="button" data-game-change-setup="" class="self-start rounded-lg border border-line px-4 py-2 text-sm hover:border-accent" onclick={on_change_setup}>{"Change setup"}</button>
                    <GameSession key={session_key} rounds={rounds} source={source_label} />
                }
            } else {
                <GameSetup title="Sentence Relay" description="Reorder words into meaningful Mandarin sentences." path="/games/sentence-relay" sets={sets} source={source_label} available={0} on_start={on_start} />
                <VocabularyFallback state={vocabulary} />
            }
        </div>
    }
}

#[function_component(GameSession)]
fn game_session(props: &GameSessionProps) -> Html {
    let initial_rounds = props.rounds.clone();
    let state = use_reducer(move || SentenceController(SentenceState::from_rounds(initial_rounds)));
    let on_select = {
        let state = state.clone();
        Callback::from(move |token| state.dispatch(SentenceAction::Select(token)))
    };
    let on_remove = {
        let state = state.clone();
        Callback::from(move |token| state.dispatch(SentenceAction::Remove(token)))
    };
    let on_undo = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(SentenceAction::Undo))
    };
    let on_clear = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(SentenceAction::Clear))
    };
    let on_check = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(SentenceAction::Check))
    };
    let on_restart = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(SentenceAction::Restart))
    };
    let game = &state.0;
    html! {
        <GameShell source={props.source.clone()} title="Sentence Relay" step={game.step} total={game.total()} mistakes={game.mistakes} notice={game.notice}>
            if game.complete() {
                <GameSummary rounds={game.total()} mistakes={game.mistakes} on_restart={on_restart} />
            } else {
                <SentenceRound game={game.clone()} on_select={on_select} on_remove={on_remove} on_undo={on_undo} on_clear={on_clear} on_check={on_check} />
            }
        </GameShell>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentence_controller_requires_a_new_attempt_after_wrong_check() {
        let start = Rc::new(SentenceController::default());
        let selected = [0, 1, 2, 3].into_iter().fold(start, |game, index| {
            game.reduce(SentenceAction::Select(index))
        });
        let first = selected.reduce(SentenceAction::Check);
        let duplicate = first.reduce(SentenceAction::Check);
        assert_eq!(duplicate.0.mistakes, 1);
    }

    #[test]
    fn sentence_controller_clears_notice_when_editing() {
        let state = Rc::new(SentenceController::default());
        let populated = [0, 1, 2, 3].into_iter().fold(state, |state, token| {
            state.reduce(SentenceAction::Select(token))
        });
        let wrong = populated.reduce(SentenceAction::Check);
        assert_eq!(wrong.0.mistakes, 1);
        let edited = wrong.reduce(SentenceAction::Undo);
        assert_eq!(edited.0.notice, None);
    }
}
