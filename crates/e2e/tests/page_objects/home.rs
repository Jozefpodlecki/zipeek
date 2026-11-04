use thirtyfour::prelude::*;

pub async fn brand(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver
        .query(By::Css("header a[href='#/']"))
        .desc("Home brand link")
        .single()
        .await
}
