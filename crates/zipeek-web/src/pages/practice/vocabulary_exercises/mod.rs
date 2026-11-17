use std::{cell::Cell, collections::HashMap, rc::Rc};

use yew::prelude::*;
use yew_router::prelude::Link;
use zipeek_core::Word;

use crate::{hooks::use_dictionary, models::VocabularyExerciseOptions, route::Route};

mod answer;
mod character;
mod character_builder;
mod mosaic;
mod progress;
mod type_pinyin;
mod vocabulary;
mod word_mosaics;

use character_builder::CharacterBuilder;
use progress::{ProgressAction, SessionProgress};
use type_pinyin::TypePinyin;
pub(crate) use vocabulary::load_words;
use word_mosaics::WordMosaics;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VocabularyExerciseMode {
    TypePinyin,
    CharacterBuilder,
    WordMosaics,
}

impl VocabularyExerciseMode {
    fn title(self) -> &'static str {
        match self {
            Self::TypePinyin => "Type pinyin",
            Self::CharacterBuilder => "Character builder",
            Self::WordMosaics => "Word mosaics",
        }
    }

    fn words_per_round(self) -> usize {
        match self {
            Self::WordMosaics => 8,
            Self::TypePinyin | Self::CharacterBuilder => 1,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct VocabularyExercisePageProps {
    #[prop_or_default]
    pub session_query: Option<HashMap<String, String>>,
    #[prop_or_default]
    pub on_completed: Option<Callback<()>>,
    #[prop_or_default]
    pub instance_seed: u64,
    pub exercise_options: VocabularyExerciseOptions,
}

#[function_component(VocabularyExercisePage)]
pub fn vocabulary_exercise_page(props: &VocabularyExercisePageProps) -> Html {
    let dictionary = use_dictionary();
    let query = props.session_query.clone().unwrap_or_default();
    let exercise_options = props.exercise_options;
    let mode = match exercise_options {
        VocabularyExerciseOptions::TypePinyin(_) => VocabularyExerciseMode::TypePinyin,
        VocabularyExerciseOptions::CharacterBuilder(_) => VocabularyExerciseMode::CharacterBuilder,
        VocabularyExerciseOptions::WordMosaics => VocabularyExerciseMode::WordMosaics,
    };
    let round_limit = query
        .get("rounds")
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|count| (1..=20).contains(count));
    let progress = use_reducer(SessionProgress::default);
    let pool = use_state(|| None::<Result<Vec<Word>, String>>);

    {
        let dictionary = dictionary.clone();
        let progress = progress.clone();
        let pool = pool.clone();
        let instance_seed = props.instance_seed;
        use_effect_with(
            (query.clone(), mode, instance_seed),
            move |(query, mode, _)| {
                let alive = Rc::new(Cell::new(true));
                let pending = alive.clone();
                let query = query.clone();
                let unique_meanings = *mode == VocabularyExerciseMode::WordMosaics;
                pool.set(None);
                progress.dispatch(ProgressAction::Reset);
                wasm_bindgen_futures::spawn_local(async move {
                    let result = load_words(&dictionary, &query, unique_meanings).await;
                    if pending.get() {
                        pool.set(Some(result));
                    }
                });
                move || alive.set(false)
            },
        );
    }

    let Some(Ok(words)) = &*pool else {
        return html! {
            <main class="mx-auto max-w-4xl rounded-3xl border border-line bg-surface p-8">
                <h1 class="text-2xl font-bold">{mode.title()}</h1>
                <p
                    role={if matches!(&*pool, Some(Err(_))) { "alert" } else { "status" }}
                    aria-live="polite"
                    class="mt-4"
                >
                    {match &*pool {
                        None => "Preparing your selected vocabulary…".to_owned(),
                        Some(Err(error)) => format!("Could not load vocabulary: {error}"),
                        _ => String::new(),
                    }}
                </p>
                <Link<Route> to={Route::PracticeSetup} classes="mt-6 inline-block underline">{"Back to practice"}</Link<Route>>
            </main>
        };
    };
    let words = words.clone();
    let width = mode.words_per_round();
    if words.len() < width {
        return html! {
            <main class="mx-auto max-w-3xl rounded-2xl border border-line bg-surface p-8" role="alert">
                {format!("Not enough eligible words for {}. Select another vocabulary source.", mode.title())}
                <div class="mt-4">
                    <Link<Route> to={Route::PracticeSetup}>{"Back to practice"}</Link<Route>>
                </div>
            </main>
        };
    }
    if progress.completed {
        return html! {
            <main data-component="vocabulary-exercise-completed" class="mx-auto max-w-3xl space-y-5 rounded-3xl border border-line bg-surface p-8 text-center">
                <h1 class="text-2xl font-bold">{"Practice complete"}</h1>
                <p class="text-muted">{format!("{} rounds completed", progress.round + 1)}</p>
                <Link<Route> to={Route::PracticeSetup} classes="inline-flex min-h-11 items-center rounded-xl bg-brand-action px-5 font-semibold text-white">{"Back to practice"}</Link<Route>>
            </main>
        };
    }

    let on_round_completed = {
        let progress = progress.clone();
        let on_session_completed = props.on_completed.clone();
        Callback::from(move |round_mistakes: u32| {
            if round_limit.is_some_and(|limit| progress.round + 1 >= limit) {
                if let Some(callback) = &on_session_completed {
                    callback.emit(());
                } else {
                    progress.dispatch(ProgressAction::Finish {
                        mistakes: round_mistakes,
                    });
                }
            } else {
                progress.dispatch(ProgressAction::NextRound {
                    mistakes: round_mistakes,
                });
            }
        })
    };
    let index = (props.instance_seed as usize % words.len() + progress.round * width) % words.len();
    let seed = props.instance_seed ^ (progress.round * width) as u64;
    let exercise = match exercise_options {
        VocabularyExerciseOptions::CharacterBuilder(options) => html! {
            <CharacterBuilder
                key={format!("character-{index}-{}", progress.round)}
                word={words[index].clone()}
                words={words.clone()}
                seed={seed}
                options={options}
                on_completed={on_round_completed}
            >
                <Link<Route> to={Route::PracticeSetup} classes="rounded-xl border border-line px-6 py-4 font-semibold">{"Exit practice"}</Link<Route>>
            </CharacterBuilder>
        },
        VocabularyExerciseOptions::TypePinyin(options) => html! {
            <TypePinyin
                key={format!("pinyin-{index}-{}", progress.round)}
                word={words[index].clone()}
                options={options}
                on_completed={on_round_completed}
            >
                <Link<Route> to={Route::PracticeSetup} classes="rounded-xl border border-line px-6 py-4 font-semibold">{"Exit practice"}</Link<Route>>
            </TypePinyin>
        },
        VocabularyExerciseOptions::WordMosaics => {
            let group = (0..width)
                .map(|offset| words[(index + offset) % words.len()].clone())
                .collect::<Vec<_>>();
            html! {
                <WordMosaics
                    key={format!("mosaics-{index}-{}", progress.round)}
                    words={group}
                    seed={seed}
                    on_completed={on_round_completed}
                />
            }
        }
    };

    html! {
        <main data-component="vocabulary-exercise" class="mx-auto w-full max-w-5xl p-3 sm:p-6">
            <section class="rounded-3xl border border-line bg-surface p-5 sm:p-9 space-y-7 shadow-lg">
                <header class="flex flex-wrap items-center justify-between gap-3">
                    <h1 class="text-2xl font-bold">{mode.title()}</h1>
                    <span class="text-sm text-muted">{format!("Round {} · Mistakes {}", progress.round + 1, progress.mistakes)}</span>
                </header>
                {exercise}
                {if matches!(exercise_options, VocabularyExerciseOptions::WordMosaics) {
                    html! { <Link<Route> to={Route::PracticeSetup} classes="inline-flex rounded-xl border border-line px-6 py-4 font-semibold">{"Exit practice"}</Link<Route>> }
                } else { Html::default() }}
            </section>
        </main>
    }
}
