use std::{collections::HashMap, rc::Rc};

use wasm_bindgen::JsCast;
use web_sys::{Element, WheelEvent};
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::province_quest::{GeoDataset, MapRegion, zoom_view_box};

use super::viewport::{DragGesture, MapPoint, MapViewport, ViewBox, wheel_delta_pixels};

mod region;

use region::{MapRegion as MapRegionPath, RegionVisualState};

#[derive(Properties)]
pub struct RegionMapProps {
    pub map: Rc<GeoDataset>,
    pub parent_id: Option<String>,
    pub parent_picker: bool,
    pub selected_id: Option<String>,
    pub correct_id: Option<String>,
    pub on_select: Callback<String>,
    pub disabled: bool,
    pub highlighted: bool,
    pub reveal_names: bool,
}

impl PartialEq for RegionMapProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.map, &other.map)
            && self.parent_id == other.parent_id
            && self.parent_picker == other.parent_picker
            && self.selected_id == other.selected_id
            && self.correct_id == other.correct_id
            && self.disabled == other.disabled
            && self.highlighted == other.highlighted
            && self.reveal_names == other.reveal_names
            && self.on_select == other.on_select
    }
}

pub(super) fn eligible_parent_regions(map: &GeoDataset) -> Vec<&MapRegion> {
    map.provinces()
        .into_iter()
        .filter(|province| !map.children_of(&province.id).is_empty())
        .collect()
}

fn selectable_regions<'a>(map: &'a GeoDataset, parent: Option<&str>, parent_picker: bool) -> Vec<&'a MapRegion> {
    match parent {
        Some(parent) => map.children_of(parent),
        None if parent_picker => eligible_parent_regions(map),
        None => map.provinces(),
    }
}

