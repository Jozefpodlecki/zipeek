use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{WebDriverActions, translator},
};

async fn input_validation_and_clearing(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/translate-sentence").await?;
    driver.element(translator::ROOT).await?;
    let input = driver.element(translator::INPUT).await?;
    let submit = driver.element(translator::SUBMIT).await?;
    zipeek_e2e::e2e_ensure!(
        !submit.is_enabled().await?,
        "Empty translation should not be submitted"
    );
    input.send_keys("你好").await?;
    zipeek_e2e::e2e_ensure_eq!(input.prop("value").await?.as_deref(), Some("你好"));
    let clear = driver.element(translator::CLEAR).await?;
    clear.click().await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .element(translator::INPUT)
            .await?
            .prop("value")
            .await?
            .as_deref(),
        Some("")
    );
    zipeek_e2e::e2e_ensure!(
        !driver
            .element(translator::SUBMIT)
            .await?
            .is_enabled()
            .await?
    );
    ctx.checkpoint("translation-input-validation").await?;
    Ok(())
}

async fn direction_switch_updates_input_contract(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/translate-sentence").await?;
    driver.element(translator::ROOT).await?;
    let choices = driver.elements(translator::DIRECTION).await?;
    zipeek_e2e::e2e_ensure_eq!(choices.len(), 2, "Translation needs two direction choices");
    for index in 0..choices.len() {
        translator::choose_direction(driver, index).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_all_css(&format!("{}[aria-selected='true']", translator::DIRECTION))
                .await?
                .len(),
            1
        );
        let input_label = driver
            .element(translator::INPUT)
            .await?
            .attr("aria-label")
            .await?
            .unwrap_or_default();
        zipeek_e2e::e2e_ensure!(
            matches!(
                input_label.as_str(),
                "Chinese source sentence" | "English source sentence"
            ),
            "Unexpected input accessibility label: {input_label}"
        );
    }
    ctx.checkpoint("translation-direction-switch").await?;
    Ok(())
}

#[tokio::test]
async fn translator_interactions() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("translator-input-validation", input_validation_and_clearing)
        .await?;
    harness
        .run(
            "translator-direction-change",
            direction_switch_updates_input_contract,
        )
        .await?;
    harness.finish().await
}
