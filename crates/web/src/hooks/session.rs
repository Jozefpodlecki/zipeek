use crate::{
    components::QuizSessionContext,
    reducers::{
        matching::{CardChoice, MatchState},
        mcq::McqState,
        practice::PracticeStatus,
        session::{ActiveRound, SessionAction, SessionKey, SessionState},
    },
    route::Route,
    services::{
        exercise_types::{query_pairs, random_type},
        practice_url::fresh_offset,
    },
};
use gloo::timers::callback::Timeout;
use std::{collections::HashSet, rc::Rc};
use yew::prelude::*;
use yew_router::prelude::use_navigator;
use zipeek_core::practice::{ExerciseVariant, SeedDisposition, advance_offset};
use zipeek_exercises::session::ExerciseResult;

#[derive(Clone, PartialEq)]
pub(crate) struct PracticeControls {
    pub state: Rc<SessionState>,
    pub ready: bool,
    mcq_select: Callback<usize>,
    match_select: Callback<CardChoice>,
    escape: Callback<()>,
    advance: Callback<()>,
    restart: Callback<()>,
    back_to_setup: Callback<()>,
    navigation_failed: bool,
}
pub(crate) struct McqControls {
    pub state: Rc<McqState>,
    pub ready: bool,
    pub required: u64,
    pub select: Callback<usize>,
    pub advance: Callback<()>,
    pub restart: Callback<()>,
    pub back_to_setup: Callback<()>,
    pub navigation_failed: bool,
}
pub(crate) struct MatchingControls {
    pub state: Rc<MatchState>,
    pub ready: bool,
    pub required: u64,
    pub select: Callback<CardChoice>,
    pub escape: Callback<()>,
    pub advance: Callback<()>,
    pub restart: Callback<()>,
    pub back_to_setup: Callback<()>,
    pub navigation_failed: bool,
}
impl PracticeControls {
    pub fn mcq(&self) -> McqControls {
        let state = match &self.state.active {
            ActiveRound::Mcq(state) => state.clone(),
            _ => Rc::new(McqState::default()),
        };
        McqControls {
            state,
            ready: self.ready,
            required: self.state.required,
            select: self.mcq_select.clone(),
            advance: self.advance.clone(),
            restart: self.restart.clone(),
            back_to_setup: self.back_to_setup.clone(),
            navigation_failed: self.navigation_failed,
        }
    }
    pub fn matching(&self) -> MatchingControls {
        let state = match &self.state.active {
            ActiveRound::Matching(state) => state.clone(),
            _ => Rc::new(MatchState::default()),
        };
        MatchingControls {
            state,
            ready: self.ready,
            required: self.state.required,
            select: self.match_select.clone(),
            escape: self.escape.clone(),
            advance: self.advance.clone(),
            restart: self.restart.clone(),
            back_to_setup: self.back_to_setup.clone(),
            navigation_failed: self.navigation_failed,
        }
    }
}

