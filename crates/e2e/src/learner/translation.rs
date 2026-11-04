use std::time::Duration;
use thirtyfour::prelude::*;

use crate::{HarnessResult, ScenarioContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleTile {
    pub index: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationObservation {
    pub source: String,
    pub target_language: String,
    pub available_tiles: Vec<VisibleTile>,
}

pub struct TranslationPage<'a> {
    context: &'a ScenarioContext,
}

impl<'a> TranslationPage<'a> {
    pub fn new(context: &'a ScenarioContext) -> Self {
        Self { context }
    }

    pub async fn open(&self, vocabulary_source: &str) -> HarnessResult<()> {
        self.context.driver.goto(self.context.url(&format!(
            "/#/practice/translation?source={vocabulary_source}",
        ))).await?;
        self.context.driver.query(By::Css("[data-translation-source]")).single().await?;
        Ok(())
    }

    pub async fn configure(&self, model: &str) -> HarnessResult<()> {
        self.configure_endpoint(model, &self.context.ai_url).await
    }

    pub async fn configure_endpoint(&self, model: &str, url: &str) -> HarnessResult<()> {
        let driver = &self.context.driver;
        let endpoint = driver.find(By::Css("[data-ai-endpoint]")).await?;
        endpoint.clear().await?;
        endpoint.send_keys(url).await?;
        driver.execute(
            "const select = document.querySelector('[data-translation-direction]'); select.value = 'zh-to-en'; select.dispatchEvent(new Event('change', {bubbles: true}));",
            vec![],
        ).await?;
        let model_input = driver.find(By::Css("[data-ai-model]")).await?;
        model_input.send_keys(model).await?;
        Ok(())
    }

    pub async fn generate(&self) -> HarnessResult<()> {
        self.context.driver.find(By::Css("[data-translation-start]")).await?.click().await?;
        self.context.driver.query(By::Css("[data-translation-word-bank] [data-bank-tile]")).wait(Duration::from_secs(180), Duration::from_millis(200)).single().await?;
        Ok(())
    }

    pub async fn observe(&self) -> HarnessResult<TranslationObservation> {
        let driver = &self.context.driver;
        let sentence = driver.find(By::Css("[data-sentence-translation] [lang]")).await?;
        let source = sentence.text().await?;
        let target_language = driver.find(By::Css("[data-translation-target]")).await?.text().await?;
        let target_language = if target_language.contains("English") { "en" } else { "zh" }.to_owned();
        let mut available_tiles = Vec::new();
        for element in driver.find_all(By::Css("[data-translation-word-bank] [data-bank-tile]")).await? {
            let raw_index = element.attr("data-bank-tile").await?.ok_or("Missing tile index")?;
            let index = raw_index.parse::<usize>().map_err(|error| format!("Invalid tile index: {error}"))?;
            available_tiles.push(VisibleTile {index, text: element.text().await?});
        }
        if source.trim().is_empty() || available_tiles.is_empty() {
            return Err("Incomplete learner-visible translation observation".into());
        }
        Ok(TranslationObservation {source, target_language, available_tiles})
    }

    pub async fn select_tile(&self, tile: &VisibleTile) -> HarnessResult<()> {
        self.context.driver.find(By::Css(format!("[data-translation-word-bank] [data-bank-tile='{}']", tile.index))).await?.click().await?;
        Ok(())
    }

    pub async fn check(&self) -> HarnessResult<String> {
        self.context.driver.find(By::Css("[data-translation-submit]:not([disabled])")).await?.click().await?;
        let feedback = self.context.driver.query(By::Css("[data-translation-feedback]")).wait(Duration::from_secs(180), Duration::from_millis(200)).single().await?;
        Ok(feedback.attr("data-feedback-result").await?.ok_or("Feedback result missing")?)
    }

    pub async fn continue_after_feedback(&self) -> HarnessResult<()> {
        self.context.driver.find(By::Css("[data-feedback-continue]")).await?.click().await?;
        Ok(())
    }

    pub async fn wait_for_question(&self, question: usize) -> HarnessResult<()> {
        self.context.driver.query(By::Css(format!(
            "[data-sentence-translation][data-translation-question='{question}'] [data-translation-word-bank] [data-bank-tile]"
        ))).wait(Duration::from_secs(180), Duration::from_millis(200)).first().await?;
        Ok(())
    }
}
