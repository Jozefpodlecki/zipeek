mod gameplay;
mod pinyin;
mod results;
mod rotation;
mod setup;
mod state;
mod storage;
mod workflow;

use crate::hooks::use_dictionary;
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::use_location;
use zipeek_exercises::{crossword::CrosswordWord, practice_draft::DictionaryFilters};

use crate::hooks::use_app_environment;
use state::{CrosswordAction, CrosswordState};
use workflow::{CrosswordWorkflow, WorkflowAction};

#[function_component(CrosswordPage)]
pub fn crossword_page() -> Html {
    let environment = use_app_environment();
    let dictionary = use_dictionary();
    let initial_session_id = use_location()
        .and_then(|location| location.query::<HashMap<String, String>>().ok())
        .and_then(|query| query.get("session_id").cloned());
    let initial_environment = environment.clone();
    let session = use_reducer(move || {
        CrosswordState::from_environment(&initial_environment, initial_session_id.as_deref())
    });
    let workflow = use_reducer(CrosswordWorkflow::default);

    {
        let id = session.session_id.clone();
        let environment = environment.clone();
        use_effect_with(id, move |id| {
            storage::update_url(&environment, id);
            || ()
        });
    }
    {
        let game = session.game.clone();
        let id = session.session_id.clone();
        let bands = session.bands.clone();
        let environment = environment.clone();
        use_effect_with((id, game, bands), move |(id, game, bands)| {
            if let Some(game) = game.as_ref() {
                if !id.is_empty() {
                    storage::save(&environment, id, game, bands);
                }
            }
            || ()
        });
    }

    {
        let id = session.session_id.clone();
        let seed = session.game.as_ref().map(|game| game.seed());
        let game = session.game.clone();
        let environment = environment.clone();
        use_effect_with((id, seed), move |(id, _)| {
            if let Some(game) = game.as_ref() {
                storage::record_rotation(&environment, id, game);
            }
            || ()
        });
    }

    let pinyin = pinyin::use_crossword_pinyin(session.clone(), environment.document.clone());

    let on_toggle_band = {
        let workflow = workflow.clone();
        Callback::from(move |band| workflow.dispatch(WorkflowAction::ToggleBand(band)))
    };
    let on_start = {
        let dictionary = dictionary.clone();
        let session = session.clone();
        let workflow = workflow.clone();
        let environment = environment.clone();
        Callback::from(move |_| {
            if workflow.loading {
                return;
            }
            if workflow.bands.is_empty() {
                workflow.dispatch(WorkflowAction::Begin);
                return;
            }
            let mut filters = DictionaryFilters::default();
            filters.hsk = workflow.bands.clone();
            let dictionary = dictionary.clone();
            let session = session.clone();
            let workflow = workflow.clone();
            let history = storage::load_rotation(&environment);
            workflow.dispatch(WorkflowAction::Begin);
            spawn_local(async move {
                let result = async {
                    let db = dictionary.database().map_err(|e| e.to_string())?;
                    let words = db.crossword_words_for_filters(&filters).await.map_err(|e| e.to_string())?;
                    let vocabulary: Vec<CrosswordWord> = words.into_iter().filter_map(|word| {
                        Some(CrosswordWord {
                            answer: word.simplified,
                            english: word.english?.trim().to_owned(),
                            pinyin: word.pinyin,
                        })
                    }).collect();
                    if vocabulary.len() < 12 {
                        return Err("This selection has too few multi-character words for a crossword. Add more HSK levels.".to_owned());
                    }
                    Ok(vocabulary)
                }.await;
                match result {
                    Ok(vocabulary) => {
                        gloo::timers::callback::Timeout::new(20, move || {
                            session.dispatch(CrosswordAction::Start {
                                vocabulary,
                                bands: filters.hsk.into_iter().collect(),
                                history,
                            });
                            workflow.dispatch(WorkflowAction::Complete);
                        })
                        .forget();
                    }
                    Err(message) => workflow.dispatch(WorkflowAction::Fail(message)),
                }
            });
        })
    };
    let on_reset = {
        let session = session.clone();
        let pinyin_input = pinyin.query.clone();
        let workflow = workflow.clone();
        let environment = environment.clone();
        Callback::from(move |_| {
            if workflow.loading {
                return;
            }
            let history = storage::load_rotation(&environment);
            pinyin_input.set(String::new());
            workflow.dispatch(WorkflowAction::Regenerate);
            let workflow = workflow.clone();
            let session = session.clone();
            gloo::timers::callback::Timeout::new(20, move || {
                session.dispatch(CrosswordAction::Reset { history });
                workflow.dispatch(WorkflowAction::Complete);
            })
            .forget();
        })
    };
    let on_place = {
        let session = session.clone();
        Callback::from(move |(row, col, tile)| {
            session.dispatch(CrosswordAction::Place { row, col, tile })
        })
    };
    let on_select = {
        let session = session.clone();
        Callback::from(move |index| session.dispatch(CrosswordAction::SelectTile(index)))
    };
    let on_activate = {
        let session = session.clone();
        Callback::from(move |number| session.dispatch(CrosswordAction::ActivateClue(number)))
    };
    let on_close_pinyin = {
        let query = pinyin.query.clone();
        Callback::from(move |_| query.set(String::new()))
    };
    let highlighted = session.active_clue;
    let game = session.game.clone();
    let finished = game.as_ref().is_some_and(|game| game.finished());

    let crossword_game_content = {
        if let Some(game) = game.as_ref() {
            let crossword_in_progress_content = {
                if !finished {
                    html! {
                        <div
                            class="flex flex-wrap items-center gap-2"
                        >
                            <button
                                data-testid="crossword-new-game"
                                type="button"
                                disabled={workflow.loading}
                                class="rounded-lg bg-brand-action px-3 py-2 font-semibold text-white hover:bg-brand-hover disabled:opacity-50"
                                onclick={on_reset.clone()}
                            >
                                {if workflow.loading { "Generating…" } else { "New game" }}
                            </button>
                        </div>
                    }
                } else {
                    Html::default()
                }
            };
            html! {
                <div
                    class="flex flex-wrap items-center gap-4 text-sm"
                >
                    <span
                        class="inline-flex items-center gap-1.5"
                    >
                        <span
                            aria-hidden="true"
                        >
                            <Icon
                                data={IconData::LUCIDE_STAR}
                                width="16"
                                height="16"
                            />
                        </span>
                        <span
                            class="sr-only"
                        >
                            {"Score: "}
                        </span>
                        <strong
                            data-testid="crossword-score"
                        >
                            {game.score}
                        </strong>
                    </span>
                    <span>
                        {format!("{} / {}", game.completed(), game.total())}
                    </span>
                    <span
                        class="text-muted"
                    >
                        {format!("{} missed", game.mistakes)}
                    </span>
                    {crossword_in_progress_content}
                </div>
            }
        } else {
            Html::default()
        }
    };
    let crossword_result_or_gameplay = {
        if let Some(game) = game {
            if finished {
                html! { <results::CrosswordResults game={game} bands={session.bands.clone()} on_reset={on_reset.clone()} /> }
            } else {
                html! {
                    <gameplay::CrosswordGameplay
                        game={game} active_clue={highlighted}
                        selected_tile={session.selected_tile} last_crossing={session.last_crossing}
                        rejected_tile={session.rejected_tile} rejection_sequence={session.rejection_sequence}
                        loading={workflow.loading} on_place={on_place} on_activate={on_activate}
                        on_select={on_select.clone()}>
                        <pinyin::PinyinPopover
                            query={(*pinyin.query).clone()} input_ref={pinyin.input_ref.clone()}
                            on_input={pinyin.on_input.clone()} on_key={pinyin.on_key.clone()}
                            matches={pinyin.matches.clone()} on_select={on_select}
                            on_close={on_close_pinyin} />
                    </gameplay::CrosswordGameplay>
                }
            }
        } else {
            html! {
                <setup::CrosswordSetup selected_bands={workflow.bands.clone()}
                    loading={workflow.loading} error={workflow.error.clone()} notice={session.notice}
                    on_toggle_band={on_toggle_band} on_start={on_start} />
            }
        }
    };

    html! {
        <section
            data-component="hanzi-crosswords"
            class="mx-auto flex w-full flex-col rounded-2xl border border-line bg-surface p-3 lg:fixed lg:inset-0 lg:z-20 lg:box-border lg:!m-0 lg:!h-dvh lg:!w-screen lg:!max-w-none lg:overflow-hidden lg:p-4"
        >
            <header
                class="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b border-line pb-3"
            >
                <div
                    class="flex items-center gap-3"
                >
                    <a
                        href="#/games"
                        aria-label="Back to all games"
                        class="rounded-lg border border-line p-2 text-muted hover:text-ink"
                    >
                        <Icon
                            data={IconData::LUCIDE_ARROW_LEFT}
                            width="20"
                            height="20"
                        />
                    </a>
                    <div>
                        <h1
                            class="text-xl font-bold"
                        >
                            {"Hanzi Crosswords"}
                        </h1>
                        <p
                            class="text-xs text-muted"
                        >
                            {"Complete intersecting Chinese words"}
                        </p>
                    </div>
                </div>
                {crossword_game_content}
            </header>
            {crossword_result_or_gameplay}
            <p
                role="status"
                aria-live="polite"
                class="sr-only"
            >
                {session.notice.unwrap_or("")}
            </p>
        </section>
    }
}
