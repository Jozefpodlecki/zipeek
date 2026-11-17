use yew::prelude::*;
use zipeek_hanzi_writing::Stroke;

const VIEWBOX_SIDE: f64 = 1024.0;

pub(super) fn median_path(stroke: &Stroke) -> Option<String> {
    let mut points = stroke.points.iter();
    let first = points.next()?;
    let mut path = format!(
        "M {:.2} {:.2}",
        first.x * VIEWBOX_SIDE,
        first.y * VIEWBOX_SIDE
    );
    for point in points {
        path.push_str(&format!(
            " L {:.2} {:.2}",
            point.x * VIEWBOX_SIDE,
            point.y * VIEWBOX_SIDE
        ));
    }
    Some(path)
}

pub(super) fn replay_median_path(stroke: &Stroke) -> Option<String> {
    let mut points = stroke.points.iter();
    let first = points.next()?;
    let mut path = format!(
        "M {:.2} {:.2}",
        first.x * VIEWBOX_SIDE,
        900.0 - first.y * VIEWBOX_SIDE
    );
    for point in points {
        path.push_str(&format!(
            " L {:.2} {:.2}",
            point.x * VIEWBOX_SIDE,
            900.0 - point.y * VIEWBOX_SIDE
        ));
    }
    Some(path)
}

fn preview_endpoint(stroke: &Stroke, fraction: f64) -> Option<zipeek_hanzi_writing::Point> {
    let first = stroke.points.first()?;
    let total = stroke.points.windows(2).map(|pair| {
        (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y)
    }).sum::<f64>();
    if total <= f64::EPSILON {
        return None;
    }
    let mut remaining = total * fraction.clamp(0.0, 0.5);
    for pair in stroke.points.windows(2) {
        let distance = (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y);
        if distance <= f64::EPSILON {
            continue;
        }
        if remaining <= distance {
            let ratio = remaining / distance;
            return zipeek_hanzi_writing::Point::new(
                pair[0].x + (pair[1].x - pair[0].x) * ratio,
                pair[0].y + (pair[1].y - pair[0].y) * ratio,
            ).ok();
        }
        remaining -= distance;
    }
    Some(*stroke.points.last().unwrap_or(first))
}

pub(super) fn partial_stroke_path(stroke: &Stroke, fraction: f64) -> Option<String> {
    let first = stroke.points.first()?;
    let total = stroke.points.windows(2).map(|pair| {
        (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y)
    }).sum::<f64>();
    if total <= f64::EPSILON {
        return None;
    }
    let mut remaining = total * fraction.clamp(0.0, 0.5);
    let mut path = format!("M {:.2} {:.2}", first.x * VIEWBOX_SIDE, first.y * VIEWBOX_SIDE);
    for pair in stroke.points.windows(2) {
        let distance = (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y);
        if distance <= f64::EPSILON {
            continue;
        }
        let ratio = (remaining / distance).min(1.0);
        let x = pair[0].x + (pair[1].x - pair[0].x) * ratio;
        let y = pair[0].y + (pair[1].y - pair[0].y) * ratio;
        path.push_str(&format!(" L {:.2} {:.2}", x * VIEWBOX_SIDE, y * VIEWBOX_SIDE));
        remaining -= distance * ratio;
        if remaining <= f64::EPSILON {
            break;
        }
    }
    Some(path)
}

#[derive(Properties, PartialEq)]
pub(super) struct DirectionGuideProps {
    pub stroke: Stroke,
    pub stroke_index: usize,
    pub rejection_count: usize,
}

