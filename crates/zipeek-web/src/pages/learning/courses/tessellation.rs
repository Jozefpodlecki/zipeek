#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Point {
    pub x: f64,
    pub y: f64,
}

pub(super) const WIDTH: f64 = 1000.0;
pub(super) const HEIGHT: f64 = 600.0;

fn clip_half_plane(polygon: &[Point], a: f64, b: f64, c: f64) -> Vec<Point> {
    if polygon.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(polygon.len() + 1);
    let mut previous = polygon[polygon.len() - 1];
    let mut previous_distance = a * previous.x + b * previous.y - c;
    for &current in polygon {
        let current_distance = a * current.x + b * current.y - c;
        if (previous_distance <= 1e-9) != (current_distance <= 1e-9) {
            let fraction = previous_distance / (previous_distance - current_distance);
            result.push(Point {
                x: previous.x + fraction * (current.x - previous.x),
                y: previous.y + fraction * (current.y - previous.y),
            });
        }
        if current_distance <= 1e-9 {
            result.push(current);
        }
        previous = current;
        previous_distance = current_distance;
    }
    result
}

pub(super) fn voronoi(sites: &[Point]) -> Vec<Vec<Point>> {
    sites
        .iter()
        .enumerate()
        .map(|(index, site)| {
            let mut polygon = vec![
                Point { x: 0.0, y: 0.0 },
                Point { x: WIDTH, y: 0.0 },
                Point {
                    x: WIDTH,
                    y: HEIGHT,
                },
                Point { x: 0.0, y: HEIGHT },
            ];
            for (other_index, other) in sites.iter().enumerate() {
                if index == other_index {
                    continue;
                }
                let a = 2.0 * (other.x - site.x);
                let b = 2.0 * (other.y - site.y);
                let c = other.x * other.x + other.y * other.y - site.x * site.x - site.y * site.y;
                polygon = clip_half_plane(&polygon, a, b, c);
                if polygon.is_empty() {
                    break;
                }
            }
            polygon
        })
        .collect()
}

fn control_point(a: Point, b: Point) -> Point {
    let (start, end) = if a.x < b.x || (a.x == b.x && a.y < b.y) {
        (a, b)
    } else {
        (b, a)
    };
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let distance = dx.hypot(dy);
    let on_border = (start.x.abs() < 1e-6 && end.x.abs() < 1e-6)
        || ((start.x - WIDTH).abs() < 1e-6 && (end.x - WIDTH).abs() < 1e-6)
        || (start.y.abs() < 1e-6 && end.y.abs() < 1e-6)
        || ((start.y - HEIGHT).abs() < 1e-6 && (end.y - HEIGHT).abs() < 1e-6);
    let hash = (start.x * 0.017 + end.x * 0.031 + start.y * 0.043 + end.y * 0.059).sin();
    let displacement = if on_border || distance < 1e-6 {
        0.0
    } else {
        hash * (distance * 0.035).min(2.8)
    };
    Point {
        x: (start.x + end.x) / 2.0 - dy / distance.max(1e-6) * displacement,
        y: (start.y + end.y) / 2.0 + dx / distance.max(1e-6) * displacement,
    }
}

pub(super) fn curved_path(polygon: &[Point]) -> String {
    let Some(&start) = polygon.first() else {
        return String::new();
    };
    let mut path = format!("M {:.5} {:.5}", start.x, start.y);
    for (a, b) in polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
    {
        let control = control_point(*a, *b);
        path.push_str(&format!(
            " Q {:.5} {:.5} {:.5} {:.5}",
            control.x, control.y, b.x, b.y
        ));
    }
    path.push_str(" Z");
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reversed_edge_has_the_same_control_point() {
        let a = Point { x: 150.0, y: 180.0 };
        let b = Point { x: 250.0, y: 280.0 };
        assert_eq!(control_point(a, b), control_point(b, a));
    }

    #[test]
    fn outer_edges_stay_on_canvas() {
        let a = Point { x: 0.0, y: 100.0 };
        let b = Point { x: 0.0, y: 250.0 };
        assert_eq!(control_point(a, b).x, 0.0);
    }

    #[test]
    fn clipping_covers_the_canvas() {
        let polygons = voronoi(&[Point { x: 150.0, y: 300.0 }, Point { x: 850.0, y: 300.0 }]);
        assert_eq!(polygons.len(), 2);
        assert!(
            polygons
                .iter()
                .all(|polygon| curved_path(polygon).contains(" Q "))
        );
    }
}
