mod exercise_tile;

use crate::reducers::practice_setup_flow::QUICK_EXERCISES;
use yew::prelude::*;
use yew_icons::IconData;
use zipeek_exercises::{definition::required_external_services, practice_draft::PracticeDraft};

use self::exercise_tile::ExerciseTile;

struct ExerciseConfig {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    icon: IconData,
    tab: &'static str,
}

const EXERCISES: [ExerciseConfig; 16] = [
    ExerciseConfig {
        id: "write-hanzi",
        title: "Write Hanzi",
        description: "Trace vocabulary characters in canonical stroke order.",
        icon: IconData::LUCIDE_PENCIL,
        tab: "write-hanzi",
    },
    ExerciseConfig {
        id: "matching",
        title: "Matching",
        description: "Pair Chinese words with meanings or readings.",
        icon: IconData::LUCIDE_SHUFFLE,
        tab: "matching",
    },
    ExerciseConfig {
        id: "multiple-choice",
        title: "Multiple choice",
        description: "Choose the correct word or reading.",
        icon: IconData::LUCIDE_GRADUATION_CAP,
        tab: "multiple-choice",
    },
    ExerciseConfig {
        id: "sentence-translation",
        title: "Sentence translation",
        description: "Translate generated sentences with a word bank.",
        icon: IconData::LUCIDE_BOOK_OPEN,
        tab: "translation",
    },
    ExerciseConfig {
        id: "speaking",
        title: "Pronunciation",
        description: "Record, replay, and improve your Mandarin pronunciation.",
        icon: IconData::LUCIDE_VOLUME_2,
        tab: "speaking",
    },
    ExerciseConfig {
        id: "listening-choice",
        title: "Listening",
        description: "Hear Mandarin audio and choose the correct Hanzi.",
        icon: IconData::LUCIDE_VOLUME_2,
        tab: "listening",
    },
    ExerciseConfig {
        id: "type-pinyin",
        title: "Type pinyin",
        description: "Type a word's Mandarin reading.",
        icon: IconData::LUCIDE_KEYBOARD,
        tab: "type-pinyin",
    },
    ExerciseConfig {
        id: "character-builder",
        title: "Character builder",
        description: "Arrange Hanzi tiles to form a word.",
        icon: IconData::LUCIDE_LAYERS,
        tab: "character-builder",
    },
    ExerciseConfig {
        id: "word-mosaics",
        title: "Word mosaics",
        description: "Match Chinese words with their meanings.",
        icon: IconData::LUCIDE_LAYOUT_GRID,
        tab: "word-mosaics",
    },
    ExerciseConfig {
        id: "character-hunt",
        title: "Character Hunt",
        description: "Find real Chinese words by joining 2–3 Hanzi in a grid.",
        icon: IconData::LUCIDE_GRID,
        tab: "hunt",
    },
    ExerciseConfig {
        id: "fill-blank",
        title: "Fill in the Blank",
        description: "Complete a sentence with the missing Chinese word or grammar particle.",
        icon: IconData::LUCIDE_FILE_TEXT,
        tab: "fill-blank",
    },
    ExerciseConfig {
        id: "audio-dictation",
        title: "Audio Dictation",
        description: "Listen to Mandarin and type what you hear in Chinese characters.",
        icon: IconData::LUCIDE_HEADPHONES,
        tab: "audio-dictation",
    },
    ExerciseConfig {
        id: "sentence-builder",
        title: "Sentence Builder",
        description: "Arrange Chinese phrase tiles into a natural sentence.",
        icon: IconData::LUCIDE_LAYERS,
        tab: "sentence-builder",
    },
    ExerciseConfig {
        id: "sentence-transformation",
        title: "Sentence Transformation",
        description: "Rewrite a sentence using a target Chinese grammar pattern and word-bank tiles.",
        icon: IconData::LUCIDE_LAYERS,
        tab: "sentence-transformation",
    },
    ExerciseConfig {
        id: "spot-mistake",
        title: "Spot the Mistake",
        description: "Identify a grammar mistake and type a valid replacement. Practice only.",
        icon: IconData::LUCIDE_SEARCH,
        tab: "spot-mistake",
    },
    ExerciseConfig {
        id: "image-vocabulary",
        title: "Picture vocabulary",
        description: "Recognize words in an approved generated image.",
        icon: IconData::LUCIDE_IMAGE,
        tab: "image",
    },
];

