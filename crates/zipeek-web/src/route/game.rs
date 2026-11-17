use super::Route;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameId {
    IconPairs,
    ProvinceQuest,
    TimeWeaver,
    ToneDetective,
    CloudMatch,
    Crosswords,
    ClassifierMarket,
    ListeningNavigator,
    SentenceRelay,
    MarketBargain,
    RadicalForge,
    DialogueQuest,
}

impl GameId {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::IconPairs => "icon-pairs",
            Self::ProvinceQuest => "province-quest",
            Self::TimeWeaver => "time-weaver",
            Self::ToneDetective => "tone-detective",
            Self::CloudMatch => "cloud-match",
            Self::Crosswords => "crosswords",
            Self::ClassifierMarket => "classifier-market",
            Self::ListeningNavigator => "listening-navigator",
            Self::SentenceRelay => "sentence-relay",
            Self::MarketBargain => "market-bargain",
            Self::RadicalForge => "radical-forge",
            Self::DialogueQuest => "dialogue-quest",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "icon-pairs" => Some(Self::IconPairs),
            "province-quest" => Some(Self::ProvinceQuest),
            "time-weaver" => Some(Self::TimeWeaver),
            "tone-detective" => Some(Self::ToneDetective),
            "cloud-match" => Some(Self::CloudMatch),
            "crosswords" => Some(Self::Crosswords),
            "classifier-market" => Some(Self::ClassifierMarket),
            "listening-navigator" => Some(Self::ListeningNavigator),
            "sentence-relay" => Some(Self::SentenceRelay),
            "market-bargain" => Some(Self::MarketBargain),
            "radical-forge" => Some(Self::RadicalForge),
            "dialogue-quest" => Some(Self::DialogueQuest),
            _ => None,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::IconPairs => "Icon Pairs",
            Self::ProvinceQuest => "China Map Challenge",
            Self::TimeWeaver => "Time Weaver",
            Self::ToneDetective => "Tone Detective",
            Self::CloudMatch => "Cloud Match",
            Self::Crosswords => "Hanzi Crosswords",
            Self::ClassifierMarket => "Classifier Market",
            Self::ListeningNavigator => "Listening Navigator",
            Self::SentenceRelay => "Sentence Relay",
            Self::MarketBargain => "Market Bargain",
            Self::RadicalForge => "Radical Forge",
            Self::DialogueQuest => "Dialogue Quest",
        }
    }

    pub fn route(self) -> Route {
        Route::Game {
            slug: self.slug().to_owned(),
        }
    }
}

#[cfg(test)]
mod game_route_tests {
    use super::{GameId, Route};

    #[test]
    fn every_game_has_a_unique_round_trip_route() {
        let games = [
            GameId::IconPairs,
            GameId::ProvinceQuest,
            GameId::TimeWeaver,
            GameId::ToneDetective,
            GameId::CloudMatch,
            GameId::Crosswords,
            GameId::ClassifierMarket,
            GameId::ListeningNavigator,
            GameId::SentenceRelay,
            GameId::MarketBargain,
            GameId::RadicalForge,
            GameId::DialogueQuest,
        ];
        for (index, game) in games.iter().enumerate() {
            assert_eq!(GameId::from_slug(game.slug()), Some(*game));
            assert_eq!(
                game.route(),
                Route::Game {
                    slug: game.slug().to_owned(),
                }
            );
            assert!(!game.title().is_empty());
            for earlier in &games[..index] {
                assert_ne!(game.slug(), earlier.slug());
            }
        }
        assert_eq!(GameId::from_slug("unknown"), None);
        assert_eq!(GameId::from_slug(""), None);
    }
}
