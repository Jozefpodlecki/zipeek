use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn docs_reader_navigation(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/docs/README.md").await?;
    driver.query_single("[data-component='docs-page']").await?;
    let article = driver.query_single("[data-docs-markdown]").await?;
    zipeek_e2e::e2e_ensure!(article.text().await?.contains("Documentation index"));
    ctx.checkpoint("docs-rendered-markdown").await?;

    driver
        .query_single("aside[aria-label='On this page'] button")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .current_url()
            .await?
            .to_string()
            .contains("/docs/README.md"),
        "On-page navigation must not replace the Docs route"
    );
    driver
        .query_single("nav[aria-label='Document reading order'] a")
        .await?
        .click()
        .await?;
    driver.query_single("[data-component='docs-page']").await?;
    zipeek_e2e::e2e_ensure!(
        driver.current_url().await?.to_string().contains("/docs/"),
        "Previous/next navigation should stay in the Docs shell"
    );
    Ok(())
}

#[tokio::test]
async fn docs_reader_navigation_and_markdown_rendering() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness.run("docs_reader", docs_reader_navigation).await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}

async fn docs_cross_document_heading(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    let heading = "e2e-evidence-verification";
    ctx.open_route("/docs/project-management~tasks.md?heading=e2e-evidence-verification")
        .await?;
    driver
        .query_single("#e2e-evidence-verification")
        .await?;
    let positioned = driver
        .execute(
            "const heading = document.getElementById('e2e-evidence-verification'); if (!heading) return false; const top = heading.getBoundingClientRect().top; return top >= -100 && top < window.innerHeight;",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        positioned,
        "Deep-linked Docs heading must be scrolled into view"
    );
    ctx.checkpoint("docs-deep-link-positioned").await?;
    driver.refresh().await?;
    driver
        .query_single("#e2e-evidence-verification")
        .await?;
    let positioned_after_reload = driver
        .execute(
            "const heading = document.getElementById('e2e-evidence-verification'); if (!heading) return false; const top = heading.getBoundingClientRect().top; return top >= -100 && top < window.innerHeight;",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        positioned_after_reload,
        "Docs heading deep link must survive browser reload"
    );
    ctx.navigate_in_app(zipeek_e2e::AppRoute::new("/docs/README.md")?)
        .await?;
    driver.query_single("#documentation-rules").await?;
    driver.back().await?;
    driver.query_single(&format!("#{heading}")).await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .current_url()
            .await?
            .as_str()
            .contains("heading=e2e-evidence-verification"),
        "Browser back must restore the Docs heading route"
    );
    Ok(())
}

#[tokio::test]
async fn docs_cross_document_heading_deep_link() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("docs_cross_document_heading", docs_cross_document_heading)
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}

async fn docs_all_sections_and_issue_images(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    let sections = [
        ("getting-started/README.md", "Getting started"),
        ("product-ux/product.md", "Product & UX"),
        ("learning-practice/practice/practice-session-builder.md", "Learning & Practice"),
        ("games/sentence-relay.md", "Games"),
        ("dictionary-data/schema/db.md", "Dictionary & Data"),
        ("architecture/architecture.md", "Architecture"),
        ("engineering/coding-standards.md", "Engineering"),
        ("testing-quality/e2e/e2e-architecture.md", "Testing & Quality"),
        ("build-deployment/ci.md", "Build & Deployment"),
        ("project-management/tasks.md", "Project Management"),
        ("archive-history/RELEASE_HISTORY.md", "Archive & History"),
        ("issues/practice/ISS-002-practice-path-layout-and-empty-guidance.md", "Issues"),
    ];
    for (path, section) in sections {
        ctx.open_route(&format!("/docs/{}", path.replace('/', "~")))
            .await?;
        let article = driver.query_single("[data-docs-markdown]").await?;
        zipeek_e2e::e2e_ensure!(
            !article.text().await?.trim().is_empty(),
            "Docs section {section} rendered an empty article: {path}"
        );
        let breadcrumb = driver.query_single("nav[aria-label='Breadcrumb']").await?;
        zipeek_e2e::e2e_ensure!(
            breadcrumb.text().await?.contains(section),
            "Docs article {path} has an incorrect section breadcrumb: {section}"
        );
    }
    ctx.checkpoint("docs-twelve-sections-rendered").await?;
    let image = driver.query_single("[data-docs-markdown] img").await?;
    let valid_image = driver
        .execute(
            "const image = document.querySelector('[data-docs-markdown] img'); return image.decode().then(() => image.naturalWidth > 0, () => false);",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(valid_image, "Docs issue image did not decode");
    zipeek_e2e::e2e_ensure!(
        image.get_attribute("src").await?.is_some(),
        "Docs issue image is missing its source"
    );
    ctx.checkpoint("docs-issue-image-rendered").await?;
    Ok(())
}

#[tokio::test]
async fn docs_twelve_sections_and_issue_image_rendering() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("docs_twelve_sections", docs_all_sections_and_issue_images)
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}

async fn docs_search_finds_current_article(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/docs/README.md").await?;
    let search = driver.query_single("[data-docs-search]").await?;
    search.send_keys("Nonblocking vocabulary eligibility").await?;
    let article_link = driver
        .query_single("a[href*='dictionary-data~vocabulary~eligibility-counting.md']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        article_link.is_displayed().await?,
        "Docs search must expose the current eligibility article"
    );
    ctx.checkpoint("docs-search-result").await?;
    article_link.click().await?;
    let article = driver.query_single("[data-docs-markdown]").await?;
    zipeek_e2e::e2e_ensure!(
        article.text().await?.contains("Nonblocking vocabulary eligibility"),
        "Docs search result must open the matching article"
    );
    ctx.checkpoint("docs-search-navigation").await?;
    Ok(())
}

#[tokio::test]
async fn docs_search_and_result_navigation() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("docs_search_navigation", docs_search_finds_current_article)
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
