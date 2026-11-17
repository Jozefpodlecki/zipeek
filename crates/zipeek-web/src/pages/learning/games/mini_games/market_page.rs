use std::rc::Rc;

use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::mini_games::market::{
    MarketAction, MarketDifficulty, MarketState, MarketSupport, ROUND_COUNT,
};

use crate::hooks::use_app_environment;

#[derive(Clone, Debug, PartialEq)]
struct MarketController(MarketState);

impl Reducible for MarketController {
    type Action = MarketAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = self.0.clone();
        state.apply(action);
        Rc::new(Self(state))
    }
}

fn random_seed() -> Result<u64, String> {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
    Ok(u64::from_le_bytes(bytes))
}

#[derive(Properties, PartialEq)]
struct MarketSetupProps {
    difficulty: MarketDifficulty,
    support: MarketSupport,
    on_support: Callback<MarketSupport>,
    on_difficulty: Callback<MarketDifficulty>,
    on_start: Callback<MouseEvent>,
    error: Option<String>,
}

#[function_component(MarketSetup)]
fn market_setup(props: &MarketSetupProps) -> Html {
    html! {
        <section
            data-game-setup="Market Bargain"
            class="space-y-6 rounded-2xl border border-line bg-surface p-5 sm:p-8"
        >
            <header class="space-y-2">
                <p class="text-sm font-semibold text-accent">{"Learn through play"}</p>
                <h1 class="text-3xl font-bold">{"Market Bargain"}</h1>
                <p class="text-muted">{"Listen to Mandarin prices and choose the matching amount."}</p>
            </header>
            <fieldset class="space-y-3">
                <legend class="text-lg font-semibold">{"Price difficulty"}</legend>
                <div class="grid gap-3 sm:grid-cols-3">
                    {for [MarketDifficulty::Easy, MarketDifficulty::Medium, MarketDifficulty::Hard].into_iter().map(|difficulty| {
                        let selected = props.difficulty == difficulty;
                        let onclick = props.on_difficulty.reform(move |_: MouseEvent| difficulty);
                        let button_classes = format!(
                            "min-h-20 rounded-xl border px-4 py-3 text-left transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent {}",
                            if selected { "border-accent bg-raised" } else { "border-line hover:border-accent" },
                        );
                        let (title, example) = match difficulty {
                            MarketDifficulty::Easy => ("Easy", "¥25.00"),
                            MarketDifficulty::Medium => ("Medium", "¥0.50 / ¥25.30"),
                            MarketDifficulty::Hard => ("Hard", "¥0.25 / ¥25.34"),
                        };
                        html! {
                            <button
                                type="button"
                                key={title}
                                data-market-difficulty={title.to_lowercase()}
                                aria-pressed={selected.to_string()}
                                onclick={onclick}
                                class={button_classes}
                            >
                                <span class="block font-semibold">{title}</span>
                                <span class="block text-sm text-muted">{example}</span>
                            </button>
                        }
                    })}
                </div>
            </fieldset>
            <fieldset class="space-y-3">
                <legend class="text-lg font-semibold">{"Learning support"}</legend>
                <p class="text-sm text-muted">{"Choose how much reading help you want. Price difficulty stays independent."}</p>
                <div class="grid gap-3 sm:grid-cols-3">
                    {for [MarketSupport::Guided, MarketSupport::Reading, MarketSupport::Listening].into_iter().map(|support| {
                        let selected = props.support == support;
                        let onclick = props.on_support.reform(move |_: MouseEvent| support);
                        let (key, description) = match support {
                            MarketSupport::Guided => ("guided", "Hanzi and Pinyin always visible"),
                            MarketSupport::Reading => ("reading", "Hanzi visible; reveal Pinyin"),
                            MarketSupport::Listening => ("listening", "Audio first; reveal text if needed"),
                        };
                        let support_classes = format!(
                            "min-h-24 rounded-xl border px-4 py-3 text-left transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent {}",
                            if selected { "border-accent bg-raised" } else { "border-line hover:border-accent" },
                        );
                        html! {
                            <button
                                type="button"
                                key={key}
                                data-market-support={key}
                                aria-pressed={selected.to_string()}
                                onclick={onclick}
                                class={support_classes}
                            >
                                <span class="block font-semibold">{support.label()}</span>
                                <span class="mt-1 block text-sm text-muted">{description}</span>
                            </button>
                        }
                    })}
                </div>
            </fieldset>
            <div class="flex flex-wrap items-center gap-3 rounded-xl border border-line bg-raised px-4 py-3 text-sm">
                <span class="font-semibold">{"Price vocabulary"}</span>
                <span>{"块 kuài · yuan"}</span>
                <span>{"毛 máo · jiao (0.1 yuan)"}</span>
                <span>{"分 fēn · fen (0.01 yuan)"}</span>
            </div>
            <div class="flex flex-wrap items-center justify-between gap-4 rounded-xl bg-raised p-4">
                <span class="text-sm text-muted">{format!("{ROUND_COUNT} new random prices per game")}</span>
                <button
                    type="button"
                    data-game-start=""
                    onclick={props.on_start.clone()}
                    class="min-h-11 rounded-xl bg-brand-action px-6 font-semibold text-white"
                >
                    {"Start game"}
                </button>
            </div>
            if let Some(error) = &props.error {
                <p role="alert" class="text-sm text-rose-400">{error}</p>
            }
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct MarketPricePromptProps {
    hanzi: String,
    pinyin: String,
    support: MarketSupport,
    revealed: bool,
    on_listen: Callback<MouseEvent>,
    on_reveal: Callback<MouseEvent>,
}

#[function_component(MarketPricePrompt)]
fn market_price_prompt(props: &MarketPricePromptProps) -> Html {
    let show_hanzi = props.support.show_hanzi(props.revealed);
    let show_pinyin = props.support.show_pinyin(props.revealed);
    html! {
        <section data-market-price-prompt="" class="flex flex-col items-center justify-center gap-4 rounded-xl border border-line bg-raised p-5 sm:flex-row">
            <button
                type="button"
                data-market-listen=""
                onclick={props.on_listen.clone()}
                aria-label="Listen to price again"
                title="Listen to price again"
                class="inline-flex size-12 shrink-0 items-center justify-center rounded-xl bg-brand-action text-white focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
                <Icon data={IconData::LUCIDE_VOLUME_2} width="24" height="24" />
            </button>
            <div class="min-w-0 flex-1 space-y-2 text-center sm:text-left">
                if show_hanzi {
                    <p data-market-hanzi="" class="text-2xl font-semibold tracking-wide sm:text-3xl">{props.hanzi.clone()}</p>
                } else {
                    <p data-market-hidden="" class="text-sm text-muted">{"Listen and choose the amount"}</p>
                }
                if show_pinyin {
                    <p data-market-pinyin="" class="text-base text-muted">{props.pinyin.clone()}</p>
                }
                if !props.revealed && props.support != MarketSupport::Guided {
                    <button
                        type="button"
                        data-market-reveal=""
                        onclick={props.on_reveal.clone()}
                        class="rounded-lg border border-line px-3 py-2 text-sm text-accent hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                    >
                        {"Reveal help"}
                    </button>
                }
            </div>
        </section>
    }
}

#[function_component(MarketBargainPage)]
pub fn market_page() -> Html {
    let playing = use_state(|| false);
    let difficulty = use_state(|| MarketDifficulty::Easy);
    let support = use_state(|| MarketSupport::Guided);
    let state = use_reducer(|| MarketController(MarketState::default()));
    let audio_error = use_state(|| None::<String>);
    let setup_error = use_state(|| None::<String>);
    let environment = use_app_environment();

    let on_difficulty = {
        let difficulty = difficulty.clone();
        Callback::from(move |selected: MarketDifficulty| difficulty.set(selected))
    };
    let on_support = {
        let support = support.clone();
        Callback::from(move |selected: MarketSupport| support.set(selected))
    };
    let on_start = {
        let state = state.clone();
        let playing = playing.clone();
        let difficulty = difficulty.clone();
        let support = support.clone();
        let setup_error = setup_error.clone();
        let audio_error = audio_error.clone();
        Callback::from(move |_: MouseEvent| match random_seed() {
            Ok(seed) => {
                state.dispatch(MarketAction::NewGame(*difficulty, *support, seed));
                audio_error.set(None);
                setup_error.set(None);
                playing.set(true);
            }
            Err(error) => setup_error.set(Some(format!("Could not generate prices: {error}"))),
        })
    };
    let on_setup = {
        let playing = playing.clone();
        let audio_error = audio_error.clone();
        Callback::from(move |_: MouseEvent| {
            audio_error.set(None);
            playing.set(false);
        })
    };
    let on_listen = {
        let speech = environment.speech.clone();
        let audio_error = audio_error.clone();
        let prompt = state.0.current().map(|question| question.prompt.clone());
        Callback::from(move |_: MouseEvent| {
            if let Some(prompt) = &prompt {
                audio_error.set(
                    crate::services::practice_audio::speak(&speech, prompt)
                        .err()
                        .map(|_| "Audio playback unavailable".to_owned()),
                );
            }
        })
    };
    let on_reveal = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(MarketAction::RevealHelp))
    };
    let on_continue = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(MarketAction::Continue))
    };
    let on_answer = {
        let state = state.clone();
        Callback::from(move |index: usize| state.dispatch(MarketAction::Answer(index)))
    };
    let on_restart = {
        let state = state.clone();
        let difficulty = difficulty.clone();
        let support = support.clone();
        let setup_error = setup_error.clone();
        let playing = playing.clone();
        Callback::from(move |_: MouseEvent| match random_seed() {
            Ok(seed) => {
                state.dispatch(MarketAction::NewGame(*difficulty, *support, seed));
                setup_error.set(None);
            }
            Err(error) => {
                playing.set(false);
                setup_error.set(Some(format!("Could not generate prices: {error}")));
            }
        })
    };
    let completed = state.0.step.min(state.0.questions.len());
    let total = state.0.questions.len();
    let progress_percent = if total == 0 {
        0
    } else {
        completed * 100 / total
    };

    html! {
        <main class="mx-auto w-full max-w-3xl px-4 py-8">
            if !*playing {
                <MarketSetup
                    difficulty={*difficulty}
                    support={*support}
                    on_support={on_support}
                    on_difficulty={on_difficulty}
                    on_start={on_start}
                    error={(*setup_error).clone()}
                />
            } else {
                <section data-market-bargain="" class="space-y-6 rounded-2xl border border-line bg-surface p-5 sm:p-8">
                    <header class="flex items-center justify-between gap-3">
                        <div class="space-y-1">
                            <h1 class="text-2xl font-bold">{"Market Bargain"}</h1>
                            <p class="text-sm text-muted">{format!("{} · {}", state.0.difficulty.label(), state.0.support.label())}</p>
                        </div>
                        <button
                            type="button"
                            data-market-settings=""
                            onclick={on_setup}
                            title="Change settings"
                            aria-label="Change settings"
                            class="inline-flex size-11 items-center justify-center rounded-xl border border-line text-muted transition-colors hover:border-accent hover:text-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
                        >
                            <Icon data={IconData::LUCIDE_SETTINGS} width="20" height="20" />
                        </button>
                    </header>
                    <div class="space-y-2">
                        <div class="flex items-center justify-between gap-3 text-sm text-muted">
                            <span data-game-progress="">{if state.0.complete() { format!("{total} of {total} complete") } else { format!("Round {} of {total}", completed + 1) }}</span>
                            <span>{format!("{} mistakes", state.0.mistakes)}</span>
                        </div>
                        <div
                            role="progressbar"
                            aria-label="Prices completed"
                            aria-valuemin="0"
                            aria-valuemax={total.to_string()}
                            aria-valuenow={completed.to_string()}
                            class="h-2.5 w-full overflow-hidden rounded-full bg-raised"
                        >
                            <div
                                class="h-full rounded-full bg-brand-action transition-[width] duration-300 motion-reduce:transition-none"
                                style={format!("width: {progress_percent}%")}
                            />
                        </div>
                    </div>
                    if state.0.complete() {
                        <div data-game-complete="" class="space-y-4 text-center">
                            <h2 class="text-xl font-bold">{"Market complete"}</h2>
                            <p data-market-assisted-count="" class="text-sm text-muted">{format!("{} answers used revealed help", state.0.assisted_rounds)}</p>
                            <button
                                type="button"
                                onclick={on_restart}
                                class="rounded-xl bg-brand-action px-5 py-3 text-white"
                            >
                                {"Play again"}
                            </button>
                        </div>
                    } else if let Some(question) = state.0.current() {
                        <p class="text-sm text-muted">{"Listen to the spoken price, then choose the matching amount."}</p>
                        <MarketPricePrompt
                            hanzi={question.prompt.clone()}
                            pinyin={question.pinyin.clone()}
                            support={state.0.support}
                            revealed={state.0.help_revealed || state.0.awaiting_continue}
                            on_listen={on_listen}
                            on_reveal={on_reveal}
                        />
                        if let Some(error) = &*audio_error {
                            <p role="alert" class="text-rose-400">{error.clone()}</p>
                        }
                        <div class="grid gap-3 sm:grid-cols-3">
                            {for question.choices.iter().enumerate().map(|(index, choice)| {
                                let onclick = on_answer.reform(move |_: MouseEvent| index);
                                html! {
                                    <button
                                        type="button"
                                        key={index}
                                        data-market-choice={index.to_string()}
                                        disabled={state.0.awaiting_continue || state.0.rejected.contains(&index)}
                                        onclick={onclick}
                                        class={if state.0.awaiting_continue && index == question.answer {
                                            "min-h-20 rounded-xl border border-emerald-400 bg-emerald-950/40 px-3 text-2xl font-bold text-emerald-200"
                                        } else {
                                            "min-h-20 rounded-xl border border-line bg-raised px-3 text-2xl font-bold hover:border-accent disabled:opacity-40"
                                        }}
                                    >
                                        {choice.clone()}
                                    </button>
                                }
                            })}
                        </div>
                        if state.0.awaiting_continue {
                            <div data-market-review="" role="status" class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-emerald-500/40 bg-emerald-950/30 p-4">
                                <div class="space-y-1">
                                    <p class="font-semibold text-emerald-300">{format!("Correct: {}", question.choices[question.answer])}</p>
                                    <p class="text-sm text-muted">{format!("{} · {}", question.prompt, question.pinyin)}</p>
                                </div>
                                <button type="button" data-market-continue="" onclick={on_continue} class="min-h-11 rounded-xl bg-brand-action px-5 py-2 font-semibold text-white">
                                    {"Continue"}
                                </button>
                            </div>
                        }
                        if !state.0.awaiting_continue && !state.0.rejected.is_empty() {
                            <p role="status" class="text-amber-400">{"Try another amount."}</p>
                        }
                    }
                </section>
            }
        </main>
    }
}
