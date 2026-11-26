use std::{cell::Cell, collections::HashMap, rc::Rc};

use yew::prelude::*;
use zipeek_core::Word;
use zipeek_hanzi_writing::hanzi_characters;

use crate::{
    components::audio::listen_button::ListenButton, hooks::use_dictionary,
    models::WriteHanziOptions, pages::learning::hanzi_writing::HanziWriting,
};

use super::vocabulary_exercises::load_words;

#[derive(Properties, PartialEq)]
pub struct WriteHanziExerciseProps {
    pub vocabulary: HashMap<String, String>,
    pub seed: u64,
    pub options: WriteHanziOptions,
    pub on_completed: Callback<()>,
    pub on_ready_change: Callback<bool>,
}

#[derive(Default)]
struct CompletionGate {
    last_key: Option<String>,
}

impl CompletionGate {
    fn accept(&mut self, key: &str) -> bool {
        if self.last_key.as_deref() == Some(key) {
            return false;
        }
        self.last_key = Some(key.to_owned());
        true
    }
}

#[function_component(WriteHanziExercise)]
pub fn write_hanzi_exercise(props: &WriteHanziExerciseProps) -> Html {
    let dictionary = use_dictionary();
    let loaded = use_state(|| None::<(u64, HashMap<String, String>, Result<Vec<Word>, String>)>);
    let character_index = use_state(|| 0usize);
    let retry_epoch = use_state(|| 0u64);
    let completion_gate = use_mut_ref(CompletionGate::default);
    {
        let dictionary = dictionary.clone();
        let loaded = loaded.clone();
        let character_index = character_index.clone();
        use_effect_with(
            (props.vocabulary.clone(), props.seed, *retry_epoch),
            move |(query, seed, _)| {
                let active = Rc::new(Cell::new(true));
                let live = active.clone();
                let query = query.clone();
                let seed = *seed;
                character_index.set(0);
                loaded.set(None);
                wasm_bindgen_futures::spawn_local(async move {
                    let result = load_words(&dictionary, &query, false).await.map(|words| {
                        words
                            .into_iter()
                            .filter(|word| !hanzi_characters(&word.simplified).is_empty())
                            .collect::<Vec<_>>()
                    });
                    if live.get() {
                        loaded.set(Some((seed, query, result)));
                    }
                });
                move || active.set(false)
            },
        );
    }
    let retry_load = {
        let retry_epoch = retry_epoch.clone();
        Callback::from(move |_: MouseEvent| {
            retry_epoch.set((*retry_epoch).wrapping_add(1));
        })
    };
    let Some((seed, source, result)) = &*loaded else {
        return html! { <p role="status">{"Loading writing vocabulary..."}</p> };
    };
    if *seed != props.seed || source != &props.vocabulary {
        return html! { <p role="status">{"Loading writing vocabulary..."}</p> };
    }
    let words = match result {
        Ok(words) if !words.is_empty() => words,
        Ok(_) => {
            return html! {
                <p role="alert">{"No eligible words for Hanzi writing. Select another vocabulary source."}</p>
            };
        }
        Err(error) => {
            return html! {
                <div class="space-y-3" role="alert">
                    <p>{error}</p>
                    <button
                        type="button"
                        data-action="retry-writing-vocabulary"
                        onclick={retry_load}
                        class="min-h-11 rounded-lg border border-line px-4 font-semibold"
                    >
                        {"Retry loading vocabulary"}
                    </button>
                </div>
            };
        }
    };
    let word = &words[(props.seed as usize) % words.len()];
    let characters = hanzi_characters(&word.simplified);
    let Some(character) = characters.get(*character_index) else {
        return html! { <p role="status">{"Preparing the next character..."}</p> };
    };
    let on_next = {
        let completion_gate = completion_gate.clone();
        let completion_key = format!(
            "{}:{}:{}:{}",
            props.seed, word.id, *character_index, *retry_epoch
        );
        let character_index = character_index.clone();
        let on_completed = props.on_completed.clone();
        let count = characters.len();
        Callback::from(move |_: ()| {
            if !completion_gate.borrow_mut().accept(&completion_key) {
                return;
            }
            if *character_index + 1 < count {
                character_index.set(*character_index + 1);
            } else {
                on_completed.emit(());
            }
        })
    };
    html! {
        <section
            data-exercise="write-hanzi"
            class="mx-auto flex w-full max-w-3xl flex-col gap-4"
        >
            <header class="space-y-2" data-writing-word-prompt="">
                <div class="flex flex-wrap items-center gap-2">
                    <h2 class="text-2xl font-bold">{"Write "}<span lang="zh-Hans">{word.simplified.clone()}</span></h2>
                    if props.options.playable_audio {
                        <ListenButton hanzi={word.simplified.clone()} compact=true />
                    }
                </div>
                if props.options.show_pinyin && !word.pinyin.trim().is_empty() {
                    <p lang="zh-Latn" class="text-base font-medium text-accent" data-writing-pinyin="">{word.pinyin.clone()}</p>
                }
                if props.options.show_meaning {
                    if let Some(meaning) = word.english.as_deref().filter(|meaning| !meaning.trim().is_empty()) {
                        <p class="text-sm text-muted" data-writing-meaning="">{meaning.to_owned()}</p>
                    } else {
                        <p class="text-sm text-muted" data-writing-meaning-unavailable="">{"Meaning unavailable"}</p>
                    }
                }
            </header>
            <HanziWriting
                key={format!("{}-{}", word.id, *character_index)}
                selected={Some((character.to_string(), word.pinyin.clone(), word.english.clone().unwrap_or_default()))}
                on_completed={Some(on_next)}
                on_completion_change={if *character_index + 1 == characters.len() {
                    Some(props.on_ready_change.clone())
                } else {
                    None
                }}
                options={Some(props.options)}
            />
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::CompletionGate;

    #[test]
    fn completion_is_once_per_character_but_allows_the_next_character() {
        let mut gate = CompletionGate::default();
        assert!(gate.accept("seed:word:0:retry"));
        assert!(!gate.accept("seed:word:0:retry"));
        assert!(gate.accept("seed:word:1:retry"));
        assert!(!gate.accept("seed:word:1:retry"));
        assert!(gate.accept("seed:word:0:next-retry"));
    }
}
