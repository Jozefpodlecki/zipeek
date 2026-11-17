use zipeek_e2e::{Harness, HarnessResult};

#[path = "platform/bootstrap.rs"]
mod bootstrap;
#[path = "platform/database.rs"]
mod database;
#[path = "practice/fixed_mcq.rs"]
mod fixed_mcq;
#[path = "learning/learner_journeys.rs"]
mod learner_journeys;
#[path = "practice/mcq.rs"]
mod mcq;
#[path = "practice/practice.rs"]
mod practice;
#[path = "platform/providers.rs"]
mod providers;
#[path = "platform/recent_routes.rs"]
mod recent_routes;
#[path = "platform/routing.rs"]
mod routing;
#[path = "practice/setup.rs"]
mod setup;
#[path = "platform/shell.rs"]
mod shell;
#[path = "platform/styling.rs"]
mod styling;
#[path = "learning/translator_diagnostics.rs"]
mod translator_diagnostics;
#[path = "learning/word_sets.rs"]
mod word_sets;

pub async fn run(harness: &mut Harness) -> HarnessResult<()> {
    harness
        .run(
            "quick_write_hanzi_summary",
            setup::quick_write_hanzi_summary,
        )
        .await?;
    harness
        .run(
            "translator_fixture_requests_are_observable",
            translator_diagnostics::fixture_requests_are_observable,
        )
        .await?;
    harness
        .run(
            "translator_structured_rejection_falls_back",
            translator_diagnostics::structured_rejection_returns_plain_translation,
        )
        .await?;
    harness
        .run(
            "translator_structured_success_shows_analysis",
            translator_diagnostics::structured_success_shows_analysis,
        )
        .await?;
    harness
        .run(
            "translator_streaming_shows_progress",
            translator_diagnostics::streaming_progress_visible_before_result,
        )
        .await?;
    harness
        .run(
            "practice_progress_and_configuration_journey",
            learner_journeys::practice_progress_and_configuration_journey,
        )
        .await?;
    harness
        .run(
            "rapid_hsk_selection_remains_responsive",
            learner_journeys::rapid_hsk_selection_remains_responsive,
        )
        .await?;
    harness
        .run(
            "character_hunt_easy_session_launch",
            learner_journeys::character_hunt_easy_session_launch,
        )
        .await?;
    harness
        .run(
            "homepage_tiles_navigate_to_product_areas",
            learner_journeys::homepage_tiles_navigate_to_product_areas,
        )
        .await?;
    harness
        .run(
            "secondary_product_routes_render",
            learner_journeys::secondary_product_routes_render,
        )
        .await?;
    harness
        .run(
            "word_set_rename_switch_and_delete",
            learner_journeys::word_set_rename_switch_and_delete,
        )
        .await?;
    harness
        .run(
            "browse_filter_listen_and_save_word",
            learner_journeys::browse_filter_listen_and_save_word,
        )
        .await?;
    harness
        .run(
            "quick_practice_summary_defaults_to_fixed_five",
            learner_journeys::quick_practice_summary_defaults_to_fixed_five,
        )
        .await?;
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
            "setup_progress_mode_and_responsive_layout",
            setup::setup_progress_mode_and_responsive_layout,
        )
        .await?;
    harness
        .run(
            "setup_selection_guard_and_history",
            setup::setup_selection_guard_and_history,
        )
        .await?;
    harness
        .run(
            "listening_choice_session_and_restore",
            setup::listening_choice_session_and_restore,
        )
        .await?;
    harness
        .run(
            "sentence_builder_session_and_restore",
            setup::sentence_builder_session_and_restore,
        )
        .await?;
    harness
        .run(
            "ai_sentence_uuid_generates_restores_and_continues",
            setup::ai_sentence_uuid_generates_restores_and_continues,
        )
        .await?;
    harness
        .run(
            "ai_practice_requires_model_before_launch",
            setup::ai_practice_requires_model_before_launch,
        )
        .await?;
    harness
        .run(
            "character_hunt_uuid_restore_and_skip",
            setup::character_hunt_uuid_restore_and_skip,
        )
        .await?;
    harness
        .run(
            "spot_mistake_identify_repair_and_restore",
            setup::spot_mistake_identify_repair_and_restore,
        )
        .await?;
    harness
        .run(
            "sentence_transformation_answer_and_restore",
            setup::sentence_transformation_answer_and_restore,
        )
        .await?;
    harness
        .run(
            "audio_dictation_play_and_restore",
            setup::audio_dictation_play_and_restore,
        )
        .await?;
    harness
        .run(
            "authored_practice_continue_and_restore",
            setup::authored_practice_continue_and_restore,
        )
        .await?;
    harness
        .run(
            "standalone_practice_playlist_selection",
            setup::standalone_practice_playlist_selection,
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
            "planned_start_cancel_and_restart_preserves_finite_session",
            practice::planned_start_cancel_and_restart_preserves_finite_session,
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
            "session_only_practice_routes_reject_retired_urls",
            routing::session_only_practice_routes_reject_retired_urls,
        )
        .await?;
    harness
        .run(
            "first_visit_enters_app_before_database_readiness",
            shell::first_visit_enters_app_before_database_readiness,
        )
        .await?;
    harness
        .run(
            "onboarding_opt_in_persists_and_updates_header",
            shell::onboarding_opt_in_persists_and_updates_header,
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
    harness
        .run(
            "personal_sets_persist_across_navigation_and_backup_rejects_invalid_data",
            word_sets::personal_sets_persist_across_navigation_and_backup_rejects_invalid_data,
        )
        .await?;
    harness
        .run(
            "dictionary_search_is_independent_of_saved_word_sets",
            word_sets::dictionary_search_is_independent_of_saved_word_sets,
        )
        .await?;
    harness
        .run(
            "deleting_personal_set_persists_without_touching_dictionary",
            word_sets::deleting_personal_set_persists_without_touching_dictionary,
        )
        .await?;
    harness
        .run(
            "crossword_click_to_place_and_reset",
            recent_routes::crossword_click_to_place_and_reset,
        )
        .await?;
    harness
        .run(
            "settings_font_and_theme_are_independent",
            recent_routes::settings_font_and_theme_are_independent,
        )
        .await?;
    harness
        .run(
            "background_presets_persist_without_changing_display_mode",
            recent_routes::background_presets_persist_without_changing_display_mode,
        )
        .await?;
    harness
        .run(
            "dictionary_autocomplete_keyboard_and_pagination",
            recent_routes::dictionary_autocomplete_keyboard_and_pagination,
        )
        .await?;
    harness.prove_failure_isolation().await?;
    Ok(())
}

pub async fn run_ai_fixture(harness: &mut Harness) -> HarnessResult<()> {
    harness
        .run(
            "ai_practice_requires_model_before_launch",
            setup::ai_practice_requires_model_before_launch,
        )
        .await?;
    harness
        .run(
            "ai_sentence_uuid_generates_restores_and_continues",
            setup::ai_sentence_uuid_generates_restores_and_continues,
        )
        .await?;
    Ok(())
}
