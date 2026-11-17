mod game;
mod setup;
mod vocabulary;

use game::{CloudAction, CloudSession};
use game::{CloudPoolKey, CloudPoolResult, current_result};
use setup::CloudSetupState;
use std::{cell::Cell, rc::Rc};

use crate::hooks::use_dictionary;
use crate::hooks::use_word_sets;
use gloo::events::EventListener;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::cloud_match::CloudSide;

use crate::hooks::use_app_environment;

#[function_component(CloudMatchPage)]
pub fn cloud_match_page() -> Html {
    let environment = use_app_environment();
    let dictionary = use_dictionary();
    let sets = use_word_sets();
    let setup = use_reducer_eq(CloudSetupState::default);
    let pool = use_state(|| None::<CloudPoolResult>);
    let session = use_reducer_eq(CloudSession::default);
    let available_sets = sets.state.sets.as_ref().ok();
    let word_ids = available_sets
        .and_then(|items| {
            setup
                .selected_set
                .parse::<crate::services::word_sets::WordSetId>()
                .ok()
                .and_then(|id| crate::services::word_sets::find(items, &id))
                .or_else(|| {
                    if setup.selected_set.is_empty() {
                        items.first()
                    } else {
                        None
                    }
                })
        })
        .map(|set| set.word_ids.clone())
        .unwrap_or_default();
    let pool_key = CloudPoolKey {
        source: setup.source.clone(),
        standard: setup.standard,
        level: setup.level,
        word_ids,
    };
    let loaded_pool = current_result(&pool_key, &pool);

    {
        let dictionary = dictionary.clone();
        let pool = pool.clone();
        use_effect_with(pool_key.clone(), move |key| {
            let live = Rc::new(Cell::new(true));
            let pending = live.clone();
            let key = key.clone();
            pool.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let result = vocabulary::load_pairs(
                    &dictionary,
                    &key.source,
                    key.standard,
                    key.level,
                    key.word_ids.clone(),
                )
                .await;
                if pending.get() {
                    pool.set(Some(CloudPoolResult { key, result }));
                }
            });
            move || live.set(false)
        });
    }

    let in_game = session.game.is_some();
    {
        let document = environment.document.clone();
        let session = session.clone();
        use_effect_with((), move |_| {
            let observed_document = document.clone();
            let listener = EventListener::new(&document, "visibilitychange", move |_| {
                if observed_document.hidden() {
                    session.dispatch(CloudAction::PauseWhenHidden);
                }
            });
            move || drop(listener)
        });
    }
    {
        let session = session.clone();
        let sequence = session.feedback_sequence;
        let has_feedback = !session.feedback.is_empty();
        use_effect_with((sequence, has_feedback), move |(sequence, visible)| {
            let timeout = if *visible {
                let session = session.clone();
                let sequence = *sequence;
                Some(gloo::timers::callback::Timeout::new(850, move || {
                    session.dispatch(CloudAction::ClearFeedback(sequence));
                }))
            } else {
                None
            };
            move || drop(timeout)
        });
    }
    let playing = session
        .game
        .as_ref()
        .is_some_and(|game| !game.finished && !game.paused);
    {
        let session = session.clone();
        use_effect_with(playing, move |running| {
            let interval = if *running {
                Some(gloo::timers::callback::Interval::new(1000, move || {
                    session.dispatch(CloudAction::Tick);
                }))
            } else {
                None
            };
            move || drop(interval)
        });
    }

    let on_start = {
        let pool = pool.clone();
        let pool_key = pool_key.clone();
        let session = session.clone();
        let mode = setup.mode;
        let difficulty = setup.difficulty;
        let game_length = setup.length;
        let life_limit = setup.lives;
        let spawn_mode = setup.spawn;
        Callback::from(move |_| {
            if let Some(Ok(words)) = current_result(&pool_key, &pool) {
                let seed = chrono::Utc::now().timestamp_millis() as u64;
                match zipeek_exercises::cloud_match::CloudGame::new_with_spawn_mode(
                    words.clone(),
                    mode,
                    difficulty,
                    seed,
                    game_length,
                    life_limit,
                    spawn_mode,
                ) {
                    Ok(game) => session.dispatch(CloudAction::Start(game)),
                    Err(error) => session.dispatch(CloudAction::StartFailed(error)),
                }
            }
        })
    };
    let on_back = {
        let session = session.clone();
        Callback::from(move |_| session.dispatch(CloudAction::ReturnToSetup))
    };
    let on_stop = {
        let session = session.clone();
        let window = environment.window;
        Callback::from(move |_| {
            let confirmed = window
                .confirm_with_message(
                    "End this game and return to setup? Your current score will be lost.",
                )
                .ok()
                .unwrap_or(false);
            if confirmed {
                session.dispatch(CloudAction::ReturnToSetup);
            }
        })
    };
    let on_finish_endless = {
        let session = session.clone();
        Callback::from(move |_| session.dispatch(CloudAction::Finish))
    };
    let on_pause = {
        let session = session.clone();
        Callback::from(move |_| session.dispatch(CloudAction::Pause))
    };
    let choose = {
        let session = session.clone();
        let speech = environment.speech.clone();
        Callback::from(move |(side, id): (CloudSide, i32)| {
            if let Some(round) = &session.game {
                if !round.finished && !round.paused {
                    if let Some(pair) = round.wave_pairs().iter().find(|pair| pair.id == id) {
                        if round.has_hanzi_on_side(pair, side) {
                            if let Err(error) =
                                crate::services::practice_audio::speak(&speech, &pair.hanzi)
                            {
                                log::warn!("Cloud Match pronunciation unavailable: {error:?}");
                            }
                        }
                    }
                }
            }
            session.dispatch(CloudAction::Choose(side, id));
        })
    };
    let cloud_match_content = {
        if let Some(round) = &session.game {
            let completion_overlay = if round.finished {
                game::summary_view(round, on_start.clone(), on_back.clone())
            } else {
                Html::default()
            };
            let pause_overlay = if round.paused && !round.finished {
                html! {
                    <game::CloudPauseOverlay
                        round={round.clone()}
                        on_resume={on_pause.clone()}
                    />
                }
            } else {
                Html::default()
            };
            html! {
                <section
                    class="relative flex h-dvh min-h-0 w-full flex-col max-[520px]:min-h-dvh"
                >
                    {game::hud_view(round, &session.feedback, on_pause.clone(), on_stop.clone(), on_finish_endless.clone())}
                    <game::CloudField
                        key={format!("{}-{}", round.seed, round.wave)}
                        round={round.clone()}
                        choose={choose.clone()}
                        show_match_hint={setup.highlight_partner}
                        incorrect_pairs={session.incorrect_pairs.clone()}
                        pause_overlay={pause_overlay}
                        completion_overlay={completion_overlay}
                    />
                </section>
            }
        } else {
            html! {
                <setup::CloudSetupView
                    setup={setup.clone()}
                    pool={loaded_pool.cloned()}
                    set_names={available_sets.cloned().unwrap_or_default()}
                    setup_error={session.setup_error.clone()}
                    on_start={on_start.clone()}
                />
            }
        }
    };
    let cloud_match_navigation = if in_game {
        Html::default()
    } else {
        html! {
            <header
                class="flex items-center gap-3 border-b border-line pb-3"
            >
                <a
                    href="#/games"
                    class="grid size-10 shrink-0 scroll-mt-24 place-items-center rounded-lg border border-line text-accent hover:bg-raised"
                    aria-label="All games"
                >
                    <Icon
                        data={IconData::LUCIDE_ARROW_LEFT}
                        width="20"
                        height="20"
                    />
                </a>
                <div>
                    <h1
                        class="flex items-center gap-2 text-2xl font-bold"
                    >
                        <Icon
                            data={IconData::LUCIDE_CLOUD}
                            width="25"
                            height="25"
                        />
                        {"Cloud Match"}
                    </h1>
                    <p
                        class="text-sm text-muted"
                    >
                        {"Find matching words as the clouds drift across the sky."}
                    </p>
                </div>
            </header>
        }
    };

    html! {
        <main
            data-component="cloud-match"
            class={if in_game {"fixed inset-0 z-[100] w-full min-h-dvh bg-[#11243d]"} else {"mx-auto flex w-full max-w-6xl flex-col gap-5 py-4"}}
        >
            {cloud_match_navigation}
            {cloud_match_content}
        </main>
    }
}