#[function_component(DirectionGuide)]
pub(super) fn direction_guide(props: &DirectionGuideProps) -> Html {
    let Some(start) = props.stroke.points.first() else {
        return Html::default();
    };
    let fraction = match props.rejection_count {
        0 => 0.25,
        1 => 0.35,
        _ => 0.5,
    };
    let Some(path) = partial_stroke_path(&props.stroke, fraction) else {
        return Html::default();
    };
    let gradient_id = format!("hanzi-hint-fade-{}", props.stroke_index);
    let endpoint = preview_endpoint(&props.stroke, fraction).unwrap_or(*start);
    html! {
        <g data-hanzi-direction-guide="" data-hanzi-guide-stroke={props.stroke_index.to_string()}
            data-hanzi-hint-fraction={fraction.to_string()}>
            <defs>
                <linearGradient id={gradient_id.clone()} gradientUnits="userSpaceOnUse"
                    x1={(start.x * VIEWBOX_SIDE).to_string()}
                    y1={(start.y * VIEWBOX_SIDE).to_string()}
                    x2={(endpoint.x * VIEWBOX_SIDE).to_string()}
                    y2={(endpoint.y * VIEWBOX_SIDE).to_string()}>
                    <stop offset="0%" stop-color="#fbbf24" stop-opacity="0.9" />
                    <stop offset="100%" stop-color="#fbbf24" stop-opacity="0.25" />
                </linearGradient>
            </defs>
            <path data-hanzi-stroke-preview="" d={path} fill="none"
                stroke={format!("url(#{gradient_id})")}
                stroke-width="19" stroke-linecap="round" stroke-linejoin="round" />
            <circle data-hanzi-next-stroke-start=""
                cx={(start.x * VIEWBOX_SIDE).to_string()}
                cy={(start.y * VIEWBOX_SIDE).to_string()}
                r="6" fill="#fbbf24" />
        </g>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_hanzi_writing::Point;

    #[test]
    fn preserves_hook_vertices_in_svg_path() {
        let stroke = Stroke::new(vec![
            Point::new(0.1, 0.2).unwrap(),
            Point::new(0.8, 0.2).unwrap(),
            Point::new(0.7, 0.3).unwrap(),
        ])
        .unwrap();
        assert_eq!(
            median_path(&stroke).as_deref(),
            Some("M 102.40 204.80 L 819.20 204.80 L 716.80 307.20")
        );
    }

    #[test]
    fn replay_mask_uses_the_same_reference_coordinates_as_the_outline() {
        let stroke = Stroke::new(vec![
            Point::new(0.1, 0.2).unwrap(),
            Point::new(0.8, 0.2).unwrap(),
            Point::new(0.7, 0.3).unwrap(),
        ])
        .unwrap();
        assert_eq!(
            replay_median_path(&stroke).as_deref(),
            Some("M 102.40 695.20 L 819.20 695.20 L 716.80 592.80")
        );
        assert_eq!(
            median_path(&stroke).as_deref(),
            Some("M 102.40 204.80 L 819.20 204.80 L 716.80 307.20")
        );
    }

    #[test]
    fn replay_mask_preserves_the_full_reference_vertical_extent() {
        let stroke = Stroke::new(vec![
            Point::new(0.0, 0.0).unwrap(),
            Point::new(1.0, 1.0).unwrap(),
        ])
        .unwrap();
        assert_eq!(
            replay_median_path(&stroke).as_deref(),
            Some("M 0.00 900.00 L 1024.00 -124.00")
        );
    }

    #[test]
    fn preview_follows_arc_length_across_a_corner() {
        let stroke = Stroke::new(vec![
            Point::new(0.1, 0.2).unwrap(),
            Point::new(0.5, 0.2).unwrap(),
            Point::new(0.5, 0.6).unwrap(),
        ]).unwrap();
        assert_eq!(partial_stroke_path(&stroke, 0.25).as_deref(),
            Some("M 102.40 204.80 L 307.20 204.80"));
        assert_eq!(partial_stroke_path(&stroke, 0.5).as_deref(),
            Some("M 102.40 204.80 L 512.00 204.80"));
    }

    #[test]
    fn preview_rejects_zero_length_reference() {
        let stroke = Stroke { points: vec![Point::new(0.3, 0.3).unwrap()] };
        assert_eq!(partial_stroke_path(&stroke, 0.25), None);
    }

    #[test]
    fn empty_stroke_has_no_visual_path() {
        assert!(median_path(&Stroke { points: vec![] }).is_none());
    }
}
