use zipeek_e2e::journeys::{FullRunConfig, FullRunKind, run_journey};
use zipeek_e2e::{Harness, HarnessResult, SuiteKind};

#[tokio::test]
#[ignore = "End-to-end scoring/verified evidence integration is not implemented; run explicitly for screenshots and blocked-prerequisite diagnostics"]
async fn full_run_with_pedagogical_data() -> HarnessResult<()> {
    let config = FullRunConfig::from_environment(FullRunKind::Pedagogical)?;
    let budget = config.max_duration;
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("full_run_pedagogical", move |ctx| async move {
            tokio::time::timeout(budget, run_journey(ctx, config))
                .await
                .map_err(|_| -> zipeek_e2e::HarnessError {
                    format!(
                        "Pedagogical full run timed out after {} minutes",
                        budget.as_secs() / 60
                    )
                    .into()
                })?
        })
        .await?;
    harness.finish().await
}
