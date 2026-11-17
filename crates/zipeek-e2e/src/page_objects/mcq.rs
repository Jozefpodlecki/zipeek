use crate::HarnessResult;
use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

const OPTION_SELECTOR: &str = "[data-mcq-option]";
const READY_SELECTOR: &str = "[data-mcq] [data-mcq-option='0']:not([disabled])";

pub async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver.query_named(READY_SELECTOR, "Mcq - ready").await
}

pub async fn option(driver: &WebDriver, index: usize) -> WebDriverResult<WebElement> {
    driver
        .find_css(format!("[data-mcq-option='{index}']"))
        .await
}

pub async fn answer(driver: &WebDriver, correct: bool) -> HarnessResult<()> {
    ready(driver).await?;
    let correct_index = super::round::mcq_correct_index(driver).await?;
    let options = driver.find_all_css(OPTION_SELECTOR).await?;
    crate::e2e_ensure_eq!(
        options.len(),
        3,
        "MCQ must expose exactly three options before answering"
    );
    let option_index = if correct {
        correct_index
    } else {
        (correct_index + 1) % 3
    };
    option(driver, option_index).await?.click().await?;
    driver
        .query_named("[data-mcq-continue]:not([disabled])", "Mcq - answer")
        .await?
        .send_keys(Key::Enter)
        .await?;
    Ok(())
}

pub async fn assert_answer_locked(driver: &WebDriver) -> HarnessResult<()> {
    let options = driver.find_all_css(OPTION_SELECTOR).await?;
    crate::e2e_ensure_eq!(options.len(), 3, "MCQ should render exactly three answers");
    for (index, option) in options.iter().enumerate() {
        crate::e2e_ensure!(
            !option.is_enabled().await?,
            "MCQ option {index} remained enabled after submitting an answer"
        );
    }
    let pressed = driver
        .find_all_css("[data-mcq-option][aria-pressed='true']")
        .await?;
    crate::e2e_ensure_eq!(
        pressed.len(),
        1,
        "Exactly one MCQ choice should be selected"
    );
    let correct_markers = driver
        .find_all_css("[data-mcq-answer-marker='correct']")
        .await?;
    crate::e2e_ensure_eq!(
        correct_markers.len(),
        1,
        "MCQ feedback must identify exactly one correct answer"
    );
    crate::e2e_ensure!(
        driver
            .find_css("[data-mcq-answer-feedback]")
            .await?
            .text()
            .await?
            .contains("correct answer"),
        "MCQ feedback must explain the correct answer"
    );
    Ok(())
}
