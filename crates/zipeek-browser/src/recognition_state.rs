#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasPoint {
    pub x: f64,
    pub y: f64,
}

pub fn input_kind(has_image: bool, stroke_count: usize) -> &'static str {
    match (has_image, stroke_count > 0) {
        (true, true) => "mixed",
        (true, false) => "image",
        (false, true) => "drawing",
        (false, false) => "empty",
    }
}

pub fn allowed_image(mime: &str, size: u64) -> bool {
    matches!(mime, "image/png" | "image/jpeg" | "image/webp") && (1..=5_000_000).contains(&size)
}

pub fn canvas_point(
    client_x: f64,
    client_y: f64,
    rect_left: f64,
    rect_top: f64,
    rect_width: f64,
    rect_height: f64,
    canvas_width: f64,
    canvas_height: f64,
) -> Option<CanvasPoint> {
    if !rect_width.is_finite()
        || !rect_height.is_finite()
        || rect_width <= 0.0
        || rect_height <= 0.0
        || !client_x.is_finite()
        || !client_y.is_finite()
    {
        return None;
    }
    Some(CanvasPoint {
        x: (client_x - rect_left) * canvas_width / rect_width,
        y: (client_y - rect_top) * canvas_height / rect_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_states_match_workspace_contract() {
        assert_eq!(input_kind(false, 0), "empty");
        assert_eq!(input_kind(false, 2), "drawing");
        assert_eq!(input_kind(true, 0), "image");
        assert_eq!(input_kind(true, 1), "mixed");
    }

    #[test]
    fn image_limits_are_bounded() {
        assert!(allowed_image("image/png", 1));
        assert!(allowed_image("image/webp", 5_000_000));
        assert!(allowed_image("image/jpeg", 200));
        assert!(!allowed_image("image/svg+xml", 123));
        assert!(!allowed_image("image/png", 0));
        assert!(!allowed_image("image/png", 5_000_001));
    }

    #[test]
    fn points_scale_with_responsive_canvas() {
        assert_eq!(
            canvas_point(60.0, 45.0, 10.0, 5.0, 100.0, 80.0, 400.0, 160.0),
            Some(CanvasPoint { x: 200.0, y: 80.0 })
        );
        assert_eq!(
            canvas_point(60.0, 45.0, 10.0, 5.0, 0.0, 80.0, 400.0, 160.0),
            None
        );
        assert_eq!(
            canvas_point(f64::NAN, 45.0, 10.0, 5.0, 100.0, 80.0, 400.0, 160.0),
            None
        );
        assert_eq!(
            canvas_point(110.0, 85.0, 10.0, 5.0, 100.0, 80.0, 400.0, 160.0),
            Some(CanvasPoint { x: 400.0, y: 160.0 })
        );
    }
}
