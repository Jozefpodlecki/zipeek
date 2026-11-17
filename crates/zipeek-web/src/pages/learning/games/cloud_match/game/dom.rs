use super::motion::{CloudId, CloudWorld};
use std::cell::Cell;
use wasm_bindgen::JsCast;

pub(super) struct CloudDom {
    arena: web_sys::HtmlElement,
    tiles: Vec<(CloudId, web_sys::HtmlElement, Cell<i32>)>,
    arena_width: Cell<i32>,
    active_scene: Cell<Option<u8>>,
}

impl CloudDom {
    pub fn bind(arena: web_sys::HtmlElement, world: &CloudWorld) -> Self {
        let mut tiles = Vec::new();
        for id in world.ids() {
            let side = if id.side == 0 { "left" } else { "right" };
            let selector = format!(
                "[data-cloud-side='{side}'][data-cloud-pair-id='{}']:not([data-cloud-ghost])",
                id.pair,
            );
            if let Ok(Some(node)) = arena.query_selector(&selector) {
                if let Ok(tile) = node.dyn_into::<web_sys::HtmlElement>() {
                    let width = tile.offset_width();
                    tiles.push((id, tile, Cell::new(width)));
                }
            }
        }
        let arena_width = Cell::new(arena.client_width());
        Self {
            arena,
            tiles,
            arena_width,
            active_scene: Cell::new(None),
        }
    }

    pub fn paint(&self, world: &CloudWorld) {
        let scene = ((chrono::Utc::now().timestamp_millis() / 60_000).rem_euclid(5)) as u8;
        if self.active_scene.get() != Some(scene) {
            self.active_scene.set(Some(scene));
            for index in 0..5 {
                let selector = format!("[data-cloud-scene='{index}']");
                if let Ok(Some(element)) = self.arena.query_selector(&selector) {
                    let active = if index == scene { "true" } else { "false" };
                    let _ = element.set_attribute("data-sky-active", active);
                }
            }
        }
        let current_width = self.arena.client_width();
        if current_width <= 0 {
            return;
        }
        if self.arena_width.get() != current_width {
            self.arena_width.set(current_width);
            for (_, tile, measured_width) in &self.tiles {
                measured_width.set(tile.offset_width());
            }
        }
        let width = f64::from(current_width);
        for (id, tile, measured_width) in &self.tiles {
            if measured_width.get() <= 0 {
                continue;
            }
            let Some(motion) = world.motion(*id) else {
                continue;
            };
            let Some(x) = world.position(*id, width, f64::from(measured_width.get())) else {
                continue;
            };
            let visibility = if tile.get_attribute("data-cloud-matched").as_deref() == Some("true")
            {
                "hidden"
            } else {
                "visible"
            };
            let position = format!(
                "top:{}%;transform:translate3d({x:.2}px,0,0);visibility:{visibility}",
                motion.top
            );
            if let Err(error) = tile.set_attribute("style", &position) {
                log::warn!("Could not position Cloud Match tile: {error:?}");
            }
            if let Some(progress) = world.progress(*id) {
                let progress_value = format!("{progress:.6}");
                let _ = tile.set_attribute("data-cloud-motion-progress", &progress_value);
                let _ = tile.set_attribute("data-cloud-velocity", &format!("{:.2}", motion.speed));
            }
        }
    }
}
