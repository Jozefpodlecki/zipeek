use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, PointerEvent};
use zipeek_hanzi_writing::{CanvasTransform, InkCapture, Point, Stroke};

pub(crate) fn normalized_pointer(
    event: &PointerEvent,
    canvas: &HtmlCanvasElement,
) -> Option<Point> {
    let bounds = canvas.get_bounding_client_rect();
    let origin = Point::new(bounds.left(), bounds.top()).ok()?;
    let coordinate = Point::new(f64::from(event.client_x()), f64::from(event.client_y())).ok()?;
    let transform = CanvasTransform::new(
        origin,
        bounds.width(),
        bounds.height(),
        f64::from(canvas.width()),
        f64::from(canvas.height()),
    )
    .ok()?;
    Some(transform.client_to_normalized(coordinate))
}

fn render_stroke(context: &CanvasRenderingContext2d, points: &[Point], width: f64, height: f64) {
    let Some(first) = points.first() else {
        return;
    };
    context.begin_path();
    context.move_to(first.x * width, first.y * height);
    if points.len() == 1 {
        context.line_to(first.x * width + 0.01, first.y * height + 0.01);
    } else {
        for point in &points[1..] {
            context.line_to(point.x * width, point.y * height);
        }
    }
    context.stroke();
}

pub(crate) fn paint(canvas: &HtmlCanvasElement, ink: &InkCapture) {
    let Some(context) = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|context| context.dyn_into::<CanvasRenderingContext2d>().ok())
    else {
        return;
    };
    let width = f64::from(canvas.width());
    let height = f64::from(canvas.height());
    context.clear_rect(0.0, 0.0, width, height);
    context.set_fill_style_str("#111e35");
    context.fill_rect(0.0, 0.0, width, height);
    context.set_stroke_style_str("#344766");
    context.set_line_width(1.0);
    context
        .set_line_dash(&js_sys::Array::of2(&6.into(), &7.into()))
        .ok();
    context.begin_path();
    context.move_to(width / 2.0, 0.0);
    context.line_to(width / 2.0, height);
    context.move_to(0.0, height / 2.0);
    context.line_to(width, height / 2.0);
    context.stroke();
    context.set_line_dash(&js_sys::Array::new()).ok();
    context.set_stroke_style_str("#79b7ff");
    context.set_line_width((width * 0.013).max(3.0));
    context.set_line_cap("round");
    context.set_line_join("round");
    for Stroke { points } in ink.strokes() {
        render_stroke(&context, points, width, height);
    }
    if let Some(points) = ink.active_points() {
        render_stroke(&context, points, width, height);
    }
}
