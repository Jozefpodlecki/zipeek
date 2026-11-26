use std::rc::Rc;

use yew::prelude::*;
use zipeek_exercises::province_quest::GeoDataset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RegionVisualState {
    Default,
    Highlighted,
    Correct,
    Incorrect,
}

impl RegionVisualState {
    pub(super) fn for_region(
        is_correct: bool,
        is_selected: bool,
        highlighted: bool,
        reveal_names: bool,
    ) -> Self {
        if is_correct && highlighted && !reveal_names {
            Self::Highlighted
        } else if is_correct {
            Self::Correct
        } else if is_selected {
            Self::Incorrect
        } else {
            Self::Default
        }
    }

    fn fill(self) -> &'static str {
        match self {
            Self::Default => "var(--province-map-fill)",
            Self::Highlighted => "var(--province-map-hint)",
            Self::Correct => "var(--province-map-correct)",
            Self::Incorrect => "var(--province-map-wrong)",
        }
    }
}

#[derive(Properties)]
pub(super) struct MapRegionProps {
    pub dataset: Rc<GeoDataset>,
    pub index: usize,
    pub display_index: usize,
    pub state: RegionVisualState,
    pub reveal_names: bool,
    pub disabled: bool,
    pub on_click_select: Callback<String>,
    pub on_keyboard_select: Callback<String>,
}

impl PartialEq for MapRegionProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.dataset, &other.dataset)
            && self.index == other.index
            && self.display_index == other.display_index
            && self.state == other.state
            && self.reveal_names == other.reveal_names
            && self.disabled == other.disabled
            && self.on_click_select == other.on_click_select
            && self.on_keyboard_select == other.on_keyboard_select
    }
}

#[function_component(MapRegion)]
pub(super) fn map_region(props: &MapRegionProps) -> Html {
    let region = &props.dataset.regions[props.index];
    let name = if props.reveal_names {
        region.hanzi.clone()
    } else if props.state == RegionVisualState::Highlighted {
        format!("Highlighted map region {}", props.display_index)
    } else {
        format!("Map region {}", props.display_index)
    };
    let onclick = {
        let on_select = props.on_click_select.clone();
        let id = region.id.clone();
        let disabled = props.disabled;
        Callback::from(move |_: MouseEvent| {
            if !disabled {
                on_select.emit(id.clone());
            }
        })
    };
    let onkeydown = {
        let on_select = props.on_keyboard_select.clone();
        let id = region.id.clone();
        let disabled = props.disabled;
        Callback::from(move |event: KeyboardEvent| {
            if !disabled && (event.key() == "Enter" || event.key() == " ") {
                event.prevent_default();
                on_select.emit(id.clone());
            }
        })
    };
    let interaction_class = if props.disabled {
        "cursor-grab"
    } else {
        "cursor-pointer hover:brightness-125 focus:stroke-[var(--province-map-focus)] focus:stroke-[3px]"
    };
    let class = format!("transition-colors focus:outline-none {interaction_class}");
    html! {
        <path key={region.id.clone()} d={region.path.clone()}
            role={if props.disabled { "img" } else { "button" }}
            tabindex={if props.disabled { "-1" } else { "0" }}
            aria-label={name.clone()} aria-disabled={props.disabled.to_string()}
            data-region={region.id.clone()} fill={props.state.fill()} fill-rule="evenodd"
            stroke="var(--province-map-stroke)" stroke-width="1.1"
            class={class} onclick={onclick} onkeydown={onkeydown}>
            if props.reveal_names { <title>{name}</title> }
        </path>
    }
}

#[cfg(test)]
mod tests {
    use super::RegionVisualState;

    #[test]
    fn visual_state_preserves_priority_and_hint_visibility() {
        assert_eq!(
            RegionVisualState::for_region(false, false, false, false),
            RegionVisualState::Default
        );
        assert_eq!(
            RegionVisualState::for_region(false, true, true, false),
            RegionVisualState::Incorrect
        );
        assert_eq!(
            RegionVisualState::for_region(true, true, true, false),
            RegionVisualState::Highlighted
        );
        assert_eq!(
            RegionVisualState::for_region(true, false, true, true),
            RegionVisualState::Correct
        );
        assert_eq!(
            RegionVisualState::for_region(true, false, false, false),
            RegionVisualState::Correct
        );
    }

    #[test]
    fn visual_states_use_theme_tokens() {
        assert_eq!(
            RegionVisualState::Default.fill(),
            "var(--province-map-fill)"
        );
        assert_eq!(
            RegionVisualState::Highlighted.fill(),
            "var(--province-map-hint)"
        );
        assert_eq!(
            RegionVisualState::Correct.fill(),
            "var(--province-map-correct)"
        );
        assert_eq!(
            RegionVisualState::Incorrect.fill(),
            "var(--province-map-wrong)"
        );
    }
}
