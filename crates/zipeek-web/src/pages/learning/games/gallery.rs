use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use crate::route::{GameId, Route};

const PAGE_CAPACITY: usize = 9;
const CARD_CLASS: &str = "group relative flex min-h-28 flex-col gap-3 rounded-2xl border border-line bg-surface p-4 text-ink no-underline transition-colors hover:border-accent hover:bg-raised focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameCardLayout {
    Regular,
    Featured,
}

impl GameCardLayout {
    fn slots(self) -> usize {
        match self {
            Self::Regular => 1,
            Self::Featured => 4,
        }
    }
}

struct GameEntry {
    title: &'static str,
    description: &'static str,
    icon: IconData,
    route: Route,
    layout: GameCardLayout,
}

fn available_games() -> Vec<GameEntry> {
    vec![
        GameEntry {
            title: "China Map Challenge",
            description: "Explore China's provinces, capitals and prefectures on an interactive map.",
            icon: IconData::LUCIDE_MAP,
            route: GameId::ProvinceQuest.route(),
            layout: GameCardLayout::Featured,
        },
        GameEntry {
            title: "Icon Pairs",
            description: "Match meaningful icons to related Chinese words in six-pair rounds.",
            icon: IconData::LUCIDE_GRID,
            layout: GameCardLayout::Regular,
            route: GameId::IconPairs.route(),
        },
        GameEntry {
            title: "Time Weaver",
            description: "Build dates in Hanzi, identify weekdays and choose between three Chinese answers.",
            icon: IconData::LUCIDE_CLOCK,
            layout: GameCardLayout::Regular,
            route: GameId::TimeWeaver.route(),
        },
        GameEntry {
            title: "Tone Detective",
            description: "Listen to Mandarin syllables and identify the four tones.",
            icon: IconData::LUCIDE_EAR,
            layout: GameCardLayout::Regular,
            route: GameId::ToneDetective.route(),
        },
        GameEntry {
            title: "Cloud Match",
            description: "Pair Hanzi, Pinyin and meanings as clouds move across the sky.",
            icon: IconData::LUCIDE_CLOUD,
            layout: GameCardLayout::Regular,
            route: GameId::CloudMatch.route(),
        },
        GameEntry {
            title: "Hanzi Crosswords",
            description: "Build connected Chinese words using clues and character tiles.",
            icon: IconData::LUCIDE_GRID,
            layout: GameCardLayout::Regular,
            route: GameId::Crosswords.route(),
        },
        GameEntry {
            title: "Classifier Market",
            description: "Practise measure words with authored questions filtered by your vocabulary.",
            icon: IconData::LUCIDE_PACKAGE,
            layout: GameCardLayout::Regular,
            route: GameId::ClassifierMarket.route(),
        },
        GameEntry {
            title: "Listening Navigator",
            description: "Follow spoken directions in vocabulary-filtered routes around a city.",
            icon: IconData::LUCIDE_MAP,
            layout: GameCardLayout::Regular,
            route: GameId::ListeningNavigator.route(),
        },
        GameEntry {
            title: "Sentence Relay",
            description: "Reorder authored sentences using words from your chosen vocabulary.",
            icon: IconData::LUCIDE_PUZZLE,
            layout: GameCardLayout::Featured,
            route: GameId::SentenceRelay.route(),
        },
        GameEntry {
            title: "Market Bargain",
            description: "Listen to prices in Mandarin and choose the correct amount.",
            icon: IconData::LUCIDE_PACKAGE,
            layout: GameCardLayout::Regular,
            route: GameId::MarketBargain.route(),
        },
        GameEntry {
            title: "Radical Forge",
            description: "Identify the building blocks of Chinese characters.",
            icon: IconData::LUCIDE_GRID,
            layout: GameCardLayout::Regular,
            route: GameId::RadicalForge.route(),
        },
        GameEntry {
            title: "Dialogue Quest",
            description: "Make conversational choices in AI-generated Mandarin dialogues.",
            icon: IconData::LUCIDE_EAR,
            layout: GameCardLayout::Regular,
            route: GameId::DialogueQuest.route(),
        },
    ]
}

fn paginate_games(games: Vec<GameEntry>) -> Vec<Vec<GameEntry>> {
    let mut pages = vec![Vec::new()];
    let mut used = 0;
    for game in games {
        let slots = game.layout.slots();
        if used + slots > PAGE_CAPACITY {
            pages.push(Vec::new());
            used = 0;
        }
        pages
            .last_mut()
            .expect("gallery has an initial page")
            .push(game);
        used += slots;
    }
    pages
}

