use crate::geometry::{GeometryError, Stroke, normalize, resample};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonError {
    StrokeCountMismatch { expected: usize, actual: usize },
    Geometry(GeometryError),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeComparison {
    pub average_distance: f64,
    pub maximum_distance: f64,
    pub sample_count: usize,
}

pub fn compare_strokes(
    reference: &[Stroke],
    candidate: &[Stroke],
    samples_per_stroke: usize,
) -> Result<StrokeComparison, ComparisonError> {
    if reference.len() != candidate.len() {
        return Err(ComparisonError::StrokeCountMismatch {
            expected: reference.len(),
            actual: candidate.len(),
        });
    }
    if samples_per_stroke < 2 || reference.is_empty() {
        return Err(ComparisonError::Geometry(GeometryError::InvalidSampleCount));
    }
    let mut sum = 0.0;
    let mut max: f64 = 0.0;
    let mut count = 0;
    for (expected, actual) in reference.iter().zip(candidate) {
        let normalized_reference =
            normalize(&resample(expected, samples_per_stroke).map_err(ComparisonError::Geometry)?);
        let normalized_candidate =
            normalize(&resample(actual, samples_per_stroke).map_err(ComparisonError::Geometry)?);
        for (a, b) in normalized_reference
            .points
            .iter()
            .zip(&normalized_candidate.points)
        {
            let distance = a.distance(*b);
            sum += distance;
            max = max.max(distance);
            count += 1;
        }
    }
    Ok(StrokeComparison {
        average_distance: sum / count as f64,
        maximum_distance: max,
        sample_count: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    fn s(points: &[(f64, f64)]) -> Stroke {
        Stroke::new(
            points
                .iter()
                .map(|&(x, y)| Point::new(x, y).unwrap())
                .collect(),
        )
        .unwrap()
    }

    #[test]
    fn matching_shape_under_translation_and_scale_is_equal() {
        let a = s(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
        let b = s(&[(10.0, 20.0), (12.0, 22.0), (14.0, 20.0)]);
        let comparison = compare_strokes(&[a], &[b], 9).unwrap();
        assert!(comparison.average_distance < 1e-12);
        assert_eq!(comparison.sample_count, 9);
    }

    #[test]
    fn reversed_direction_is_detectable() {
        let a = s(&[(0.0, 0.0), (1.0, 1.0)]);
        let b = s(&[(1.0, 1.0), (0.0, 0.0)]);
        assert!(compare_strokes(&[a], &[b], 8).unwrap().average_distance > 0.5);
    }

    #[test]
    fn stroke_count_and_sample_count_are_validated() {
        let a = s(&[(0.0, 0.0)]);
        assert_eq!(
            compare_strokes(&[a.clone()], &[], 8),
            Err(ComparisonError::StrokeCountMismatch {
                expected: 1,
                actual: 0
            })
        );
        assert_eq!(
            compare_strokes(&[a.clone()], &[a], 1),
            Err(ComparisonError::Geometry(GeometryError::InvalidSampleCount))
        );
    }
}
