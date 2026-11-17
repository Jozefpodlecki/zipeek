use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const PAGE: &str = "[data-component='settings-page']";
pub const ACTIVE_PANEL: &str = "[data-settings-active-panel]";
pub const FONT_CHOICES: &str = "[data-hanzi-font-options]";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Section {
    Appearance,
    Characters,
    Ai,
    Images,
    Practice,
    Data,
}

impl Section {
    pub const ALL: [Self; 6] = [
        Self::Appearance,
        Self::Characters,
        Self::Ai,
        Self::Images,
        Self::Practice,
        Self::Data,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Appearance => "appearance",
            Self::Characters => "characters",
            Self::Ai => "ai",
            Self::Images => "images",
            Self::Practice => "practice",
            Self::Data => "data",
        }
    }
}

pub async fn select_section(driver: &WebDriver, section: Section) -> WebDriverResult<()> {
    let selector = format!("[data-settings-tab-button='{}']", section.key());
    driver
        .query_named(&selector, "Settings - select section")
        .await?
        .click()
        .await?;
    driver
        .query_named(
            &format!("{PAGE}[data-settings-tab='{}']", section.key()),
            "Settings - select section",
        )
        .await?;
    let selected_selector =
        format!("{selector}[aria-current='true'][aria-controls='settings-active-panel']");
    driver
        .query_named(&selected_selector, "Settings - selected navigation button")
        .await?;
    driver
        .query_named(
            "#settings-active-panel[role='region']",
            "Settings - controlled section",
        )
        .await?;
    Ok(())
}

pub async fn choose_font(driver: &WebDriver, key: &str) -> WebDriverResult<()> {
    select_section(driver, Section::Characters).await?;
    let selector = format!("[data-hanzi-font-option='{key}']");
    driver
        .query_named(&selector, "Settings - choose font")
        .await?
        .click()
        .await?;
    driver
        .query_named(
            &format!("{selector}[aria-pressed='true']"),
            "Settings - choose font",
        )
        .await?;
    driver
        .query_named(
            &format!("html[data-hanzi-font='{key}']"),
            "Selected Chinese font applied to document",
        )
        .await?;
    Ok(())
}

pub async fn choose_character_size(driver: &WebDriver, key: &str) -> WebDriverResult<()> {
    select_section(driver, Section::Characters).await?;
    let selector = format!("[data-character-size-option='{key}']");
    driver
        .query_named(&selector, "Settings - choose character size")
        .await?
        .click()
        .await?;
    driver
        .query_named(
            &format!("{selector}[aria-pressed='true']"),
            "Settings - choose character size",
        )
        .await?;
    driver
        .query_named(
            &format!("html[data-character-size='{key}']"),
            "Character size applied to document",
        )
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Section;

    #[test]
    fn every_settings_section_has_a_unique_nonempty_key() {
        let keys: Vec<_> = Section::ALL.into_iter().map(Section::key).collect();
        assert_eq!(keys.len(), 6);
        assert!(keys.iter().all(|key| !key.is_empty()));
        let unique: std::collections::BTreeSet<_> = keys.iter().copied().collect();
        assert_eq!(unique.len(), keys.len());
    }
}
