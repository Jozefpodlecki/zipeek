use yew::prelude::*;
use zipeek_core::{HskLevel, Word};

use super::ranking;

#[derive(Properties, PartialEq)]
pub struct VocabularyResultsProps {
    pub results: Option<Result<Vec<Word>, String>>,
    pub mode: String,
    pub query: String,
    pub submitted: String,
    pub selected: Option<Word>,
    pub expanded: bool,
    pub highlighted: usize,
    pub visible_count: usize,
    pub on_more: Callback<MouseEvent>,
    pub on_select: Callback<Word>,
    pub on_open: Callback<MouseEvent>,
}

pub(super) fn visible_words(props: &VocabularyResultsProps) -> Vec<Word> {
    let Some(Ok(words)) = &props.results else {
        return Vec::new();
    };

    let mut matches: Vec<Word> = if props.mode == "sets" && !props.query.trim().is_empty() {
        words
            .iter()
            .filter(|word| ranking::score(word, &props.query) > 0)
            .cloned()
            .collect()
    } else {
        words.clone()
    };

    if !props.query.trim().is_empty() {
        ranking::rank(&mut matches, &props.query);
    }

    matches.truncate(props.visible_count);
    matches
}

#[function_component(VocabularyResults)]
pub fn vocabulary_results(props: &VocabularyResultsProps) -> Html {
    if !props.expanded {
        if let Some(word) = &props.selected {
            return html! {
                <div
                    class="rounded-xl border border-line bg-raised p-3 space-y-2"
                    data-write-hanzi-selection=""
                >
                    <div class="flex items-center justify-between gap-2">
                        <span class="text-xs font-medium text-muted">
                            {"Selected word"}
                        </span>
                        <button
                            type="button"
                            onclick={props.on_open.clone()}
                            class="rounded-md px-2 py-1 text-xs text-accent hover:bg-surface focus-visible:outline-none"
                        >
                            {"Change"}
                        </button>
                    </div>

                    <div class="flex flex-wrap items-baseline gap-2">
                        <strong lang="zh-Hans" class="text-2xl">
                            {&word.simplified}
                        </strong>
                        <span class="text-sm text-accent">
                            {&word.pinyin}
                        </span>
                    </div>

                    <p class="text-xs text-muted">
                        {word
                            .english
                            .as_deref()
                            .unwrap_or("Meaning unavailable")
                            .replace('|', " · ")}
                    </p>
                </div>
            };
        }

        return Html::default();
    }

    if props.mode == "search" && props.query.trim() != props.submitted.trim() {
        return html! {
            <p role="status" class="text-sm text-muted">
                {"Searching…"}
            </p>
        };
    }

    let matches = visible_words(props);

    let total_matches = props
        .results
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(|words| {
            if props.mode == "sets" && !props.query.trim().is_empty() {
                words
                    .iter()
                    .filter(|word| ranking::score(word, &props.query) > 0)
                    .count()
            } else {
                words.len()
            }
        })
        .unwrap_or(0);

    match &props.results {
        None => html! {
            <p role="status" class="text-sm text-muted">
                {"Loading vocabulary…"}
            </p>
        },

        Some(Err(error)) => html! {
            <p role="alert" class="text-sm text-red-500">
                {error}
            </p>
        },

        Some(Ok(_)) if matches.is_empty() => {
            let message = if props.mode == "sets" {
                "No matching words in this set."
            } else if props.query.trim().is_empty() {
                "Enter English, Hanzi or Pinyin to search."
            } else {
                "No matching dictionary entries found."
            };

            html! {
                <p class="text-sm text-muted" role="status">
                    {message}
                </p>
            }
        }

        Some(Ok(_)) => html! {
            <div
                class="min-h-0 space-y-2 overflow-y-auto overscroll-contain"
                aria-label="Available Chinese words"
            >
                <div class="flex items-center justify-between gap-2 text-xs text-muted">
                    <span>{"Suggestions"}</span>
                    <span>
                        {format!("Showing {} of {}", matches.len(), total_matches)}
                    </span>
                </div>

                <ul
                    id="write-hanzi-suggestions"
                    role="listbox"
                    aria-label="Vocabulary suggestions"
                    class="space-y-1"
                >
                    {for matches.iter().enumerate().map(|(index, word)| {
                        let selected = props
                            .selected
                            .as_ref()
                            .is_some_and(|choice| choice.id == word.id);

                        let active = index == props.highlighted;

                        let on_select = props.on_select.clone();
                        let choice = word.clone();

                        let on_choose = Callback::from(move |_| {
                            on_select.emit(choice.clone());
                        });

                        let button_class = classes!(
                            "w-full",
                            "rounded-lg",
                            "border",
                            "px-3",
                            "py-2",
                            "text-left",
                            "transition-colors",
                            "motion-reduce:transition-none",
                            "focus-visible:outline-none",
                            if active {
                                "border-accent/70"
                            } else {
                                "border-line"
                            },
                            if active {
                                "bg-brand-action/15"
                            } else {
                                "bg-raised"
                            },
                            (!active).then_some("hover:border-accent/40"),
                        );

                        let badges = [
                            word.hsk2_level.map(HskLevel::Hsk2),
                            word.hsk3_level.map(HskLevel::Hsk3),
                        ]
                        .into_iter()
                        .flatten()
                        .map(|level| {
                            html! {
                                <span class="rounded border border-line px-1.5 text-[10px] text-muted">
                                    {level.badge_label()}
                                </span>
                            }
                        });

                        html! {
                            <li key={word.id.as_i32()} class="list-none">
                                <button
                                    type="button"
                                    role="option"
                                    aria-selected={selected.to_string()}
                                    id={format!("write-hanzi-suggestion-{index}")}
                                    tabindex="-1"
                                    onclick={on_choose}
                                    class={button_class}
                                >
                                    <span class="flex flex-wrap items-baseline gap-2">
                                        <strong lang="zh-Hans" class="text-lg">
                                            {&word.simplified}
                                        </strong>

                                        <span class="text-xs text-accent">
                                            {&word.pinyin}
                                        </span>

                                        if selected {
                                            <span class="text-[10px] text-accent">
                                                {"Selected"}
                                            </span>
                                        }
                                    </span>

                                    <span class="block line-clamp-2 text-xs text-muted">
                                        {word
                                            .english
                                            .as_deref()
                                            .unwrap_or("Meaning unavailable")
                                            .replace('|', " · ")}
                                    </span>

                                    <span class="mt-1 flex flex-wrap gap-1">
                                        {for badges}
                                    </span>
                                </button>
                            </li>
                        }
                    })}
                </ul>

                if total_matches > matches.len() {
                    <button
                        type="button"
                        onclick={props.on_more.clone()}
                        class="w-full rounded-lg border border-line bg-raised py-2 text-sm text-accent hover:border-accent/50"
                    >
                        {format!(
                            "Show more ({} remaining)",
                            total_matches - matches.len()
                        )}
                    </button>
                }
            </div>
        },
    }
}
