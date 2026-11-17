use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const BRAND: &str = crate::page_objects::selectors::shell::BRAND_LINK;

pub async fn brand(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver.query_named(BRAND, "Home brand link").await
}

#[cfg(test)]
mod tests {
    use super::BRAND;

    #[test]
    fn brand_link_targets_the_root_hash_route() {
        assert_eq!(BRAND, "[data-brand-link] a");
    }
}
