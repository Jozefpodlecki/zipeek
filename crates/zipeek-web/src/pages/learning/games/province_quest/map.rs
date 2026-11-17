use std::rc::Rc;

use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::province_quest::{GeoDataset, MapRegion, zoom_view_box};

#[derive(Properties)]
pub struct RegionMapProps {
    pub map: Rc<GeoDataset>,
    pub parent_id: Option<String>,
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
            && self.selected_id == other.selected_id
            && self.correct_id == other.correct_id
            && self.disabled == other.disabled
            && self.highlighted == other.highlighted
            && self.reveal_names == other.reveal_names
            && self.on_select == other.on_select
    }
}

fn selectable_regions<'a>(map: &'a GeoDataset, parent: Option<&str>) -> Vec<&'a MapRegion> {
    match parent {
        None => map.provinces(),
        Some(parent) => map.children_of(parent),
    }
}

fn parse_view_box(value: &str) -> Option<[f64; 4]> {
    let parts = value
        .split_whitespace()
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if parts.len() != 4
        || !parts.iter().all(|part| part.is_finite())
        || parts[2] <= 0.0
        || parts[3] <= 0.0
    {
        return None;
    }
    Some([parts[0], parts[1], parts[2], parts[3]])
}

fn viewport(base: [f64; 4], zoom: f64, pan: (f64, f64)) -> String {
    let zoom = zoom.clamp(1.0, 5.0);
    let width = base[2] / zoom;
    let height = base[3] / zoom;
    let max_x = (base[2] - width) / 2.0;
    let max_y = (base[3] - height) / 2.0;
    let x = base[0] + (base[2] - width) / 2.0 + pan.0.clamp(-max_x, max_x);
    let y = base[1] + (base[3] - height) / 2.0 + pan.1.clamp(-max_y, max_y);
    format!("{x} {y} {width} {height}")
}

