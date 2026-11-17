use super::{
    dom::CloudDom,
    exit,
    motion::{CloudId, CloudMotion, CloudWorld},
    tile,
};
use gloo::render::{AnimationFrame, request_animation_frame};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use yew::prelude::*;
use zipeek_exercises::cloud_match::{CloudGame, CloudSide};

#[derive(Properties, PartialEq)]
pub(crate) struct CloudFieldProps {
    pub round: CloudGame,
    pub choose: Callback<(CloudSide, i32)>,
    pub show_match_hint: bool,
    pub incorrect_pairs: Vec<(CloudSide, i32)>,
    pub pause_overlay: Html,
    pub completion_overlay: Html,
}

fn schedule_frame(
    dom: Rc<CloudDom>,
    world: Rc<RefCell<CloudWorld>>,
    paused: Rc<Cell<bool>>,
    live: Rc<Cell<bool>>,
    frame: Rc<RefCell<Option<AnimationFrame>>>,
) {
    let next_frame = frame.clone();
    let handle = request_animation_frame(move |timestamp| {
        next_frame.borrow_mut().take();
        if !live.get() {
            return;
        }
        let mut world_state = world.borrow_mut();
        world_state.tick(timestamp, paused.get());
        dom.paint(&world_state);
        drop(world_state);
        schedule_frame(dom, world, paused, live, next_frame);
    });
    *frame.borrow_mut() = Some(handle);
}

#[function_component(CloudField)]
pub(crate) fn field(props: &CloudFieldProps) -> Html {
    let root = use_node_ref();
    let paused = use_mut_ref(|| Rc::new(Cell::new(false)));
    let retained_world = use_mut_ref(|| None::<(usize, u64, Rc<RefCell<CloudWorld>>)>);
    let pause_state = paused.borrow().clone();
    pause_state.set(props.round.paused || props.round.finished);

    {
        let root = root.clone();
        let paused = pause_state.clone();
        let mut entries = Vec::new();
        for side in [CloudSide::Left, CloudSide::Right] {
            for (lane, pair) in props.round.ordered_wave_all(side) {
                let motion = CloudMotion::new(
                    pair.id,
                    side,
                    lane,
                    props.round.wave,
                    props.round.difficulty,
                );
                entries.push((CloudId::new(side, pair.id), side, motion));
            }
        }
        let roster = entries.iter().map(|(id, _, _)| *id).collect::<Vec<_>>();
        let retained_world = retained_world.clone();
        let wave = props.round.wave;
        let seed = props.round.seed;
        use_effect_with((wave, seed, props.round.difficulty, roster), move |_| {
            let world = {
                let mut retained = retained_world.borrow_mut();
                let existing = retained
                    .as_ref()
                    .and_then(|(stored_wave, stored_seed, world)| {
                        if *stored_wave == wave && *stored_seed == seed {
                            Some(world.clone())
                        } else {
                            None
                        }
                    });
                if let Some(world) = existing {
                    world.borrow_mut().sync(entries);
                    world
                } else {
                    let world = Rc::new(RefCell::new(CloudWorld::new(entries)));
                    *retained = Some((wave, seed, world.clone()));
                    world
                }
            };
            let live = Rc::new(Cell::new(true));
            let frame = Rc::new(RefCell::new(None));
            if let Some(arena) = root.cast::<web_sys::HtmlElement>() {
                let dom = Rc::new(CloudDom::bind(arena, &world.borrow()));
                dom.paint(&world.borrow());
                world.borrow_mut().activate();
                schedule_frame(dom, world, paused, live.clone(), frame.clone());
            }
            move || {
                live.set(false);
                frame.borrow_mut().take();
            }
        });
    }

    let field_choose = {
        let choose = props.choose.clone();
        let root = root.clone();
        let selected = props.round.selected;
        Callback::from(move |chosen: (CloudSide, i32)| {
            if let Some(first) = selected {
                if first.0 != chosen.0 && first.1 == chosen.1 {
                    if let Some(arena) = root.cast::<web_sys::HtmlElement>() {
                        exit::celebrate(&arena, first, chosen);
                    }
                }
            }
            choose.emit(chosen);
        })
    };
    let clouds = [CloudSide::Left, CloudSide::Right]
        .into_iter()
        .flat_map(|side| {
            let choose = field_choose.clone();
            props
                .round
                .ordered_wave_all(side)
                .into_iter()
                .map(move |(index, pair)| {
                    tile::view(
                        &props.round,
                        side,
                        index,
                        pair,
                        choose.clone(),
                        props.show_match_hint,
                        &props.incorrect_pairs,
                    )
                })
        })
        .collect::<Vec<_>>();

    let scenes = ["coast", "sunrise", "meadow", "sunset", "twilight"];
    html! {
        <div
            ref={root}
            data-cloud-motion-controller=""
            class="relative isolate min-h-80 flex-1 overflow-hidden bg-[linear-gradient(180deg,#14253f_0%,#263d5b_38%,#42637c_72%,#567b91_100%)] max-[520px]:h-84 max-[520px]:flex-none"
        >
            <div class="pointer-events-none absolute inset-0 z-0" aria-hidden="true">
                {for scenes.iter().enumerate().map(|(index, scene)| {
                    let background = format!(
                        "background-image:linear-gradient(180deg,rgba(6,22,45,.34),rgba(9,29,55,.22) 52%,rgba(9,29,55,.40)),url('cloud-match-skies/{scene}.webp')"
                    );
                    html! {
                        <div
                            key={index}
                            data-cloud-scene={index.to_string()}
                            data-sky-active="false"
                            class="pointer-events-none absolute inset-0 bg-cover bg-center bg-no-repeat opacity-0 transition-opacity duration-[4000ms] data-[sky-active=true]:opacity-100 motion-reduce:transition-none"
                            style={background}
                        />
                    }
                })}
            </div>
            <div
                class="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_45%_28%_at_50%_58%,rgba(200,229,246,.08),transparent_90%)]"
                aria-hidden="true"
            />
            {for clouds}
            {props.pause_overlay.clone()}
            {props.completion_overlay.clone()}
        </div>
    }
}
