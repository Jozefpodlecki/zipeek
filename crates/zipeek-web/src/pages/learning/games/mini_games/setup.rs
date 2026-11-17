use yew::prelude::*;

use super::vocabulary_picker::VocabularyPicker;
use crate::services::word_sets::UserWordSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamePhase {
    Setup,
    Playing,
    Results,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameFlowAction {
    Start,
    Finish,
    PlayAgain,
    ChangeSetup,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameFlow {
    pub phase: GamePhase,
}

impl Default for GameFlow {
    fn default() -> Self {
        Self {
            phase: GamePhase::Setup,
        }
    }
}

impl Reducible for GameFlow {
    type Action = GameFlowAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        use GameFlowAction::*;
        let next = match (self.phase, action) {
            (GamePhase::Setup, Start) | (GamePhase::Results, PlayAgain) => GamePhase::Playing,
            (GamePhase::Playing, Finish) => GamePhase::Results,
            (_, ChangeSetup) => GamePhase::Setup,
            (phase, _) => phase,
        };
        std::rc::Rc::new(Self { phase: next })
    }
}

#[derive(Properties, PartialEq)]
pub struct GameSetupProps {
    pub title: &'static str,
    pub description: &'static str,
    pub path: &'static str,
    pub sets: Vec<UserWordSet>,
    pub source: String,
    pub available: usize,
    pub on_start: Callback<MouseEvent>,
}

#[function_component(GameSetup)]
pub fn game_setup(props: &GameSetupProps) -> Html {
    html! {
        <section data-game-setup={props.title} class="mx-auto flex w-full max-w-3xl flex-col gap-6 rounded-2xl border border-line bg-surface p-5 sm:p-8">
            <header class="space-y-2">
                <p class="text-sm font-semibold text-accent">{"Ready to play?"}</p>
                <h1 class="text-3xl font-bold">{props.title}</h1>
                <p class="leading-relaxed text-muted">{props.description}</p>
            </header>
            <div class="space-y-3">
                <h2 class="text-lg font-semibold">{"Choose vocabulary"}</h2>
                <VocabularyPicker path={props.path} sets={props.sets.clone()} />
                <p data-game-source="" class="text-sm text-muted">{props.source.clone()}</p>
            </div>
            <div class="flex flex-wrap items-center justify-between gap-4 rounded-xl bg-raised p-4">
                <span data-game-available="" class="text-sm text-muted">{format!("{} available rounds", props.available)}</span>
                <button type="button" data-game-start="" class="min-h-12 rounded-xl bg-brand-action px-6 font-semibold text-white disabled:cursor-not-allowed disabled:opacity-40" disabled={props.available == 0} onclick={props.on_start.clone()}>{"Start game"}</button>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn flow_requires_explicit_start_and_preserves_state_boundaries() {
        let setup = Rc::new(GameFlow::default());
        assert_eq!(
            setup.clone().reduce(GameFlowAction::Finish).phase,
            GamePhase::Setup
        );
        let playing = setup.reduce(GameFlowAction::Start);
        assert_eq!(playing.phase, GamePhase::Playing);
        assert_eq!(
            playing.clone().reduce(GameFlowAction::Start).phase,
            GamePhase::Playing
        );
        let results = playing.reduce(GameFlowAction::Finish);
        assert_eq!(results.phase, GamePhase::Results);
        assert_eq!(
            results.clone().reduce(GameFlowAction::PlayAgain).phase,
            GamePhase::Playing
        );
        assert_eq!(
            results.reduce(GameFlowAction::ChangeSetup).phase,
            GamePhase::Setup
        );
    }
}

#[derive(Properties, PartialEq)]
pub struct SimpleGameSetupProps {
    pub title: &'static str,
    pub description: &'static str,
    pub rounds: usize,
    pub on_start: Callback<MouseEvent>,
}

#[function_component(SimpleGameSetup)]
pub fn simple_game_setup(props: &SimpleGameSetupProps) -> Html {
    html! {
        <section data-game-setup={props.title} class="mx-auto flex w-full max-w-3xl flex-col gap-5 rounded-2xl border border-line bg-surface p-6 sm:p-8">
            <p class="text-sm font-semibold text-accent">{"Learn through play"}</p>
            <h1 class="text-3xl font-bold">{props.title}</h1>
            <p class="leading-relaxed text-muted">{props.description}</p>
            <div class="flex flex-wrap items-center justify-between gap-3 rounded-xl bg-raised p-4">
                <span class="text-sm text-muted">{format!("{} authored rounds", props.rounds)}</span>
                <button type="button" data-game-start="" class="min-h-12 rounded-xl bg-brand-action px-6 font-semibold text-white" onclick={props.on_start.clone()}>{"Start game"}</button>
            </div>
        </section>
    }
}
