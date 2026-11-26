use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use yew_router::prelude::Link;

use crate::route::Route;
use zipeek_exercises::province_quest::{
    Action, AnswerMode, Difficulty, LearningSupport, Phase, ROUND_LIMIT, Session, build_deck,
};

use super::{
    choices::HanziChoices,
    data::load_map,
    map::{RegionMap, eligible_parent_regions},
    prompt::ProvincePrompt,
    review::ProvinceReview,
    setup::ProvinceSetup,
    state::{GameError, GamePhase, ProvinceQuestAction, ProvinceQuestState},
};

#[function_component(ProvinceQuestPage)]
pub fn province_quest_page() -> Html {
    let state = use_reducer(ProvinceQuestState::default);
    let on_difficulty = {
        let state = state.clone();
        Callback::from(move |value| state.dispatch(ProvinceQuestAction::SetDifficulty(value)))
    };
    let on_support = {
        let state = state.clone();
        Callback::from(move |value| state.dispatch(ProvinceQuestAction::SetSupport(value)))
    };
    let on_start = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            if matches!(&state.phase, GamePhase::Loading { .. }) {
                return;
            }
            let request = state.request_generation().wrapping_add(1);
            let difficulty = state.difficulty;
            let support = state.support;
            state.dispatch(ProvinceQuestAction::Start);
            let state = state.clone();
            spawn_local(async move {
                let result = async {
                    let loaded = load_map(difficulty == Difficulty::Hard)
                        .await
                        .map_err(GameError::Map)?;
                    let mut seed = [0u8; 8];
                    getrandom::fill(&mut seed).map_err(|failure| {
                        GameError::Seed(format!("Random seed unavailable: {failure}"))
                    })?;
                    let questions =
                        build_deck(&loaded, difficulty, u64::from_le_bytes(seed), ROUND_LIMIT)
                            .map_err(|failure| {
                                GameError::Deck(format!("No usable map questions: {failure:?}"))
                            })?;
                    Ok::<_, GameError>((
                        Rc::new(loaded),
                        Session::new(difficulty, support, questions),
                    ))
                }
                .await;
                match result {
                    Ok((dataset, session)) => state.dispatch(ProvinceQuestAction::Loaded {
                        request,
                        dataset,
                        session,
                    }),
                    Err(error) => {
                        state.dispatch(ProvinceQuestAction::LoadFailed { request, error })
                    }
                }
            });
        })
    };
    let on_setup = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| state.dispatch(ProvinceQuestAction::BackToSetup))
    };
    let on_answer = {
        let state = state.clone();
        Callback::from(move |id| {
            state.dispatch(ProvinceQuestAction::Session(Action::SelectMap(id)))
        })
    };
    let on_hanzi = {
        let state = state.clone();
        Callback::from(move |id| {
            state.dispatch(ProvinceQuestAction::Session(Action::SelectHanzi(id)))
        })
    };
    let on_reveal = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch(ProvinceQuestAction::Session(Action::RevealHint))
        })
    };
    let on_continue = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.dispatch(ProvinceQuestAction::Session(Action::Continue))
        })
    };
    let on_pick_parent = {
        let state = state.clone();
        Callback::from(move |id: String| {
            state.dispatch(ProvinceQuestAction::SetActiveProvince(Some(id)));
        })
    };
    let on_parent = {
        let state = state.clone();
        Callback::from(move |event: Event| {
            let input: web_sys::HtmlSelectElement = event.target_unchecked_into();
            let value = input.value();
            state.dispatch(ProvinceQuestAction::SetActiveProvince(
                (!value.is_empty()).then_some(value),
            ));
        })
    };
    let empty_session = Session::new(state.difficulty, state.support, Vec::new());
    let (map, game) = match &state.phase {
        GamePhase::Playing { dataset, session } => (Some(dataset), session),
        _ => (None, &empty_session),
    };
    let playing = matches!(&state.phase, GamePhase::Playing { .. });
    let loading = matches!(&state.phase, GamePhase::Loading { .. });
    let error = match &state.phase {
        GamePhase::Failed(error) => Some(error.to_string()),
        _ => None,
    };
    let question = game.current();
    let current_region = map.and_then(|dataset| {
        question.and_then(|question| {
            dataset
                .regions
                .iter()
                .find(|region| region.id == question.region_id)
        })
    });
    let is_review = matches!(&game.phase, Phase::Reviewing { .. });
    let answer_mode = question.map(|question| question.answer_mode);
    let choice_round = answer_mode == Some(AnswerMode::SelectHanzi);
    let selected_id = if let Phase::Reviewing { selected_id, .. } = &game.phase {
        Some(selected_id.clone())
    } else {
        None
    };
    html! {
        <main class="mx-auto flex h-full min-h-0 w-full max-w-7xl flex-col gap-2 overflow-y-auto lg:overflow-hidden" data-province-quest="">
            <nav class="flex shrink-0 justify-end" aria-label="Game navigation">
                <Link<Route> to={Route::Games} classes="inline-flex min-h-10 items-center gap-2 rounded-lg border border-line bg-surface px-3 text-sm font-semibold text-ink focus-visible:outline-2 focus-visible:outline-accent" >
                    <Icon data={IconData::LUCIDE_ARROW_LEFT} width="18px" height="18px" />
                    {"Exit"}
                </Link<Route>>
            </nav>
            if !playing {
                <ProvinceSetup
                    difficulty={state.difficulty}
                    support={state.support}
                    on_difficulty={on_difficulty}
                    on_support={on_support}
                    on_start={on_start}
                    error={error}
                    loading={loading}
                />
                if loading {<p role="status" class="mt-3 text-muted">{"Loading geographic data…"}</p>}
            } else if matches!(&game.phase, Phase::Complete) {
                <section class="space-y-4 rounded-2xl border border-line bg-surface p-6">
                    <h1 class="text-2xl font-bold">{"China Map Challenge complete"}</h1>
                    <p>{format!("{} locations · {} mistakes · {} assisted answers",game.questions.len(),game.mistakes,game.assisted_answers)}</p>
                    <button class="rounded-xl bg-brand-action px-5 py-3 text-white" onclick={on_setup.clone()}> {"Back to setup"} </button>
                </section>
            } else if let (Some(dataset),Some(region),Some(question)) = (map,current_region,question) {
                <section class="flex min-h-0 flex-1 flex-col gap-2 rounded-2xl border border-line bg-surface p-2 sm:p-3" data-province-gameplay="">
                    <header class="flex shrink-0 items-center justify-between gap-3">
                        <div><h1 class="text-base font-bold sm:text-lg">{"China Map Challenge"}</h1><p class="text-xs text-muted">{format!("{} / {} · {} mistakes", game.round+1,game.questions.len(),game.mistakes)}</p></div>
                        <div class="flex items-center gap-2">
                            <button type="button" aria-label="Game settings" title="Game settings" class="grid size-10 place-items-center rounded-lg border border-line text-ink focus-visible:outline-2 focus-visible:outline-accent" onclick={on_setup}>
                                <Icon data={IconData::LUCIDE_SETTINGS} width="19px" height="19px" />
                            </button>
                        </div>
                    </header>
                    <ProvincePrompt region={region.clone()} kind={question.kind} answer_mode={question.answer_mode} support={game.support} revealed={game.hint_revealed} reviewing={is_review} on_reveal={on_reveal}/>
                    if game.difficulty == Difficulty::Hard && !choice_round && !is_review {
                        <label class="block space-y-2 text-sm"> <span>{"Zoom to province"}</span>
                            <select class="w-full rounded-lg border border-line bg-raised p-3" onchange={on_parent}>
                                <option value="">{"Choose province"}</option>
                                {for eligible_parent_regions(dataset).into_iter().map(|parent| html! { <option value={parent.id.clone()} selected={state.active_province.as_deref()==Some(parent.id.as_str())}>{&parent.hanzi}</option> })}
                            </select>
                        </label>
                    }
                    <RegionMap
                        map={Rc::clone(dataset)}
                        parent_id={if game.difficulty==Difficulty::Hard {
                            if choice_round || is_review { question.parent_id.clone() } else { state.active_province.clone() }
                        } else { None }}
                        parent_picker={game.difficulty == Difficulty::Hard && !choice_round && !is_review && state.active_province.is_none()}
                        selected_id={selected_id.clone()}
                        correct_id={if is_review || choice_round {Some(question.region_id.clone())} else {None}}
                        highlighted={choice_round}
                        reveal_names={is_review}
                        on_select={if game.difficulty == Difficulty::Hard && !choice_round && !is_review && state.active_province.is_none() { on_pick_parent } else { on_answer }}
                        disabled={is_review || choice_round}
                    />
                    if choice_round {
                        <HanziChoices
                            map={Rc::clone(dataset)}
                            options={question.options.clone()}
                            selected_id={selected_id.clone()}
                            correct_id={if is_review { Some(question.region_id.clone()) } else { None }}
                            disabled={is_review}
                            support={game.support}
                            revealed={game.hint_revealed}
                            on_select={on_hanzi}
                        />
                    }
                    if let Phase::Reviewing {correct,..} = &game.phase {
                        <ProvinceReview correct={*correct} region={region.clone()} on_continue={on_continue}/>
                    }
                </section>
            }
        </main>
    }
}
