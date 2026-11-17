use std::time::Duration;
use thirtyfour::prelude::*;

#[allow(async_fn_in_trait)]
pub trait WebDriverActions {
    async fn find_css(&self, selector: impl AsRef<str>) -> WebDriverResult<WebElement>;
    async fn find_all_css(&self, selector: impl AsRef<str>) -> WebDriverResult<Vec<WebElement>>;
    async fn query_single(&self, selector: impl AsRef<str>) -> WebDriverResult<WebElement>;
    async fn query_first(&self, selector: impl AsRef<str>) -> WebDriverResult<WebElement>;
    async fn query_named(
        &self,
        selector: impl AsRef<str>,
        description: &str,
    ) -> WebDriverResult<WebElement>;
    async fn query_first_named(
        &self,
        selector: impl AsRef<str>,
        description: &str,
    ) -> WebDriverResult<WebElement>;
    async fn query_wait_single(
        &self,
        selector: impl AsRef<str>,
        timeout: Duration,
        interval: Duration,
    ) -> WebDriverResult<WebElement>;
    async fn query_wait_first(
        &self,
        selector: impl AsRef<str>,
        timeout: Duration,
        interval: Duration,
    ) -> WebDriverResult<WebElement>;
    async fn query_with_text(
        &self,
        selector: impl AsRef<str>,
        text: &str,
    ) -> WebDriverResult<WebElement>;
    async fn element(&self, selector: &str) -> WebDriverResult<WebElement>;
    async fn click(&self, selector: &str) -> WebDriverResult<()>;
    async fn click_with_text(&self, selector: &str, text: &str) -> WebDriverResult<()>;
    async fn elements(&self, selector: &str) -> WebDriverResult<Vec<WebElement>>;
    async fn is_present(&self, selector: &str) -> WebDriverResult<bool>;
}

impl WebDriverActions for WebDriver {
    async fn find_css(&self, selector: impl AsRef<str>) -> WebDriverResult<WebElement> {
        self.find(By::Css(selector.as_ref())).await
    }

    async fn find_all_css(&self, selector: impl AsRef<str>) -> WebDriverResult<Vec<WebElement>> {
        self.find_all(By::Css(selector.as_ref())).await
    }

    async fn query_single(&self, selector: impl AsRef<str>) -> WebDriverResult<WebElement> {
        self.query(By::Css(selector.as_ref()))
            .desc(selector.as_ref())
            .single()
            .await
    }

    async fn query_first(&self, selector: impl AsRef<str>) -> WebDriverResult<WebElement> {
        self.query(By::Css(selector.as_ref()))
            .desc(selector.as_ref())
            .first()
            .await
    }

    async fn query_named(
        &self,
        selector: impl AsRef<str>,
        description: &str,
    ) -> WebDriverResult<WebElement> {
        self.query(By::Css(selector.as_ref()))
            .desc(description)
            .single()
            .await
    }

    async fn query_first_named(
        &self,
        selector: impl AsRef<str>,
        description: &str,
    ) -> WebDriverResult<WebElement> {
        self.query(By::Css(selector.as_ref()))
            .desc(description)
            .first()
            .await
    }

    async fn query_wait_single(
        &self,
        selector: impl AsRef<str>,
        timeout: Duration,
        interval: Duration,
    ) -> WebDriverResult<WebElement> {
        self.query(By::Css(selector.as_ref()))
            .wait(timeout, interval)
            .single()
            .await
    }

    async fn query_wait_first(
        &self,
        selector: impl AsRef<str>,
        timeout: Duration,
        interval: Duration,
    ) -> WebDriverResult<WebElement> {
        self.query(By::Css(selector.as_ref()))
            .wait(timeout, interval)
            .first()
            .await
    }

    async fn query_with_text(
        &self,
        selector: impl AsRef<str>,
        text: &str,
    ) -> WebDriverResult<WebElement> {
        let description = format!("CSS selector: {} with text: {text}", selector.as_ref());
        self.query(By::Css(selector.as_ref()))
            .with_text(text.to_owned())
            .desc(&description)
            .single()
            .await
    }

    async fn element(&self, selector: &str) -> WebDriverResult<WebElement> {
        self.query_single(selector).await
    }

    async fn click(&self, selector: &str) -> WebDriverResult<()> {
        self.query_single(selector).await?.click().await
    }

    async fn click_with_text(&self, selector: &str, text: &str) -> WebDriverResult<()> {
        self.query_with_text(selector, text).await?.click().await
    }

    async fn elements(&self, selector: &str) -> WebDriverResult<Vec<WebElement>> {
        self.find_all_css(selector).await
    }

    async fn is_present(&self, selector: &str) -> WebDriverResult<bool> {
        Ok(!self.elements(selector).await?.is_empty())
    }
}

#[allow(async_fn_in_trait)]
pub trait WebElementQueryActions {
    async fn find_css(&self, selector: impl AsRef<str>) -> WebDriverResult<WebElement>;
    async fn find_all_css(&self, selector: impl AsRef<str>) -> WebDriverResult<Vec<WebElement>>;
    async fn query_single(&self, selector: impl AsRef<str>) -> WebDriverResult<WebElement>;
    async fn query_named(
        &self,
        selector: impl AsRef<str>,
        description: &str,
    ) -> WebDriverResult<WebElement>;
}

impl WebElementQueryActions for WebElement {
    async fn find_css(&self, selector: impl AsRef<str>) -> WebDriverResult<WebElement> {
        self.find(By::Css(selector.as_ref())).await
    }

    async fn find_all_css(&self, selector: impl AsRef<str>) -> WebDriverResult<Vec<WebElement>> {
        self.find_all(By::Css(selector.as_ref())).await
    }

    async fn query_single(&self, selector: impl AsRef<str>) -> WebDriverResult<WebElement> {
        self.query(By::Css(selector.as_ref())).single().await
    }

    async fn query_named(
        &self,
        selector: impl AsRef<str>,
        description: &str,
    ) -> WebDriverResult<WebElement> {
        self.query(By::Css(selector.as_ref()))
            .desc(description)
            .single()
            .await
    }
}
