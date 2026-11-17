use std::rc::Rc;
use yew::prelude::*;
use zipeek_exercises::mini_games::{
    Direction, NAVIGATION_ROUTE, NavigatorAction, NavigatorState, navigator_rounds,
};

use super::{
    navigator::NavigatorRound,
    setup::{GameFlow, GameFlowAction, GamePhase, GameSetup},
    shell::GameShell,
    summary::GameSummary,
    vocabulary::{GameVocabulary, VocabularyFallback, use_game_vocabulary},
};
use crate::hooks::use_app_environment;

#[derive(Clone, Debug, Default, PartialEq)]
struct NavigatorController(NavigatorState);

impl Reducible for NavigatorController {
    type Action = NavigatorAction;

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

#[function_component(ListeningNavigatorPage)]
pub fn navigator_page() -> Html {
    let (vocabulary, sets) = use_game_vocabulary();
    let rounds = match &vocabulary {
        GameVocabulary::Starter => Some((0..NAVIGATION_ROUTE.len()).collect::<Vec<_>>()),
        GameVocabulary::Ready(data) => data
            .result
            .as_ref()
            .ok()
            .map(|words| navigator_rounds(words.iter().map(|word| word.simplified.as_str()))),
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
                    <GameSetup title="Listening Navigator" description="Listen to Mandarin directions and find your way through the city." path="/games/listening-navigator" sets={sets} source={source_label} available={0} on_start={on_start} />
                } else if flow.phase == GamePhase::Setup {
                    <GameSetup title="Listening Navigator" description="Listen to Mandarin directions and find your way through the city." path="/games/listening-navigator" sets={sets} source={source_label} available={rounds.len()} on_start={on_start} />
                } else {
                    <button type="button" data-game-change-setup="" class="self-start rounded-lg border border-line px-4 py-2 text-sm hover:border-accent" onclick={on_change_setup}>{"Change setup"}</button>
                    <GameSession key={session_key} rounds={rounds} source={source_label} />
                }
            } else {
                <GameSetup title="Listening Navigator" description="Listen to Mandarin directions and find your way through the city." path="/games/listening-navigator" sets={sets} source={source_label} available={0} on_start={on_start} />
                <VocabularyFallback state={vocabulary} />
            }
        </div>
    }
}

#[function_component(GameSession)]
fn game_session(props: &GameSessionProps) -> Html {
    let initial_rounds = props.rounds.clone();
    let state =
        use_reducer(move || NavigatorController(NavigatorState::from_rounds(initial_rounds)));
    let audio_error = use_state(|| None::<String>);
    let environment = use_app_environment();
    let on_move = {
        let state = state.clone();
        Callback::from(move |direction: Direction| state.dispatch(NavigatorAction::Move(direction)))
    };
    let on_restart = {
        let state = state.clone();
        let audio_error = audio_error.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch(NavigatorAction::Restart);
            audio_error.set(None);
        })
    };
    let on_listen = {
        let speech = environment.speech.clone();
        let audio_error = audio_error.clone();
        let current = state.0.current();
        Callback::from(move |_: MouseEvent| {
            if let Some(direction) = current {
                audio_error.set(
                    crate::services::practice_audio::speak(&speech, direction.phrase())
                        .err()
                        .map(|_| {
                            "Mandarin speech playback is unavailable in this browser.".to_owned()
                        }),
                );
            }
        })
    };
    let game = &state.0;
    html! {
        <GameShell source={props.source.clone()} title="Listening Navigator" step={game.step} total={game.total()} mistakes={game.mistakes} notice={game.notice}>
            if game.complete() {
                <GameSummary rounds={game.total()} mistakes={game.mistakes} on_restart={on_restart} />
            } else {
                <NavigatorRound game={game.clone()} on_move={on_move} on_listen={on_listen} audio_error={(*audio_error).clone()} />
            }
        </GameShell>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigator_controller_uses_playable_filtered_decks() {
        let controller = Rc::new(NavigatorController(NavigatorState::from_rounds(vec![
            2, 3, 4,
        ])));
        assert_eq!(controller.0.rounds, vec![2, 4]);
        let next = controller.reduce(NavigatorAction::Move(Direction::Right));
        assert_eq!(next.0.position, (2, 1));
    }

    #[test]
    fn navigator_controller_restarts_its_own_position() {
        let state = Rc::new(NavigatorController::default());
        let moved = state.reduce(NavigatorAction::Move(Direction::Left));
        assert_eq!(moved.0.position, (0, 1));
        let reset = moved.reduce(NavigatorAction::Restart);
        assert_eq!(reset.0, NavigatorState::default());
    }
}
