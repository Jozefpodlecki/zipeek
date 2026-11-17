use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{
        WebDriverActions, navigation,
        routes::{ROUTES, RouteCase},
    },
};

#[derive(Clone, Copy)]
enum LayoutArea {
    Practice,
    Other,
}

impl LayoutArea {
    fn includes(self, path: &str) -> bool {
        match self {
            Self::Practice => path == "/practice" || path.starts_with("/practice/"),
            Self::Other => !(path == "/practice" || path.starts_with("/practice/")),
        }
    }
}

async fn check_layout(ctx: &ScenarioContext, case: &RouteCase, width: u32) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route(&case.navigation_path()).await?;
    driver
        .element(case.marker)
        .await
        .map_err(|error| format!("{} failed to render at {width}px: {error}", case.path))?;
    zipeek_e2e::e2e_ensure!(
        driver.element(case.marker).await?.is_displayed().await?,
        "{} rendered but its route marker is hidden at {width}px",
        case.path
    );
    let overflow = navigation::has_horizontal_overflow(driver).await?;
    if overflow {
        let measured = driver
            .execute(
                zipeek_e2e::js_scripts::accessibility::RESPONSIVE_MATRIX_CHECK_LAYOUT_01,
                vec![],
            )
            .await?;
        return Err(format!(
            "Horizontal overflow at {} ({}px): {}",
            case.path,
            width,
            measured.json()
        )
        .into());
    }
    Ok(())
}

async fn verify_area(ctx: ScenarioContext, area: LayoutArea, width: u32) -> HarnessResult<()> {
    let observed = ctx.viewport().responsive(width, 850, width < 600).await?;
    let mut failures = Vec::new();
    for case in ROUTES.iter().filter(|case| area.includes(case.path)) {
        if let Err(error) = check_layout(&ctx, case, observed.width).await {
            failures.push(format!("{}: {error}", case.path));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} routes failed the {width}px requested ({}px measured) layout check:\n{}",
            failures.len(),
            observed.width,
            failures.join("\n")
        )
        .into())
    }
}

async fn narrow_phone_practice(ctx: ScenarioContext) -> HarnessResult<()> {
    verify_area(ctx, LayoutArea::Practice, 320).await
}
async fn narrow_phone_other(ctx: ScenarioContext) -> HarnessResult<()> {
    verify_area(ctx, LayoutArea::Other, 320).await
}
async fn mobile_practice(ctx: ScenarioContext) -> HarnessResult<()> {
    verify_area(ctx, LayoutArea::Practice, 390).await
}
async fn mobile_other(ctx: ScenarioContext) -> HarnessResult<()> {
    verify_area(ctx, LayoutArea::Other, 390).await
}
async fn tablet_practice(ctx: ScenarioContext) -> HarnessResult<()> {
    verify_area(ctx, LayoutArea::Practice, 768).await
}
async fn tablet_other(ctx: ScenarioContext) -> HarnessResult<()> {
    verify_area(ctx, LayoutArea::Other, 768).await
}

#[tokio::test]
async fn responsive_matrix() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("responsive-narrow-phone-practice", narrow_phone_practice)
        .await?;
    harness
        .run("responsive-narrow-phone-other", narrow_phone_other)
        .await?;
    harness
        .run("responsive-mobile-practice", mobile_practice)
        .await?;
    harness.run("responsive-mobile-other", mobile_other).await?;
    harness
        .run("responsive-tablet-practice", tablet_practice)
        .await?;
    harness.run("responsive-tablet-other", tablet_other).await?;
    harness.finish().await
}

#[test]
fn responsive_matrix_covers_all_routes() {
    for case in ROUTES {
        assert_eq!(
            [LayoutArea::Practice, LayoutArea::Other]
                .into_iter()
                .filter(|area| area.includes(case.path))
                .count(),
            1
        );
    }
}
