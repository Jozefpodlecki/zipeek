use super::route_source;
use std::collections::HashMap;

use yew::prelude::*;
use yew_router::prelude::{use_location, use_navigator};
use zipeek_core::{HskLevel, HskSystem};
use zipeek_exercises::practice_draft::{SessionLength, VocabularySource};

use crate::{
    components::practice::setup::practice_flow_navigation::PracticeStep,
    components::{
        PracticeExercisesPanel, PracticeFlowNavigation, PracticePathSelector, PracticeSetupSummary,
        PracticeStepFooter, PracticeVocabularyPanel,
    },
    hooks::{SavedExclusions, use_app_environment, use_saved_exclusions},
    reducers::{
        practice_setup_flow::{PracticeSetupAction, PracticeSetupFlow},
        setup::{SetupAction, SetupState},
    },
    services::{practice_session_store::PracticeSessionStore, word_sets},
};

#[function_component(PracticeSetup)]
pub fn practice_setup() -> Html {
    let state = use_reducer_eq(SetupState::default);
    let session_revision = use_state(|| 0_u64);
    {
        let revision = session_revision.clone();
        use_effect_with((), move |_| {
            let window = gloo::utils::window();
            let on_change = {
                let revision = revision.clone();
                gloo::events::EventListener::new(
                    &window,
                    crate::services::practice_session_store::SESSION_CHANGE_EVENT,
                    move |_| {
                        revision.set(revision.wrapping_add(1));
                    },
                )
            };
            let on_external_change =
                gloo::events::EventListener::new(&window, "storage", move |_| {
                    revision.set(revision.wrapping_add(1));
                });
            move || drop((on_change, on_external_change))
        });
    }
    let saved = use_saved_exclusions();
    let navigator = use_navigator().expect("Practice setup requires the router provider");
    let environment = use_app_environment();
    let storage = environment.local_storage.clone();
    let personal_sets = use_state(move || word_sets::read(&storage));
    let location = use_location();
    let preselected_source = route_source::preselected_source(
        location
            .as_ref()
            .and_then(|location| location.query::<HashMap<String, String>>().ok())
            .as_ref(),
    );
    let flow = use_reducer(|| PracticeSetupFlow::with_source(Some(preselected_source)));
    let profiles =
        super::profiles::use_profile_controls(environment.local_storage.clone(), flow.clone());
    let draft = &flow.draft;
    let on_path = {
        let flow = flow.clone();
        Callback::from(move |advanced: bool| {
            flow.dispatch(PracticeSetupAction::SelectPath(advanced));
        })
    };
    let on_select_step = {
        let flow = flow.clone();
        Callback::from(move |step: PracticeStep| {
            flow.dispatch(PracticeSetupAction::SelectStep(step));
        })
    };
    let on_back = {
        let flow = flow.clone();
        Callback::from(move |(): ()| {
            flow.dispatch(PracticeSetupAction::Back);
        })
    };
    let on_continue = {
        let flow = flow.clone();
        Callback::from(move |(): ()| {
            flow.dispatch(PracticeSetupAction::Continue);
        })
    };
    {
        let dispatch = state.dispatcher();
        use_effect_with(draft.vocabulary.clone(), move |source| {
            if let VocabularySource::Dictionary(filters) = source {
                if let Some(band) = filters.hsk.iter().next() {
                    let level = HskSystem::from_number(band.system).and_then(|system| {
                        HskLevel::try_from((system, i32::from(band.level))).ok()
                    });
                    if let Some(level) = level {
                        dispatch.dispatch(SetupAction::SelectHsk(level));
                    }
                }
            }
        });
    }
    let on_vocabulary = {
        let flow = flow.clone();
        Callback::from(move |source: VocabularySource| {
            flow.dispatch(PracticeSetupAction::SetVocabulary(source));
        })
    };
    let on_toggle_hsk_band = {
        let flow = flow.clone();
        Callback::from(move |band: zipeek_exercises::practice_draft::HskBand| {
            flow.dispatch(PracticeSetupAction::ToggleHskBand(band));
        })
    };
    let on_select_hsk_standard = {
        let flow = flow.clone();
        Callback::from(move |system: u8| {
            flow.dispatch(PracticeSetupAction::SelectHskStandard(system));
        })
    };
    let selected_variants = draft.active_formats();
    let personal_reference = match &draft.vocabulary {
        VocabularySource::PersonalSet { reference } => reference.as_str(),
        VocabularySource::Dictionary(_) => "",
    };
    let snapshot_error = match &*personal_sets {
        Ok(sets) => word_sets::practice_snapshot(sets, personal_reference).err(),
        Err(error) if !personal_reference.is_empty() => {
            Some(format!("Word Sets unavailable: {error}"))
        }
        Err(_) => None,
    };
    let _session_revision = *session_revision;
    let session_inventory =
        PracticeSessionStore::new(environment.local_storage.clone()).recoverable_sessions();
    let session_creation_blocked = match &session_inventory {
        Ok(inventory) => !inventory.sessions.is_empty() || inventory.invalid_records > 0,
        Err(_) => true,
    };
    let eligibility = super::validation::evaluate(
        draft,
        &selected_variants,
        &personal_sets,
        matches!(saved.status, SavedExclusions::Ready(_)),
        snapshot_error.is_none(),
        session_creation_blocked,
        super::validation::ai_model_configured(),
    );
    let can_start = eligibility.can_start;
    let start_gate_message = eligibility.message;
    let on_start_any = super::launch::start(
        navigator.clone(),
        (*draft).clone(),
        (*personal_sets).clone(),
        environment.local_storage.clone(),
        flow.exercise_options,
        can_start,
        flow.clone(),
    );
    let on_length = {
        let flow = flow.clone();
        Callback::from(move |length: SessionLength| {
            flow.dispatch(PracticeSetupAction::SetLength(length));
        })
    };
    let on_count = {
        let flow = flow.clone();
        Callback::from(move |count: u16| {
            flow.dispatch(PracticeSetupAction::SetTranslationCount(count));
        })
    };
    let active_panel = match flow.step {
        PracticeStep::Path => html! {
            <PracticePathSelector
                advanced={flow.advanced}
                on_select={on_path.clone()}
                profiles={(*profiles.profiles).clone()}
                pending_delete={(*profiles.pending_delete).clone()}
                on_load_profile={profiles.on_load.clone()}
                on_request_delete={profiles.on_request_delete.clone()}
                on_confirm_delete={profiles.on_confirm_delete.clone()}
                on_cancel_delete={profiles.on_cancel_delete.clone()}
            />
        },
        PracticeStep::Exercises => html! {
            <PracticeExercisesPanel flow={flow.clone()} />
        },
        PracticeStep::Vocabulary => html! {
            <PracticeVocabularyPanel
                source={draft.vocabulary.clone()}
                personal_sets={(*personal_sets).clone()}
                on_change={on_vocabulary.clone()}
                on_toggle_hsk_band={on_toggle_hsk_band.clone()}
                on_select_hsk_standard={on_select_hsk_standard.clone()}
            />
        },
        PracticeStep::Summary => html! {
            <PracticeSetupSummary
                draft={(*draft).clone()}
                variants={selected_variants.clone()}
                plan_rounds={draft.length.resumable_rounds()}
                can_start={can_start}
                start_failed={flow.start_failed}
                gate_message={start_gate_message.to_owned()}
                on_length={on_length.clone()}
                on_count={on_count.clone()}
                on_start={on_start_any.clone()}
                on_back={on_back.clone()}
                profile_name={(*profiles.name).clone()}
                profile_error={(*profiles.error).clone()}
                profile_saved_notice={(*profiles.saved_notice).clone()}
                loaded_profile={(*profiles.loaded).clone()}
                profile_advanced={flow.advanced}
                profile_options={flow.exercise_options}
                on_profile_name={profiles.on_name_change.clone()}
                on_save_profile={profiles.on_save.clone()}
            />
        },
    };

    let practice_step_footer = {
        if flow.step == PracticeStep::Summary {
            Html::default()
        } else {
            html! {
                <PracticeStepFooter
                    current={flow.step}
                    exercise_stage={flow.exercise_stage}
                    can_continue={flow.step != PracticeStep::Exercises || !draft.exercise_ids.is_empty()}
                    embedded={flow.step == PracticeStep::Exercises}
                    on_back={on_back}
                    on_continue={on_continue}
                />
            }
        }
    };

    if session_creation_blocked {
        let existing = session_inventory
            .as_ref()
            .ok()
            .and_then(|inventory| inventory.sessions.first());
        let warning = match &session_inventory {
            Ok(inventory) if inventory.invalid_records > 0 => Some(format!(
                "{} saved Practice record(s) need recovery. No records were deleted.",
                inventory.invalid_records,
            )),
            Err(error) => Some(format!(
                "Could not check saved Practice sessions: {error:?}"
            )),
            _ => None,
        };
        return html! {
            <super::session_recovery::PracticeSessionRecovery
                session={existing.cloned()}
                warning={warning}
            />
        };
    }

    html! {
        <section
            data-component="practice-setup-page"
            data-practice-setup=""
            class={classes!(
                "flex", "w-full", "min-h-0", "flex-col", "gap-3",
                if flow.step == PracticeStep::Exercises {
                    classes!("md:h-[calc(100dvh-var(--topbar-height)-1.5rem)]", "md:overflow-hidden")
                } else {
                    classes!("md:h-full", "md:overflow-y-auto")
                }
            )}
        >
            <PracticeFlowNavigation
                current={flow.step}
                furthest={flow.furthest}
                on_select={on_select_step.clone()}
            />
            if flow.step == PracticeStep::Exercises {
                <div
                    data-practice-exercise-workspace=""
                    class="flex min-h-0 flex-col rounded-xl border border-line bg-surface/95 md:h-0 md:flex-1 md:overflow-hidden"
                >
                    {practice_step_footer}
                    <div class="flex min-h-0 flex-col px-3 pb-3 md:h-0 md:flex-1 md:overflow-hidden">
                        {active_panel}
                    </div>
                </div>
            } else {
                {practice_step_footer}
                <div class="min-h-0">
                    {active_panel}
                </div>
            }
            {if flow.step != PracticeStep::Summary {
                if let Some(error) = &*profiles.error {
                    html! { <p role="alert" data-practice-profile-error="" class="text-sm text-red-400">{error.clone()}</p> }
                } else { Html::default() }
            } else { Html::default() }}
        </section>
    }
}
