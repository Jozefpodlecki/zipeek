use zipeek_e2e::ChromeDevTools;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{HarnessResult, ScenarioContext};

pub async fn shell_and_card_styling(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    let media = driver
        .execute(
            zipeek_e2e::js_scripts::platform::STYLING_SHELL_AND_CARD_STYLING_01,
            vec![],
        )
        .await?;
    ctx.checkpoint("styling-initial-state").await?;
    ChromeDevTools::new(&driver).media(media.json()).await?;
    for (saved, theme) in [(true, "dark"), (false, "light")] {
        zipeek_e2e::page_objects::browser::set_preference(driver, "dark", serde_json::json!(saved))
            .await?;
        driver.refresh().await?;
        driver.query_single("[data-database-state='ready']").await?;
        for width in [400, 640, 768, 1024, 1280, 1920] {
            ctx.viewport().responsive(width, 1000, width < 768).await?;
            driver.action_chain().move_to(0, 0).perform().await?;
            driver
                .execute_async(
                    zipeek_e2e::js_scripts::platform::STYLING_SHELL_AND_CARD_STYLING_02,
                    vec![],
                )
                .await?;
            let failures = driver
                .execute(
                    include_str!("../../../fixtures/browser_probes/styling_acceptance.js"),
                    vec![],
                )
                .await?
                .convert::<Vec<String>>()?;
            zipeek_e2e::e2e_ensure!(failures.is_empty(), "{theme} at {width}px: {failures:?}");
            zipeek_e2e::e2e_ensure!(
                zipeek_e2e::page_objects::browser::no_horizontal_overflow(driver).await?,
                "{theme} at {width}px caused horizontal page overflow"
            );
            let metrics = driver
                .execute(
                    include_str!("../../../fixtures/browser_probes/styling.js"),
                    vec![],
                )
                .await?
                .convert::<String>()?;
            std::fs::write(ctx.artifacts.join(format!("{theme}-{width}.json")), metrics)?;
            driver
                .screenshot(&ctx.artifacts.join(format!("{theme}-{width}.png")))
                .await?;
        }
    }
    ctx.viewport().responsive(400, 800, true).await?;
    driver
        .execute(
            zipeek_e2e::js_scripts::platform::STYLING_SHELL_AND_CARD_STYLING_03,
            vec![],
        )
        .await?;
    driver
        .screenshot(&ctx.artifacts.join("light-400-footer.png"))
        .await?;
    let toggle = driver.find_css("[data-theme-toggle]").await?;
    toggle.send_keys(" ").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(
                zipeek_e2e::js_scripts::platform::STYLING_SHELL_AND_CARD_STYLING_04,
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    let tile = driver.find_css("[data-navigation-tiles] > a").await?;
    driver
        .action_chain()
        .move_to_element_center(&tile)
        .perform()
        .await?;
    let motion = driver
        .execute(
            zipeek_e2e::js_scripts::platform::STYLING_SHELL_AND_CARD_STYLING_05,
            vec![],
        )
        .await?
        .convert::<(bool, String, String, String)>()?;
    zipeek_e2e::e2e_ensure!(
        motion.0 && motion.1 == "none" && motion.2 == "none" && motion.3 == "none",
        "Reduced-motion state: {motion:?}"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(
                zipeek_e2e::js_scripts::platform::STYLING_SHELL_AND_CARD_STYLING_06,
                vec![]
            )
            .await?
            .convert::<bool>()?,
        "Style tests must retain reduced-motion emulation through the final viewport"
    );
    ctx.checkpoint_visible(
        "styling-shell-remains-rendered",
        "[data-home], [data-component='browse-page']",
    )
    .await?;
    ctx.checkpoint("styling-verified-result").await?;
    Ok(())
}
