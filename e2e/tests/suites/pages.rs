use e2e::{HarnessResult, ScenarioContext};

pub async fn pages_dictionary_and_workers_use_the_zipeek_base_path(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    driver.goto(ctx.url("/#/browse")).await?;
    crate::shell_page::ready(&driver).await?;
    assert!(driver.execute("const local = performance.getEntriesByType('resource').map(item => new URL(item.name)).filter(url => url.origin === location.origin); return local.some(url => url.pathname === '/zipeek/hanzi.duckdb') && local.some(url => url.pathname.includes('/zipeek/vendor/duckdb/')) && local.every(url => url.pathname.startsWith('/zipeek/'));", vec![]).await?.convert::<bool>()?);
    Ok(())
}
