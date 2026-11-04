use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;
use zipeek_ai::{Evaluation, Sentence, STARTER_SETS, evaluation_messages, generation_messages, parse_evaluation, parse_sentence, starter_set};

use crate::services::ai;

#[derive(Clone, PartialEq)]
enum Stage {
    Ready,
    Generating,
    Answering(Sentence),
    Evaluating(Sentence),
    Feedback(Sentence, Evaluation),
    Failed(String, Option<Sentence>),
}

#[function_component(SentenceTranslation)]
pub fn sentence_translation() -> Html {
    let set_id = use_state(|| "travel".to_owned());
    let endpoint = use_state(|| "http://127.0.0.1:1234/v1/".to_owned());
    let model = use_state(String::new);
    let answer = use_state(String::new);
    let stage = use_state(|| Stage::Ready);

    let on_set = {
        let set_id = set_id.clone();
        let stage = stage.clone();
        Callback::from(move |event: Event| {
            set_id.set(event.target_unchecked_into::<HtmlSelectElement>().value());
            stage.set(Stage::Ready);
        })
    };
    let on_endpoint = {
        let endpoint = endpoint.clone();
        Callback::from(move |event: InputEvent| endpoint.set(event.target_unchecked_into::<HtmlInputElement>().value()))
    };
    let on_model = {
        let model = model.clone();
        Callback::from(move |event: InputEvent| model.set(event.target_unchecked_into::<HtmlInputElement>().value()))
    };
    let on_answer = {
        let answer = answer.clone();
        Callback::from(move |event: InputEvent| answer.set(event.target_unchecked_into::<HtmlTextAreaElement>().value()))
    };
    let generate = {
        let set_id = set_id.clone();
        let endpoint = endpoint.clone();
        let model = model.clone();
        let stage = stage.clone();
        let answer = answer.clone();
        Callback::from(move |_| {
            let Some(set) = starter_set(&set_id) else {
                stage.set(Stage::Failed("Choose a valid Word Set".to_owned(), None));
                return;
            };
            if model.trim().is_empty() {
                stage.set(Stage::Failed("Enter a model identifier from LM Studio".to_owned(), None));
                return;
            }
            let endpoint = (*endpoint).clone();
            let model = (*model).clone();
            let stage = stage.clone();
            answer.set(String::new());
            stage.set(Stage::Generating);
            spawn_local(async move {
                let result = match ai::complete(&endpoint, &model, generation_messages(set)).await {
                    Ok(raw) => parse_sentence(&raw, set).map(Stage::Answering),
                    Err(error) => Err(error),
                };
                stage.set(result.unwrap_or_else(|error| Stage::Failed(error, None)));
            });
        })
    };
    let evaluate = {
        let stage = stage.clone();
        let answer = answer.clone();
        let endpoint = endpoint.clone();
        let model = model.clone();
        Callback::from(move |_| {
            let Stage::Answering(sentence) = &*stage else { return };
            if answer.trim().is_empty() { return }
            let sentence = sentence.clone();
            let submitted_answer = (*answer).clone();
            let endpoint = (*endpoint).clone();
            let model = (*model).clone();
            let stage = stage.clone();
            stage.set(Stage::Evaluating(sentence.clone()));
            spawn_local(async move {
                let sentence_for_retry = sentence.clone();
                let result = match ai::complete(&endpoint, &model, evaluation_messages(&sentence.chinese, &submitted_answer)).await {
                    Ok(raw) => parse_evaluation(&raw).map(|evaluation| Stage::Feedback(sentence, evaluation)),
                    Err(error) => Err(error),
                };
                stage.set(result.unwrap_or_else(|error| Stage::Failed(error, Some(sentence_for_retry))));
            });
        })
    };
    let reset = {
        let stage = stage.clone();
        Callback::from(move |_| {
            if let Stage::Failed(_, Some(sentence)) = &*stage {
                stage.set(Stage::Answering(sentence.clone()));
            } else {
                stage.set(Stage::Ready);
            }
        })
    };
    let selected = starter_set(&set_id);
    html! {
        <section data-sentence-translation="" class="mx-auto flex max-w-3xl flex-col gap-6">
            <header class="flex flex-col gap-2">
                <h1 class="text-3xl font-bold tracking-tight">{"Sentence translation"}</h1>
                <p class="text-sm text-muted">{"Experimental Chinese-to-English practice using a local LM Studio model. Model output may be incorrect."}</p>
            </header>
            <div class="grid gap-4 sm:grid-cols-2">
                <label class="flex flex-col gap-2 text-sm font-semibold">{"Word Set"}
                    <select data-word-set="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*set_id).clone()} onchange={on_set}>
                        {for STARTER_SETS.iter().map(|set| html! { <option value={set.id}>{set.name}</option> })}
                    </select>
                </label>
                <label class="flex flex-col gap-2 text-sm font-semibold">{"LM Studio model ID"}
                    <input data-ai-model="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*model).clone()} oninput={on_model} placeholder="Loaded model identifier" />
                </label>
                <label class="flex flex-col gap-2 text-sm font-semibold sm:col-span-2">{"Local API base URL"}
                    <input data-ai-endpoint="" class="min-h-11 rounded-lg border border-line bg-surface px-3" value={(*endpoint).clone()} oninput={on_endpoint} />
                </label>
            </div>
            {if let Some(set) = selected {html! {
                <p class="text-sm text-muted">{format!("Vocabulary: {}", set.vocabulary.join(" · "))}</p>
            }} else {Html::default()}}
            {match &*stage {
                Stage::Ready => html!{<button type="button" data-translation-start="" class="min-h-12 rounded-xl bg-blue-600 px-5 py-3 font-semibold text-white" onclick={generate.clone()}>{"Generate sentence"}</button>},
                Stage::Generating => html!{<p role="status">{"Generating sentence…"}</p>},
                Stage::Answering(sentence) | Stage::Evaluating(sentence) | Stage::Feedback(sentence, _) => html!{
                    <div class="flex flex-col gap-4">
                        <div class="rounded-xl border border-line p-5">
                            <p class="text-2xl font-semibold" lang="zh-Hans">{&sentence.chinese}</p>
                            <p class="mt-2 text-sm text-muted">{"Translate this sentence into English."}</p>
                        </div>
                        <label class="flex flex-col gap-2 text-sm font-semibold">{"Your translation"}
                            <textarea data-translation-answer="" class="min-h-28 rounded-lg border border-line bg-surface p-3" value={(*answer).clone()} oninput={on_answer} disabled={!matches!(&*stage,Stage::Answering(_))} />
                        </label>
                        {match &*stage {
                            Stage::Answering(_) => html!{<button type="button" data-translation-submit="" disabled={answer.trim().is_empty()} class="min-h-12 rounded-xl bg-blue-600 px-5 py-3 font-semibold text-white disabled:opacity-50" onclick={evaluate}>{"Check translation"}</button>},
                            Stage::Evaluating(_) => html!{<p role="status">{"Evaluating translation…"}</p>},
                            Stage::Feedback(_, feedback) => html!{<div data-translation-feedback="" class="flex flex-col gap-3 rounded-xl border border-line p-4" role="status">
                                <p class="font-semibold">{if feedback.accurate {"Meaning preserved"} else {"Review translation"}}</p>
                                <p>{&feedback.feedback}</p>
                                <p>{format!("Suggested translation: {}", feedback.suggested_translation)}</p>
                                <button type="button" class="min-h-11 rounded-lg border border-line px-4" onclick={generate.clone()}>{"Next sentence"}</button>
                            </div>},
                            _ => Html::default(),
                        }}
                    </div>
                },
                Stage::Failed(message, _) => html!{<div role="alert" class="flex flex-col gap-3 rounded-xl border border-line p-4">
                    <p>{message}</p>
                    <button type="button" class="min-h-11 rounded-lg border border-line px-4" onclick={reset}>{"Retry"}</button>
                </div>},
            }}
        </section>
    }
}
