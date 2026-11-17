use zipeek_e2e::{HarnessResult, ScenarioContext};

pub async fn pages_dictionary_and_workers_use_the_zipeek_base_path(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/browse").await?;
    zipeek_e2e::page_objects::shell::ready(driver).await?;
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::dictionary_row_count(driver).await? > 0,
        "Pages build must expose nonempty verified dictionary metadata"
    );
    ctx.checkpoint_visible("browse-dictionary-ready", "[data-database-state=\"ready\"]")
        .await?;
    let background = zipeek_e2e::page_objects::shell::background_source(driver).await?;
    zipeek_e2e::e2e_ensure!(
        background.contains("public/backgrounds/")
            || background.starts_with("data:image/webp;base64,"),
        "Background must use a bundled asset or inline default: {background}"
    );
    let resources = driver
        .execute(
            include_str!("../../../fixtures/browser_probes/base_path.js"),
            vec![],
        )
        .await?;
    let report = resources.json();
    zipeek_e2e::e2e_ensure_eq!(
        report["dictionary"],
        true,
        "Dictionary did not use the project base path: {report}"
    );
    zipeek_e2e::e2e_ensure_eq!(
        report["vendor"],
        true,
        "DuckDB vendor assets were not loaded under the project base path: {report}"
    );
    zipeek_e2e::e2e_ensure_eq!(
        report["paths"],
        true,
        "A same-origin resource escaped the project base path: {report}"
    );
    ctx.checkpoint_visible(
        "pages-base-path-resources-verified",
        "[data-database-state=\"ready\"]",
    )
    .await?;
    Ok(())
}
