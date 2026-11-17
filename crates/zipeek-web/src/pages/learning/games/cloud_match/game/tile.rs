use super::{label, motion::CloudMotion, shape};
use yew::prelude::*;
use zipeek_exercises::cloud_match::{CloudGame, CloudPair, CloudSide, Difficulty};

#[derive(Properties, PartialEq)]
pub(super) struct CloudTileProps {
    pub id: i32,
    pub wave: usize,
    pub side: CloudSide,
    pub lane: usize,
    pub difficulty: Difficulty,
    pub label: String,
    pub selected: bool,
    pub hint_glow: bool,
    pub incorrect: bool,
    pub primary_label: bool,
    pub disabled: bool,
    pub matched: bool,
    pub on_choose: Callback<(CloudSide, i32)>,
}

#[function_component(CloudTile)]
fn tile(props: &CloudTileProps) -> Html {
    let motion = CloudMotion::new(
        props.id,
        props.side,
        props.lane,
        props.wave,
        props.difficulty,
    );
    let visible_label = label::compact_label(&props.label, props.primary_label);
    let wider = if props.primary_label {
        visible_label.chars().count() >= 5
    } else {
        visible_label.chars().count() >= 12
    };
    let (width_class, height_class) = if wider {
        ("w-[clamp(17rem,24vw,23rem)]", "h-[clamp(8rem,11vw,10rem)]")
    } else {
        match motion.variation % 3 {
            0 => (
                "w-[clamp(12.5rem,18vw,17rem)]",
                "h-[clamp(6.5rem,8.5vw,8rem)]",
            ),
            1 => (
                "w-[clamp(13rem,19vw,18rem)]",
                "h-[clamp(6.8rem,9vw,8.5rem)]",
            ),
            _ => (
                "w-[clamp(14rem,20.5vw,19rem)]",
                "h-[clamp(7.1rem,9.5vw,9rem)]",
            ),
        }
    };
    let side = props.side;
    let id = props.id;
    let choose = props.on_choose.clone();
    let on_choose = Callback::from(move |_| choose.emit((side, id)));
    let is_primary = props.primary_label;
    let gradient_id = format!("cloud-ice-{}-{}-{}", props.wave, u8::from(!is_primary), id);
    let accessible = if is_primary {
        "Chinese vocabulary cloud"
    } else {
        "Translation or Pinyin cloud"
    };
    let tile_classes = classes!(
        "group",
        "absolute",
        "left-0",
        "z-[3]",
        "grid",
        "cursor-pointer",
        "place-items-center",
        "border-0",
        "bg-transparent",
        "p-0",
        "text-center",
        "font-semibold",
        "text-[#17354f]",
        "invisible",
        "will-change-transform",
        "focus-visible:rounded-[3rem]",
        "focus-visible:outline-3",
        "focus-visible:outline-offset-[-1px]",
        "focus-visible:outline-sky-300",
        "contrast-more:focus-visible:outline-[#244d70]",
        "max-sm:h-[6.2rem]",
        "max-sm:w-[45%]",
        "motion-reduce:!transform-none",
        "motion-reduce:data-[cloud-side=left]:!left-[3%]",
        "motion-reduce:data-[cloud-side=right]:!left-auto",
        "motion-reduce:data-[cloud-side=right]:!right-[3%]",
        "motion-reduce:will-change-auto",
        "disabled:cursor-default",
        "data-[cloud-matched=true]:!invisible",
        "data-[cloud-matched=true]:pointer-events-none",
        width_class,
        height_class,
    );
    html! {
        <button
            type="button"
            data-cloud-tile=""
            data-cloud-pair-id={props.id.to_string()}
            data-cloud-wide={wider.to_string()}
            data-cloud-side={if props.side == CloudSide::Left { "left" } else { "right" }}
            data-cloud-wave={props.wave.to_string()}
            data-cloud-lane={props.lane.to_string()}
            data-cloud-primary={props.primary_label.to_string()}
            data-cloud-hint={props.hint_glow.to_string()}
            data-cloud-incorrect={props.incorrect.to_string()}
            aria-pressed={props.selected.to_string()}
            disabled={props.disabled || props.matched}
            aria-hidden={props.matched.to_string()}
            data-cloud-matched={props.matched.to_string()}
            class={tile_classes}
            title={props.label.clone()}
            aria-label={format!("{accessible}: {}", props.label)}
            onclick={on_choose}
        >
            {shape::view(motion.variation, &gradient_id)}
            {label::view(&props.label, is_primary)}
        </button>
    }
}

pub(super) fn view(
    round: &CloudGame,
    side: CloudSide,
    index: usize,
    pair: &CloudPair,
    choose: Callback<(CloudSide, i32)>,
    show_match_hint: bool,
    incorrect_pairs: &[(CloudSide, i32)],
) -> Html {
    let label = round.label_on_side(pair, side);
    let hint_glow = round.selected.is_some_and(|(selected_side, selected_id)| {
        selected_side != side && selected_id == pair.id
    });
    html! {
        <CloudTile
            key={format!("{}-{:?}-{}", round.wave, side, pair.id)}
            id={pair.id}
            wave={round.wave}
            side={side}
            lane={index}
            difficulty={round.difficulty}
            label={label.to_owned()}
            selected={round.selected == Some((side, pair.id))}
            hint_glow={show_match_hint && hint_glow}
            incorrect={incorrect_pairs.contains(&(side, pair.id))}
            primary_label={round.label_on_side(pair, side) == pair.labels(round.mode).0}
            disabled={round.paused || round.finished}
            matched={round.matched.contains(&pair.id)}
            on_choose={choose}
        />
    }
}