#[derive(Properties, PartialEq)]
struct GameCardContentProps {
    title: &'static str,
    description: &'static str,
    icon: IconData,
}

#[function_component(GameCardContent)]
fn game_card_content(props: &GameCardContentProps) -> Html {
    html! {
        <>
            <span class="grid size-10 place-items-center rounded-xl border border-line bg-raised text-accent">
                <Icon data={props.icon} width="24px" height="24px" />
            </span>
            <div class="space-y-1.5">
                <h2 class="text-lg font-bold">{props.title}</h2>
                <p class="text-sm leading-snug text-muted">{props.description}</p>
            </div>
        </>
    }
}

#[function_component(GamesPage)]
pub fn games_page() -> Html {
    let page = use_state(|| 0_usize);
    let pages = paginate_games(available_games());
    let page_count = pages.len();
    let page_index = (*page).min(page_count.saturating_sub(1));
    let previous_page = {
        let page = page.clone();
        Callback::from(move |_| page.set(page_index.saturating_sub(1)))
    };
    let next_page = {
        let page = page.clone();
        Callback::from(move |_| page.set((page_index + 1).min(page_count - 1)))
    };
    let visible_games = pages.into_iter().nth(page_index).unwrap_or_default()
        .into_iter().map(|game| {
            let featured = game.layout == GameCardLayout::Featured;
            let classes = if featured {
                format!("{CARD_CLASS} md:col-span-2 md:row-span-2 min-h-64 justify-between")
            } else { CARD_CLASS.to_owned() };
            html! {
                <Link<Route> key={game.title} to={game.route} classes={Classes::from(classes)}>
                    <GameCardContent title={game.title} description={game.description} icon={game.icon.clone()} />
                    if featured {
                        <div data-game-featured={game.title} class="flex flex-1 items-center justify-center rounded-xl border border-line/60 bg-raised/40" aria-hidden="true">
                            <Icon data={game.icon} width="88px" height="88px" />
                        </div>
                    }
                </Link<Route>>
            }
        }).collect::<Html>();
    html! {
        <main data-component="games-gallery" class="mx-auto flex w-full max-w-5xl flex-col gap-4 py-3">
            <header class="space-y-1">
                <p class="text-sm font-semibold text-accent">{"Learn through play"}</p>
                <p class="text-sm text-muted">
                    {"Play curated rounds or choose HSK and Word Set vocabulary in supported games."}
                </p>
            </header>
            <section class="grid auto-rows-fr grid-flow-row-dense gap-3 md:grid-cols-2 xl:grid-cols-3" aria-label="Available games">
                {visible_games}
            </section>
            <nav aria-label="Games pages" class="flex items-center justify-center gap-4 py-1">
                <button
                    type="button"
                    aria-label="Previous games page"
                    class="grid size-10 place-items-center rounded-xl border border-line bg-surface text-ink transition-colors hover:bg-raised disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={page_index == 0}
                    onclick={previous_page}
                >
                    <Icon data={IconData::LUCIDE_CHEVRON_LEFT} width="20px" height="20px"/>
                </button>
                <span class="min-w-24 text-center text-sm text-muted" aria-live="polite">
                    {format!("Page {} of {}", page_index + 1, page_count)}
                </span>
                <button
                    type="button"
                    aria-label="Next games page"
                    class="grid size-10 place-items-center rounded-xl border border-line bg-surface text-ink transition-colors hover:bg-raised disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={page_index + 1 == page_count}
                    onclick={next_page}
                >
                    <Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="20px" height="20px"/>
                </button>
            </nav>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn featured_tiles_use_four_grid_slots_on_each_page() {
        let pages = paginate_games(available_games());
        assert!(pages.len() >= 2);
        assert!(
            pages
                .iter()
                .flatten()
                .any(|game| game.title == "Icon Pairs")
        );
        assert!(
            pages
                .iter()
                .flatten()
                .any(|game| game.title == "Time Weaver")
        );
        assert_eq!(pages[0][0].title, "China Map Challenge");
        assert_eq!(pages[0][0].layout, GameCardLayout::Featured);
        assert!(
            pages
                .iter()
                .flatten()
                .any(|game| game.title == "Sentence Relay"
                    && game.layout == GameCardLayout::Featured)
        );
    }

    #[test]
    fn no_gallery_page_exceeds_grid_capacity() {
        for page in paginate_games(available_games()) {
            assert!(page.iter().map(|game| game.layout.slots()).sum::<usize>() <= PAGE_CAPACITY);
        }
    }
}
