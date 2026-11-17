use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{
        WebDriverActions,
        routes::{ROUTES, RouteCase},
    },
};

#[derive(Clone, Copy)]
enum Area {
    Core,
    Practice,
    Learning,
}

impl Area {
    fn includes(self, path: &str) -> bool {
        match self {
            Self::Practice => path.starts_with("/practice/"),
            Self::Learning => {
                path.starts_with("/games/")
                    || path.starts_with("/courses/")
                    || matches!(path, "/write-hanzi" | "/translate-sentence")
            }
            Self::Core => !Self::Practice.includes(path) && !Self::Learning.includes(path),
        }
    }
}

async fn visit_and_check(ctx: &ScenarioContext, case: &RouteCase) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route(&case.navigation_path()).await?;
    let marker = driver
        .element(case.marker)
        .await
        .map_err(|error| format!("{} did not render {}: {error}", case.path, case.marker))?;
    zipeek_e2e::e2e_ensure!(
        marker.is_displayed().await?,
        "{} rendered an invisible route marker",
        case.path
    );
    let issues = driver
        .execute(
            zipeek_e2e::js_scripts::accessibility::ACCESSIBILITY_MATRIX_VISIT_AND_CHECK_01,
            vec![],
        )
        .await?;
    let value = issues.json();
    let repeated = value
        .get("repeated")
        .and_then(serde_json::Value::as_array)
        .ok_or("Accessibility probe did not return duplicate ID list")?;
    let unnamed = value
        .get("unnamed")
        .and_then(serde_json::Value::as_array)
        .ok_or("Accessibility probe did not return button name list")?;
    zipeek_e2e::e2e_ensure!(
        repeated.is_empty(),
        "{} has duplicate element IDs: {repeated:?}",
        case.path
    );
    zipeek_e2e::e2e_ensure!(
        unnamed.is_empty(),
        "{} has buttons with no discernible name: {unnamed:?}",
        case.path
    );
    Ok(())
}

async fn area_accessibility(ctx: ScenarioContext, area: Area) -> HarnessResult<()> {
    let mut failures = Vec::new();
    for case in ROUTES.iter().filter(|case| area.includes(case.path)) {
        if let Err(error) = visit_and_check(&ctx, case).await {
            failures.push(format!("{}: {error}", case.path));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} accessibility route checks failed:\n{}",
            failures.len(),
            failures.join("\n")
        )
        .into())
    }
}

async fn core_accessibility(ctx: ScenarioContext) -> HarnessResult<()> {
    area_accessibility(ctx, Area::Core).await
}

async fn practice_accessibility(ctx: ScenarioContext) -> HarnessResult<()> {
    area_accessibility(ctx, Area::Practice).await
}

async fn learning_accessibility(ctx: ScenarioContext) -> HarnessResult<()> {
    area_accessibility(ctx, Area::Learning).await
}

#[tokio::test]
async fn accessibility_matrix() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("a11y-core-route-contract", core_accessibility)
        .await?;
    harness
        .run("a11y-practice-route-contract", practice_accessibility)
        .await?;
    harness
        .run("a11y-learning-route-contract", learning_accessibility)
        .await?;
    harness.finish().await
}

#[test]
fn accessibility_partition_accounts_for_every_route() {
    for route in ROUTES {
        assert_eq!(
            [Area::Core, Area::Practice, Area::Learning]
                .iter()
                .filter(|area| area.includes(route.path))
                .count(),
            1,
            "Route {} must be assigned to exactly one accessibility group",
            route.path
        );
    }
}