#[derive(Properties, PartialEq)]
pub(super) struct ExerciseSelectorProps {
    pub draft: PracticeDraft,
    pub advanced: bool,
    pub on_toggle: Callback<&'static str>,
}

#[function_component(ExerciseSelector)]
pub(super) fn exercise_selector(props: &ExerciseSelectorProps) -> Html {
    html! {
        <section
            class="flex min-h-0 flex-col gap-3"
            data-component="practice-builder"
        >
            <div
                role="group"
                aria-label="Choose exercise types"
                data-practice-tabs=""
                tabindex="0"
                class="rounded-xl focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
            >
                <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    {for EXERCISES.iter()
                        .filter(|exercise| props.advanced || QUICK_EXERCISES.contains(&exercise.id))
                        .map(|exercise| {
                        html! {
                            <ExerciseTile
                                key={exercise.id}
                                id={exercise.id}
                                title={exercise.title}
                                description={exercise.description}
                                icon={exercise.icon}
                                tab={exercise.tab}
                                selected={props.draft.has_exercise(exercise.id)}
                                services={required_external_services(exercise.id)}
                                on_toggle={props.on_toggle.clone()}
                            />
                        }
                        })}
                </div>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn exercise_identifiers_and_tabs_are_unique() {
        let ids: HashSet<_> = EXERCISES.iter().map(|exercise| exercise.id).collect();
        let tabs: HashSet<_> = EXERCISES.iter().map(|exercise| exercise.tab).collect();

        assert_eq!(ids.len(), EXERCISES.len());
        assert_eq!(tabs.len(), EXERCISES.len());
    }

    #[test]
    fn quick_exercises_match_the_path_description() {
        assert_eq!(QUICK_EXERCISES.len(), 8);
        assert!(
            QUICK_EXERCISES
                .iter()
                .all(|id| EXERCISES.iter().any(|exercise| exercise.id == *id))
        );
        assert!(QUICK_EXERCISES.contains(&"write-hanzi"));
        assert!(QUICK_EXERCISES.contains(&"listening-choice"));
        assert!(!QUICK_EXERCISES.contains(&"sentence-translation"));
        assert!(!QUICK_EXERCISES.contains(&"sentence-builder"));
        assert!(!QUICK_EXERCISES.contains(&"spot-mistake"));
    }

    #[test]
    fn every_exercise_is_available_from_practice_selection() {
        assert_eq!(EXERCISES.len(), 16);
        assert!(
            EXERCISES
                .iter()
                .any(|exercise| exercise.id == "write-hanzi")
        );
        assert!(
            EXERCISES
                .iter()
                .any(|exercise| exercise.id == "listening-choice")
        );
        assert!(
            !EXERCISES
                .iter()
                .any(|exercise| exercise.id == "handwriting-recall")
        );
        assert!(
            EXERCISES
                .iter()
                .any(|exercise| exercise.id == "spot-mistake" && exercise.tab == "spot-mistake")
        );
        assert!(EXERCISES.iter().any(
            |exercise| exercise.id == "sentence-builder" && exercise.tab == "sentence-builder"
        ));
        assert!(
            EXERCISES
                .iter()
                .any(|exercise| exercise.id == "sentence-transformation"
                    && exercise.tab == "sentence-transformation")
        );
        assert!(EXERCISES.iter().any(|exercise| exercise.id == "speaking"));
        assert!(
            EXERCISES
                .iter()
                .any(|exercise| exercise.id == "image-vocabulary")
        );
    }
}
