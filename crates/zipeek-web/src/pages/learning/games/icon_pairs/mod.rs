mod catalogue;
mod round;

use std::collections::BTreeSet;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::Icon;

use self::catalogue::{IconWord, all_words, catalogue, icon_count};
use self::round::{PAIRS_PER_ROUND, Round, Tile};
use crate::hooks::use_dictionary;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Curated,
    Hsk,
}

#[derive(Clone)]
enum BoardStatus {
    Ready(Round),
    Loading,
    Error(String),
}

fn filter_attested(pool: Vec<IconWord>, hsk_words: &BTreeSet<String>) -> Vec<IconWord> {
    pool.into_iter()
        .filter(|word| hsk_words.contains(word.hanzi))
        .collect()
}

#[function_component(IconPairsPage)]
pub fn icon_pairs_page() -> Html {
    let dictionary = use_dictionary();
    let source = use_state(|| Source::Curated);
    let standard = use_state(|| 3_u8);
    let level = use_state(|| 1_u8);
    let board = use_state(|| None::<BoardStatus>);
    let active_pool = use_state(Vec::<IconWord>::new);
    let seed = use_state(|| 17_u64);

    let start = {
        let dictionary = dictionary.clone();
        let source = source.clone();
        let standard = standard.clone();
        let level = level.clone();
        let board = board.clone();
        let active_pool = active_pool.clone();
        let seed = seed.clone();
        Callback::from(move |_: MouseEvent| {
            let pool = all_words();
            let next_seed = seed.wrapping_add(0x9e37_79b9);
            seed.set(next_seed);
            if *source == Source::Curated {
                active_pool.set(pool.clone());
                board.set(Some(BoardStatus::Ready(Round::new(pool, next_seed, 1))));
                return;
            }
            board.set(Some(BoardStatus::Loading));
            let board = board.clone();
            let active_pool = active_pool.clone();
            let dictionary = dictionary.clone();
            let standard = *standard;
            let level = *level;
            spawn_local(async move {
                let result = async {
                    let database = dictionary.database().map_err(|error| error.to_string())?;
                    let words = database
                        .game_words_for_hsk(standard, level)
                        .await
                        .map_err(|error| error.to_string())?;
                    let allowed = words
                        .into_iter()
                        .map(|word| word.simplified)
                        .collect::<BTreeSet<_>>();
                    Ok::<Vec<IconWord>, String>(filter_attested(pool, &allowed))
                }
                .await;
                match result {
                    Ok(pool) if pool.iter().map(|word| word.icon_key).collect::<BTreeSet<_>>().len() >= PAIRS_PER_ROUND => {
                        let round = Round::new(pool.clone(), next_seed, 1);
                        active_pool.set(pool);
                        board.set(Some(BoardStatus::Ready(round)));
                    }
                    Ok(_) => board.set(Some(BoardStatus::Error(
                        "Fewer than six distinct icon–word pairs were found at this HSK level. Choose another level or the full catalogue.".to_owned()
                    ))),
                    Err(error) => board.set(Some(BoardStatus::Error(format!("Could not load HSK vocabulary: {error}")))),
                }
            });
        })
    };
    let setup = {
        let board = board.clone();
        Callback::from(move |_: MouseEvent| board.set(None))
    };
    let on_next = {
        let board = board.clone();
        let active_pool = active_pool.clone();
        let seed = seed.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(BoardStatus::Ready(current)) = &*board {
                let next_seed = seed.wrapping_add(0x9e37_79b9);
                seed.set(next_seed);
                board.set(Some(BoardStatus::Ready(Round::new(
                    (*active_pool).clone(),
                    next_seed,
                    current.round_number + 1,
                ))));
            }
        })
    };
    html! {
        <main data-component="icon-pairs-game" class="mx-auto flex w-full max-w-5xl min-w-0 flex-col gap-3 px-3 py-3 sm:px-4">
            <header class="flex items-start justify-between gap-3">
                <div class="space-y-1">
                    <h1 class="text-xl font-bold text-ink">{"Icon Pairs"}</h1>
                    <p class="text-xs text-muted">{"Match related concepts. The icon need not describe the Chinese word literally."}</p>
                </div>
                if board.is_some() {
                    <button type="button" onclick={setup} class="shrink-0 rounded-lg border border-line bg-surface px-3 py-2 text-sm font-semibold hover:bg-raised">{"Settings"}</button>
                }
            </header>
            {match &*board {
                None => html! {
                    <section class="mx-auto w-full max-w-2xl space-y-5 rounded-2xl border border-line bg-surface p-5 sm:p-7" aria-label="Icon Pairs setup">
                        <div class="space-y-1">
                            <h2 class="text-lg font-semibold text-ink">{"Choose vocabulary"}</h2>
                            <p class="text-sm text-muted">{"Use the curated mappings or restrict matches to words attested in an HSK level."}</p>
                        </div>
                        <div class="flex flex-wrap gap-2" role="group" aria-label="Vocabulary source">
                            {for [(Source::Curated, "All mapped words"), (Source::Hsk, "HSK vocabulary")].into_iter().map(|(value, label)| {
                                let selected = *source == value;
                                let source = source.clone();
                                let onclick = Callback::from(move |_: MouseEvent| source.set(value));
                                html! { <button type="button" aria-pressed={selected.to_string()} {onclick}
                                    class={format!("rounded-lg border px-4 py-2 text-sm font-semibold {}", if selected {"border-accent bg-brand-action text-white"} else {"border-line hover:bg-raised"})}>{label}</button> }
                            })}
                        </div>
                        if *source == Source::Hsk {
                            <div class="space-y-3">
                                <p class="text-sm font-semibold">{"HSK standard"}</p>
                                <div class="flex gap-2" role="group" aria-label="HSK standard">
                                    {for [2_u8, 3_u8].into_iter().map(|value| {
                                        let chosen = *standard == value;
                                        let standard = standard.clone();
                                        let level = level.clone();
                                        let onclick = Callback::from(move |_: MouseEvent| { standard.set(value); level.set(1); });
                                        html! { <button type="button" {onclick} aria-pressed={chosen.to_string()}
                                            class={format!("rounded-lg border px-4 py-2 text-sm {}", if chosen {"border-accent bg-brand-action text-white"} else {"border-line"})}>{format!("HSK {value}.0")}</button> }
                                    })}
                                </div>
                                <p class="text-sm font-semibold">{"Level"}</p>
                                <div class="flex flex-wrap gap-2" role="group" aria-label="HSK level">
                                    {for (1..=if *standard == 2 {6} else {7}).map(|value| {
                                        let chosen = *level == value;
                                        let level = level.clone();
                                        let onclick = Callback::from(move |_: MouseEvent| level.set(value));
                                        html! { <button type="button" {onclick} aria-pressed={chosen.to_string()}
                                            class={format!("min-w-11 rounded-lg border px-3 py-2 text-sm {}", if chosen {"border-accent bg-brand-action text-white"} else {"border-line"})}>{if value == 7 {"7–9".to_owned()} else {value.to_string()}}</button> }
                                    })}
                                </div>
                            </div>
                        }
                        <p class="text-xs text-muted">{format!("{} mapped words across {} icons ({} associations). Each round uses {PAIRS_PER_ROUND} unique icons and words.", catalogue().len(), icon_count(), all_words().len())}</p>
                        <button type="button" onclick={start} class="rounded-xl bg-brand-action px-6 py-3 font-semibold text-white hover:bg-brand-hover">{"Start matching"}</button>
                    </section>
                },
                Some(BoardStatus::Loading) => html! { <section class="mx-auto w-full max-w-lg rounded-xl border border-line bg-surface p-8 text-center" role="status"><span class="loading-spinner" aria-hidden="true" /><p class="mt-3 text-sm">{"Checking mapped words against HSK vocabulary…"}</p></section> },
                Some(BoardStatus::Error(message)) => html! { <section class="mx-auto w-full max-w-lg rounded-xl border border-line bg-surface p-6" role="alert"><h2 class="font-semibold">{"Unable to start this HSK level"}</h2><p class="mt-2 text-sm text-muted">{message}</p><button type="button" onclick={start.clone()} class="mt-4 rounded-lg bg-brand-action px-4 py-2 text-white">{"Try again"}</button></section> },
                Some(BoardStatus::Ready(round)) => html! {
                    <section class="flex min-w-0 flex-col gap-3" aria-label="Icon Pairs board">
                        <div class="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-line bg-surface px-4 py-2 text-sm" aria-label="Game progress">
                            <span>{format!("Round {} · {} pairs", round.round_number, round.words.len())}</span>
                            <span data-icon-pairs-score="">{format!("{} matched · {} attempts", round.matched.len(), round.attempts)}</span>
                        </div>
                        <div class="grid min-w-0 grid-cols-3 gap-2 sm:grid-cols-4 sm:gap-3" role="group" aria-label="Mixed icon and Chinese word tiles">
                            {for round.tiles.iter().map(|tile| {
                                let tile = tile.clone();
                                let item = &round.words[tile.word_index];
                                let matched = round.matched.contains(&tile.word_index);
                                let selected = round.selected.as_ref() == Some(&tile);
                                let word_index = tile.word_index;
                                let is_icon = tile.icon;
                                let onclick = {
                                    let board = board.clone();
                                    Callback::from(move |_: MouseEvent| {
                                        if let Some(BoardStatus::Ready(old)) = &*board {
                                            let mut next = old.clone();
                                            next.choose(Tile {word_index, icon: is_icon});
                                            board.set(Some(BoardStatus::Ready(next)));
                                        }
                                    })
                                };
                                html! {
                                    <button type="button" key={format!("{}-{word_index}", if is_icon {"i"} else {"w"})}
                                        data-icon-pairs-tile="" data-icon-choice={is_icon.then(|| word_index.to_string())}
                                        data-hanzi-choice={(!is_icon).then(|| word_index.to_string())}
                                        aria-pressed={selected.to_string()} disabled={matched} {onclick}
                                        aria-label={if is_icon {format!("{} icon", item.icon_key.replace("lucide:", "").replace('-', " "))} else {item.hanzi.to_owned()}}
                                        class={format!("flex min-h-20 min-w-0 flex-col items-center justify-center gap-1 rounded-xl border px-2 py-2 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action sm:min-h-24 {}", if matched {"border-accent bg-accent-soft opacity-40"} else if selected {"border-accent bg-accent-soft ring-2 ring-accent"} else {"border-line bg-surface hover:border-accent hover:bg-raised"})}>
                                        if is_icon { <Icon data={item.icon.clone()} width="32px" height="32px" /> }
                                        else { <span lang="zh-Hans" class="text-xl font-semibold text-ink sm:text-2xl">{item.hanzi}</span> }
                                    </button>
                                }
                            })}
                        </div>
                        <div class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-line bg-surface px-4 py-3">
                            <p class="text-sm text-muted" role="status" aria-live="polite">{&round.feedback}</p>
                            if round.complete() { <button type="button" onclick={on_next} class="rounded-lg bg-brand-action px-4 py-2 text-sm font-semibold text-white">{"Next round"}</button> }
                        </div>
                    </section>
                }
            }}
            <p class="text-xs text-muted">{"Local practice. Icon associations are curated and are not automatically treated as HSK mastery."}</p>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hsk_filter_uses_dictionary_attestation() {
        let words = all_words();
        let allowed = BTreeSet::from(["太阳".to_owned(), "家".to_owned()]);
        let selected = filter_attested(words, &allowed);
        assert_eq!(selected.len(), 2);
    }
}
