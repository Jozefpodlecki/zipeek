use super::setup::SimpleGameSetup;
use yew::prelude::*;
use zipeek_exercises::mini_games::radical::{QUESTIONS, RadicalAction, RadicalState};

#[derive(Clone, Debug, PartialEq)]
struct GameController(RadicalState);

impl Reducible for GameController {
    type Action = RadicalAction;
    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut game = self.0.clone();
        game.apply(action);
        std::rc::Rc::new(Self(game))
    }
}

#[function_component(RadicalForgePage)]
pub fn radical_page() -> Html {
    let playing = use_state(|| false);
    let state = use_reducer(|| GameController(RadicalState::default()));
    let on_start = {
        let playing = playing.clone();
        Callback::from(move |_: MouseEvent| playing.set(true))
    };
    let on_setup = {
        let state = state.clone();
        let playing = playing.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch(RadicalAction::Restart);
            playing.set(false);
        })
    };
    let on_answer = {
        let state = state.clone();
        Callback::from(move |index: usize| state.dispatch(RadicalAction::Answer(index)))
    };
    let on_restart = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(RadicalAction::Restart))
    };
    html! {
        <main class="mx-auto w-full max-w-3xl px-4 py-8">
            if !*playing {
                <SimpleGameSetup title="Radical Forge" description="Identify the real components of Chinese characters. These are authored component decompositions, not stroke-tracing exercises." rounds={QUESTIONS.len()} on_start={on_start} />
            } else {
                <section data-radical-forge="" class="space-y-6 rounded-2xl border border-line bg-surface p-5 sm:p-8">
                    <div class="flex items-center justify-between gap-3">
                        <h1 class="text-2xl font-bold">{"Radical Forge"}</h1>
                        <button type="button" onclick={on_setup} class="rounded-lg border border-line px-4 py-2">{"Change setup"}</button>
                    </div>
                    <p data-game-progress="" class="text-sm text-muted">{format!("{} / {} characters | {} mistakes", state.0.step, QUESTIONS.len(), state.0.mistakes)}</p>
                    <progress class="w-full" value={state.0.step.to_string()} max={QUESTIONS.len().to_string()} />
                    if state.0.complete() {
                        <div data-game-complete="" class="space-y-4 text-center">
                            <h2 class="text-xl font-bold">{"Characters completed"}</h2>
                            <button type="button" onclick={on_restart.clone()} class="rounded-xl bg-brand-action px-5 py-3 text-white">{"Play again"}</button>
                        </div>
                    } else if let Some(question) = state.0.current() {
                        <p class="text-sm text-muted">{"Which components make this character?"}</p>
                        <div class="rounded-xl bg-raised p-6 text-center">
                            <p lang="zh-Hans" class="text-7xl font-semibold">{question.character}</p>
                            <p class="mt-3 text-muted">{question.meaning}</p>
                        </div>
                        <div class="grid gap-3 sm:grid-cols-3">
                            {for question.choices.iter().enumerate().map(|(index, choice)| {
                                let onclick = on_answer.reform(move |_: MouseEvent| index);
                                html! { <button type="button" key={index} data-radical-choice={index.to_string()} disabled={state.0.rejected == Some(index)} onclick={onclick} class="min-h-20 rounded-xl border border-line bg-raised text-2xl hover:border-accent disabled:opacity-40">{*choice}</button> }
                            })}
                        </div>
                        if state.0.rejected.is_some() { <p role="status" class="text-amber-400">{"Try another decomposition."}</p> }
                    }
                </section>
            }
        </main>
    }
}
