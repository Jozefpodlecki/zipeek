use yew::*;
use yew_router::{Routable, prelude::Redirect};

use crate::{
    components::{BrowseProvider, DatabaseBoundary, WordSetsProvider},
    pages::courses::VocabularyCoursePage,
    pages::*,
};

mod game;
pub use game::GameId;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/sentences")]
    Sentences,
    #[at("/record")]
    Record,
    #[at("/random")]
    Random,
    #[at("/practice")]
    PracticeSetup,
    #[at("/practice/session/:id")]
    LocalPracticeSession { id: String },
    #[at("/games")]
    Games,
    #[at("/games/:slug")]
    Game { slug: String },
    #[at("/recognize-hanzi")]
    RecognizeHanzi,
    #[at("/write-hanzi")]
    WriteHanzi,
    #[at("/translate-sentence")]
    TranslateSentence,
    #[at("/browse")]
    Browse,
    #[at("/themes")]
    Themes,
    #[at("/courses")]
    Courses,
    #[at("/courses/grammar")]
    GrammarCourse,
    #[at("/courses/vocabulary")]
    VocabularyCourse,
    #[at("/courses/grammar/topic/:region")]
    GrammarExpanded { region: usize },
    #[at("/courses/grammar/assessment/:course")]
    GrammarAssessment { course: String },
    #[at("/courses/grammar/:slug")]
    GrammarLesson { slug: String },
    #[at("/courses/pinyin")]
    PinyinCourse,
    #[at("/courses/pinyin/:lesson")]
    CourseLesson { lesson: String },
    #[at("/stats")]
    Stats,
    #[at("/settings")]
    Settings,
    #[at("/debug")]
    Debug,
    #[at("/docs")]
    Docs,
    #[at("/docs/:file")]
    DocsArticle { file: String },
    #[at("/profile")]
    Profile,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    let without_dictionary = matches!(
        routes,
        Route::NotFound
            | Route::Sentences
            | Route::TranslateSentence
            | Route::Settings
            | Route::Stats
            | Route::Debug
            | Route::Docs
            | Route::DocsArticle { .. }
            | Route::Games
            | Route::Game { .. }
            | Route::Themes
            | Route::Courses
            | Route::GrammarCourse
            | Route::VocabularyCourse
            | Route::PinyinCourse
            | Route::CourseLesson { .. }
            | Route::GrammarLesson { .. }
            | Route::GrammarExpanded { .. }
            | Route::GrammarAssessment { .. }
    );
    let page = match routes {
        Route::Home => html! { <Home /> },
        Route::Sentences => html! { <SentencesPage /> },
        Route::Random => html! { <Random /> },
        Route::Record => html! { <RecordPage /> },
        Route::PracticeSetup => html! { <PracticeSetup /> },
        Route::LocalPracticeSession { id } => {
            html! { <LocalPracticeSession key={id.clone()} id={id.clone()} /> }
        }
        Route::RecognizeHanzi => {
            html! { <WordSetsProvider><RecognizeHanziPage /></WordSetsProvider> }
        }
        Route::WriteHanzi => html! { <WordSetsProvider>
            <WriteHanziPage />
        </WordSetsProvider> },
        Route::Games => html! { <GamesPage /> },
        Route::Game { slug } => match GameId::from_slug(&slug) {
            Some(GameId::ProvinceQuest) => html! { <ProvinceQuestPage /> },
            Some(GameId::IconPairs) => html! { <IconPairsPage /> },
            Some(GameId::TimeWeaver) => html! { <TimeWeaverPage /> },
            Some(GameId::ToneDetective) => html! { <ToneDetectivePage /> },
            Some(GameId::CloudMatch) => {
                html! { <WordSetsProvider><CloudMatchPage /></WordSetsProvider> }
            }
            Some(GameId::Crosswords) => html! { <CrosswordPage /> },
            Some(GameId::ClassifierMarket) => {
                html! { <WordSetsProvider><ClassifierMarketPage /></WordSetsProvider> }
            }
            Some(GameId::ListeningNavigator) => {
                html! { <WordSetsProvider><ListeningNavigatorPage /></WordSetsProvider> }
            }
            Some(GameId::SentenceRelay) => {
                html! { <WordSetsProvider><SentenceRelayPage /></WordSetsProvider> }
            }
            Some(GameId::MarketBargain) => html! { <MarketBargainPage /> },
            Some(GameId::RadicalForge) => html! { <RadicalForgePage /> },
            Some(GameId::DialogueQuest) => {
                html! { <WordSetsProvider><DialogueQuestPage /></WordSetsProvider> }
            }
            None => html! { <NotFound /> },
        },
        Route::TranslateSentence => html! { <SentenceTranslatorPage /> },
        Route::Browse => html! {
            <BrowseProvider>
                <WordSetsProvider>
                    <Browse />
                </WordSetsProvider>
            </BrowseProvider>
        },
        Route::Themes => html! { <Redirect<Route> to={Route::Settings} /> },
        Route::Courses => html! { <Redirect<Route> to={Route::VocabularyCourse} /> },
        Route::GrammarCourse => html! { <CoursesPage /> },
        Route::VocabularyCourse => {
            html! { <WordSetsProvider><VocabularyCoursePage /></WordSetsProvider> }
        }
        Route::GrammarLesson { slug } => {
            html! { <AuthoredGrammarLessonPage key={slug.clone()} slug={slug.clone()} /> }
        }
        Route::GrammarExpanded { region } => {
            html! { <GrammarExpandedLessonPage region={region} /> }
        }
        Route::GrammarAssessment { course } => {
            html! { <GrammarAssessmentPage key={course.clone()} course={course.clone()} /> }
        }
        Route::PinyinCourse => html! { <PinyinCoursePage /> },
        Route::CourseLesson { lesson } => html! { <CourseLessonPage
            lesson={lesson}
        /> },
        Route::Settings => html! { <SettingsPage /> },
        Route::Stats => html! { <StatsPage /> },
        Route::Debug => {
            if cfg!(debug_assertions) {
                html! { <DebugPage /> }
            } else {
                html! { <NotFound /> }
            }
        }
        Route::Docs | Route::DocsArticle { .. } => html! { <DocsPage /> },
        Route::Profile => html! {
            <WordSetsProvider>
                <ProfilePage />
            </WordSetsProvider>
        },
        Route::NotFound => html! { <NotFound /> },
    };
    if without_dictionary {
        html! {
            <>
                {page}
            </>
        }
    } else {
        html! {
            <DatabaseBoundary>
                {page}
            </DatabaseBoundary>
        }
    }
}

