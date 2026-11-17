use crate::page_objects::WebElementQueryActions;
use crate::page_objects::{WebDriverActions, selectors::practice};
use std::time::Duration;

use crate::{HarnessResult, ScenarioContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleTile {
    pub index: usize,
    pub text: String,
    pub solution_order: Option<usize>,
    pub is_first: bool,
    pub is_last: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationObservation {
    pub source: String,
    pub target_language: String,
    pub available_tiles: Vec<VisibleTile>,
}

impl TranslationObservation {
    pub fn canonical_tiles(&self) -> Result<Vec<&VisibleTile>, String> {
        let count = self.available_tiles.len();
        let mut ordered = vec![None; count];
        for tile in &self.available_tiles {
            let position = tile
                .solution_order
                .ok_or_else(|| format!("Tile {} is missing its solution position", tile.index))?;
            if position >= count {
                return Err(format!(
                    "Tile {} has out-of-range solution position {position}",
                    tile.index
                ));
            }
            if tile.is_first != (position == 0) || tile.is_last != (position + 1 == count) {
                return Err(format!(
                    "Tile {} has inconsistent sentence boundary flags",
                    tile.index
                ));
            }
            if ordered[position].replace(tile).is_some() {
                return Err(format!("Duplicate canonical solution position {position}"));
            }
        }
        ordered
            .into_iter()
            .enumerate()
            .map(|(position, tile)| {
                tile.ok_or_else(|| format!("Missing canonical solution position {position}"))
            })
            .collect()
    }
}

pub struct TranslationPage<'a> {
    context: &'a ScenarioContext,
}

impl<'a> TranslationPage<'a> {
    pub fn new(context: &'a ScenarioContext) -> Self {
        Self { context }
    }

    pub async fn open(&self, vocabulary_source: &str) -> HarnessResult<()> {
        self.context
            .driver
            .goto(self.context.url(&format!(
                "/#/practice/translation?source={vocabulary_source}&direction=zh-to-en",
            )))
            .await?;
        self.context
            .driver
            .query_single("[data-sentence-translation]")
            .await?;
        Ok(())
    }

    pub async fn configure(&self, model: &str) -> HarnessResult<()> {
        self.configure_endpoint(model, &self.context.ai_url).await
    }

    pub async fn configure_endpoint(&self, model: &str, url: &str) -> HarnessResult<()> {
        let driver = &self.context.driver;
        let previous_url = driver.current_url().await?;
        driver.goto(self.context.url("/#/settings")).await?;
        crate::page_objects::settings::select_section(
            driver,
            crate::page_objects::settings::Section::Ai,
        )
        .await?;
        let updated = driver
            .execute_async(
                crate::js_scripts::translation::TRANSLATION_CONFIGURE_ENDPOINT_01,
                vec![serde_json::json!(url), serde_json::json!(model)],
            )
            .await?
            .convert::<bool>()?;
        if !updated {
            return Err("AI Settings inputs were not rendered".into());
        }
        driver.find_css("[data-ai-save]").await?.click().await?;
        let saved = driver
            .execute(
                crate::js_scripts::translation::TRANSLATION_CONFIGURE_ENDPOINT_02,
                vec![],
            )
            .await?
            .convert::<serde_json::Value>()?;
        if saved["model"].as_str() != Some(model) || saved["url"].as_str() != Some(url) {
            return Err(format!("AI Settings were not persisted: {saved}").into());
        }
        driver.goto(previous_url.as_str()).await?;
        Ok(())
    }

    pub async fn open_from_practice(&self, direction: &str) -> HarnessResult<()> {
        self.open_from_practice_with_count(direction, 5).await
    }

    pub async fn open_from_practice_with_count(
        &self,
        direction: &str,
        count: usize,
    ) -> HarnessResult<()> {
        if !matches!(count, 0 | 5 | 10) {
            return Err("Translation count must be Endless, 5 or 10".into());
        }
        if !matches!(direction, "mixed" | "zh-to-en" | "en-to-zh") {
            return Err(format!("Unsupported translation direction: {direction}").into());
        }
        let driver = &self.context.driver;
        driver.goto(self.context.url("/#/practice")).await?;
        driver.click(practice::ADVANCED_PATH).await?;
        driver.click(practice::SENTENCE_TRANSLATION).await?;
        let selected_exercises = driver.find_all_css(practice::SELECTED_EXERCISES).await?;
        if selected_exercises.len() != 1 {
            return Err(format!(
                "Expected only Sentence translation, found {} selected exercises",
                selected_exercises.len()
            )
            .into());
        }

        driver.click(practice::CONFIGURE).await?;
        driver
            .query_single("[data-translation-direction-setup]")
            .await?;
        driver
            .query_single(format!(
                "[name='practice-translation-direction'][value='{direction}']"
            ))
            .await?
            .click()
            .await?;
        let selected = driver
            .query_single(format!(
                "[name='practice-translation-direction'][value='{direction}']:checked"
            ))
            .await?;
        if !selected.is_selected().await? {
            return Err("Practice did not select the requested translation direction".into());
        }
        driver
            .query_single("[data-practice-continue]:not([disabled])")
            .await?
            .click()
            .await?;
        driver
            .query_single("[data-practice-step-panel='vocabulary']")
            .await?;
        driver
            .query_single("[data-hsk-band='3:1'][aria-pressed='true']")
            .await?;
        driver
            .query_single("[data-practice-continue]:not([disabled])")
            .await?
            .click()
            .await?;
        driver
            .query_single("[data-translation-setup-count]")
            .await?;
        let label = match count {
            0 => "Endless",
            5 => "5 sentences",
            10 => "10 sentences",
            _ => unreachable!(),
        };
        driver
            .query_single(format!("[data-setup-choice='{label}']"))
            .await?
            .click()
            .await?;
        driver
            .query_single("[data-translation-setup-start]:not([disabled])")
            .await?
            .click()
            .await?;
        driver
            .query_wait_single(
                "[data-practice-local-session]",
                Duration::from_secs(30),
                Duration::from_millis(100),
            )
            .await?;
        let url = driver.current_url().await?;
        if !url.as_str().contains("/practice/session/") {
            return Err(format!("Practice did not create a resumable translation: {url}").into());
        }
        let stored = driver
            .execute(
                crate::js_scripts::translation::TRANSLATION_OPEN_FROM_PRACTICE_WITH_COUNT_01,
                vec![],
            )
            .await?
            .convert::<Vec<serde_json::Value>>()?;
        let saved = stored
            .iter()
            .find(|session| {
                session["steps"].as_array().is_some_and(|steps| {
                    steps
                        .iter()
                        .all(|step| step["exercise_id"] == "sentence-translation")
                })
            })
            .ok_or("Practice did not persist the translation plan")?;
        let expected_steps = if count == 0 { 2 } else { count };
        if saved["translation_direction"].as_str() != Some(direction)
            || saved["steps"]
                .as_array()
                .is_none_or(|steps| steps.len() != expected_steps)
            || saved["endless_translation"] != (count == 0)
        {
            return Err(format!(
                "Practice did not persist the selected direction and count: {saved}"
            )
            .into());
        }
        if !driver.find_all_css(
            "[data-ai-model], [data-ai-endpoint], [data-translation-source], [data-translation-direction]"
        ).await?.is_empty() {
            return Err("Exercise exposes configuration controls owned by Settings or Practice".into());
        }
        Ok(())
    }

    pub async fn generate(&self) -> HarnessResult<()> {
        let driver = &self.context.driver;
        if driver
            .find_all_css("[data-practice-local-session]")
            .await?
            .is_empty()
        {
            driver
                .find_css("[data-translation-start]")
                .await?
                .click()
                .await?;
        }
        driver.query_wait_first("[data-translation-word-bank] [data-bank-tile], [data-generated-translation] [data-bank-tile]", Duration::from_secs(180), Duration::from_millis(200)).await?;
        Ok(())
    }

    pub async fn observe(&self) -> HarnessResult<TranslationObservation> {
        let driver = &self.context.driver;
        let session_host = !driver
            .find_all_css("[data-generated-translation]")
            .await?
            .is_empty();
        let sentence = driver
            .find_css(if session_host {
                "[data-generated-translation-source]"
            } else {
                "[data-sentence-translation] [lang]"
            })
            .await?;
        let source = sentence.text().await?;
        let target_language = driver
            .find_css(if session_host {
                "[data-generated-translation-target]"
            } else {
                "[data-translation-target]"
            })
            .await?
            .text()
            .await?;
        let target_language = if target_language.contains("English") {
            "en"
        } else {
            "zh"
        }
        .to_owned();
        let mut available_tiles = Vec::new();
        for element in driver
            .find_all_css(if session_host {
                "[data-generated-translation] [data-bank-tile]"
            } else {
                "[data-translation-word-bank] [data-bank-tile]"
            })
            .await?
        {
            let raw_index = element
                .attr("data-bank-tile")
                .await?
                .ok_or("Missing tile index")?;
            let index = raw_index
                .parse::<usize>()
                .map_err(|error| format!("Invalid tile index: {error}"))?;
            let position = element
                .attr("data-solution-order")
                .await?
                .ok_or("Missing solution order attribute")?;
            let solution_order = if position == "-1" {
                None
            } else {
                Some(position.parse::<usize>().map_err(|error| {
                    format!("Invalid solution position for tile {index}: {error}")
                })?)
            };
            let first = element.attr("data-solution-first").await?;
            let last = element.attr("data-solution-last").await?;
            available_tiles.push(VisibleTile {
                index,
                text: element.text().await?,
                solution_order,
                is_first: first.as_deref() == Some("true"),
                is_last: last.as_deref() == Some("true"),
            });
        }
        if source.trim().is_empty() || available_tiles.is_empty() {
            return Err("Incomplete learner-visible translation observation".into());
        }
        Ok(TranslationObservation {
            source,
            target_language,
            available_tiles,
        })
    }

    pub async fn select_tile(&self, tile: &VisibleTile) -> HarnessResult<()> {
        let driver = &self.context.driver;
        let selector = if !driver
            .find_all_css("[data-generated-translation]")
            .await?
            .is_empty()
        {
            format!(
                "[data-generated-translation] [data-bank-tile='{}']",
                tile.index
            )
        } else {
            format!(
                "[data-translation-word-bank] [data-bank-tile='{}']",
                tile.index
            )
        };
        driver.find_css(selector).await?.click().await?;
        Ok(())
    }

    pub async fn check(&self) -> HarnessResult<String> {
        let driver = &self.context.driver;
        let local = !driver
            .find_all_css("[data-generated-translation]")
            .await?
            .is_empty();
        let submit = if local {
            "[data-generated-translation-check]:not([disabled])"
        } else {
            "[data-translation-submit]:not([disabled])"
        };
        driver.find_css(submit).await?.click().await?;
        let selector = if local {
            "[data-generated-translation-feedback]"
        } else {
            "[data-translation-feedback]"
        };
        let feedback = driver
            .query_wait_single(
                selector,
                Duration::from_secs(180),
                Duration::from_millis(200),
            )
            .await?;
        let outcome = if local {
            feedback
                .find_css("[data-generated-translation-result]")
                .await?
                .attr("data-generated-translation-result")
                .await?
        } else {
            feedback.attr("data-feedback-result").await?
        };
        outcome.ok_or_else(|| "Translation feedback result missing".into())
    }

    pub async fn continue_after_feedback(&self) -> HarnessResult<()> {
        let driver = &self.context.driver;
        let button = if !driver
            .find_all_css("[data-generated-translation]")
            .await?
            .is_empty()
        {
            "[data-generated-translation-next]"
        } else {
            "[data-feedback-continue]"
        };
        driver.find_css(button).await?.click().await?;
        Ok(())
    }

    pub async fn wait_for_practice_progress(&self, question: usize) -> HarnessResult<()> {
        let label = format!("Exercise {question} of 5");
        let settled = self
            .context
            .driver
            .execute_async(
                crate::js_scripts::translation::TRANSLATION_WAIT_FOR_PRACTICE_PROGRESS_01,
                vec![serde_json::json!(label)],
            )
            .await?
            .convert::<bool>()?;
        if !settled {
            return Err(format!("Practice did not advance to exercise {question} in time").into());
        }
        Ok(())
    }

    pub async fn wait_for_question(&self, question: usize) -> HarnessResult<()> {
        let driver = &self.context.driver;
        let current = format!("Exercise {question} of 5");
        if !driver
            .find_all_css("[data-practice-local-session]")
            .await?
            .is_empty()
        {
            let question_visible = driver
                .execute_async(
                    crate::js_scripts::translation::TRANSLATION_WAIT_FOR_QUESTION_01,
                    vec![serde_json::json!(current)],
                )
                .await?
                .convert::<bool>()?;
            if !question_visible {
                return Err(format!("Translation did not reach {current} before timeout").into());
            }
            driver
                .query_wait_first(
                    "[data-generated-translation] [data-bank-tile]",
                    Duration::from_secs(180),
                    Duration::from_millis(200),
                )
                .await?;
        } else {
            driver.query_wait_first(format!(
                "[data-sentence-translation][data-translation-question='{question}'] [data-translation-word-bank] [data-bank-tile]"
            ), Duration::from_secs(180), Duration::from_millis(200)).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(positions: &[usize]) -> TranslationObservation {
        TranslationObservation {
            source: "我坐地铁。".to_owned(),
            target_language: "en".to_owned(),
            available_tiles: positions
                .iter()
                .enumerate()
                .map(|(index, position)| VisibleTile {
                    index,
                    text: format!("tile {index}"),
                    solution_order: Some(*position),
                    is_first: *position == 0,
                    is_last: *position + 1 == positions.len(),
                })
                .collect(),
        }
    }

    #[test]
    fn canonical_tiles_sort_by_solution_position_not_bank_index() {
        let observed = observation(&[2, 0, 1]);
        let indices = observed
            .canonical_tiles()
            .unwrap()
            .iter()
            .map(|tile| tile.index)
            .collect::<Vec<_>>();
        assert_eq!(indices, [1, 2, 0]);
    }

    #[test]
    fn canonical_tiles_reject_missing_duplicate_or_bad_boundary_metadata() {
        let mut observed = observation(&[2, 0, 1]);
        observed.available_tiles[0].solution_order = None;
        assert!(observed.canonical_tiles().is_err());
        let observed = observation(&[0, 0, 2]);
        assert!(observed.canonical_tiles().is_err());
        let mut observed = observation(&[2, 0, 1]);
        observed.available_tiles[1].is_first = false;
        assert!(observed.canonical_tiles().is_err());
    }
}
