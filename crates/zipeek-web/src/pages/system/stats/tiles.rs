use std::collections::BTreeSet;

use yew::prelude::*;

use super::{
    data::{SessionRecord, WordRecord},
    fixture::EvidenceStatus,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum TileId {
    ReviewWords,
    UnrecordedWords,
    RecognitionRecords,
    ListeningRecords,
    SpeakingRecords,
    SentenceRecords,
    WritingRecords,
    RecognitionReview,
    ListeningReview,
    SpeakingReview,
    SentenceReview,
    WritingReview,
    ActiveDates,
    CompletedSessions,
    PracticeSessions,
    GameSessions,
    ScoredSessions,
    UnscoredSessions,
    PracticeTypes,
    GameTypes,
}

impl TileId {
    pub const ALL: [Self; 20] = [
        Self::ReviewWords,
        Self::UnrecordedWords,
        Self::RecognitionRecords,
        Self::ListeningRecords,
        Self::SpeakingRecords,
        Self::SentenceRecords,
        Self::WritingRecords,
        Self::RecognitionReview,
        Self::ListeningReview,
        Self::SpeakingReview,
        Self::SentenceReview,
        Self::WritingReview,
        Self::ActiveDates,
        Self::CompletedSessions,
        Self::PracticeSessions,
        Self::GameSessions,
        Self::ScoredSessions,
        Self::UnscoredSessions,
        Self::PracticeTypes,
        Self::GameTypes,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::ReviewWords => "Words flagged for review",
            Self::UnrecordedWords => "Words without activity markers",
            Self::RecognitionRecords => "Recognition activity",
            Self::ListeningRecords => "Listening activity",
            Self::SpeakingRecords => "Speaking activity",
            Self::SentenceRecords => "Sentence-use activity",
            Self::WritingRecords => "Writing activity",
            Self::RecognitionReview => "Recognition review markers",
            Self::ListeningReview => "Listening review markers",
            Self::SpeakingReview => "Speaking review markers",
            Self::SentenceReview => "Sentence-use review markers",
            Self::WritingReview => "Writing review markers",
            Self::ActiveDates => "Dates with activity",
            Self::CompletedSessions => "Sample sessions",
            Self::PracticeSessions => "Practice sessions",
            Self::GameSessions => "Game sessions",
            Self::ScoredSessions => "Sessions with sample outcomes",
            Self::UnscoredSessions => "Sessions without scored outcomes",
            Self::PracticeTypes => "Distinct practice types",
            Self::GameTypes => "Distinct game types",
        }
    }

    pub const fn group(self) -> &'static str {
        match self {
            Self::ReviewWords | Self::UnrecordedWords => "Vocabulary",
            Self::RecognitionRecords
            | Self::ListeningRecords
            | Self::SpeakingRecords
            | Self::SentenceRecords
            | Self::WritingRecords
            | Self::RecognitionReview
            | Self::ListeningReview
            | Self::SpeakingReview
            | Self::SentenceReview
            | Self::WritingReview => "Activity coverage",
            _ => "Practice history",
        }
    }

    pub const fn explanation(self) -> &'static str {
        match self {
            Self::ReviewWords => {
                "Words with one or more review markers, not a proficiency judgement."
            }
            Self::UnrecordedWords => "Words without an activity or review marker in any category.",
            Self::RecognitionRecords
            | Self::ListeningRecords
            | Self::SpeakingRecords
            | Self::SentenceRecords
            | Self::WritingRecords => {
                "Words with a sample activity marker in this category; not scored competency evidence."
            }
            Self::RecognitionReview
            | Self::ListeningReview
            | Self::SpeakingReview
            | Self::SentenceReview
            | Self::WritingReview => {
                "Words with a sample review marker in this category; not a failed assessment."
            }
            Self::ActiveDates => "Distinct fixture dates with a recorded sample session.",
            Self::CompletedSessions => "Count of completed sessions, irrespective of scoring.",
            Self::PracticeSessions => "Sessions associated with registered Practice exercises.",
            Self::GameSessions => "Game sessions, shown by activity type.",
            Self::ScoredSessions => {
                "Sessions with a sample outcome field; not necessarily assessed."
            }
            Self::UnscoredSessions => "Sessions without a correctness outcome.",
            Self::PracticeTypes => {
                "Distinct registered Practice exercise identities represented in fixture history."
            }
            Self::GameTypes => "Distinct game activity identities represented in fixture history.",
        }
    }

    pub fn value(self, words: &[WordRecord], sessions: &[SessionRecord]) -> usize {
        let status = |word: &WordRecord| match self {
            Self::RecognitionRecords | Self::RecognitionReview => word.recognition,
            Self::ListeningRecords | Self::ListeningReview => word.listening,
            Self::SpeakingRecords | Self::SpeakingReview => word.speaking,
            Self::SentenceRecords | Self::SentenceReview => word.sentence,
            _ => word.writing,
        };
        match self {
            Self::ReviewWords => words
                .iter()
                .filter(|word| {
                    [
                        word.recognition,
                        word.listening,
                        word.speaking,
                        word.sentence,
                        word.writing,
                    ]
                    .contains(&EvidenceStatus::NeedsReview)
                })
                .count(),
            Self::UnrecordedWords => words
                .iter()
                .filter(|word| {
                    [
                        word.recognition,
                        word.listening,
                        word.speaking,
                        word.sentence,
                        word.writing,
                    ]
                    .iter()
                    .all(|value| *value == EvidenceStatus::NeedsEvidence)
                })
                .count(),
            Self::RecognitionRecords
            | Self::ListeningRecords
            | Self::SpeakingRecords
            | Self::SentenceRecords
            | Self::WritingRecords => words
                .iter()
                .filter(|word| status(word) == EvidenceStatus::Observed)
                .count(),
            Self::RecognitionReview
            | Self::ListeningReview
            | Self::SpeakingReview
            | Self::SentenceReview
            | Self::WritingReview => words
                .iter()
                .filter(|word| status(word) == EvidenceStatus::NeedsReview)
                .count(),
            Self::ActiveDates => sessions
                .iter()
                .map(|row| row.date.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            Self::CompletedSessions => sessions.len(),
            Self::PracticeSessions => sessions
                .iter()
                .filter(|row| row.category == "Practice")
                .count(),
            Self::GameSessions => sessions
                .iter()
                .filter(|row| row.category == "Games")
                .count(),
            Self::ScoredSessions => sessions.iter().filter(|row| row.correct.is_some()).count(),
            Self::UnscoredSessions => sessions.iter().filter(|row| row.correct.is_none()).count(),
            Self::PracticeTypes => sessions
                .iter()
                .filter(|row| row.category == "Practice")
                .map(|row| row.activity_id.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            Self::GameTypes => sessions
                .iter()
                .filter(|row| row.category == "Games")
                .map(|row| row.activity_id.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TileSelection {
    pub order: Vec<TileId>,
    pub visible: BTreeSet<TileId>,
}

impl Default for TileSelection {
    fn default() -> Self {
        Self {
            order: TileId::ALL.to_vec(),
            visible: BTreeSet::new(),
        }
    }
}

impl TileSelection {
    pub fn preset(preset: super::state::DashboardPreset) -> Self {
        let mut next = Self::default();
        match preset {
            super::state::DashboardPreset::Focused => {}
            super::state::DashboardPreset::Activity => {
                next.visible.extend([
                    TileId::ActiveDates,
                    TileId::PracticeSessions,
                    TileId::GameSessions,
                ]);
            }
            super::state::DashboardPreset::Detailed => {
                next.visible.extend(TileId::ALL);
            }
        }
        next
    }

    pub fn toggle(&mut self, tile: TileId) {
        if !self.visible.remove(&tile) {
            self.visible.insert(tile);
        }
    }

    pub fn move_tile(&mut self, tile: TileId, direction: i8) {
        if let Some(index) = self.order.iter().position(|candidate| *candidate == tile) {
            let adjacent = match direction {
                -1 if index > 0 => Some(index - 1),
                1 if index + 1 < self.order.len() => Some(index + 1),
                _ => None,
            };
            if let Some(other) = adjacent {
                self.order.swap(index, other);
            }
        }
    }
}

#[derive(Properties, PartialEq)]
pub(super) struct ExtraTilesProps {
    pub selection: TileSelection,
}

#[function_component(ExtraTiles)]
pub(super) fn extra_tiles(props: &ExtraTilesProps) -> Html {
    let loaded = super::context::use_loaded_section();
    let super::data::SectionPayload::Overview { words, sessions } = loaded.as_ref() else {
        unreachable!("Overview projection expected")
    };
    let visible = props
        .selection
        .order
        .iter()
        .copied()
        .filter(|tile| props.selection.visible.contains(tile))
        .collect::<Vec<_>>();
    if visible.is_empty() {
        return html! {};
    }
    html! {
        <section class="space-y-3" data-stats-extra-tiles="" aria-label="Dashboard metrics">
            <h2 class="sr-only">{"Metrics"}</h2>
            <div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4">
                {for visible.into_iter().map(|tile| html! {
                    <article key={tile.label()} class="min-w-0 rounded-xl border border-line bg-surface p-3">
                        <h3 class="text-xs font-medium text-muted">{tile.label()}</h3>
                        <p class="mt-1 text-xl font-semibold tabular-nums">{tile.value(words, sessions).to_string()}</p>
                        <p class="sr-only">{tile.explanation()}</p>
                    </article>
                })}
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_twenty_unique_entries() {
        assert_eq!(TileId::ALL.len(), 20);
        assert_eq!(
            TileId::ALL.iter().copied().collect::<BTreeSet<_>>().len(),
            20
        );
    }

    #[test]
    fn visibility_and_order_can_be_changed_independently() {
        let mut tiles = TileSelection::default();
        tiles.toggle(TileId::GameSessions);
        assert!(tiles.visible.contains(&TileId::GameSessions));
        tiles.move_tile(TileId::GameSessions, -1);
        assert_eq!(
            tiles
                .order
                .iter()
                .filter(|tile| **tile == TileId::GameSessions)
                .count(),
            1
        );
        tiles.toggle(TileId::GameSessions);
        assert!(!tiles.visible.contains(&TileId::GameSessions));
        assert_eq!(
            TileSelection::preset(super::super::state::DashboardPreset::Detailed)
                .visible
                .len(),
            20
        );
    }

    #[test]
    fn sample_aggregation_never_claims_assessed_performance() {
        let words = super::super::data::demo_words_for_tests();
        let sessions = super::super::data::demo_sessions_for_tests();
        assert_eq!(
            TileId::CompletedSessions.value(&words, &sessions),
            sessions.len()
        );
        assert_eq!(
            TileId::ScoredSessions.value(&words, &sessions)
                + TileId::UnscoredSessions.value(&words, &sessions),
            sessions.len()
        );
        assert_eq!(
            TileId::PracticeSessions.value(&words, &sessions)
                + TileId::GameSessions.value(&words, &sessions),
            sessions.len()
        );
    }
}
