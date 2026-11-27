use std::{cell::Cell, collections::HashMap, rc::Rc};

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_core::Word;
use zipeek_exercises::listening_choice::{ListeningQuestion, ListeningWord, prepare};

use crate::{
    hooks::{use_app_environment, use_dictionary},
    models::ListeningChoiceOptions,
    services::{
        practice_audio,
        practice_session_store::{ExerciseAssessment, ExerciseVerdict},
    },
};

use super::{
    feedback::{ExerciseReview, FeedbackVerdict},
    instruction::InstructionIcon,
    surface::ExerciseSurface,
    vocabulary_exercises::load_words,
};

#[derive(Clone, PartialEq)]
struct ListeningPrompt {
    question: ListeningQuestion,
    words: Vec<Word>,
}

impl ListeningPrompt {
    fn word(&self, id: i32) -> Option<&Word> {
        self.words.iter().find(|word| word.id.as_i32() == id)
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct ResumableListeningChoiceProps {
    pub vocabulary: HashMap<String, String>,
    pub seed: u64,
    pub options: ListeningChoiceOptions,
    pub on_completed: Callback<()>,
    #[prop_or_default]
    pub on_assessed: Option<Callback<ExerciseAssessment>>,
}

#[function_component(ResumableListeningChoice)]
pub(crate) fn resumable_listening_choice(props: &ResumableListeningChoiceProps) -> Html {
    let dictionary = use_dictionary();
    let environment = use_app_environment();
    let prepared = use_state(|| None::<Result<ListeningPrompt, String>>);
    let selected = use_state(|| None::<i32>);
    let audio_error = use_state(String::new);
    {
        let speech = environment.speech.clone();
        use_effect_with((props.vocabulary.clone(), props.seed), move |_| {
            let _ = speech.cancel();
            move || {
                let _ = speech.cancel();
            }
        });
    }
    {
        let dictionary = dictionary.clone();
        let prepared = prepared.clone();
        let selected = selected.clone();
        let audio_error = audio_error.clone();
        use_effect_with(
            (props.vocabulary.clone(), props.seed),
            move |(vocabulary, seed)| {
                let alive = Rc::new(Cell::new(true));
                let pending = alive.clone();
                let vocabulary = vocabulary.clone();
                let seed = *seed;
                prepared.set(None);
                selected.set(None);
                audio_error.set(String::new());
                spawn_local(async move {
                    let result = load_words(&dictionary, &vocabulary, false)
                    .await
                    .and_then(|words| {
                        let candidates = words
                            .iter()
                            .map(|word| ListeningWord {
                                id: word.id.as_i32(),
                                hanzi: word.simplified.clone(),
                            })
                            .collect::<Vec<_>>();
                        prepare(&candidates, seed)
                            .map(|question| ListeningPrompt { question, words })
                            .map_err(|error| format!("Cannot prepare listening choices: {error:?}. Select at least three distinct vocabulary words."))
                    });
                    if pending.get() {
                        prepared.set(Some(result));
                    }
                });
                move || alive.set(false)
            },
        );
    }
    {
        let speech = environment.speech.clone();
        let audio_error = audio_error.clone();
        use_effect_with(
            ((*prepared).clone(), props.options.autoplay),
            move |(prompt, autoplay)| {
                if *autoplay {
                    if let Some(Ok(prompt)) = prompt {
                        if let Err(error) =
                            practice_audio::speak(&speech, &prompt.question.spoken_hanzi)
                        {
                            audio_error.set(format!("Automatic Mandarin playback unavailable: {error:?}. Use the speaker to try again."));
                        }
                    }
                }
                || ()
            },
        );
    }

    let play = {
        let prepared = prepared.clone();
        let audio_error = audio_error.clone();
        let speech = environment.speech.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(Ok(prompt)) = &*prepared {
                match practice_audio::speak(&speech, &prompt.question.spoken_hanzi) {
                    Ok(()) => audio_error.set(String::new()),
                    Err(error) => audio_error.set(format!("Mandarin audio unavailable: {error:?}")),
                }
            }
        })
    };
    let body = match &*prepared {
        None => html! { <p role="status">{"Loading listening vocabulary..."}</p> },
        Some(Err(error)) => html! { <p role="alert">{error}</p> },
        Some(Ok(prompt)) => {
            let question = &prompt.question;
            let answer = (*selected).and_then(|id| question.grade(id).ok());
            let answered = answer.is_some();
            let correct_word = prompt.word(question.correct_id);
            let continue_session = {
                let on_completed = props.on_completed.clone();
                let on_assessed = props.on_assessed.clone();
                Callback::from(move |_: MouseEvent| {
                    if let (Some(correct), Some(on_assessed)) = (answer, &on_assessed) {
                        on_assessed.emit(ExerciseAssessment {
                            verdict: if correct {
                                ExerciseVerdict::Correct
                            } else {
                                ExerciseVerdict::Incorrect
                            },
                            attempts: 1,
                            mistakes: u16::from(!correct),
                            hints: 0,
                            assisted: false,
                            item_evidence: Vec::new(),
                        });
                    } else {
                        on_completed.emit(());
                    }
                })
            };
            html! {
                <div class="space-y-4">
                    <div class="flex flex-col items-center gap-3 rounded-xl border border-line bg-raised px-4 py-6 text-center">
                        <button type="button" class="flex h-16 w-16 items-center justify-center rounded-full bg-brand-action text-white shadow-md transition-colors hover:brightness-110 focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-accent"
                            aria-label="Play Mandarin pronunciation" title="Play Mandarin pronunciation" onclick={play} data-listening-play="">
                            <Icon data={IconData::LUCIDE_VOLUME_2} width="30" height="30" />
                        </button>
                        <p class="font-semibold">{"Listen to the word"}</p>
                        <p class="text-xs text-muted">{"Select the matching Chinese word. Use the speaker to listen again."}</p>
                    </div>
                    <div role="group" aria-label="Choose the word you heard" class="grid gap-3 sm:grid-cols-3">
                        {for question.choices.iter().map(|choice| {
                            let id = choice.id;
                            let is_selected = *selected == Some(id);
                            let is_correct = answered && id == question.correct_id;
                            let word = prompt.word(id);
                            let selected_for_click = selected.clone();
                            let on_choose = Callback::from(move |_: MouseEvent| {
                                if (*selected_for_click).is_none() {
                                    selected_for_click.set(Some(id));
                                }
                            });
                            let border = if is_correct { "border-green-600" } else if is_selected && answered { "border-amber-500" } else if is_selected { "border-brand-action" } else { "border-line" };
                            html! {
                                <button key={id} type="button" data-listening-choice-id={id.to_string()}
                                    class={classes!("flex", "min-h-24", "flex-col", "items-center", "justify-center", "gap-1", "rounded-xl", "border-2", "bg-raised", "p-3", "text-center", "transition-colors", "focus-visible:outline-2", "focus-visible:outline-offset-2", "focus-visible:outline-accent", border)}
                                    aria-pressed={is_selected.to_string()} disabled={answered} onclick={on_choose}>
                                    <span class="text-2xl font-semibold">{&choice.hanzi}</span>
                                    {if props.options.pinyin.visible(answered) {
                                        html! { <span class="text-sm text-accent">{word.map(|word| word.pinyin.as_str()).unwrap_or("")}</span> }
                                    } else { Html::default() }}
                                    {if props.options.meaning.visible(answered) {
                                        html! { <span class="text-xs text-muted">{word.and_then(|word| word.english.as_deref()).and_then(|gloss| gloss.split('|').next()).unwrap_or("")}</span> }
                                    } else { Html::default() }}
                                </button>
                            }
                        })}
                    </div>
                    <div class="min-h-28" data-listening-feedback="">
                        {if let Some(correct) = answer {
                            html! {
                                <div class="space-y-3">
                                    <ExerciseReview verdict={if correct { FeedbackVerdict::Correct } else { FeedbackVerdict::Incorrect }} on_continue={continue_session.clone()} listening_choice=true>
                                        <p class="text-sm font-medium">
                                            {format!("The spoken word was {}.", question.spoken_hanzi)}
                                        </p>
                                        {if props.options.pinyin.visible(true) || props.options.meaning.visible(true) {
                                            html! { <p class="text-sm text-muted">
                                                {if props.options.pinyin.visible(true) { correct_word.map(|word| word.pinyin.as_str()).unwrap_or("") } else { "" }}
                                                {if props.options.pinyin.visible(true) && props.options.meaning.visible(true) { " · " } else { "" }}
                                                {if props.options.meaning.visible(true) { correct_word.and_then(|word| word.english.as_deref()).and_then(|gloss| gloss.split('|').next()).unwrap_or("") } else { "" }}
                                            </p> }
                                        } else { Html::default() }}
                                    </ExerciseReview>
                                </div>
                            }
                        } else { Html::default() }}
                    </div>
                </div>
            }
        }
    };
    html! {
        <div data-listening-session="" class="w-full">
            <ExerciseSurface icon={InstructionIcon::Listen} instruction="Listen and choose the matching Chinese word.">
            {body}
            {if audio_error.is_empty() { Html::default() } else { html! { <p role="alert">{&*audio_error}</p> } }}
            </ExerciseSurface>
        </div>
    }
}
