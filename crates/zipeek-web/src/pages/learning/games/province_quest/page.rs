use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use zipeek_exercises::province_quest::{
    Action, AnswerMode, Difficulty, GeoDataset, LearningSupport, Phase, ROUND_LIMIT, Session,
    build_deck,
};

use super::{
    choices::HanziChoices, data::load_map, map::RegionMap, prompt::ProvincePrompt,
    review::ProvinceReview, setup::ProvinceSetup,
};

#[derive(Clone, Debug, PartialEq)]
struct ProvinceController(Session);

impl Reducible for ProvinceController {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut game = self.0.clone();
        game.apply(action);
        Rc::new(Self(game))
    }
}

#[function_component(ProvinceQuestPage)]
pub fn province_quest_page() -> Html {
    let difficulty = use_state(|| Difficulty::Easy);
    let support = use_state(|| LearningSupport::Guided);
    let game = use_reducer(|| {
        ProvinceController(Session::new(
            Difficulty::Easy,
            LearningSupport::Guided,
            Vec::new(),
        ))
    });
    let map = use_state(|| None::<Rc<GeoDataset>>);
    let playing = use_state(|| false);
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);
    let active_province = use_state(|| None::<String>);
    let on_difficulty = {
        let difficulty = difficulty.clone();
        Callback::from(move |new_difficulty| difficulty.set(new_difficulty))
    };
    let on_support = {
        let support = support.clone();
        Callback::from(move |new_support| support.set(new_support))
    };
    let on_start = {
        let map = map.clone();
        let playing = playing.clone();
        let loading = loading.clone();
        let error = error.clone();
        let difficulty = difficulty.clone();
        let support = support.clone();
        let game = game.clone();
        let active_province = active_province.clone();
        Callback::from(move |_: MouseEvent| {
            if *loading {
                return;
            }
            let map = map.clone();
            let playing = playing.clone();
            let loading = loading.clone();
            let error = error.clone();
            let game = game.clone();
            let active_province = active_province.clone();
            let difficulty = *difficulty;
            let support = *support;
            loading.set(true);
            error.set(None);
            spawn_local(async move {
                let result = load_map(difficulty == Difficulty::Hard).await;
                match result {
                    Ok(loaded) => {
                        let mut bytes = [0u8; 8];
                        if let Err(failure) = getrandom::fill(&mut bytes) {
                            error.set(Some(format!("Random seed unavailable: {failure}")));
                        } else {
                            match build_deck(
                                &loaded,
                                difficulty,
                                u64::from_le_bytes(bytes),
                                ROUND_LIMIT,
                            ) {
                                Ok(questions) => {
                                    let fresh = Session::new(difficulty, support, questions);
                                    map.set(Some(Rc::new(loaded)));
                                    active_province.set(None);
                                    playing.set(true);
                                    error.set(None);
                                    game.dispatch(Action::NewGame(fresh));
                                }
                                Err(failure) => {
                                    error.set(Some(format!("No usable map questions: {failure:?}")))
                                }
                            }
                        }
                    }
                    Err(failure) => error.set(Some(failure)),
                }
                loading.set(false);
            });
        })
    };
    let on_setup = {
        let playing = playing.clone();
        Callback::from(move |_: MouseEvent| playing.set(false))
    };
    let on_answer = {
        let game = game.clone();
        Callback::from(move |id| game.dispatch(Action::SelectMap(id)))
    };
    let on_hanzi = {
        let game = game.clone();
        Callback::from(move |id| game.dispatch(Action::SelectHanzi(id)))
    };
    let on_reveal = {
        let game = game.clone();
        Callback::from(move |_: MouseEvent| game.dispatch(Action::RevealHint))
    };
    let on_continue = {
        let game = game.clone();
        let active_province = active_province.clone();
        Callback::from(move |_: MouseEvent| {
            active_province.set(None);
            game.dispatch(Action::Continue);
        })
    };
    let on_parent = {
        let active_province = active_province.clone();
        Callback::from(move |event: Event| {
            let input: web_sys::HtmlSelectElement = event.target_unchecked_into();
            active_province.set(Some(input.value()));
        })
    };
    let question = game.0.current();
    let current_region = map.as_ref().and_then(|dataset| {
        question.and_then(|question| {
            dataset
                .regions
                .iter()
                .find(|region| region.id == question.region_id)
        })
    });
    let is_review = matches!(&game.0.phase, Phase::Reviewing { .. });
    let answer_mode = question.map(|question| question.answer_mode);
    let choice_round = answer_mode == Some(AnswerMode::SelectHanzi);
    let selected_id = if let Phase::Reviewing { selected_id, .. } = &game.0.phase {
        Some(selected_id.clone())
    } else {
        None
    };
    html! {
        <main class="mx-auto w-full max-w-5xl px-3 py-2 md:max-h-[calc(100dvh-100px)] md:overflow-y-auto" data-province-quest="">
            if !*playing {
                <ProvinceSetup
                    difficulty={*difficulty}
                    support={*support}
                    on_difficulty={on_difficulty}
                    on_support={on_support}
                    on_start={on_start}
                    error={(*error).clone()}
                    loading={*loading}
                />
                if *loading {<p role="status" class="mt-3 text-muted">{"Loading geographic data…"}</p>}
            } else if matches!(&game.0.phase, Phase::Complete) {
                <section class="space-y-4 rounded-2xl border border-line bg-surface p-6">
                    <h1 class="text-2xl font-bold">{"China Map Challenge complete"}</h1>
                    <p>{format!("{} locations · {} mistakes · {} assisted answers",game.0.questions.len(),game.0.mistakes,game.0.assisted_answers)}</p>
                    <button class="rounded-xl bg-brand-action px-5 py-3 text-white" onclick={on_setup.clone()}> {"Back to setup"} </button>
                </section>
            } else if let (Some(dataset),Some(region),Some(question)) = (map.as_ref(),current_region,question) {
                <section class="space-y-2 rounded-2xl border border-line bg-surface p-3 sm:p-4">
                    <header class="flex items-center justify-between gap-3">
                        <div><h1 class="text-xl font-bold">{"China Map Challenge"}</h1><p class="text-sm text-muted">{format!("Round {} of {} · {} mistakes", game.0.round+1,game.0.questions.len(),game.0.mistakes)}</p></div>
                        <button type="button" class="rounded-xl border border-line px-4 py-2" onclick={on_setup}> {"Settings"} </button>
                    </header>
                    <ProvincePrompt region={region.clone()} kind={question.kind} answer_mode={question.answer_mode} support={game.0.support} revealed={game.0.hint_revealed} reviewing={is_review} on_reveal={on_reveal}/>
                    if game.0.difficulty == Difficulty::Hard && !choice_round && !is_review {
                        <label class="block space-y-2 text-sm"> <span>{"Zoom to province"}</span>
                            <select class="w-full rounded-lg border border-line bg-raised p-3" onchange={on_parent}>
                                <option value="">{"Choose province"}</option>
                                {for dataset.provinces().into_iter().map(|parent| html! { <option value={parent.id.clone()} selected={active_province.as_deref()==Some(parent.id.as_str())}>{&parent.hanzi}</option> })}
                            </select>
                        </label>
                    }
                    if game.0.difficulty != Difficulty::Hard || active_province.is_some() || choice_round {
                    <RegionMap
                        map={dataset.clone()}
                        parent_id={if game.0.difficulty==Difficulty::Hard {
                            if choice_round || is_review { question.parent_id.clone() } else { (*active_province).clone() }
                        } else { None }}
                        selected_id={selected_id.clone()}
                        correct_id={if is_review || choice_round {Some(question.region_id.clone())} else {None}}
                        highlighted={choice_round}
                        reveal_names={is_review}
                        on_select={on_answer}
                        disabled={is_review || choice_round}
                    />
                    } else {
                        <p role="status" class="rounded-xl bg-raised p-4 text-muted">{"Choose a province to zoom into its prefecture-level divisions."}</p>
                    }
                    if choice_round {
                        <HanziChoices
                            map={dataset.clone()}
                            options={question.options.clone()}
                            selected_id={selected_id.clone()}
                            correct_id={if is_review { Some(question.region_id.clone()) } else { None }}
                            disabled={is_review}
                            support={game.0.support}
                            revealed={game.0.hint_revealed}
                            on_select={on_hanzi}
                        />
                    }
                    if let Phase::Reviewing {correct,..} = &game.0.phase {
                        <ProvinceReview correct={*correct} region={region.clone()} on_continue={on_continue}/>
                    }
                </section>
            }
        </main>
    }
}
