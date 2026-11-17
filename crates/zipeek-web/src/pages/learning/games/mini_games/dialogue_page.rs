use super::{
    ai_content,
    setup::GameSetup,
    vocabulary::{GameVocabulary, VocabularyFallback, use_game_vocabulary},
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use zipeek_exercises::mini_games::dialogue::{DialogueAction, DialogueRound, DialogueState};

#[derive(Clone, Debug, PartialEq)]
struct GameController(DialogueState);

impl Reducible for GameController {
    type Action = DialogueAction;
    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut game = self.0.clone();
        game.apply(action);
        std::rc::Rc::new(Self(game))
    }
}

#[derive(Clone, Debug, PartialEq)]
enum DialogueView {
    Setup,
    Generating,
    Failed(String),
    Playing(DialogueState),
}

#[derive(Properties, PartialEq)]
struct DialoguePlayProps {
    initial: DialogueState,
    on_setup: Callback<MouseEvent>,
}

#[function_component(DialoguePlay)]
fn dialogue_play(props: &DialoguePlayProps) -> Html {
    let initial = props.initial.clone();
    let state = use_reducer(move || GameController(initial));
    let on_answer = {
        let state = state.clone();
        Callback::from(move |index: usize| state.dispatch(DialogueAction::Answer(index)))
    };
    let on_restart = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(DialogueAction::Restart))
    };
    html! {
        <section data-dialogue-quest="" class="mx-auto flex w-full max-w-3xl flex-col gap-6 rounded-2xl border border-line bg-surface p-5 sm:p-8">
            <header class="flex items-center justify-between gap-3">
                <h1 class="text-2xl font-bold">{"Dialogue Quest"}</h1>
                <button type="button" onclick={props.on_setup.clone()} class="rounded-lg border border-line px-4 py-2">{"Change setup"}</button>
            </header>
            <p data-game-progress="" class="text-sm text-muted">{format!("{} / {} conversations | {} mistakes", state.0.step, state.0.rounds.len(), state.0.mistakes)}</p>
            <progress class="w-full" value={state.0.step.to_string()} max={state.0.rounds.len().to_string()} />
            if state.0.complete() {
                <div data-game-complete="" class="space-y-3 text-center">
                    <h2 class="text-xl font-bold">{"Conversation complete"}</h2>
                    <button type="button" class="rounded-lg bg-brand-action px-5 py-3 text-white" onclick={on_restart}>{"Play again"}</button>
                </div>
            } else if let Some(round) = state.0.current() {
                <p class="text-sm text-muted">{round.context.clone()}</p>
                <div class="rounded-xl bg-raised p-5">
                    <p class="text-sm font-semibold text-accent">{round.speaker.clone()}</p>
                    <p class="mt-2 text-2xl" lang="zh-Hans">{round.prompt.clone()}</p>
                </div>
                <h2 class="font-semibold">{"Choose your response"}</h2>
                <div class="grid gap-3">
                    {for round.choices.iter().enumerate().map(|(index, choice)| {
                        let onclick = on_answer.reform(move |_: MouseEvent| index);
                        html! { <button type="button" key={index} data-dialogue-choice={index.to_string()} disabled={state.0.rejected == Some(index)} onclick={onclick} class="min-h-14 rounded-xl border border-line bg-raised px-5 py-3 text-left text-lg hover:border-accent disabled:opacity-40">{choice.clone()}</button> }
                    })}
                </div>
                if state.0.rejected.is_some() { <p role="status" class="text-amber-400">{"Try another response."}</p> }
            }
        </section>
    }
}

#[function_component(DialogueQuestPage)]
pub fn dialogue_page() -> Html {
    let (vocabulary, sets) = use_game_vocabulary();
    let view = use_state(|| DialogueView::Setup);
    let request = use_mut_ref(|| 0_u64);
    let source = vocabulary.label();
    let source_key = match &vocabulary {
        GameVocabulary::Ready(data) => format!("{:?}", data.source),
        _ => source.clone(),
    };
    let words = match &vocabulary {
        GameVocabulary::Starter => Some(Vec::new()),
        GameVocabulary::Ready(data) => data.result.as_ref().ok().map(|entries| {
            entries
                .iter()
                .map(|word| word.simplified.clone())
                .collect::<Vec<_>>()
        }),
        _ => None,
    };
    {
        let request = request.clone();
        let view = view.clone();
        use_effect_with(source_key, move |_| {
            *request.borrow_mut() += 1;
            view.set(DialogueView::Setup);
        });
    }
    let on_start = {
        let request = request.clone();
        let view = view.clone();
        let words = words.clone();
        let source = source.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(words) = words.clone() else { return };
            *request.borrow_mut() += 1;
            let ticket = *request.borrow();
            view.set(DialogueView::Generating);
            let view = view.clone();
            let request = request.clone();
            let source = source.clone();
            spawn_local(async move {
                let result = ai_content::generate_dialogues(&source, &words)
                    .await
                    .and_then(|rounds: Vec<DialogueRound>| {
                        DialogueState::new(rounds).map_err(str::to_owned)
                    });
                if *request.borrow() == ticket {
                    view.set(match result {
                        Ok(game) => DialogueView::Playing(game),
                        Err(error) => DialogueView::Failed(error),
                    });
                }
            });
        })
    };
    let on_setup = {
        let request = request.clone();
        let view = view.clone();
        Callback::from(move |_: MouseEvent| {
            *request.borrow_mut() += 1;
            view.set(DialogueView::Setup);
        })
    };
    html! {
        <div class="mx-auto w-full max-w-3xl px-4 py-8">
            if let DialogueView::Playing(game) = &*view {
                <DialoguePlay key={source.clone()} initial={game.clone()} on_setup={on_setup} />
            } else {
                <GameSetup title="Dialogue Quest" description="Generate Mandarin conversations with your configured AI model and choose appropriate responses." path="/games/dialogue-quest" sets={sets} source={source} available={if words.is_some() && !matches!(&*view, DialogueView::Generating) {4} else {0}} on_start={on_start} />
                if let DialogueView::Generating = &*view { <p role="status" class="mt-4 text-center">{"Generating Mandarin conversations..."}</p> }
                if let DialogueView::Failed(error) = &*view { <p role="alert" class="mt-4 rounded-lg border border-red-500 p-4">{error.clone()}</p> }
                <VocabularyFallback state={vocabulary} />
            }
        </div>
    }
}
