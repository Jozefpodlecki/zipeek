use zipeek_e2e::journeys::{FullRunConfig, FullRunKind, run_journey};
use zipeek_e2e::{Harness, HarnessResult, SuiteKind};

#[tokio::test]
#[ignore = "Requires a fresh Chrome profile, running deterministic AI/image fixtures, and independent OPFS inspection; run explicitly for full acceptance"]
async fn full_run_without_pedagogical_data() -> HarnessResult<()> {
    let config = FullRunConfig::from_environment(FullRunKind::Standard)?;
    let budget = config.max_duration;
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("full_run_standard", move |ctx| async move {
            tokio::time::timeout(budget, run_journey(ctx, config))
                .await
                .map_err(|_| -> zipeek_e2e::HarnessError {
                    format!(
                        "Standard full run timed out after {} minutes",
                        budget.as_secs() / 60
                    )
                    .into()
                })?
        })
        .await?;
    harness.finish().await
}
