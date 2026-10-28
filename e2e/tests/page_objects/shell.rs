use thirtyfour::prelude::*;

pub async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver.query(By::Css("[data-database-state='ready'][data-database-rows='12100'][data-database-engine='v1.4.3']")).single().await
}