#[function_component(RegionMap)]
pub fn region_map(props: &RegionMapProps) -> Html {
    let regions = selectable_regions(&props.map, props.parent_id.as_deref());
    let base_view_box = props
        .parent_id
        .as_deref()
        .and_then(|id| props.map.regions.iter().find(|region| region.id == id))
        .and_then(zoom_view_box)
        .unwrap_or_else(|| props.map.view_box.clone());
    let base = parse_view_box(&base_view_box).unwrap_or([0.0, 0.0, 1000.0, 600.0]);
    let zoom = use_state(|| 1.0_f64);
    let pan = use_state(|| (0.0_f64, 0.0_f64));
    let dragging = use_mut_ref(|| None::<(i32, f64, f64)>);
    let dragged = use_mut_ref(|| false);
    {
        let zoom = zoom.clone();
        let pan = pan.clone();
        use_effect_with(props.parent_id.clone(), move |_| {
            zoom.set(1.0);
            pan.set((0.0, 0.0));
        });
    }
    let view_box = viewport(base, *zoom, *pan);
    let on_zoom_in = {
        let zoom = zoom.clone();
        Callback::from(move |_: MouseEvent| zoom.set((*zoom * 1.5).min(5.0)))
    };
    let on_zoom_out = {
        let zoom = zoom.clone();
        let pan = pan.clone();
        Callback::from(move |_: MouseEvent| {
            let next = (*zoom / 1.5).max(1.0);
            zoom.set(next);
            if next == 1.0 {
                pan.set((0.0, 0.0));
            }
        })
    };
    let on_reset = {
        let zoom = zoom.clone();
        let pan = pan.clone();
        Callback::from(move |_: MouseEvent| {
            zoom.set(1.0);
            pan.set((0.0, 0.0));
        })
    };
    let on_pointer_down = {
        let dragging = dragging.clone();
        let dragged = dragged.clone();
        Callback::from(move |event: PointerEvent| {
            if event.button() != 0 {
                return;
            }
            *dragging.borrow_mut() = Some((
                event.pointer_id(),
                event.client_x() as f64,
                event.client_y() as f64,
            ));
            *dragged.borrow_mut() = false;
        })
    };
    let on_pointer_move = {
        let dragging = dragging.clone();
        let dragged = dragged.clone();
        let pan = pan.clone();
        let zoom = zoom.clone();
        Callback::from(move |event: PointerEvent| {
            let Some((id, x, y)) = *dragging.borrow() else {
                return;
            };
            if id != event.pointer_id() {
                return;
            }
            let dx = event.client_x() as f64 - x;
            let dy = event.client_y() as f64 - y;
            if dx.abs() + dy.abs() < 3.0 {
                return;
            }
            *dragged.borrow_mut() = true;
            let rendered_width = event
                .current_target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| element.get_bounding_client_rect().width())
                .unwrap_or(600.0)
                .max(1.0);
            let factor = base[2] / *zoom / rendered_width;
            pan.set((pan.0 - dx * factor, pan.1 - dy * factor));
            *dragging.borrow_mut() = Some((id, event.client_x() as f64, event.client_y() as f64));
        })
    };
    let on_pointer_end = {
        let dragging = dragging.clone();
        Callback::from(move |_: PointerEvent| {
            *dragging.borrow_mut() = None;
        })
    };
    let paths = regions.into_iter().enumerate().map(|(index, region)| {
        let id = region.id.clone();
        let is_correct = props.correct_id.as_ref() == Some(&id);
        let name = if props.reveal_names {
            region.hanzi.clone()
        } else if props.highlighted && is_correct {
            format!("Highlighted map region {}", index + 1)
        } else {
            format!("Map region {}", index + 1)
        };
        let is_selected = props.selected_id.as_ref() == Some(&id);
        let selected = props.on_select.clone();
        let dragged_click = dragged.clone();
        let onclick = {
            let id = id.clone();
            let disabled = props.disabled;
            Callback::from(move |_: MouseEvent| {
                if *dragged_click.borrow() { *dragged_click.borrow_mut() = false; return; }
                if !disabled { selected.emit(id.clone()); }
            })
        };
        let selected = props.on_select.clone();
        let onkeydown = {
            let id = id.clone();
            let disabled = props.disabled;
            Callback::from(move |event: KeyboardEvent| {
                if !disabled && (event.key() == "Enter" || event.key() == " ") {
                    event.prevent_default();
                    selected.emit(id.clone());
                }
            })
        };
        let fill = if is_correct {
            if props.highlighted && !props.reveal_names { "var(--province-map-hint)" }
            else { "var(--province-map-correct)" }
        } else if is_selected { "var(--province-map-wrong)" }
        else { "var(--province-map-fill)" };
        let interaction_class = if props.disabled { "cursor-grab" }
            else { "cursor-pointer hover:brightness-125 focus:stroke-[var(--province-map-focus)] focus:stroke-[3px]" };
        html! {
            <path key={id} d={region.path.clone()}
                role={if props.disabled {"img"} else {"button"}}
                tabindex={if props.disabled {"-1"} else {"0"}}
                aria-label={name.clone()} aria-disabled={props.disabled.to_string()}
                data-region={region.id.clone()} fill={fill} fill-rule="evenodd"
                stroke="var(--province-map-stroke)" stroke-width="1.1"
                class={format!("transition-colors focus:outline-none {interaction_class}")}
                onclick={onclick} onkeydown={onkeydown}>
                if props.reveal_names { <title>{name}</title> }
            </path>
        }
    }).collect::<Html>();
    html! {
        <section class="min-w-0 max-w-[min(100%,95dvh)] overflow-hidden rounded-xl border border-line bg-surface p-2" data-province-map="">
            <div class="mb-2 flex flex-wrap items-center justify-between gap-2 px-1">
                <span class="text-xs text-muted">{"Zoom for small regions. Drag the map to pan."}</span>
                <div class="flex gap-1" aria-label="Map navigation controls" data-province-map-controls="">
                    <button type="button" aria-label="Zoom in" title="Zoom in" disabled={*zoom >= 5.0}
                        onclick={on_zoom_in} class="grid size-9 place-items-center rounded-lg border border-line bg-raised text-ink disabled:opacity-40">
                        <Icon data={IconData::LUCIDE_PLUS} width="17px" height="17px" />
                    </button>
                    <button type="button" aria-label="Zoom out" title="Zoom out" disabled={*zoom <= 1.0}
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
                aria-label={if props.disabled { "Administrative region map" } else { "Select an administrative region on the map" }}
                class="block aspect-[1.65] w-full touch-none select-none cursor-grab active:cursor-grabbing"
                onpointerdown={on_pointer_down} onpointermove={on_pointer_move}
                onpointerup={on_pointer_end.clone()} onpointercancel={on_pointer_end}>
                {paths}
            </svg>
            <p class="px-2 pt-2 text-[11px] leading-relaxed text-muted">{format!("Geography: {} · {} · {}", props.map.source, props.map.version, props.map.attribution)}</p>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn map_viewport_clamps_pan_and_zoom() {
        assert_eq!(
            viewport([0.0, 0.0, 100.0, 60.0], 1.0, (900.0, 200.0)),
            "0 0 100 60"
        );
        assert_eq!(
            viewport([0.0, 0.0, 100.0, 60.0], 2.0, (0.0, 0.0)),
            "25 15 50 30"
        );
        assert!(parse_view_box("0 0 100 -20").is_none());
    }
}
