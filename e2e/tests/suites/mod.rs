use e2e::{Harness, HarnessResult};

mod bootstrap;
mod database;
mod fixed_mcq;
mod mcq;
mod practice;
mod providers;
mod routing;
mod setup;
mod shell;
mod styling;

pub async fn run(harness: &mut Harness) -> HarnessResult<()> {
    harness
        .run(
            "fixed_mcq_counts_stats_and_restart",
            fixed_mcq::fixed_mcq_counts_stats_and_restart,
        )
        .await?;
    harness
        .run(
            "fixed_summary_motion_layout_and_setup",
            fixed_mcq::fixed_summary_motion_layout_and_setup,
        )
        .await?;
    harness
        .run(
            "endless_mcq_answers_lock_and_continue",
            mcq::endless_mcq_answers_lock_and_continue,
        )
        .await?;
    harness
        .run(
            "endless_mcq_seed_wrap_and_responsive_layout",
            mcq::endless_mcq_seed_wrap_and_responsive_layout,
        )
        .await?;
    harness
        .run(
            "setup_controls_generate_all_six_practice_urls",
            setup::setup_controls_generate_all_six_practice_urls,
        )
        .await?;
    harness
        .run(
            "setup_exclusion_failure_retry_and_reset_error",
            setup::setup_exclusion_failure_retry_and_reset_error,
        )
        .await?;
    harness
        .run(
            "practice_rounds_normalize_and_replay",
            practice::practice_rounds_normalize_and_replay,
        )
        .await?;
    harness
        .run(
            "late_queries_and_retry_keep_current_route",
            practice::late_queries_and_retry_keep_current_route,
        )
        .await?;
    harness
        .run(
            "unknown_route_recovers_while_dictionary_fails",
            routing::unknown_route_recovers_while_dictionary_fails,
        )
        .await?;
    harness
        .run(
            "navigation_and_rapid_theme_commands_preserve_dictionary",
            providers::navigation_and_rapid_theme_commands_preserve_dictionary,
        )
        .await?;
    harness
        .run(
            "mounted_app_waits_for_dictionary_download",
            bootstrap::mounted_app_waits_for_dictionary_download,
        )
        .await?;
    harness
        .run(
            "bootstrap_loader_is_centered_and_landscape_survives_both_themes",
            bootstrap::bootstrap_loader_is_centered_and_landscape_survives_both_themes,
        )
        .await?;
    harness
        .run(
            "dictionary_failure_retry_and_real_read_only_queries",
            database::dictionary_failure_retry_and_real_read_only_queries,
        )
        .await?;
    harness
        .run(
            "adapter_reads_all_rows_and_rejects_writes",
            database::adapter_reads_all_rows_and_rejects_writes,
        )
        .await?;
    harness
        .run(
            "practice_query_decoding_preserves_shared_url",
            routing::practice_query_decoding_preserves_shared_url,
        )
        .await?;
    harness
        .run("landing_page_loads", shell::landing_page_loads)
        .await?;
    harness
        .run(
            "responsive_shell_preserves_navigation_theme_and_keyboard_access",
            shell::responsive_shell_preserves_navigation_theme_and_keyboard_access,
        )
        .await?;
    harness
        .run(
            "drawer_collapses_and_compact_desktop_has_no_unnecessary_scrollbar",
            shell::drawer_collapses_and_compact_desktop_has_no_unnecessary_scrollbar,
        )
        .await?;
    harness
        .run("shell_and_card_styling", styling::shell_and_card_styling)
        .await?;
    harness.prove_failure_isolation().await?;
    Ok(())
}
