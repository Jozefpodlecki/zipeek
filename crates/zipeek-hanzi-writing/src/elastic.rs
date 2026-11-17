use crate::geometry::{Point, Stroke};

#[derive(Debug, Clone, Copy)]
pub(super) struct ShapeFit {
    pub mean: f64,
    pub deviation: f64,
}

pub(super) fn compare(reference: &Stroke, candidate: &Stroke) -> ShapeFit {
    let reference_points = &reference.points;
    let candidate_points = &candidate.points;
    let count = reference_points.len();
    let drift = centroid(candidate_points);
    let anchor = centroid(reference_points);
    let shift = Point {
        x: (drift.x - anchor.x) * 0.35,
        y: (drift.y - anchor.y) * 0.35,
    };
    let adjusted: Vec<Point> = candidate_points
        .iter()
        .map(|point| Point {
            x: point.x - shift.x,
            y: point.y - shift.y,
        })
        .collect();

    let radius = (count / 5).max(2);
    let width = count + 1;
    let mut costs = vec![f64::INFINITY; width * width];
    let mut lengths = vec![0usize; width * width];
    costs[0] = 0.0;
    for i in 1..=count {
        for j in (i.saturating_sub(radius).max(1))..=(i + radius).min(count) {
            let distance = reference_points[i - 1].distance(adjusted[j - 1]);
            let predecessor = [
                (i - 1, j - 1, 0.0),
                (i - 1, j, 0.012),
                (i, j - 1, 0.012),
            ];
            let (previous, best_cost) = predecessor
                .iter()
                .filter_map(|&(a, b, penalty)| {
                    let index = a * width + b;
                    let cost = costs[index] + penalty;
                    cost.is_finite().then_some((index, cost))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap_or((0, f64::INFINITY));
            let index = i * width + j;
            costs[index] = best_cost + distance;
            lengths[index] = lengths[previous] + 1;
        }
    }
    let end = count * width + count;
    let mean = costs[end] / lengths[end].max(1) as f64;
    let deviation = robust_coverage(reference_points, &adjusted)
        .max(robust_coverage(&adjusted, reference_points));
    ShapeFit { mean, deviation }
}

fn centroid(points: &[Point]) -> Point {
    let (x, y) = points.iter().fold((0.0, 0.0), |(x, y), point| {
        (x + point.x, y + point.y)
    });
    Point {
        x: x / points.len() as f64,
        y: y / points.len() as f64,
    }
}

fn robust_coverage(source: &[Point], target: &[Point]) -> f64 {
    let mut errors: Vec<f64> = source
        .iter()
        .map(|point| {
            target
                .windows(2)
                .map(|segment| point_segment_distance(*point, segment[0], segment[1]))
                .fold(f64::INFINITY, f64::min)
        })
        .collect();
    errors.sort_by(f64::total_cmp);
    errors[(errors.len() * 9 / 10).min(errors.len() - 1)]
}

fn point_segment_distance(point: Point, a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let length_squared = dx * dx + dy * dy;
    if length_squared <= f64::EPSILON {
        return point.distance(a);
    }
    let t = (((point.x - a.x) * dx + (point.y - a.y) * dy) / length_squared)
        .clamp(0.0, 1.0);
    point.distance(Point {
        x: a.x + dx * t,
        y: a.y + dy * t,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stroke(points: &[(f64, f64)]) -> Stroke {
        Stroke {
            points: points.iter().map(|&(x, y)| Point { x, y }).collect(),
        }
    }

    #[test]
    fn identical_strokes_have_zero_error() {
        let sample = stroke(&[(0.1, 0.2), (0.5, 0.4), (0.8, 0.8)]);
        let fit = compare(&sample, &sample);
        assert!(fit.mean < 1e-12);
        assert!(fit.deviation < 1e-12);
    }

    #[test]
    fn curved_detour_is_not_equivalent_to_a_straight_line() {
        let straight = stroke(&[(0.2, 0.3), (0.5, 0.3), (0.8, 0.3)]);
        let detour = stroke(&[(0.2, 0.3), (0.5, 0.7), (0.8, 0.3)]);
        let fit = compare(&straight, &detour);
        assert!(fit.deviation > 0.2);
    }
}
