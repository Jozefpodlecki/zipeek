use std::rc::Rc;
use yew::prelude::*;
use zipeek_exercises::mini_games::{
    CLASSIFIER_QUESTIONS, ClassifierAction, ClassifierState, classifier_rounds,
};

use super::{
    classifier::ClassifierRound,
    setup::{GameFlow, GameFlowAction, GamePhase, GameSetup},
    shell::GameShell,
    summary::GameSummary,
    vocabulary::{GameVocabulary, VocabularyFallback, use_game_vocabulary},
};

#[derive(Clone, Debug, Default, PartialEq)]
struct ClassifierController(ClassifierState);

impl Reducible for ClassifierController {
    type Action = ClassifierAction;

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

#[function_component(ClassifierMarketPage)]
pub fn classifier_page() -> Html {
    let (vocabulary, sets) = use_game_vocabulary();
    let rounds = match &vocabulary {
        GameVocabulary::Starter => Some((0..CLASSIFIER_QUESTIONS.len()).collect::<Vec<_>>()),
        GameVocabulary::Ready(data) => data
            .result
            .as_ref()
            .ok()
            .map(|words| classifier_rounds(words.iter().map(|word| word.simplified.as_str()))),
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
                    <GameSetup title="Classifier Market" description="Choose the correct Chinese measure word for each item." path="/games/classifier-market" sets={sets} source={source_label} available={0} on_start={on_start} />
                } else if flow.phase == GamePhase::Setup {
                    <GameSetup title="Classifier Market" description="Choose the correct Chinese measure word for each item." path="/games/classifier-market" sets={sets} source={source_label} available={rounds.len()} on_start={on_start} />
                } else {
                    <button type="button" data-game-change-setup="" class="self-start rounded-lg border border-line px-4 py-2 text-sm hover:border-accent" onclick={on_change_setup}>{"Change setup"}</button>
                    <GameSession key={session_key} rounds={rounds} source={source_label} />
                }
            } else {
                <GameSetup title="Classifier Market" description="Choose the correct Chinese measure word for each item." path="/games/classifier-market" sets={sets} source={source_label} available={0} on_start={on_start} />
                <VocabularyFallback state={vocabulary} />
            }
        </div>
    }
}

#[function_component(GameSession)]
fn game_session(props: &GameSessionProps) -> Html {
    let initial_rounds = props.rounds.clone();
    let state =
        use_reducer(move || ClassifierController(ClassifierState::from_rounds(initial_rounds)));
    let on_answer = {
        let state = state.clone();
        Callback::from(move |choice| state.dispatch(ClassifierAction::Answer(choice)))
    };
    let on_restart = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(ClassifierAction::Restart))
    };
    let game = &state.0;
    html! {
        <GameShell source={props.source.clone()} title="Classifier Market" step={game.step} total={game.total()} mistakes={game.mistakes} notice={game.notice}>
            if game.complete() {
                <GameSummary rounds={game.total()} mistakes={game.mistakes} on_restart={on_restart} />
            } else {
                <ClassifierRound game={game.clone()} on_answer={on_answer} />
            }
        </GameShell>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_controller_does_not_double_count_an_unchanged_choice() {
        let initial = Rc::new(ClassifierController::default());
        let first = initial.reduce(ClassifierAction::Answer(2));
        let repeated = first.reduce(ClassifierAction::Answer(2));
        assert_eq!(repeated.0.mistakes, 1);
        assert_eq!(repeated.0.last_wrong_choice, Some(2));
    }

    #[test]
    fn classifier_controller_keeps_feedback_with_transition() {
        let initial = Rc::new(ClassifierController::default());
        let wrong = initial.reduce(ClassifierAction::Answer(2));
        assert_eq!(wrong.0.mistakes, 1);
        assert_eq!(wrong.0.notice, Some("Try another measure word"));
        let reset = wrong.reduce(ClassifierAction::Restart);
        assert_eq!(reset.0, ClassifierState::default());
    }
}
