use yew::prelude::*;
use zipeek_core::sentence_tiles::is_complete;

use crate::components::{SentenceWordBank, ThemedBackground, TranslationFeedback};

use super::{actions::Callbacks, settings, state::{Stage, TranslationState}};

pub(super) fn render(state: &TranslationState, actions: Callbacks) -> Html {
    html! {
        <section data-component="sentence-translation-page" data-sentence-translation=""
            data-translation-mode={if state.round_limit.is_some() { "fixed" } else { "endless" }}
            data-translation-question={(*state.exercise_index.borrow()).to_string()}
            class="relative isolate mx-auto flex max-w-3xl flex-col gap-6 rounded-2xl p-4">
            <ThemedBackground categories={vec![(*state.set_id).clone()]} />
            <header class="flex flex-col gap-2">
                <h1 class="text-3xl font-bold tracking-tight">{"Sentence translation"}</h1>
                <p class="text-sm text-muted">{"Experimental two-way Mandarin/English translation using a local LM Studio model. Model output may be incorrect."}</p>
            </header>
            {settings::render(state)}
            <div data-component="translation-session-stats" class="flex flex-wrap gap-4 rounded-lg border border-line bg-surface p-3 text-sm" role="status">
                <span>{format!("Completed: {}", state.quiz_session.session().entries.len())}</span>
                <span>{format!("Correct: {}", state.quiz_session.session().totals().earned)}</span>
                <span>{format!("Mistakes: {}", state.quiz_session.session().totals().mistakes)}</span>
            </div>
            <p class="text-sm text-muted">{"Sentence generation uses words from the selected source. HSK and saved sets are read from the installed local dictionary."}</p>
            <p data-translation-progress="" class="text-sm font-semibold">{
                if let Some(limit) = state.round_limit {
                    format!("Translation exercises: {} / {limit}", *state.completed)
                } else {
                    format!("Translation exercises: {} completed", *state.completed)
                }
            }</p>
            {match &*state.stage {
                Stage::Complete => html! {
                    <div data-translation-complete="" role="status" class="rounded-xl border border-line bg-surface p-6">
                        <h2 class="text-xl font-semibold">{"Translation session complete"}</h2>
                        <p>{format!("Completed {} translation exercises.", *state.completed)}</p>
                        <a class="mt-4 inline-flex min-h-11 items-center rounded-lg bg-blue-600 px-5 text-white" href="#/practice">{"Back to setup"}</a>
                    </div>
                },
                Stage::Ready => html! {
                    <button type="button" data-translation-start="" class="min-h-12 rounded-xl bg-blue-600 px-5 py-3 font-semibold text-white" onclick={actions.generate.clone()}>{"Generate sentence"}</button>
                },
                Stage::Generating => html! { <p role="status">{"Generating sentence…"}</p> },
                Stage::Answering(sentence) | Stage::Evaluating(sentence) | Stage::Feedback(sentence, _) => html! {
                    <div class="flex flex-col gap-4">
                        <div class="rounded-xl border border-line p-5">
                            {if sentence.direction == "zh-to-en" {
                                html! { <p class="text-2xl font-semibold" lang="zh-Hans">{&sentence.chinese}</p> }
                            } else {
                                html! { <p class="text-2xl font-semibold" lang="en">{&sentence.english}</p> }
                            }}
                            <p data-translation-target="" class="mt-2 text-sm text-muted">{
                                if sentence.direction == "zh-to-en" { "Translate into English." } else { "Translate into Chinese." }
                            }</p>
                        </div>
                        <SentenceWordBank
                            words={sentence.word_bank.clone()}
                            target_language={if sentence.direction == "en-to-zh" { "zh" } else { "en" }}
                            order={(*state.bank_order).clone()}
                            selected={(*state.selected_tiles).clone()}
                            disabled={!matches!(&*state.stage, Stage::Answering(_))}
                            on_add={actions.add_tile.clone()}
                            on_remove={actions.remove_tile.clone()}
                            on_reorder={actions.reorder_tile.clone()}
                        />
                        {match &*state.stage {
                            Stage::Answering(_) => html! {
                                <button type="button" data-translation-submit="" disabled={!is_complete(&state.selected_tiles, sentence.word_bank.len())} class="min-h-12 rounded-xl bg-blue-600 px-5 py-3 font-semibold text-white disabled:opacity-50" onclick={actions.evaluate.clone()}>{"Check translation"}</button>
                            },
                            Stage::Evaluating(_) => html! { <p role="status">{"Evaluating translation…"}</p> },
                            Stage::Feedback(_, feedback) => html! {
                                <TranslationFeedback
                                    evaluation={feedback.clone()}
                                    reference={if sentence.direction == "zh-to-en" { sentence.english.clone() } else { sentence.chinese.clone() }}
                                    on_continue={actions.generate.clone()}
                                />
                            },
                            _ => Html::default(),
                        }}
                    </div>
                },
                Stage::Failed(message, _) => html! {
                    <div role="alert" class="flex flex-col gap-3 rounded-xl border border-line p-4">
                        <p>{message}</p>
                        <button type="button" class="min-h-11 rounded-lg border border-line px-4" onclick={actions.reset.clone()}>{"Retry"}</button>
                    </div>
                },
            }}
        </section>
    }
}
