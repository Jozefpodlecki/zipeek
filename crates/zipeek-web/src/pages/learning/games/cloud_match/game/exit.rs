use gloo::{render::request_animation_frame, timers::callback::Timeout};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use zipeek_exercises::cloud_match::CloudSide;

fn side_name(side: CloudSide) -> &'static str {
    match side {
        CloudSide::Left => "left",
        CloudSide::Right => "right",
    }
}

pub(super) fn celebrate(root: &HtmlElement, first: (CloudSide, i32), second: (CloudSide, i32)) {
    let Some(parent) = root.owner_document().and_then(|doc| doc.body()) else {
        return;
    };
    for (side, pair_id) in [first, second] {
        let selector = format!(
            "[data-cloud-side='{}'][data-cloud-pair-id='{pair_id}']:not([data-cloud-ghost])",
            side_name(side),
        );
        let Ok(Some(source)) = root.query_selector(&selector) else {
            continue;
        };
        let rect = source.get_bounding_client_rect();
        let Ok(clone) = source.clone_node_with_deep(true) else {
            continue;
        };
        let Ok(ghost) = clone.dyn_into::<HtmlElement>() else {
            continue;
        };
        let x = rect.left();
        let y = rect.top();
        let css = format!(
            "position:fixed;left:{x:.2}px;top:{y:.2}px;width:{:.2}px;height:{:.2}px;transform:none;visibility:visible;opacity:1;pointer-events:none;z-index:6;transition:opacity 350ms ease-out,transform 350ms ease-out",
            rect.width(),
            rect.height(),
        );
        let _ = ghost.set_attribute("style", &css);
        let _ = ghost.set_attribute("data-cloud-ghost", "");
        let _ = ghost.set_attribute("data-cloud-success", "true");
        let _ = ghost.set_attribute("aria-hidden", "true");
        ghost.set_tab_index(-1);
        let _ = ghost.class_list().add_1("motion-reduce:!transition-none");
        let _ = ghost.set_attribute("disabled", "");
        if parent.append_child(&ghost).is_err() {
            continue;
        }
        let _ = ghost.get_bounding_client_rect();
        let fade = ghost.clone();
        let css = css.clone();
        let frame = request_animation_frame(move |_| {
            let _ = fade.set_attribute(
                "style",
                &format!("{css};opacity:0;transform:translateY(-9px) scale(.96)"),
            );
        });
        Timeout::new(420, move || {
            ghost.remove();
            drop(frame);
        })
        .forget();
    }
}
