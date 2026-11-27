mod explore;
mod home;
pub mod learning;
pub mod practice;
mod profile;
mod record;
mod system;
pub mod translation;

pub use explore::{browse, random, word_sets};
pub use home::{Home, SentencesPage};
pub use learning::courses;
pub use learning::crossword;
pub use learning::games;
pub use learning::hanzi_writing;
pub use learning::write_hanzi;
pub use record::RecordPage;
pub use system::{debug, docs, error, not_found, settings, stats, themes};
pub use translation::{sentence_practice, sentence_translator, speaking};

pub use browse::Browse;
pub use courses::{
    AuthoredGrammarLessonPage, CourseLessonPage, CoursesPage, GrammarAssessmentPage,
    GrammarExpandedLessonPage, PinyinCoursePage,
};
pub use crossword::CrosswordPage;
pub use debug::DebugPage;
pub use docs::DocsPage;
pub use error::ErrorPage;
pub use games::{
    CharacterHuntPage, ClassifierMarketPage, CloudMatchPage, DialogueQuestPage, GamesPage,
    IconPairsPage, ListeningNavigatorPage, MarketBargainPage, ProvinceQuestPage, RadicalForgePage,
    SentenceRelayPage, TimeWeaverPage, ToneDetectivePage,
};
pub use hanzi_writing::HanziWriting;
pub use not_found::NotFound;
pub use practice::setup::PracticeSetup;
pub use profile::ProfilePage;
pub use random::Random;
pub use sentence_practice::SentencePracticePage;
pub use sentence_translator::SentenceTranslatorPage;
pub use settings::SettingsPage;
pub use speaking::Speaking;
pub use stats::StatsPage;
pub use themes::ThemesPage;
pub use word_sets::WordSetsPage;
pub use write_hanzi::WriteHanziPage;

pub use practice::image_vocabulary::ImageVocabularyPage;
pub use practice::vocabulary_exercises::VocabularyExercisePage;

pub use practice::listening_choice::ListeningChoicePage;
pub use practice::local_session::LocalPracticeSession;

pub use learning::recognize_hanzi::RecognizeHanziPage;
