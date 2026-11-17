use zipeek_e2e::page_objects;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind};

use page_objects::routes::{ROUTES, RouteCase};

async fn prepare_browser(ctx: &ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    driver.query_single("[data-component='app-shell']").await?;
    page_objects::browser::seed_default_preferences(driver).await?;
    driver.refresh().await?;
    Ok(())
}

async fn assert_route(ctx: &ScenarioContext, case: &RouteCase) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route(&case.navigation_path()).await?;
    let element = driver.query_single(case.marker).await.map_err(|error| {
        format!(
            "Route {} failed to render {}: {error}",
            case.path, case.marker
        )
    })?;
    zipeek_e2e::e2e_ensure!(
        element.is_displayed().await?,
        "Route {} rendered a hidden marker",
        case.path
    );
    let title = driver.title().await?;
    let not_found_expected = if case.path == "/debug" {
        !driver
            .find_all_css("[data-route-placeholder]")
            .await?
            .is_empty()
    } else {
        false
    };
    if case.path == "/debug" {
        let debug_present = !driver
            .find_all_css("[data-component='debug-page']")
            .await?
            .is_empty();
        zipeek_e2e::e2e_ensure!(
            debug_present != not_found_expected,
            "Debug route must render either its development page or the release-mode 404"
        );
    }
    zipeek_e2e::e2e_ensure!(
        title.ends_with(" · Zipeek") && (title.starts_with("Page not found") == not_found_expected),
        "Route {} has unexpected document title: {title}",
        case.path
    );
    Ok(())
}

async fn assert_special_routes(ctx: &ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.goto(ctx.url("/#/themes")).await?;
    driver
        .query_single("[data-component='settings-page']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver.current_url().await?.as_str().contains("/settings"),
        "Legacy themes route must redirect to Settings"
    );
    ctx.open_route("/games/not-a-real-game").await?;
    driver.query_single("[data-route-placeholder]").await?;
    zipeek_e2e::e2e_ensure!(driver.title().await?.starts_with("Page not found"));
    driver.goto(ctx.url("/#/404")).await?;
    driver.query_single("[data-route-placeholder]").await?;
    zipeek_e2e::e2e_ensure!(driver.title().await?.starts_with("Page not found"));
    Ok(())
}

#[derive(Clone, Copy)]
enum RouteGroup {
    Core,
    Practice,
    Features,
}

impl RouteGroup {
    fn contains(self, path: &str) -> bool {
        match self {
            Self::Practice => path == "/practice" || path.starts_with("/practice/"),
            Self::Features => {
                path.starts_with("/games") || matches!(path, "/write-hanzi" | "/translate-sentence")
            }
            Self::Core => !(Self::Practice.contains(path) || Self::Features.contains(path)),
        }
    }
}

async fn route_group(ctx: ScenarioContext, group: RouteGroup) -> HarnessResult<()> {
    prepare_browser(&ctx).await?;
    let mut failures = Vec::new();
    for case in ROUTES.iter().filter(|case| group.contains(case.path)) {
        if let Err(error) = assert_route(&ctx, case).await {
            failures.push(format!("{}: {error}", case.path));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} route checks failed:\n{}",
            failures.len(),
            failures.join("\n")
        )
        .into())
    }
}

async fn core_routes(ctx: ScenarioContext) -> HarnessResult<()> {
    route_group(ctx, RouteGroup::Core).await
}

async fn practice_routes(ctx: ScenarioContext) -> HarnessResult<()> {
    route_group(ctx, RouteGroup::Practice).await
}

async fn other_feature_routes(ctx: ScenarioContext) -> HarnessResult<()> {
    route_group(ctx, RouteGroup::Features).await
}

async fn redirects_and_not_found(ctx: ScenarioContext) -> HarnessResult<()> {
    prepare_browser(&ctx).await?;
    assert_special_routes(&ctx).await
}

async fn pages_core(ctx: ScenarioContext) -> HarnessResult<()> {
    route_group(ctx, RouteGroup::Core).await
}

async fn pages_practice(ctx: ScenarioContext) -> HarnessResult<()> {
    route_group(ctx, RouteGroup::Practice).await
}

async fn pages_features(ctx: ScenarioContext) -> HarnessResult<()> {
    route_group(ctx, RouteGroup::Features).await
}