#[hook]
pub(crate) fn use_practice_session(
    variant: ExerciseVariant,
    types: Vec<ExerciseVariant>,
    status: PracticeStatus,
    reload: Callback<()>,
) -> PracticeControls {
    let state = use_reducer_eq(SessionState::default);
    let shared_session = use_context::<QuizSessionContext>()
        .expect("QuizSessionProvider must wrap Practice");
    let navigator = use_navigator();
    let failed = use_state(|| false);
    let prepared = match status {
        PracticeStatus::Ready(prepared) => Some(prepared),
        _ => None,
    };
    let key = prepared
        .as_ref()
        .map(|prepared| SessionKey::new(variant, prepared.query.clone(), types.clone()));
    let instance_scope = use_state(|| shared_session.allocate_scope());
    let published_ids = use_mut_ref(HashSet::<String>::new);
    let pending_id = state
        .domain_session
        .as_ref()
        .and_then(|domain| domain.entries.last())
        .filter(|entry| entry.result.is_none())
        .map(|entry| format!("{}:{}", *instance_scope, entry.instance.id));
    {
        let shared_session = shared_session.clone();
        let published_ids = published_ids.clone();
        use_effect_with(pending_id, move |id| {
            let id = id.clone();
            move || {
                if let Some(id) = id {
                    if !published_ids.borrow().contains(&id) {
                        shared_session.abandon(id);
                    }
                }
            }
        });
    }
    {
        let dispatch = state.dispatcher();
        let failed = failed.clone();
        use_effect_with((key.clone(), prepared.clone()), move |(key, prepared)| {
            if let (Some(key), Some(prepared)) = (key, prepared) {
                dispatch.dispatch(SessionAction::Ready(
                    key.clone(),
                    prepared.round.content.clone(),
                    now_ms(),
                ));
                failed.set(false);
            }
        });
    }
    let ready = key.is_some() && state.key == key && state.expected.is_none();
    {
        let wrong = if ready {
            match &state.active {
                ActiveRound::Matching(s) => s.wrong.clone().map(|wrong| (s.epoch, wrong.serial)),
                _ => None,
            }
        } else {
            None
        };
        let dispatch = state.dispatcher();
        use_effect_with((key.clone(), wrong), move |(key, wrong)| {
            let timer = key.as_ref().zip(*wrong).map(|(key, (epoch, serial))| {
                let key = key.clone();
                Timeout::new(400, move || {
                    dispatch.dispatch(SessionAction::ClearWrong(key, epoch, serial))
                })
            });
            move || drop(timer)
        });
    }
    let mcq_select = use_callback(
        (key.clone(), state.dispatcher(), ready),
        move |index, (key, dispatch, ready)| {
            if *ready && let Some(key) = key {
                dispatch.dispatch(SessionAction::McqSelect(key.clone(), index));
            }
        },
    );
    let match_select = use_callback(
        (key.clone(), state.dispatcher(), state.generation, ready),
        move |choice, (key, dispatch, epoch, ready)| {
            if *ready && let Some(key) = key {
                dispatch.dispatch(SessionAction::MatchSelect(key.clone(), *epoch, choice));
            }
        },
    );
    let escape = use_callback(
        (key.clone(), state.dispatcher(), state.generation, ready),
        move |(), (key, dispatch, epoch, ready)| {
            if *ready && let Some(key) = key {
                dispatch.dispatch(SessionAction::Escape(key.clone(), *epoch));
            }
        },
    );
    let advance = {
        let state = state.clone();
        let dispatch = state.dispatcher();
        let navigator = navigator.clone();
        let prepared = prepared.clone();
        let key = key.clone();
        let failed = failed.clone();
        let reload = reload.clone();
        let shared_session = shared_session.clone();
        let instance_scope = instance_scope.clone();
        let published_ids = published_ids.clone();
        Callback::from(move |()| {
            if !ready || !state.active.complete() || state.active.stats().finished_at.is_some() {
                return;
            }
            if let (Some(key), Some(prepared), Some(navigator)) = (&key, &prepared, &navigator) {
                if state.active.last() {
                    if !publish_round(&shared_session, &state, &instance_scope) {
                        return;
                    }
                    if let Some(entry) = state.domain_session.as_ref().and_then(|session| session.entries.last()) {
                        published_ids.borrow_mut().insert(format!("{}:{}", *instance_scope, entry.instance.id));
                    }
                    dispatch.dispatch(SessionAction::Finish(key.clone(), now_ms()));
                    return;
                }
                let target = match random_type(&key.types, Some(variant)) {
                    Ok(target) => target,
                    Err(error) => {
                        log::error!("Could not choose next exercise: {error}");
                        failed.set(true);
                        return;
                    }
                };
                let mut query = prepared.query.clone();
                let offset = advance_offset(query.offset, prepared.next_pool_size, variant);
                query.offset = offset.offset;
                query.seed = SeedDisposition::Unseeded;
                let target_key = SessionKey::new(target, query.clone(), key.types.clone());
                if !publish_round(&shared_session, &state, &instance_scope) {
                    return;
                }
                if let Some(entry) = state.domain_session.as_ref().and_then(|session| session.entries.last()) {
                    published_ids.borrow_mut().insert(format!("{}:{}", *instance_scope, entry.instance.id));
                }
                match navigator
                    .push_with_query(&Route::practice(target), &query_pairs(&query, &key.types))
                {
                    Ok(()) => {
                        let unchanged = key == &target_key;
                        dispatch.dispatch(SessionAction::Advance(
                            key.clone(),
                            target_key,
                            offset.looped,
                        ));
                        failed.set(false);
                        if unchanged {
                            reload.emit(());
                        }
                    }
                    Err(error) => {
                        log::error!("Could not continue practice: {error}");
                        failed.set(true);
                    }
                }
            } else {
                log::error!("Practice Continue requires a prepared round and router");
                failed.set(true);
            }
        })
    };
    let restart = {
        let dispatch = state.dispatcher();
        let complete = ready && state.active.stats().finished_at.is_some();
        let failed = failed.clone();
        let navigator = navigator.clone();
        Callback::from(move |()| {
            if !complete {
                return;
            }
            if let (Some(key), Some(prepared), Some(navigator)) = (&key, &prepared, &navigator) {
                let target = match random_type(&key.types, Some(variant)) {
                    Ok(target) => target,
                    Err(error) => {
                        log::error!("Could not choose restart type: {error}");
                        failed.set(true);
                        return;
                    }
                };
                let mut bytes = [0u8; 8];
                if let Err(error) = getrandom::fill(&mut bytes) {
                    log::error!("Could not choose restart offset: {error}");
                    failed.set(true);
                    return;
                }
                let mut query = prepared.query.clone();
                query.offset = fresh_offset(
                    query.offset,
                    prepared.next_pool_size,
                    u64::from_le_bytes(bytes),
                );
                query.seed = SeedDisposition::Unseeded;
                let target_key = SessionKey::new(target, query.clone(), key.types.clone());
                match navigator
                    .push_with_query(&Route::practice(target), &query_pairs(&query, &key.types))
                {
                    Ok(()) => {
                        let unchanged = key == &target_key;
                        dispatch.dispatch(SessionAction::Restart(key.clone(), target_key));
                        failed.set(false);
                        if unchanged {
                            reload.emit(());
                        }
                    }
                    Err(error) => {
                        log::error!("Could not restart practice: {error}");
                        failed.set(true);
                    }
                }
            }
        })
    };
    let back_to_setup = Callback::from(move |()| {
        if let Some(navigator) = &navigator {
            navigator.push(&Route::PracticeSetup);
        }
    });
    PracticeControls {
        state: (*state).clone().into(),
        ready,
        mcq_select,
        match_select,
        escape,
        advance,
        restart,
        back_to_setup,
        navigation_failed: *failed,
    }
}
fn now_ms() -> u64 {
    js_sys::Date::now() as u64
}

fn publish_round(shared: &QuizSessionContext, state: &SessionState, scope: &str) -> bool {
    if !shared.active() {
        log::warn!("Finish or restart the active shared session before continuing practice");
        return false;
    }
    let Some(session) = &state.domain_session else { return false; };
    let Some(pending) = session.entries.last() else { return false; };
    if pending.result.is_some() { return false; }
    let stats = state.active.stats();
    let previous = session.totals();
    let possible = if matches!(&state.active, ActiveRound::Mcq(_)) { 1 } else { 4 };
    let result = ExerciseResult {
        earned: stats.correct.saturating_sub(previous.earned).min(possible),
        possible,
        mistakes: stats.mistakes.saturating_sub(previous.mistakes),
    };
    let mut instance = pending.instance.clone();
    instance.id = format!("{scope}:{}", instance.id);
    shared.submit(instance, result);
    true
}
