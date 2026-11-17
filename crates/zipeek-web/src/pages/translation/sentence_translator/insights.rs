use yew::prelude::*;
use zipeek_ai_protocol::sentence_analysis::SentenceAnalysis;
use zipeek_ai_protocol::sentence_insights::{LexicalCoverage, grammar_evidence};

use super::state::TranslationDirection;
use crate::services::database::VerifiedVocabulary;

#[derive(Properties, PartialEq)]
pub(super) struct TranslatorInsightsProps {
    pub direction: TranslationDirection,
    pub source: String,
    pub analysis: SentenceAnalysis,
    pub verified: Option<Result<Vec<VerifiedVocabulary>, String>>,
}

#[function_component(TranslatorInsights)]
pub(super) fn translator_insights(props: &TranslatorInsightsProps) -> Html {
    let analysis = &props.analysis;
    let chinese = if props.direction == TranslationDirection::ChineseToEnglish {
        props.source.as_str()
    } else {
        analysis.translation.as_str()
    };
    let coverage = lexical_coverage(analysis, props.verified.as_ref());

    let vocabulary_insights_content = {
        if analysis.vocabulary.is_empty() {
            html! { <p
                class="text-sm text-muted"
            >
                {"No vocabulary observations returned."}
            </p> }
        } else {
            html! {
                            <div
                                class="grid gap-2 sm:grid-cols-2"
                            >
                                {for analysis.vocabulary.iter().enumerate().map(|(index, word)| {
                                    let matched = props.verified.as_ref()
                                        .and_then(|result| result.as_ref().ok())
                                        .and_then(|items| items.get(index))
                                        .filter(|item| item.observation.hanzi == word.hanzi)
                                        .and_then(|item| item.dictionary.as_ref());
                                                            let matched_dictionary_entry = {if let Some(entry) = matched {
                                                html! {
                                                    <p
                                                        data-dictionary-verified="true"
                                                        class="mt-2 text-xs text-muted"
                                                    >
                                                        {format!("Dictionary verified · {}", entry.english.as_deref().unwrap_or("No definition"))}
                                                    </p>
                                                }
                                            } else {
                                                html! { <p
                                                    class="mt-2 text-xs text-muted"
                                                >
                                                    {"AI suggestion · not dictionary verified"}
                                                </p> }
                                            }};
            html! {
                                        <div
                                            data-vocabulary-observation={word.hanzi.clone()}
                                            class="rounded-xl border border-line bg-raised p-3"
                                        >
                                            <div
                                                class="flex flex-wrap items-baseline gap-2"
                                            >
                                                <strong>
                                                    {word.hanzi.clone()}
                                                </strong>
                                                <span
                                                    class="text-xs text-muted"
                                                >
                                                    {word.pinyin.clone()}
                                                </span>
                                            </div>
                                            <p
                                                class="mt-1 text-sm"
                                            >
                                                {word.meaning.clone()}
                                            </p>
                                            {matched_dictionary_entry}
                                        </div>
                                    }
                                })}
                            </div>
                        }
        }
    };
    let grammar_insights = {
        if analysis.grammar.is_empty() {
            Html::default()
        } else {
            html! {
                <div
                    class="mt-4 space-y-3 border-t border-line pt-4"
                >
                    <h3
                        class="text-sm font-semibold"
                    >
                        {"Language structures"}
                    </h3>
                    {for analysis.grammar.iter().map(|item| html! {
                        <div
                            class="rounded-lg bg-raised p-3"
                        >
                            <div
                                class="flex flex-wrap items-center justify-between gap-2"
                            >
                                <strong
                                    class="text-sm"
                                >
                                    {item.pattern.clone()}
                                </strong>
                                <span
                                    data-grammar-family=""
                                    class="text-xs text-muted"
                                >
                                    {grammar_evidence(chinese, item).family.label()}
                                </span>
                            </div>
                            <p
                                class="mt-1 text-sm text-muted"
                            >
                                {item.explanation.clone()}
                            </p>
                            <p
                                class="mt-1 text-sm"
                            >
                                {item.example_fragment.clone()}
                            </p>
                        </div>
                    })}
                </div>
            }
        }
    };

    let translation_nuance_content = {
        if analysis.nuance.is_empty() && analysis.notes.is_empty() {
            html! { <p
                class="text-sm text-muted"
            >
                {"No additional nuance returned."}
            </p> }
        } else {
            html! {
                <ul
                    class="space-y-3 text-sm leading-6"
                >
                    {for analysis.nuance.iter().chain(analysis.notes.iter()).map(|note| html! {
                        <li
                            class="flex gap-2"
                        >
                            <span
                                aria-hidden="true"
                                class="text-brand-action"
                            >
                                {"•"}
                            </span>
                            <span>
                                {note.clone()}
                            </span>
                        </li>
                    })}
                </ul>
            }
        }
    };
    let translation_verification_error = if let Some(Err(error)) = &props.verified {
        html! {
            <p
                role="status"
                class="mt-2 text-xs text-muted"
            >
                {format!("Dictionary verification unavailable: {error}")}
            </p>
        }
    } else {
        Html::default()
    };
    let estimated_hsk_badge = {
        if let Some(level) = analysis.estimated_hsk {
            html! {
                <span
                    class="rounded-lg border border-line px-3 py-2 text-sm"
                >
                    {format!("Estimated HSK {level}")}
                </span>
            }
        } else {
            Html::default()
        }
    };
    html! {
        <div
            data-sentence-analysis=""
            class="grid min-w-0 gap-4 lg:grid-cols-2"
        >
            <section
                class="min-w-0 rounded-2xl border border-line bg-surface p-5"
            >
                <h2
                    class="mb-4 font-semibold"
                >
                    {"Analysis"}
                </h2>
                <div
                    class="flex flex-wrap gap-2"
                >
                    <span
                        class="rounded-lg bg-raised px-3 py-2 text-sm font-semibold capitalize"
                    >
                        {format!("{} difficulty", analysis.difficulty)}
                    </span>
                    {for analysis.features.iter().map(|feature| html! {
                        <span
                            key={feature.clone()}
                            class="rounded-lg border border-line bg-raised px-3 py-2 text-sm"
                        >
                            {feature.clone()}
                        </span>
                    })}
                    {estimated_hsk_badge}
                </div>
                <p
                    class="mt-4 text-sm text-muted"
                >
                    {analysis.rationale.clone()}
                </p>
                <p
                    class="mt-2 text-xs text-muted"
                >
                    {"Difficulty and language features are AI estimates, not verified classifications."}
                </p>
                {grammar_insights}
            </section>
            <section
                class="min-w-0 rounded-2xl border border-line bg-surface p-5"
            >
                <h2
                    class="mb-4 font-semibold"
                >
                    {"Meaning & nuance"}
                </h2>
                {translation_nuance_content}
            </section>
            <section
                class="min-w-0 rounded-2xl border border-line bg-surface p-5 lg:col-span-2"
            >
                <h2
                    class="mb-4 font-semibold"
                >
                    {"Key vocabulary"}
                </h2>
                {vocabulary_insights_content}
                <p
                    data-lexical-coverage=""
                    class="mt-4 text-xs text-muted"
                >
                    {format!("{} of {} AI suggestions matched the dictionary · {} HSK-tagged · {} ambiguous",
                        coverage.matched, coverage.observed, coverage.hsk_tagged, coverage.ambiguous)}
                </p>
                <p
                    class="mt-1 text-xs text-muted"
                >
                    {coverage.scope_note()}
                </p>
                {translation_verification_error}
            </section>
        </div>
    }
}

fn lexical_coverage(
    analysis: &SentenceAnalysis,
    verified: Option<&Result<Vec<VerifiedVocabulary>, String>>,
) -> LexicalCoverage {
    let found = verified.and_then(|result| result.as_ref().ok());
    LexicalCoverage {
        observed: analysis.vocabulary.len(),
        matched: found.map_or(0, |items| {
            items
                .iter()
                .filter(|item| item.dictionary.is_some())
                .count()
        }),
        hsk_tagged: found.map_or(0, |items| {
            items
                .iter()
                .filter(|item| {
                    item.dictionary
                        .as_ref()
                        .is_some_and(|word| word.hsk2_level.is_some() || word.hsk3_level.is_some())
                })
                .count()
        }),
        ambiguous: found.map_or(0, |items| {
            items.iter().filter(|item| item.alternatives > 0).count()
        }),
        highest_verified_hsk3: found.and_then(|items| {
            items
                .iter()
                .filter_map(|item| item.dictionary.as_ref().and_then(|word| word.hsk3_level))
                .max()
        }),
    }
}
