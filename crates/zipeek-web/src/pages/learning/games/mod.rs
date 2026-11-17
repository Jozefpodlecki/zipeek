mod character_hunt;
mod cloud_match;
mod gallery;
mod icon_pairs;
mod mini_games;
mod province_quest;
mod time_weaver;
mod tone_detective;
pub use character_hunt::{CharacterHuntPage, ResumableCharacterHunt};
mod pairs;
pub use cloud_match::CloudMatchPage;
pub use gallery::GamesPage;
pub use icon_pairs::IconPairsPage;
pub use mini_games::{
    ClassifierMarketPage, DialogueQuestPage, ListeningNavigatorPage, MarketBargainPage,
    RadicalForgePage, SentenceRelayPage,
};
pub use province_quest::ProvinceQuestPage;
pub use tone_detective::ToneDetectivePage;

pub use time_weaver::TimeWeaverPage;
