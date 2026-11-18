#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ViewBox {
    pub fn parse(input: &str) -> Option<Self> {
        let values = input.split_whitespace().map(str::parse::<f64>)
            .collect::<Result<Vec<_>, _>>().ok()?;
        if values.len() != 4 || !values.iter().all(|number| number.is_finite())
            || values[2] <= 0.0 || values[3] <= 0.0 {
            return None;
        }
        Some(Self { x: values[0], y: values[1], width: values[2], height: values[3] })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapViewport {
    base: ViewBox,
    pub zoom: f64,
    pan: MapPoint,
}

impl MapViewport {
    pub fn new(base: ViewBox) -> Self {
        Self { base, zoom: 1.0, pan: MapPoint { x: 0.0, y: 0.0 } }
    }

    pub fn view_box(self) -> String {
        let width = self.base.width / self.zoom;
        let height = self.base.height / self.zoom;
        let max_x = (self.base.width - width) / 2.0;
        let max_y = (self.base.height - height) / 2.0;
        let x = self.base.x + max_x + self.pan.x.clamp(-max_x, max_x);
        let y = self.base.y + max_y + self.pan.y.clamp(-max_y, max_y);
        format!("{x} {y} {width} {height}")
    }

    pub fn zoom_at(&mut self, multiplier: f64, cursor: MapPoint) {
        if !multiplier.is_finite() || multiplier <= 0.0 { return; }
        let previous = self.zoom;
        self.zoom = (self.zoom * multiplier).clamp(1.0, 5.0);
        let shift = 1.0 / previous - 1.0 / self.zoom;
        self.pan.x += cursor.x.clamp(-0.5, 0.5) * self.base.width * shift;
        self.pan.y += cursor.y.clamp(-0.5, 0.5) * self.base.height * shift;
        self.constrain();
    }

    pub fn pan_by(&mut self, pixels: MapPoint, rendered_width: f64, rendered_height: f64) {
        let visible_width = self.base.width / self.zoom;
        let visible_height = self.base.height / self.zoom;
        let Some(scale) = rendered_scale(rendered_width, rendered_height, visible_width, visible_height) else {
            return;
        };
        if !pixels.x.is_finite() || !pixels.y.is_finite() {
            return;
        }
        self.pan.x -= pixels.x / scale;
        self.pan.y -= pixels.y / scale;
        self.constrain();
    }

    pub fn cursor_in_viewbox(self, cursor: MapPoint, rendered_width: f64, rendered_height: f64) -> Option<MapPoint> {
        let visible_width = self.base.width / self.zoom;
        let visible_height = self.base.height / self.zoom;
        let scale = rendered_scale(rendered_width, rendered_height, visible_width, visible_height)?;
        if !cursor.x.is_finite() || !cursor.y.is_finite() {
            return None;
        }
        let left = (rendered_width - visible_width * scale) / 2.0;
        let top = (rendered_height - visible_height * scale) / 2.0;
        Some(MapPoint {
            x: ((cursor.x - left) / (visible_width * scale) - 0.5).clamp(-0.5, 0.5),
            y: ((cursor.y - top) / (visible_height * scale) - 0.5).clamp(-0.5, 0.5),
        })
    }

    fn constrain(&mut self) {
        let max_x = self.base.width * (1.0 - 1.0 / self.zoom) / 2.0;
        let max_y = self.base.height * (1.0 - 1.0 / self.zoom) / 2.0;
        self.pan.x = self.pan.x.clamp(-max_x, max_x);
        self.pan.y = self.pan.y.clamp(-max_y, max_y);
    }
}

fn rendered_scale(container_width: f64, container_height: f64, map_width: f64, map_height: f64) -> Option<f64> {
    if ![container_width, container_height, map_width, map_height]
        .into_iter().all(|value| value.is_finite() && value > 0.0)
    {
        return None;
    }
    Some((container_width / map_width).min(container_height / map_height))
}

pub fn wheel_delta_pixels(delta: f64, mode: u32, height: f64) -> Option<f64> {
    if !delta.is_finite() || !height.is_finite() || height <= 0.0 {
        return None;
    }
    let factor = match mode {
        0 => 1.0,
        1 => 16.0,
        2 => height,
        _ => return None,
    };
    Some((delta * factor).clamp(-240.0, 240.0))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragGesture {
    pub pointer_id: i32,
    pub start: MapPoint,
    pub previous: MapPoint,
    pub moved: bool,
}

impl DragGesture {
    pub fn new(pointer_id: i32, point: MapPoint) -> Self {
        Self { pointer_id, start: point, previous: point, moved: false }
    }

    pub fn advance(&mut self, point: MapPoint) -> MapPoint {
        let was_moved = self.moved;
        if (point.x - self.start.x).hypot(point.y - self.start.y) >= 5.0 {
            self.moved = true;
        }
        let reference = if !was_moved && self.moved { self.start } else { self.previous };
        let delta = MapPoint { x: point.x - reference.x, y: point.y - reference.y };
        self.previous = point;
        delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamped_viewport_cannot_escape_bounds() {
        let mut viewport = MapViewport::new(ViewBox::parse("0 0 100 60").unwrap());
        viewport.zoom_at(2.0, MapPoint { x: 0.0, y: 0.0 });
        assert_eq!(viewport.view_box(), "25 15 50 30");
        viewport.pan_by(MapPoint { x: 10000.0, y: 10000.0 }, 600.0, 400.0);
        assert_eq!(viewport.view_box(), "0 0 50 30");
        viewport.zoom_at(0.1, MapPoint { x: 0.0, y: 0.0 });
        assert_eq!(viewport.view_box(), "0 0 100 60");
    }

    #[test]
    fn drag_threshold_measures_total_distance() {
        let mut gesture = DragGesture::new(7, MapPoint { x: 0.0, y: 0.0 });
        for x in 1..=6 {
            gesture.advance(MapPoint { x: f64::from(x), y: 0.0 });
        }
        assert!(gesture.moved);
    }

    #[test]
    fn pan_uses_svg_rendered_scale_when_letterboxed() {
        let mut viewport = MapViewport::new(ViewBox::parse("0 0 100 50").unwrap());
        viewport.zoom_at(2.0, MapPoint { x: 0.0, y: 0.0 });
        viewport.pan_by(MapPoint { x: 100.0, y: 0.0 }, 200.0, 200.0);
        assert_eq!(viewport.view_box(), "0 12.5 50 25");
    }

    #[test]
    fn cursor_accounts_for_svg_letterboxing() {
        let viewport = MapViewport::new(ViewBox::parse("0 0 100 50").unwrap());
        let left = viewport.cursor_in_viewbox(MapPoint { x: 0.0, y: 100.0 }, 200.0, 200.0).unwrap();
        assert_eq!(left.x, -0.5);
        assert_eq!(left.y, 0.0);
        let top = viewport.cursor_in_viewbox(MapPoint { x: 100.0, y: 0.0 }, 200.0, 200.0).unwrap();
        assert_eq!(top.x, 0.0);
        assert_eq!(top.y, -0.5);
    }

    #[test]
    fn wheel_delta_modes_have_comparable_behavior() {
        assert_eq!(wheel_delta_pixels(48.0, 0, 500.0), Some(48.0));
        assert_eq!(wheel_delta_pixels(3.0, 1, 500.0), Some(48.0));
        assert_eq!(wheel_delta_pixels(1.0, 2, 500.0), Some(240.0));
        assert_eq!(wheel_delta_pixels(f64::NAN, 0, 500.0), None);
    }

    #[test]
    fn crossing_drag_threshold_includes_prior_motion() {
        let mut gesture = DragGesture::new(4, MapPoint { x: 0.0, y: 0.0 });
        for x in 1..=4 {
            gesture.advance(MapPoint { x: f64::from(x), y: 0.0 });
            assert!(!gesture.moved);
        }
        let delta = gesture.advance(MapPoint { x: 5.0, y: 0.0 });
        assert_eq!(delta.x, 5.0);
    }

    #[test]
    fn invalid_boxes_are_rejected() {
        assert!(ViewBox::parse("0 0 5 -2").is_none());
        assert!(ViewBox::parse("NaN 0 5 2").is_none());
    }
}
