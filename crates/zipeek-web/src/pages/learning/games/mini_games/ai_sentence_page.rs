use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use zipeek_exercises::mini_games::generated_sentence::{
    GeneratedSentence, GeneratedSentenceAction, GeneratedSentenceState,
};

use super::{
    ai_content,
    setup::GameSetup,
    vocabulary::{GameVocabulary, VocabularyFallback, use_game_vocabulary},
};

#[derive(Clone, Debug, PartialEq)]
struct GameController(GeneratedSentenceState);

impl Reducible for GameController {
    type Action = GeneratedSentenceAction;
    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut game = self.0.clone();
        game.apply(action);
        std::rc::Rc::new(Self(game))
    }
}

#[derive(Clone, Debug, PartialEq)]
enum SentenceView {
    Setup,
    Generating,
    Failed(String),
    Playing(GeneratedSentenceState),
}

#[derive(Properties, PartialEq)]
struct SentencePlayProps {
    initial: GeneratedSentenceState,
    on_setup: Callback<MouseEvent>,
}

#[function_component(SentencePlay)]
fn sentence_play(props: &SentencePlayProps) -> Html {
    let initial = props.initial.clone();
    let state = use_reducer(move || GameController(initial));
    let on_action = {
        let state = state.clone();
        Callback::from(move |action: GeneratedSentenceAction| state.dispatch(action))
    };
    let on_undo = on_action.reform(|_: MouseEvent| GeneratedSentenceAction::Undo);
    let on_clear = on_action.reform(|_: MouseEvent| GeneratedSentenceAction::Clear);
    let on_check = on_action.reform(|_: MouseEvent| GeneratedSentenceAction::Check);
    let on_restart = on_action.reform(|_: MouseEvent| GeneratedSentenceAction::Restart);
    html! {
        <section data-ai-sentence-game="" class="mx-auto flex w-full max-w-3xl flex-col gap-5 rounded-2xl border border-line bg-surface p-5 sm:p-8">
            <header class="flex items-center justify-between gap-3">
                <h1 class="text-2xl font-bold">{"Sentence Relay"}</h1>
                <button type="button" class="rounded-lg border border-line px-3 py-2" onclick={props.on_setup.clone()}>{"Change setup"}</button>
            </header>
            <p role="status" data-game-progress="" class="text-sm text-muted">{format!("{} / {} sentences | {} mistakes", state.0.step, state.0.rounds.len(), state.0.mistakes)}</p>
            <progress class="w-full accent-blue-500" value={state.0.step.to_string()} max={state.0.rounds.len().to_string()} />
            if state.0.complete() {
                <div data-game-complete="" class="space-y-4 text-center">
                    <h2 class="text-xl font-bold">{"Session complete"}</h2>
                    <button type="button" class="rounded-lg bg-brand-action px-5 py-3 text-white" onclick={on_restart}>{"Play again"}</button>
                </div>
            } else if let Some(question) = state.0.current() {
                <p class="text-sm text-muted">{"Arrange the Chinese words to match:"}</p>
                <h2 class="text-xl font-semibold">{question.english.clone()}</h2>
                <div class="flex min-h-16 flex-wrap gap-2 rounded-lg border border-line bg-raised p-3" aria-label="Your sentence">
                    {for state.0.selected.iter().map(|&index| {
                        let on_remove = on_action.reform(move |_: MouseEvent| GeneratedSentenceAction::Remove(index));
                        html! { <button type="button" key={index} onclick={on_remove} class="rounded-lg bg-brand-action px-4 py-2 text-white">{question.tokens[index].clone()}</button> }
                    })}
                </div>
                <div class="flex flex-wrap gap-2" aria-label="Available words">
                    {for question.tokens.iter().enumerate().map(|(index, token)| {
                        let onclick = on_action.reform(move |_: MouseEvent| GeneratedSentenceAction::Select(index));
                        html! { <button type="button" key={index} data-sentence-tile={index.to_string()} disabled={state.0.selected.contains(&index)} onclick={onclick} class="min-h-12 rounded-lg border border-line bg-raised px-4 text-lg disabled:opacity-30">{token.clone()}</button> }
                    })}
                </div>
                if state.0.rejected { <p role="status" class="text-sm text-amber-400">{"Not quite. Edit your sentence and try again."}</p> }
                <div class="flex flex-wrap gap-2">
                    <button type="button" class="rounded-lg border border-line px-4 py-3" onclick={on_undo}>{"Undo"}</button>
                    <button type="button" class="rounded-lg border border-line px-4 py-3" onclick={on_clear}>{"Clear"}</button>
                    <button type="button" data-sentence-check="" class="rounded-lg bg-brand-action px-4 py-3 text-white disabled:opacity-40" disabled={state.0.selected.len() != question.tokens.len() || state.0.rejected} onclick={on_check}>{"Check sentence"}</button>
                </div>
            }
        </section>
    }
}

#[function_component(SentenceRelayPage)]
pub fn sentence_page() -> Html {
    let (vocabulary, sets) = use_game_vocabulary();
    let view = use_state(|| SentenceView::Setup);
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
            view.set(SentenceView::Setup);
        });
    }
    let on_start = {
        let view = view.clone();
        let request = request.clone();
        let source = source.clone();
        let words = words.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(words) = words.clone() else { return };
            *request.borrow_mut() += 1;
            let ticket = *request.borrow();
            view.set(SentenceView::Generating);
            let view = view.clone();
            let request = request.clone();
            let source = source.clone();
            spawn_local(async move {
                let result = ai_content::generate_sentences(&source, &words)
                    .await
                    .and_then(|rounds: Vec<GeneratedSentence>| {
                        GeneratedSentenceState::new(rounds).map_err(str::to_owned)
                    });
                if *request.borrow() == ticket {
                    view.set(match result {
                        Ok(game) => SentenceView::Playing(game),
                        Err(error) => SentenceView::Failed(error),
                    });
                }
            });
        })
    };
    let on_setup = {
        let view = view.clone();
        let request = request.clone();
        Callback::from(move |_: MouseEvent| {
            *request.borrow_mut() += 1;
            view.set(SentenceView::Setup);
        })
    };
    html! {
        <div class="mx-auto w-full max-w-3xl px-4 py-8">
            if let SentenceView::Playing(game) = &*view {
                <SentencePlay key={source.clone()} initial={game.clone()} on_setup={on_setup} />
            } else {
                <GameSetup title="Sentence Relay" description="Generate new Chinese sentences with your configured AI model. Arrange each sentence in the correct order." path="/games/sentence-relay" sets={sets} source={source} available={if words.is_some() && !matches!(&*view, SentenceView::Generating) {4} else {0}} on_start={on_start} />
                if let SentenceView::Generating = &*view { <p role="status" class="mt-4 text-center">{"Generating four Mandarin sentences..."}</p> }
                if let SentenceView::Failed(error) = &*view { <p role="alert" class="mt-4 rounded-lg border border-red-500 p-4">{error.clone()}</p> }
                <VocabularyFallback state={vocabulary} />
            }
        </div>
    }
}
