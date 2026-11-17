use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{
        WebDriverActions,
        settings::{self, Section},
    },
};

async fn native_service_controls_are_desktop_only(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    for section in [Section::Ai, Section::Images] {
        settings::select_section(driver, section).await?;
        let panel = driver
            .query_single(match section {
                Section::Ai => "[data-native-service='ai']",
                Section::Images => "[data-native-service='image']",
                _ => unreachable!(),
            })
            .await?;
        zipeek_e2e::e2e_ensure!(panel.is_displayed().await?);
        zipeek_e2e::e2e_ensure_eq!(
            panel.find_all(thirtyfour::By::Css("button")).await?.len(),
            1,
            "Browser service status should expose Refresh but no native Start/Stop controls",
        );
    }
    ctx.checkpoint("browser-settings-no-tauri-controls").await?;
    Ok(())
}

#[tokio::test]
async fn browser_settings_never_offer_native_process_controls() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run(
            "browser_native_services_absent",
            native_service_controls_are_desktop_only,
        )
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
