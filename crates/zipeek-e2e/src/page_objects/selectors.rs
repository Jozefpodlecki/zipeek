pub mod practice {
    pub const ADVANCED_PATH: &str = "[data-practice-path='advanced']";
    pub const QUICK_PATH: &str = "[data-practice-path='quick']";
    pub const SENTENCE_TRANSLATION: &str = "[data-practice-exercise='sentence-translation']";
    pub const CHARACTER_BUILDER: &str = "[data-practice-exercise='character-builder']";
    pub const CONFIGURE: &str = "[data-practice-exercise-stage='configure']";
    pub const CONTINUE: &str = "[data-practice-continue]:not([disabled])";
    pub const EXERCISE: &str = "[data-practice-exercise]";
    pub const SELECTED_EXERCISES: &str = "[data-practice-exercise][aria-pressed='true']";

    pub fn exercise(id: &str) -> String {
        format!("[data-practice-exercise='{id}']")
    }
}

pub mod image_review {
    pub const ROOT: &str = "[data-image-review]";
    pub const LOADING: &str = "[data-image-review-loading]";
    pub const ERROR: &str = "[data-image-review-error]";
    pub const APPROVE: &str = "[data-image-review-approve]";
}

pub mod word_mosaics {
    pub const ROOT: &str = "[data-exercise='word-mosaics']";
    pub const TILE: &str = "[data-mosaic-tile]";
    pub const FEEDBACK: &str = "[data-mosaic-feedback]";
    pub const CONTINUE: &str = "[data-mosaic-continue]";
    pub const PROGRESS: &str = "[data-exercise='word-mosaics'] [data-mosaic-progress]";
    pub const COMPLETED: &str = "[data-component='vocabulary-exercise-completed']";

    pub fn tile(pair: usize, meaning: bool) -> String {
        format!("[data-mosaic-tile][data-pair='{pair}'][data-meaning='{meaning}']")
    }

    pub fn matched_tiles(pair: usize) -> String {
        format!("[data-mosaic-tile][data-pair='{pair}']:disabled")
    }
}

pub mod settings {
    pub const FONT_OPTIONS: &str = "[data-hanzi-font-options]";
    pub const FONT: &str = "[data-hanzi-font-option='lxgw-wenkai']";
    pub const LARGE: &str = "[data-character-size-option='large']";
    pub const EXTRA_LARGE: &str = "[data-character-size-option='extra-large']";
    pub const LARGE_SELECTED: &str = "[data-character-size-option='large'][aria-pressed='true']";
    pub const DIAGNOSTICS: &str = "[data-duckdb-diagnostics]";
    pub const STATISTICS: &str = "[data-duckdb-diagnostics] [data-duckdb-statistics]";
}

pub mod speaking {
    pub const RECORDER: &str = "[data-component='speaking-recorder']";
    pub const INITIALIZED: &str =
        "[data-component='speaking-recorder'][data-recorder-initialized='true']";
    pub const ALERT: &str = "[data-component='speaking-recorder'] [role='alert']";
    pub const START: &str = "[data-action='start-recording']";
    pub const PLAYBACK: &str = "[data-component='speaking-playback']";
    pub const RECORDING_PHASE: &str =
        "[data-component='speaking-recorder'][data-recorder-phase='recording']";
    pub const READY_PHASE: &str =
        "[data-component='speaking-recorder'][data-recorder-phase='ready']";
    pub const STOP: &str = "[data-action='stop-recording']";
    pub const DISCARD: &str = "[data-action='discard-recording']";
    pub const ERROR_PHASE: &str =
        "[data-component='speaking-recorder'][data-recorder-phase='error']";
}

pub mod word_sets {
    pub const DELETE: &str = "[data-delete-word-set]";
    pub const SELECTABLE_OPTION: &str =
        "[data-target-word-set][data-selected-word-set-id]:not([data-selected-word-set-id=''])";
    pub const CONFIRM_DELETION: &str = "Confirm deletion";
}

pub mod browse {
    pub const REMOVE_NOUN_FILTER: &str = "button[aria-label='Remove PoS: noun']";
}

pub mod shell {
    pub const BRAND_LINK: &str = "[data-brand-link] a";
    pub const HOME: &str = "[data-home]";
    pub const MAIN_CONTENT: &str = "[data-component='main-content']";
    pub const NAVIGATION_TILES: &str = "[data-navigation-tiles] > a";
    pub const PRACTICE_PROGRESS: &str = "[data-practice-progress]";
    pub const BROWSE_QUERY: &str = "[data-component='browse-page'] [data-browse-query]";
    pub const RANDOM_PAGE: &str = "[data-component='random-page']";
}