#[function_component(RegionMap)]
pub fn region_map(props: &RegionMapProps) -> Html {
    let regions = selectable_regions(&props.map, props.parent_id.as_deref(), props.parent_picker);
    let base_view_box = props
        .parent_id
        .as_deref()
        .and_then(|id| props.map.regions.iter().find(|region| region.id == id))
        .and_then(zoom_view_box)
        .unwrap_or_else(|| props.map.view_box.clone());
    let base = ViewBox::parse(&base_view_box).unwrap_or(ViewBox { x: 0.0, y: 0.0, width: 1000.0, height: 600.0 });
    let viewport = use_state(move || MapViewport::new(base));
    let dragging = use_mut_ref(|| None::<DragGesture>);
    let suppress_click = use_mut_ref(|| false);
    {
        let viewport = viewport.clone();
        let dragging = dragging.clone();
        let suppress_click = suppress_click.clone();
        let identity = (Rc::as_ptr(&props.map) as usize, props.parent_id.clone(), base_view_box.clone());
        use_effect_with(identity, move |_| {
            viewport.set(MapViewport::new(base));
            *dragging.borrow_mut() = None;
            *suppress_click.borrow_mut() = false;
        });
    }
    let view_box = viewport.view_box();
    let on_zoom_in = {
        let viewport = viewport.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = *viewport;
            next.zoom_at(1.25, MapPoint { x: 0.0, y: 0.0 });
            viewport.set(next);
        })
    };
    let on_zoom_out = {
        let viewport = viewport.clone();
        Callback::from(move |_: MouseEvent| {
            let mut next = *viewport;
            next.zoom_at(1.0 / 1.25, MapPoint { x: 0.0, y: 0.0 });
            viewport.set(next);
        })
    };
    let on_reset = {
        let viewport = viewport.clone();
        Callback::from(move |_: MouseEvent| viewport.set(MapViewport::new(base)))
    };
    let on_pointer_down = {
        let dragging = dragging.clone();
        let suppress_click = suppress_click.clone();
        Callback::from(move |event: PointerEvent| {
            if event.button() != 0 || dragging.borrow().is_some() { return; }
            let point = MapPoint {
                x: f64::from(event.client_x()),
                y: f64::from(event.client_y()),
            };
            let gesture = DragGesture::new(event.pointer_id(), point);
            *dragging.borrow_mut() = Some(gesture);
            *suppress_click.borrow_mut() = false;
        })
    };
    let on_pointer_move = {
        let dragging = dragging.clone();
        let suppress_click = suppress_click.clone();
        let viewport = viewport.clone();
        Callback::from(move |event: PointerEvent| {
            let mut active = dragging.borrow_mut();
            let Some(gesture) = active.as_mut() else { return; };
            if gesture.pointer_id != event.pointer_id() { return; }
            let was_dragging = gesture.moved;
            let point = MapPoint {
                x: f64::from(event.client_x()),
                y: f64::from(event.client_y()),
            };
            let delta = gesture.advance(point);
            if !gesture.moved { return; }
            *suppress_click.borrow_mut() = true;
            let Some(target) = event.current_target() else { return; };
            let element: Element = target.unchecked_into();
            if !was_dragging {
                let _ = element.set_pointer_capture(event.pointer_id());
            }
            let rect = element.get_bounding_client_rect();
            let mut next = *viewport;
            next.pan_by(delta, rect.width(), rect.height());
            viewport.set(next);
        })
    };
    let on_pointer_end = {
        let dragging = dragging.clone();
        Callback::from(move |event: PointerEvent| {
            if dragging.borrow().as_ref().is_none_or(|active| active.pointer_id != event.pointer_id()) {
                return;
            }
            if let Some(target) = event.current_target() {
                let element: Element = target.unchecked_into();
                let _ = element.release_pointer_capture(event.pointer_id());
            }
            *dragging.borrow_mut() = None;
        })
    };
    let on_wheel = {
        let viewport = viewport.clone();
        Callback::from(move |event: WheelEvent| {
            let Some(target) = event.current_target() else { return; };
            let element: Element = target.unchecked_into();
            let rect = element.get_bounding_client_rect();
            if rect.width() <= 0.0 || rect.height() <= 0.0 { return; }
            event.prevent_default();
            let Some(delta) = wheel_delta_pixels(event.delta_y(), event.delta_mode(), rect.height()) else { return; };
            let mut next = *viewport;
            let point = MapPoint {
                x: f64::from(event.client_x()) - rect.left(),
                y: f64::from(event.client_y()) - rect.top(),
            };
            let Some(cursor) = next.cursor_in_viewbox(point, rect.width(), rect.height()) else {
                return;
            };
            next.zoom_at((-delta * 0.0012).exp(), cursor);
            viewport.set(next);
        })
    };
    let on_click_select = {
        let on_select = props.on_select.clone();
        let suppress_click = suppress_click.clone();
        Callback::from(move |id: String| {
            let was_suppressed = std::mem::take(&mut *suppress_click.borrow_mut());
            if !was_suppressed {
                on_select.emit(id);
            }
        })
    };
    let indices_by_id: HashMap<&str, usize> = props.map.regions.iter()
        .enumerate()
        .map(|(index, region)| (region.id.as_str(), index))
        .collect();
    let paths = regions.into_iter().enumerate().filter_map(|(display_index, region)| {
        let index = *indices_by_id.get(region.id.as_str())?;
        let state = RegionVisualState::for_region(
            props.correct_id.as_ref() == Some(&region.id),
            props.selected_id.as_ref() == Some(&region.id),
            props.highlighted,
            props.reveal_names,
        );
        Some(html! {
            <MapRegionPath
                key={region.id.clone()}
                dataset={props.map.clone()}
                index={index}
                display_index={display_index + 1}
                state={state}
                reveal_names={props.reveal_names}
                disabled={props.disabled}
                on_click_select={on_click_select.clone()}
                on_keyboard_select={props.on_select.clone()}
            />
        })
    }).collect::<Html>();
    html! {
        <section class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden rounded-xl border border-line bg-surface p-2" data-province-map="">
            <div class="mb-2 flex flex-wrap items-center justify-between gap-2 px-1">
                <span class="sr-only">{"Use the mouse wheel to zoom and drag to pan the map."}</span>
                <div class="flex gap-1" aria-label="Map navigation controls" data-province-map-controls="">
                    <button type="button" aria-label="Zoom in" title="Zoom in" disabled={viewport.zoom >= 5.0}
                        onclick={on_zoom_in} class="grid size-9 place-items-center rounded-lg border border-line bg-raised text-ink disabled:opacity-40">
                        <Icon data={IconData::LUCIDE_PLUS} width="17px" height="17px" />
                    </button>
                    <button type="button" aria-label="Zoom out" title="Zoom out" disabled={viewport.zoom <= 1.0}
                        onclick={on_zoom_out} class="grid size-9 place-items-center rounded-lg border border-line bg-raised text-ink disabled:opacity-40">
                        <Icon data={IconData::LUCIDE_MINUS} width="17px" height="17px" />
                    </button>
                    <button type="button" aria-label="Reset map view" title="Reset map view" onclick={on_reset}
                        class="grid size-9 place-items-center rounded-lg border border-line bg-raised text-ink">
                        <Icon data={IconData::LUCIDE_MAXIMIZE} width="17px" height="17px" />
                    </button>
                </div>
            </div>
            <svg viewBox={view_box} role="group"
                aria-label={if props.parent_picker { "Select a province to explore its divisions" } else if props.disabled { "Administrative region map" } else { "Select an administrative region on the map" }}
                class="block min-h-[16rem] w-full flex-1 touch-none select-none cursor-grab active:cursor-grabbing sm:min-h-[20rem] lg:min-h-0"
                preserveAspectRatio="xMidYMid meet"
                onwheel={on_wheel}
                onpointerdown={on_pointer_down} onpointermove={on_pointer_move}
                onpointerup={on_pointer_end.clone()} onpointercancel={on_pointer_end}>
                {paths}
            </svg>

        </section>
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_exercises::province_quest::RegionKind;

    fn region(id: &str, parent_id: Option<&str>) -> MapRegion {
        MapRegion {
            id: id.to_owned(),
            parent_id: parent_id.map(str::to_owned),
            hanzi: id.to_owned(),
            pinyin: String::new(),
            english: id.to_owned(),
            capital_hanzi: None,
            capital_pinyin: None,
            kind: RegionKind::Province,
            path: "M 0 0 L 5 0 L 5 5 Z".to_owned(),
            bbox: [0.0, 0.0, 5.0, 5.0],
        }
    }

    #[test]
    fn parent_picker_excludes_regions_without_prefecture_geometry() {
        let dataset = GeoDataset {
            source: String::new(),
            version: String::new(),
            attribution: String::new(),
            view_box: "0 0 100 100".to_owned(),
            regions: vec![region("44", None), region("82", None), region("4401", Some("44"))],
        };
        let available = eligible_parent_regions(&dataset);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, "44");
        assert_eq!(selectable_regions(&dataset, None, true).len(), 1);
        assert_eq!(selectable_regions(&dataset, None, false).len(), 2);
        assert_eq!(selectable_regions(&dataset, Some("44"), false).len(), 1);
    }
}