impl Route {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Sentences => "Sentences",
            Self::Random => "Random",
            Self::Record => "Record",
            Self::PracticeSetup => "Practice",
            Self::LocalPracticeSession { .. } => "Practice session",
            Self::Browse => "Browse",
            Self::Profile => "Profile",
            Self::Themes => "Themes",
            Self::Courses => "Courses",
            Self::GrammarCourse => "Grammar courses",
            Self::VocabularyCourse => "Vocabulary courses",
            Self::PinyinCourse => "Pinyin & Pronunciation",
            Self::CourseLesson { .. } => "Pinyin lesson",
            Self::GrammarLesson { slug } => {
                crate::pages::courses::authored_grammar_title_for_slug(slug)
                    .unwrap_or("Grammar lesson")
            }
            Self::GrammarExpanded { .. } => "Grammar lesson",
            Self::GrammarAssessment { .. } => "Grammar assessment",
            Self::Settings => "Settings",
            Self::Stats => "Statistics",
            Self::Debug => "Debug",
            Self::Docs | Self::DocsArticle { .. } => "Documentation",
            Self::TranslateSentence => "Translate Sentence",
            Self::RecognizeHanzi => "Recognize Hanzi",
            Self::WriteHanzi => "Write Hanzi",
            Self::Games => "Games",
            Self::Game { slug } => GameId::from_slug(slug).map_or("Page not found", GameId::title),
            Self::NotFound => "Page not found",
        }
    }
}
