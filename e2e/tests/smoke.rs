use thirtyfour::prelude::*;

#[tokio::test]
async fn landing_page_loads() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::managed(caps).await?;

    driver.goto("http://127.0.0.1:1420/").await?;

    let heading = driver
        .query(By::Css("h1"))
        .desc("Landing page heading")
        .single()
        .await?;

    assert_eq!(heading.text().await?, "Zipeek");

    driver.quit().await?;

    Ok(())
}