async fn pages_special(ctx: ScenarioContext) -> HarnessResult<()> {
    prepare_browser(&ctx).await?;
    assert_special_routes(&ctx).await
}

async fn curated_game_routes(ctx: ScenarioContext) -> HarnessResult<()> {
    prepare_browser(&ctx).await?;
    let driver = &ctx.driver;
    ctx.open_route("/games").await?;
    for (route, marker) in [
        ("classifier-market", "[data-game-setup='Classifier Market']"),
        (
            "listening-navigator",
            "[data-game-setup='Listening Navigator']",
        ),
        ("sentence-relay", "[data-game-setup='Sentence Relay']"),
        ("market-bargain", "[data-game-setup='Market Bargain']"),
        ("radical-forge", "[data-game-setup='Radical Forge']"),
        ("dialogue-quest", "[data-game-setup='Dialogue Quest']"),
    ] {
        ctx.open_route(&format!("/games/{route}")).await?;
        let game = driver.query_single(marker).await?;
        zipeek_e2e::e2e_ensure!(
            game.is_displayed().await?,
            "Game {route} must show its setup"
        );
        zipeek_e2e::e2e_ensure!(
            driver.is_present("[data-game-start]").await?,
            "Game {route} must expose Start"
        );
    }
    ctx.checkpoint("curated-game-route-matrix").await?;
    Ok(())
}

#[tokio::test]
async fn route_matrix_root() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness.run("route_matrix_core", core_routes).await?;
    harness
        .run("route_matrix_practice", practice_routes)
        .await?;
    harness
        .run("route_matrix_features", other_feature_routes)
        .await?;
    harness
        .run("route_matrix_special", redirects_and_not_found)
        .await?;
    harness.finish().await
}

#[tokio::test]
async fn route_matrix_pages() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Pages).await?;
    harness.run("pages-route-core", pages_core).await?;
    harness.run("pages-route-practice", pages_practice).await?;
    harness.run("pages-route-features", pages_features).await?;
    harness
        .run("pages-curated-game-routes", curated_game_routes)
        .await?;
    harness.run("pages-route-special", pages_special).await?;
    harness.finish().await
}

async fn cedict_browse_search(ctx: ScenarioContext) -> HarnessResult<()> {
    prepare_browser(&ctx).await?;
    let driver = &ctx.driver;
    ctx.open_route("/browse").await?;
    page_objects::browse::ready(driver).await?;
    page_objects::browse::assert_healthy(driver).await?;
    ctx.open_route("/browse?query=%E6%91%87%E6%BB%9A%E4%B9%90")
        .await?;
    page_objects::browse::assert_word_visible(driver, "摇滚乐").await?;
    ctx.checkpoint_visible("cedict-browse-result", page_objects::browse::RESULTS)
        .await?;
    Ok(())
}

#[tokio::test]
async fn browse_accepts_non_hsk_cedict_words() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("browse_accepts_non_hsk_cedict_words", cedict_browse_search)
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}

#[test]
fn route_matrix_tracks_all_declared_paths() {
    use std::collections::BTreeSet;

    let declared: BTreeSet<_> = include_str!("../../zipeek-web/src/route.rs")
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("#[at(\"")?
                .split_once("\")]")
                .map(|(path, _)| path.to_owned())
        })
        .collect();
    let exercised: BTreeSet<_> = ROUTES
        .iter()
        .map(|case| match case.path {
            "/practice/session/missing-session" => "/practice/session/:id",
            "/courses/pinyin/tones" => "/courses/pinyin/:lesson",
            other => other,
        })
        .chain(["/themes", "/404"])
        .map(ToOwned::to_owned)
        .collect();
    assert_eq!(
        declared, exercised,
        "Every declared route needs a browser smoke case"
    );
}

#[test]
fn route_groups_partition_the_root_matrix() {
    for case in ROUTES {
        let memberships = [RouteGroup::Core, RouteGroup::Practice, RouteGroup::Features]
            .into_iter()
            .filter(|group| group.contains(case.path))
            .count();
        assert_eq!(
            memberships, 1,
            "Route must belong to exactly one root group: {}",
            case.path
        );
    }
}
