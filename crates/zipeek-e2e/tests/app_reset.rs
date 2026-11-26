use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn seed_personal_data(ctx: &ScenarioContext) -> HarnessResult<()> {
    ctx.open_route("/settings?section=reset").await?;
    ctx.driver
        .query_single("[data-component='reset-settings']")
        .await?;
    let seeded = ctx.driver.execute_async(r#"
        const done = arguments[arguments.length - 1];
        (async () => {
            localStorage.setItem('zipeek.preferences.v1', '{}');
            localStorage.setItem('zipeek.reset.fixture', 'private');
            localStorage.setItem('zipeek.courses.progress.v1', '{}');
            localStorage.setItem('reset-fixture.unrelated', 'keep');
            sessionStorage.setItem('zipeek.reset.session', 'private');
            sessionStorage.setItem('reset-fixture.unrelated', 'keep');
            const root = await navigator.storage.getDirectory();
            for (const name of ['zipeek-personal-learning-v1.duckdb', 'zipeek-learning-snapshot-v1.json', 'reset-fixture-unrelated.txt']) {
                const handle = await root.getFileHandle(name, {create:true});
                const stream = await handle.createWritable();
                await stream.write(name);
                await stream.close();
            }
        })().then(() => done(true), error => done(String(error)));
    "#, vec![]).await?.convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(seeded);
    Ok(())
}

async fn assert_reset_finished(ctx: &ScenarioContext) -> HarnessResult<()> {
    ctx.driver
        .query_single("[data-component='welcome-landing']")
        .await?;
    let checked = ctx.driver.execute_async(r#"
        const done = arguments[arguments.length - 1];
        (async () => {
            const root = await navigator.storage.getDirectory();
            const names = [];
            for await (const name of root.keys()) names.push(name);
            const removed = !names.some(name => name.startsWith('zipeek-') || name.startsWith('zipeek.'));
            const cleared = [localStorage, sessionStorage].every(storage => !Object.keys(storage).some(key => (key.startsWith('zipeek.') || key === 'RUST_LOG') && key !== 'zipeek.e2e.settings.v1'));
            return removed && cleared && names.includes('reset-fixture-unrelated.txt') && localStorage.getItem('reset-fixture.unrelated') === 'keep' && sessionStorage.getItem('reset-fixture.unrelated') === 'keep' && location.hash === '#/';
        })().then(done, error => done(String(error)));
    "#, vec![]).await?.convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        checked,
        "Reset must clear app data, retain unrelated storage and return to fresh landing"
    );
    ctx.checkpoint("reset-completed-landing").await?;
    Ok(())
}

async fn confirmation_and_cleanup(ctx: ScenarioContext) -> HarnessResult<()> {
    seed_personal_data(&ctx).await?;
    ctx.driver
        .query_single("[data-reset-app]")
        .await?
        .click()
        .await?;
    ctx.driver.dismiss_alert().await?;
    let retained = ctx.driver.execute("return localStorage.getItem('zipeek.reset.fixture') === 'private' && !sessionStorage.getItem('zipeek.reset.confirmed');", vec![]).await?.convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        retained,
        "Cancel must preserve personal data and must not authorize reset"
    );
    ctx.checkpoint("reset-cancelled").await?;
    ctx.driver
        .query_single("[data-reset-app]")
        .await?
        .click()
        .await?;
    ctx.driver.accept_alert().await?;
    assert_reset_finished(&ctx).await
}

