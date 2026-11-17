use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::practice_draft::VocabularySource;

use super::{PracticeSetupSummaryProps, exercise_label, format_label, vocabulary_label};

pub(super) fn render(props: &PracticeSetupSummaryProps) -> Html {
    let selected_exercises: Vec<String> = if props.draft.playlist_exercises().len() > 1 {
        props
            .draft
            .playlist_exercises()
            .into_iter()
            .map(str::to_owned)
            .collect()
    } else {
        props.draft.exercise_ids.iter().cloned().collect()
    };
    html! {
        <div
            class="grid gap-4 md:grid-cols-2"
        >
            <section
                class="rounded-xl border border-line bg-raised p-4"
                data-practice-summary-configuration=""
            >
                <h3 class="flex items-center gap-2 font-semibold">
                    <Icon data={IconData::LUCIDE_LAYOUT_GRID} width="19px" height="19px" />
                    {"Exercises"}
                    <span class="ml-auto rounded-full bg-brand-action/15 px-2.5 py-1 text-xs font-semibold text-brand-action">
                        {props.draft.exercise_ids.len()}
                    </span>
                </h3>
                <ul class="mt-3 grid gap-2 sm:grid-cols-2" >
                    {for selected_exercises.into_iter().map(|id| html! {
                        <li key={id.clone()} class="flex items-center gap-2 rounded-lg border border-line/70 bg-surface px-3 py-2 text-sm text-ink">
                            <Icon data={exercise_icon(&id)} width="17px" height="17px" />
                            <span>{exercise_label(&id)}</span>
                        </li>
                    })}
                </ul>
                {props.draft.has_exercise("character-hunt").then(|| html! {
                    <p
                        data-character-hunt-difficulty=""
                        class="mt-2 text-xs text-muted"
                    >
                        {format!("{} difficulty", props.draft.hunt_difficulty.label())}
                    </p>
                }).unwrap_or_default()}
                {props.draft.has_exercise("matching").then(|| html! {
                    <p class="mt-2 text-xs text-muted" data-matching-summary-order="">
                        {format!("Matching columns: {}", props.draft.matching_column_order.query_value())}
                    </p>
                }).unwrap_or_default()}
                {for props.variants.iter().map(|variant| html! {
                    <p
                        key={variant.seed_token()}
                        class="mt-2 text-xs text-muted"
                    >
                        {format!("{}: {}", if variant.is_mcq() { "Multiple Choice" } else { "Matching" }, format_label(*variant))}
                    </p>
                })}
            </section>
            <section
                class="rounded-xl border border-line bg-raised p-4"
            >
                <h3 class="flex items-center gap-2 font-semibold">
                    <Icon data={IconData::LUCIDE_BOOK_OPEN} width="19px" height="19px" />
                    {"Vocabulary"}
                </h3>
                <p class="mt-3 text-sm font-medium text-ink">
                    {vocabulary_label(&props.draft.vocabulary)}
                </p>
                {match &props.draft.vocabulary {
                    VocabularySource::Dictionary(filters) => html! {
                        <div class="mt-3 flex flex-wrap gap-2" aria-label="Vocabulary filters">
                            {for filters.categories.iter().map(|category| html! {
                                <span key={category.clone()} class="rounded-full border border-brand-action/50 bg-brand-action/10 px-3 py-1 text-xs font-semibold text-ink">
                                    {category}
                                </span>
                            })}
                            {for filters.hsk.iter().map(|band| html! {
                                <span key={format!("{}-{}", band.system, band.level)} class="rounded-full border border-line bg-surface px-3 py-1 text-xs text-ink">
                                    {format!("HSK {} · {}", band.system, band.level)}
                                </span>
                            })}
                        </div>
                    },
                    VocabularySource::PersonalSet { reference } => html! {
                        <p class="mt-2 text-xs text-muted">{format!("Saved Word Set: {reference}")}</p>
                    },
                }}
                <p
                    class="mt-2 text-xs text-muted"
                    data-practice-vocabulary-validation="on-launch"
                >
                    {"Vocabulary is loaded when practice starts."}
                </p>
                {(props.draft.has_exercise("fill-blank") || props.draft.has_exercise("audio-dictation") || props.draft.has_exercise("sentence-builder") || props.draft.has_exercise("sentence-transformation") || props.draft.has_exercise("spot-mistake")).then(|| html! {
                    <p class="mt-2 text-xs text-muted" data-practice-authored-note="">
                        {"Fill in the Blank, Audio Dictation, Sentence Builder, Sentence Transformation, and Spot the Mistake use fixed example prompts rather than selected vocabulary filters. Results are practice-only."}
                    </p>
                }).unwrap_or_default()}
                {props.draft.supports_local_vocabulary_session().then(|| html! {
                    <p role="status" class="mt-3 text-xs text-muted" data-practice-local-preview="">
                        {format!("A resumable session with {} scheduled exercises will be saved in this browser.",
                            props.draft.length.resumable_rounds().unwrap_or_default())}
                    </p>
                }).unwrap_or_default()}
            </section>
        </div>
    }
}

fn exercise_icon(id: &str) -> IconData {
    match id {
        "matching" => IconData::LUCIDE_SHUFFLE,
        "multiple-choice" => IconData::LUCIDE_GRADUATION_CAP,
        "sentence-translation" => IconData::LUCIDE_BOOK_OPEN,
        "speaking" | "listening-choice" => IconData::LUCIDE_VOLUME_2,
        "type-pinyin" | "audio-dictation" => IconData::LUCIDE_KEYBOARD,
        "character-builder" | "sentence-builder" | "sentence-transformation" => {
            IconData::LUCIDE_LAYERS
        }
        "word-mosaics" => IconData::LUCIDE_LAYOUT_GRID,
        "character-hunt" => IconData::LUCIDE_GRID,
        "image-vocabulary" => IconData::LUCIDE_IMAGE,
        "fill-blank" | "spot-mistake" | "write-hanzi" => IconData::LUCIDE_PENCIL,
        _ => IconData::LUCIDE_BOOK_OPEN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_exercises_have_icons() {
        for id in [
            "matching",
            "multiple-choice",
            "speaking",
            "listening-choice",
            "character-hunt",
            "fill-blank",
            "audio-dictation",
            "sentence-builder",
            "sentence-transformation",
            "spot-mistake",
            "write-hanzi",
        ] {
            assert_ne!(super::super::exercise_label(id), "Unknown exercise");
            let _ = exercise_icon(id);
        }
    }
}
