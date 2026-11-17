use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const BOARD: &str = "[data-matching]";
pub const NEXT: &str = "[data-matching] [data-match-next]";
pub const ENABLED_NEXT: &str = "[data-matching] [data-match-next]:not([disabled])";
pub const CORRECT: &str = "[data-match-feedback='correct']";
pub const WRONG: &str = "[data-match-feedback='wrong']";
pub const SELECTED: &str = "[data-match-selected='true']";
pub const SUMMARY: &str = "[data-practice-summary] [data-summary-chart='ready']";

#[derive(Clone, Copy)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    fn attribute(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

pub fn card_selector(side: Side, index: usize) -> String {
    format!(
        "[data-match-side='{}'][data-match-index='{index}']",
        side.attribute()
    )
}

pub async fn card(driver: &WebDriver, side: Side, index: usize) -> WebDriverResult<WebElement> {
    driver
        .query_named(card_selector(side, index), "Matching - card")
        .await
}

pub async fn click_card(driver: &WebDriver, side: Side, index: usize) -> WebDriverResult<()> {
    card(driver, side, index).await?.click().await
}

pub async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver.query_named(NEXT, "Matching - ready").await
}

pub async fn continue_round(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .query_named(ENABLED_NEXT, "Matching - continue round")
        .await?
        .click()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selectors_are_scoped_and_indexed() {
        assert_eq!(
            card_selector(Side::Left, 0),
            "[data-match-side='left'][data-match-index='0']"
        );
        assert_eq!(
            card_selector(Side::Right, 3),
            "[data-match-side='right'][data-match-index='3']"
        );
        assert!(ENABLED_NEXT.starts_with(BOARD));
    }
}
