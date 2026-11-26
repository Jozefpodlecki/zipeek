#[path = "time_weaver_options.rs"]
mod options_model;
#[path = "time_weaver_pinyin.rs"]
mod pinyin;

use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::time_weaver::{DateQuestionKind, TimeWeaver};

use crate::hooks::use_app_environment;
use options_model::{GameOptions, TileSize};

#[derive(Clone, Copy, PartialEq, Eq)]
enum DragSource {
    Bank(usize),
    Answer(usize),
}

fn random_session(question_count: usize) -> TimeWeaver {
    let mut bytes = [0u8; 8];
    if let Err(error) = getrandom::fill(&mut bytes) {
        log::warn!("Unable to obtain random seed for Time Weaver: {error}");
        return TimeWeaver::with_seed_and_count(
            chrono::Utc::now().timestamp_millis() as u64,
            question_count,
        );
    }
    TimeWeaver::with_seed_and_count(u64::from_le_bytes(bytes), question_count)
}

#[function_component(TimeWeaverPage)]
pub fn time_weaver_page() -> Html {
    let environment = use_app_environment();
    let options = {
        let storage = environment.local_storage.clone();
        use_state(move || GameOptions::load(&storage))
    };
    let started = use_state(|| false);
    let audio_error = use_state(|| false);
    let game = use_state(|| random_session(15));
    let dragging = use_state(|| None::<DragSource>);
    let reset = {
        let game = game.clone();
        let started = started.clone();
        let options = options.clone();
        Callback::from(move |_: MouseEvent| {
            game.set(random_session(options.question_count));
            started.set(false);
        })
    };

    if !*started {
        let on_start = {
            let started = started.clone();
            let game = game.clone();
            let options = options.clone();
            Callback::from(move |_: MouseEvent| {
                game.set(random_session(options.question_count));
                started.set(true);
            })
        };
        let on_pinyin = {
            let options = options.clone();
            let storage = environment.local_storage.clone();
            Callback::from(move |_: MouseEvent| {
                let next = GameOptions {
                    show_pinyin: !options.show_pinyin,
                    ..*options
                };
                next.save(&storage);
                options.set(next);
            })
        };
        let on_audio = {
            let options = options.clone();
            let storage = environment.local_storage.clone();
            Callback::from(move |_: MouseEvent| {
                let next = GameOptions {
                    audio_on_click: !options.audio_on_click,
                    ..*options
                };
                next.save(&storage);
                options.set(next);
            })
        };
        let on_count = {
            let options = options.clone();
            let storage = environment.local_storage.clone();
            Callback::from(move |count: usize| {
                let next = GameOptions {
                    question_count: count,
                    ..*options
                };
                next.save(&storage);
                options.set(next);
            })
        };
        let on_size = {
            let options = options.clone();
            let storage = environment.local_storage.clone();
            Callback::from(move |size: TileSize| {
                let next = GameOptions {
                    tile_size: size,
                    ..*options
                };
                next.save(&storage);
                options.set(next);
            })
        };
        let preview_font = options.tile_size.font_class();
        return html! {
            <main data-time-weaver-setup="" class="mx-auto flex w-full max-w-2xl flex-1 items-center px-4 py-6 sm:py-10">
                <section class="w-full space-y-5 rounded-2xl border border-line bg-surface p-5 shadow-lg sm:p-8">
                    <header class="space-y-2">
                        <p class="text-xs font-semibold uppercase tracking-wide text-accent">{"Game setup"}</p>
                        <h1 class="text-2xl font-bold text-ink">{"Time Weaver"}</h1>
                        <p class="text-sm text-muted">{"Practice Chinese dates, weekdays and clock times"}</p>
                    </header>
                    <fieldset class="space-y-2">
                        <legend class="mb-2 text-sm font-semibold text-ink">{"Round length"}</legend>
                        <div class="grid grid-cols-3 gap-2" data-time-weaver-count-setting="">
                            {for [10_usize, 15, 20].into_iter().map(|count| {
                                let selected = options.question_count == count;
                                let on_click = {
                                    let on_count = on_count.clone();
                                    Callback::from(move |_: MouseEvent| on_count.emit(count))
                                };
                                html! {
                                    <button type="button" key={count} data-time-weaver-count={count.to_string()} aria-pressed={selected.to_string()} onclick={on_click}
                                        class={format!("rounded-xl border px-3 py-3 text-sm font-semibold transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action {}", if selected { "border-accent bg-brand-action text-white" } else { "border-line bg-raised text-ink hover:border-accent" })}>
                                        {format!("{count} prompts")}
                                    </button>
                                }
                            })}
                        </div>
                    </fieldset>
                    <div class="grid gap-3 sm:grid-cols-2">
                        <button type="button" role="switch" aria-checked={options.show_pinyin.to_string()} data-time-weaver-pinyin-setting="" onclick={on_pinyin}
                            class="flex min-h-20 items-center justify-between gap-3 rounded-xl border border-line bg-raised p-4 text-left transition-colors hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action">
                            <span class="space-y-1"><span class="block text-sm font-semibold text-ink">{"Show pinyin"}</span><span class="block text-xs text-muted">{"Reading beneath Chinese tiles"}</span></span>
                            <span aria-hidden="true" class={format!("flex h-6 w-11 shrink-0 items-center rounded-full p-1 transition-colors {}", if options.show_pinyin {"bg-brand-action"} else {"bg-line"})}>
                                <span class={format!("size-4 rounded-full bg-white transition-transform {}", if options.show_pinyin {"translate-x-5"} else {""})}></span>
                            </span>
                        </button>
                        <button type="button" role="switch" aria-checked={options.audio_on_click.to_string()} data-time-weaver-audio-setting="" onclick={on_audio}
                            class="flex min-h-20 items-center justify-between gap-3 rounded-xl border border-line bg-raised p-4 text-left transition-colors hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action">
                            <span class="space-y-1"><span class="block text-sm font-semibold text-ink">{"Speak on tap"}</span><span class="block text-xs text-muted">{"Play Mandarin when choosing a tile"}</span></span>
                            <span aria-hidden="true" class={format!("flex h-6 w-11 shrink-0 items-center rounded-full p-1 transition-colors {}", if options.audio_on_click {"bg-brand-action"} else {"bg-line"})}>
                                <span class={format!("size-4 rounded-full bg-white transition-transform {}", if options.audio_on_click {"translate-x-5"} else {""})}></span>
                            </span>
                        </button>
                    </div>
                    <fieldset class="space-y-2">
                        <legend class="mb-2 text-sm font-semibold text-ink">{"Chinese tile size"}</legend>
                        <div class="grid grid-cols-3 gap-2" data-time-weaver-size-setting="">
                            {for [TileSize::Compact, TileSize::Standard, TileSize::Large].into_iter().map(|size| {
                                let selected = options.tile_size == size;
                                let on_click = {
                                    let on_size = on_size.clone();
                                    Callback::from(move |_: MouseEvent| on_size.emit(size))
                                };
                                html! {
                                    <button type="button" key={size.label()} data-time-weaver-size={size.label().to_lowercase()} aria-pressed={selected.to_string()} onclick={on_click}
                                        class={format!("flex flex-col items-center gap-1.5 rounded-xl border px-2 py-3 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action {}", if selected { "border-accent bg-brand-action/15 text-ink" } else { "border-line bg-raised text-ink hover:border-accent" })}>
                                        <span lang="zh-Hans" aria-hidden="true" class={format!("font-bold {}", size.font_class())}>{"时"}</span>
                                        <span class="text-xs font-medium">{size.label()}</span>
                                    </button>
                                }
                            })}
                        </div>
                    </fieldset>
                    <div class="rounded-xl border border-line bg-raised/60 p-4" data-time-weaver-preview="">
                        <div class="flex items-center justify-between gap-2">
                            <p class="text-xs font-semibold uppercase tracking-wide text-muted">{"Live preview"}</p>
                            <span class="text-xs text-muted">{if options.audio_on_click {"Audio on tap"} else {"Audio off"}}</span>
                        </div>
                        <div class="mt-3 flex justify-center">
                            <div class="flex min-h-20 min-w-24 flex-col items-center justify-center gap-1 rounded-xl border border-line bg-surface px-5 py-3">
                                <span lang="zh-Hans" class={format!("font-semibold text-ink {}", preview_font)}>{"星期一"}</span>
                                {if options.show_pinyin {html! {<span lang="en" class="text-xs text-muted">{"xīng qī yī"}</span>}} else {Html::default()}}
                            </div>
                        </div>
                    </div>
                    <div class="space-y-2">
                        <p class="text-center text-xs text-muted">{format!("{} prompts · {} tiles · Pinyin {} · Audio {}", options.question_count, options.tile_size.label(), if options.show_pinyin {"on"} else {"off"}, if options.audio_on_click {"on"} else {"off"})}</p>
                        <button type="button" data-time-weaver-start="" onclick={on_start} class="w-full rounded-xl bg-brand-action px-6 py-3 font-semibold text-white hover:opacity-90 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action">{"Start game"}</button>
                    </div>
                </section>
            </main>
        };
    }

    let speak = {
        let speech = environment.speech.clone();
        let audio_error = audio_error.clone();
        Callback::from(move |hanzi: String| {
            if let Err(error) = crate::services::practice_audio::speak(&speech, &hanzi) {
                log::warn!("Time Weaver audio unavailable: {error:?}");
                audio_error.set(true);
            } else {
                audio_error.set(false);
            }
        })
    };

    if game.finished() {
        return html! {
            <main data-time-weaver-finished="" class="mx-auto flex w-full max-w-xl flex-1 items-center px-4 py-8">
                <section class="w-full space-y-5 rounded-2xl border border-line bg-surface p-7 text-center shadow-lg sm:p-10">
                    <p class="text-xs font-bold uppercase tracking-widest text-accent">{"Time Weaver"}</p>
                    <h1 class="text-2xl font-bold text-ink">{"Round complete"}</h1>
                    <p class="text-5xl font-bold text-ink" data-time-weaver-score="">{format!("{} / {}", game.score(), game.total())}</p>
                    <p class="text-sm text-muted">{"Correct answers"}</p>
                    <button type="button" onclick={reset} class="rounded-xl bg-brand-action px-6 py-3 font-semibold text-white hover:opacity-90 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action">
                        {"Play again"}
                    </button>
                </section>
            </main>
        };
    }

    let kind = game.question_kind();
    let review = game.reviewed();
    let is_build = kind.is_build();
    let correct_answer = game
        .correct_answer()
        .expect("active round has a correct answer");
    let prompt = game.prompt().expect("active round has a prompt");
    let on_next = {
        let game = game.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*game).clone();
            if next.next() {
                game.set(next);
            }
        })
    };
    let tiles = game.tiles().unwrap_or_default();
    let chosen = game.selected_tiles().to_vec();
    let tile_count = tiles.len();
    let tile_buttons = game.tile_order().into_iter().map(|tile_index| {
        let game = game.clone();
        let used = chosen.contains(&tile_index);
        let speak = speak.clone();
        let audio = options.audio_on_click;
        let word = tiles[tile_index].clone();
        let on_select = Callback::from(move |_: MouseEvent| {
            let mut next = (*game).clone();
            if next.select_tile(tile_index) {
                game.set(next);
                if audio { speak.emit(word.clone()); }
            }
        });
        let on_drag_start = {
            let dragging = dragging.clone();
            Callback::from(move |event: DragEvent| {
                if used || review.is_some() {
                    event.prevent_default();
                    return;
                }
                if let Some(transfer) = event.data_transfer() {
                    transfer.set_effect_allowed("move");
                    let _ = transfer.set_data("text/plain", &tile_index.to_string());
                }
                dragging.set(Some(DragSource::Bank(tile_index)));
            })
        };
        let on_drag_end = {
            let dragging = dragging.clone();
            Callback::from(move |_: DragEvent| dragging.set(None))
        };
        let reading = pinyin::reading(&tiles[tile_index]);
        let show_pinyin = options.show_pinyin;
        let tile_font = options.tile_size.font_class();
        html! {
            <button
                key={tile_index}
                type="button"
                draggable={(!used && review.is_none()).to_string()}
                ondragstart={on_drag_start}
                ondragend={on_drag_end}
                data-time-weaver-tile={tile_index.to_string()}
                disabled={used || review.is_some()}
                onclick={on_select}
                lang="zh-Hans"
                class="flex min-h-16 cursor-grab flex-col items-center justify-center gap-1 rounded-xl border border-line bg-raised px-2 py-2 font-semibold text-ink shadow-sm transition-all hover:border-accent hover:bg-surface active:cursor-grabbing disabled:cursor-default disabled:opacity-25 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action"
            >
                <span class={tile_font}>{tiles[tile_index].clone()}</span>
                {if show_pinyin { html! {<span lang="en" class="text-xs font-normal text-muted">{reading}</span>} } else {Html::default()} }
            </button>
        }
    });
    let on_remove = {
        let game = game.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*game).clone();
            if next.remove_last_tile() {
                game.set(next);
            }
        })
    };
    let on_reset_answer = {
        let game = game.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*game).clone();
            if next.reset_tiles() {
                game.set(next);
            }
        })
    };
    let on_drop_answer = {
        let game = game.clone();
        let dragging = dragging.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            let source = *dragging;
            dragging.set(None);
            let mut next = (*game).clone();
            let changed = match source {
                Some(DragSource::Bank(tile)) => next.select_tile(tile),
                Some(DragSource::Answer(from)) if !next.selected_tiles().is_empty() => {
                    let last = next.selected_tiles().len() - 1;
                    next.move_tile(from, last)
                }
                _ => false,
            };
            if changed {
                game.set(next);
            }
        })
    };
    let on_drop_bank = {
        let game = game.clone();
        let dragging = dragging.clone();
        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            let source = *dragging;
            dragging.set(None);
            if let Some(DragSource::Answer(position)) = source {
                let mut next = (*game).clone();
                if next.remove_tile(position) {
                    game.set(next);
                }
            }
        })
    };
    let on_drag_over = Callback::from(|event: DragEvent| event.prevent_default());
    let chosen_tiles = chosen.iter().enumerate().map(|(position, &tile_index)| {
        let game_for_click = game.clone();
        let speak = speak.clone();
        let audio = options.audio_on_click;
        let word = tiles[tile_index].clone();
        let on_click = Callback::from(move |_: MouseEvent| {
            let mut next = (*game_for_click).clone();
            if next.remove_tile(position) {
                game_for_click.set(next);
                if audio { speak.emit(word.clone()); }
            }
        });
        let on_drag_start = {
            let dragging = dragging.clone();
            Callback::from(move |event: DragEvent| {
                if review.is_some() {
                    event.prevent_default();
                    return;
                }
                if let Some(transfer) = event.data_transfer() {
                    transfer.set_effect_allowed("move");
                    let _ = transfer.set_data("text/plain", &tile_index.to_string());
                }
                dragging.set(Some(DragSource::Answer(position)));
            })
        };
        let on_drag_end = {
            let dragging = dragging.clone();
            Callback::from(move |_: DragEvent| dragging.set(None))
        };
        let on_drop = {
            let game = game.clone();
            let dragging = dragging.clone();
            Callback::from(move |event: DragEvent| {
                event.prevent_default();
                event.stop_propagation();
                let source = *dragging;
                dragging.set(None);
                let mut next = (*game).clone();
                let changed = match source {
                    Some(DragSource::Bank(tile)) => next.insert_tile(tile, position),
                    Some(DragSource::Answer(from)) => next.move_tile(from, position),
                    None => false,
                };
                if changed { game.set(next); }
            })
        };
        let reading = pinyin::reading(&tiles[tile_index]);
        let show_pinyin = options.show_pinyin;
        let tile_font = options.tile_size.font_class();
        html! {
            <button
                key={tile_index}
                type="button"
                lang="zh-Hans"
                aria-label={format!("Remove {} from answer; drag to rearrange", tiles[tile_index])}
                title="Click to remove or drag to rearrange"
                disabled={review.is_some()}
                draggable={review.is_none().to_string()}
                ondragstart={on_drag_start}
                ondragend={on_drag_end}
                ondragover={on_drag_over.clone()}
                ondrop={on_drop}
                onclick={on_click}
                class="flex cursor-grab flex-col items-center gap-0.5 rounded-lg border border-accent/40 bg-surface px-2.5 py-1.5 font-semibold text-ink hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action active:cursor-grabbing disabled:cursor-default"
            >
                <span class={tile_font}>{tiles[tile_index].clone()}</span>
                {if show_pinyin {html! {<span lang="en" class="text-xs font-normal text-muted">{reading}</span>}} else {Html::default()}}
            </button>
        }
    }).collect::<Html>();
    let on_submit = {
        let game = game.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = (*game).clone();
            if next.submit_tiles() {
                game.set(next);
            }
        })
    };
    let choice_buttons = game.options().map(|choices| choices.into_iter().enumerate().map(|(index, answer)| {
        let selected = game.chosen_option() == Some(index);
        let game = game.clone();
        let speak = speak.clone();
        let audio = options.audio_on_click;
        let spoken_answer = answer.clone();
        let on_select = Callback::from(move |_: MouseEvent| {
            let mut next = (*game).clone();
            if next.choose_option(index) {
                game.set(next);
                if audio { speak.emit(spoken_answer.clone()); }
            }
        });
        let correct = answer == correct_answer;
        let answer_class = match review {
            Some(_) if selected && correct => "border-emerald-400 bg-emerald-400/15",
            Some(_) if selected => "border-rose-400 bg-rose-400/15",
            Some(_) if correct => "border-emerald-400 bg-emerald-400/10",
            Some(_) => "border-line bg-raised opacity-65",
            None => "border-line bg-raised hover:border-accent hover:bg-surface",
        };
        let reading = pinyin::reading(&answer);
        let tile_font = options.tile_size.font_class();
        let show_pinyin = options.show_pinyin;
        html! {
            <button
                key={index}
                type="button"
                lang="zh-Hans"
                data-time-weaver-option={index.to_string()}
                disabled={review.is_some()}
                onclick={on_select}
                class={format!("flex min-h-18 flex-col items-center justify-center gap-1 rounded-xl border px-4 py-3 font-semibold text-ink transition-colors disabled:cursor-default focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action {answer_class}")}
            >
                <span class={tile_font}>{answer}</span>
                {if show_pinyin {html! {<span lang="en" class="text-xs font-normal text-muted">{reading}</span>}} else {Html::default()}}
            </button>
        }
    }).collect::<Html>());
    let question_label = match kind {
        DateQuestionKind::BuildHanzi => "Build the date",
        DateQuestionKind::ChooseHanzi => "Pick the Chinese date",
        DateQuestionKind::ChooseWeekday => "Pick the weekday",
        DateQuestionKind::BuildTime => "Build the time",
        DateQuestionKind::ChooseTime => "Pick the Chinese time",
    };
    let submitted_answer = game.submitted_answer();
    let weekday_context = if kind == DateQuestionKind::ChooseWeekday {
        game.date().map(|date| date.western())
    } else {
        None
    };
    let completed = game.index();
    let progress = format!("width: {}%", completed * 100 / game.total().max(1));

    html! {
        <main data-time-weaver-game="" class="mx-auto flex w-full max-w-xl flex-col justify-center gap-4 px-4 py-5 sm:py-8">
            <header class="flex items-center justify-between gap-3">
                <div class="min-w-0">
                    <h1 class="text-xl font-bold text-ink">{"Time Weaver"}</h1>
                </div>
                <div class="flex flex-wrap items-center justify-end gap-2 text-xs" data-time-weaver-progress="">
                    <span class="rounded-full border border-line bg-surface px-3 py-1.5 font-medium text-ink">{format!("Question {} of {}", game.index() + 1, game.total())}</span>
                    <span class="inline-flex items-center gap-1.5 rounded-full border border-line bg-raised px-3 py-1.5 font-semibold text-ink" aria-label={format!("Score: {} out of {} questions", game.score(), game.total())}>
                        <Icon data={IconData::LUCIDE_CHECK} width="14" height="14" />
                        <span>{"Score"}</span>
                        <span>{format!("{} / {}", game.score(), game.total())}</span>
                    </span>
                </div>
            </header>
            <div class="h-1.5 w-full overflow-hidden rounded-full bg-raised" role="progressbar" aria-label="Questions completed" aria-valuemin={"0"} aria-valuemax={game.total().to_string()} aria-valuenow={completed.to_string()}>
                <div class="h-full rounded-full bg-brand-action transition-all" style={progress}></div>
            </div>
            {if *audio_error {html! {<p role="status" class="text-center text-sm text-muted">{"Audio playback is unavailable in this browser."}</p>}} else {Html::default()}}
            <section class="rounded-2xl border border-line bg-surface p-4 shadow-lg sm:p-6" data-time-weaver-question="">
                <div class="mb-5 space-y-2 text-center">
                    <p class="text-xs font-semibold uppercase tracking-wide text-accent">{question_label}</p>
                    <p class="text-2xl font-bold tracking-tight text-ink sm:text-3xl" lang="en" data-time-weaver-prompt="">{prompt}</p>
                    {if let Some(date) = weekday_context {
                        html! {<p class="text-sm text-muted" lang="en">{date}</p>}
                    } else {Html::default()}}
                </div>
                {if is_build {
                    html! {
                        <div class="space-y-4" data-time-weaver-build="">
                            <div class="space-y-2">
                                <p class="text-xs font-semibold uppercase tracking-wide text-muted">{"Your answer"}</p>
                                <div
                                    class="flex min-h-14 flex-wrap items-center justify-center gap-1.5 rounded-xl border border-line bg-raised/60 p-2.5"
                                    aria-label="Assembled Chinese expression; drop or reorder tiles here"
                                    ondragover={on_drag_over.clone()}
                                    ondrop={on_drop_answer}
                                    data-time-weaver-assembled=""
                                >
                                    {chosen_tiles}
                                    {if chosen.is_empty() { html! {<span lang="en" class="text-sm font-normal text-muted">{"Choose or drag tiles here"}</span>} } else {Html::default()}}
                                </div>
                            </div>
                            <div class="grid grid-cols-3 gap-2 sm:grid-cols-5" aria-label="Available answer tiles" ondragover={on_drag_over} ondrop={on_drop_bank}>{for tile_buttons}</div>
                            <div class="flex items-center justify-between gap-3 pt-1">
                                <div class="flex items-center gap-2">
                                    <button type="button" title="Undo last tile" aria-label="Undo last tile" onclick={on_remove} disabled={review.is_some() || chosen.is_empty()} class="flex size-11 items-center justify-center rounded-xl border border-line bg-raised text-ink hover:border-accent disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action"><Icon data={IconData::LUCIDE_ARROW_LEFT} width="19" height="19" /></button>
                                    <button type="button" title="Reset answer" aria-label="Reset answer" onclick={on_reset_answer} disabled={review.is_some() || chosen.is_empty()} class="flex size-11 items-center justify-center rounded-xl border border-line bg-raised text-ink hover:border-accent disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action"><Icon data={IconData::LUCIDE_REFRESH_CW} width="19" height="19" /></button>
                                </div>
                                <button type="button" onclick={on_submit} disabled={review.is_some() || chosen.len() != tile_count} class="rounded-xl bg-brand-action px-5 py-2.5 text-sm font-semibold text-white hover:opacity-90 disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action" data-time-weaver-submit="">{"Check answer"}</button>
                            </div>
                        </div>
                    }
                } else {
                    html! { <div class="grid gap-2" data-time-weaver-choices="">{choice_buttons.unwrap_or_default()}</div> }
                }}
                {if let Some(correct) = review {
                    html! {
                        <section aria-live="polite" data-time-weaver-review="" class={format!("mt-4 flex flex-wrap items-center justify-between gap-3 rounded-xl border p-3 sm:p-4 {}", if correct {"border-emerald-400/50 bg-emerald-400/10"} else {"border-rose-400/50 bg-rose-400/10"})}>
                            <div class="min-w-0 space-y-1">
                                <p class={format!("flex items-center gap-2 text-base font-bold {}", if correct {"text-emerald-300"} else {"text-rose-300"})}>
                                    <Icon data={if correct {IconData::LUCIDE_CHECK} else {IconData::LUCIDE_X}} width="19" height="19" />
                                    {if correct {"Correct!"} else {"Incorrect"}}
                                </p>
                                {if !correct {
                                    html! {<p class="text-sm text-ink">{"Your answer: "}<span lang="zh-Hans" class="font-semibold">{submitted_answer.clone().unwrap_or_default()}</span></p>}
                                } else {Html::default()}}
                                <p class="text-sm text-ink">{if correct {"Answer: "} else {"Correct answer: "}}<span lang="zh-Hans" class="font-semibold">{correct_answer.clone()}</span></p>
                                {if options.show_pinyin {html! {<p class="text-sm text-muted" lang="en">{pinyin::reading(&correct_answer)}</p>}} else {Html::default()}}
                            </div>
                            <button type="button" onclick={on_next} data-time-weaver-next="" class="shrink-0 rounded-xl bg-brand-action px-5 py-2.5 text-sm font-semibold text-white hover:opacity-90 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action">{"Continue"}</button>
                        </section>
                    }
                } else {Html::default()}}
            </section>
        </main>
    }
}