async fn direct_url_requires_confirmation(ctx: ScenarioContext) -> HarnessResult<()> {
    seed_personal_data(&ctx).await?;
    ctx.driver
        .goto(format!(
            "{}/#/reset-app",
            ctx.base_url.trim_end_matches('/')
        ))
        .await?;
    ctx.driver.refresh().await?;
    let runner = ctx
        .driver
        .query_single("[data-component='reset-runner']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        runner
            .text()
            .await?
            .contains("Reset has not been confirmed")
    );
    let retained = ctx
        .driver
        .execute(
            "return localStorage.getItem('zipeek.reset.fixture') === 'private';",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(retained);
    ctx.checkpoint("unconfirmed-reset-preserves-data").await?;
    Ok(())
}

async fn another_tab_blocks_reset(ctx: ScenarioContext) -> HarnessResult<()> {
    seed_personal_data(&ctx).await?;
    let original = ctx.driver.window().await?;
    let other = ctx.driver.new_tab().await?;
    ctx.driver.switch_to_window(other.clone()).await?;
    ctx.open_route("/settings?section=reset").await?;
    ctx.driver
        .query_single("[data-component='reset-settings']")
        .await?;
    ctx.driver.switch_to_window(original.clone()).await?;
    ctx.driver
        .query_single("[data-reset-app]")
        .await?
        .click()
        .await?;
    ctx.driver.accept_alert().await?;
    let error = ctx.driver.query_single("[data-reset-error]").await?;
    zipeek_e2e::e2e_ensure!(error.text().await?.contains("Close other Zipeek tabs"));
    let retained = ctx
        .driver
        .execute(
            "return localStorage.getItem('zipeek.reset.fixture') === 'private';",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(retained);
    ctx.checkpoint("other-tab-blocks-reset").await?;
    ctx.driver.switch_to_window(other).await?;
    ctx.driver.close_window().await?;
    ctx.driver.switch_to_window(original).await?;
    ctx.driver
        .query_single("[data-reset-retry]")
        .await?
        .click()
        .await?;
    assert_reset_finished(&ctx).await
}

async fn native_failure_requires_successful_retry(ctx: ScenarioContext) -> HarnessResult<()> {
    seed_personal_data(&ctx).await?;
    ctx.driver
        .execute(
            "localStorage.setItem('reset-fixture.fail-native', '1');",
            vec![],
        )
        .await?;
    ctx.add_init_script(r#"
        window.__TAURI__ = {core: {
            isTauri: () => true,
            invoke: async (command, args) => {
                if (command === 'personal_data_reset') {
                    if (args.confirmed !== true) throw new Error('Missing reset confirmation');
                    const count = Number(sessionStorage.getItem('reset-fixture.native-calls') || '0') + 1;
                    sessionStorage.setItem('reset-fixture.native-calls', String(count));
                    if (localStorage.getItem('reset-fixture.fail-native') === '1') throw {kind:'operation', details:'Native reset fixture failure'};
                    return;
                }
                if (command === 'database_open') throw {kind:'operation',details:'Dictionary is outside this fixture'};
                return {service:'ai',phase:'stopped',running:false,healthy:false,owned:false,message:'fixture'};
            }
        }};
    "#).await?;
    ctx.driver
        .query_single("[data-reset-app]")
        .await?
        .click()
        .await?;
    ctx.driver.accept_alert().await?;
    let error = ctx.driver.query_single("[data-reset-error]").await?;
    zipeek_e2e::e2e_ensure!(error.text().await?.contains("Native reset fixture failure"));
    let retained = ctx
        .driver
        .execute(
            "return localStorage.getItem('zipeek.reset.fixture') === 'private';",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(retained);
    ctx.checkpoint("native-reset-failure").await?;
    ctx.driver
        .execute(
            "localStorage.removeItem('reset-fixture.fail-native');",
            vec![],
        )
        .await?;
    ctx.driver
        .query_single("[data-reset-retry]")
        .await?
        .click()
        .await?;
    assert_reset_finished(&ctx).await?;
    let calls = ctx
        .driver
        .execute(
            "return sessionStorage.getItem('reset-fixture.native-calls');",
            vec![],
        )
        .await?
        .convert::<String>()?;
    zipeek_e2e::e2e_ensure!(calls == "2");
    Ok(())
}

async fn run_suite(kind: SuiteKind) -> HarnessResult<()> {
    let mut harness = Harness::start(kind).await?;
    harness
        .run("reset_confirmation_and_cleanup", confirmation_and_cleanup)
        .await?;
    harness
        .run("reset_direct_url", direct_url_requires_confirmation)
        .await?;
    harness
        .run("reset_other_tab", another_tab_blocks_reset)
        .await?;
    harness
        .run(
            "reset_native_failure",
            native_failure_requires_successful_retry,
        )
        .await?;
    harness.finish().await
}

#[tokio::test]
async fn reset_at_development_root() -> HarnessResult<()> {
    run_suite(SuiteKind::Root).await
}

#[tokio::test]
async fn reset_at_pages_base_path() -> HarnessResult<()> {
    run_suite(SuiteKind::Pages).await
}
