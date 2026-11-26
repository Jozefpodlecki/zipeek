use crate::{
    elastic,
    geometry::{Point, Stroke, resample},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchConfig {
    pub samples: usize,
    pub maximum_start_error: f64,
    pub maximum_end_error: f64,
    pub maximum_average_error: f64,
    pub maximum_peak_error: f64,
    pub minimum_length_ratio: f64,
    pub maximum_length_ratio: f64,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            samples: 32,
            maximum_start_error: 0.15,
            maximum_end_error: 0.16,
            maximum_average_error: 0.105,
            maximum_peak_error: 0.16,
            minimum_length_ratio: 0.45,
            maximum_length_ratio: 1.9,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeRejection {
    TooShort,
    WrongStart,
    WrongEnd,
    WrongDirection,
    WrongOrder,
    WrongShape,
    InvalidGeometry,
}

impl StrokeRejection {
    pub fn message(self) -> &'static str {
        match self {
            Self::TooShort => "Draw the full stroke from its starting point.",
            Self::WrongStart => "Begin closer to the highlighted stroke's starting point.",
            Self::WrongEnd => "Finish closer to the highlighted stroke's endpoint.",
            Self::WrongDirection => "Draw this stroke in the indicated direction.",
            Self::WrongOrder => "That stroke comes later. Trace the highlighted stroke first.",
            Self::WrongShape => "Follow the highlighted stroke more closely.",
            Self::InvalidGeometry => "The stroke could not be evaluated. Try again.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeScore {
    pub average_distance: f64,
    pub peak_distance: f64,
    pub start_distance: f64,
    pub end_distance: f64,
}

pub fn match_stroke(
    expected: &Stroke,
    actual: &Stroke,
    config: MatchConfig,
) -> Result<StrokeScore, StrokeRejection> {
    match_stroke_inner(expected, actual, config, true)
}

fn match_stroke_inner(
    expected: &Stroke,
    actual: &Stroke,
    config: MatchConfig,
    detect_reverse: bool,
) -> Result<StrokeScore, StrokeRejection> {
    if config.samples < 3
        || config.samples > 256
        || expected
            .points
            .iter()
            .chain(actual.points.iter())
            .any(|p| !p.x.is_finite() || !p.y.is_finite())
        || expected.points.is_empty()
        || actual.points.is_empty()
        || !expected.length().is_finite()
        || !actual.length().is_finite()
        || ![
            config.maximum_start_error,
            config.maximum_end_error,
            config.maximum_average_error,
            config.maximum_peak_error,
            config.minimum_length_ratio,
            config.maximum_length_ratio,
        ]
        .iter()
        .all(|value| value.is_finite() && *value > 0.0)
        || config.minimum_length_ratio > config.maximum_length_ratio
    {
        return Err(StrokeRejection::InvalidGeometry);
    }
    let reference_length = expected.length();
    let candidate_length = actual.length();
    if reference_length < 0.012 {
        let expected_point = expected.points[0];
        let actual_point = actual.points[0];
        let distance = expected_point.distance(actual_point);
        if distance > config.maximum_start_error {
            return Err(StrokeRejection::WrongStart);
        }
        if candidate_length > 0.12 {
            return Err(StrokeRejection::WrongShape);
        }
        return Ok(StrokeScore {
            average_distance: distance,
            peak_distance: distance,
            start_distance: distance,
            end_distance: distance,
        });
    }
    if candidate_length < 0.012 {
        return Err(StrokeRejection::TooShort);
    }
    let length_ratio = candidate_length / reference_length;
    if length_ratio < config.minimum_length_ratio {
        return Err(StrokeRejection::TooShort);
    }
    if length_ratio > config.maximum_length_ratio {
        return Err(StrokeRejection::WrongShape);
    }
    let reference_start = expected.points[0];
    let reference_end = *expected
        .points
        .last()
        .ok_or(StrokeRejection::InvalidGeometry)?;
    let actual_start = actual.points[0];
    let actual_end = *actual
        .points
        .last()
        .ok_or(StrokeRejection::InvalidGeometry)?;
    let start_distance = reference_start.distance(actual_start);
    let end_distance = reference_end.distance(actual_end);
    let reference_displacement = reference_start.distance(reference_end);
    if reference_displacement > 0.045
        && reference_end.distance(actual_start) <= config.maximum_end_error
        && reference_start.distance(actual_end) <= config.maximum_start_error
    {
        let direction = (
            reference_end.x - reference_start.x,
            reference_end.y - reference_start.y,
        );
        let actual_direction = (actual_end.x - actual_start.x, actual_end.y - actual_start.y);
        if direction.0 * actual_direction.0 + direction.1 * actual_direction.1 < 0.0 {
            return Err(StrokeRejection::WrongDirection);
        }
    }
    if start_distance > config.maximum_start_error {
        return Err(StrokeRejection::WrongStart);
    }
    if end_distance > config.maximum_end_error {
        return Err(StrokeRejection::WrongEnd);
    }
    if reference_displacement > 0.045 {
        let direction = (
            reference_end.x - reference_start.x,
            reference_end.y - reference_start.y,
        );
        let actual_direction = (actual_end.x - actual_start.x, actual_end.y - actual_start.y);
        let dot = direction.0 * actual_direction.0 + direction.1 * actual_direction.1;
        if dot <= 0.0 {
            return Err(StrokeRejection::WrongDirection);
        }
    }
    let reference =
        resample(expected, config.samples).map_err(|_| StrokeRejection::InvalidGeometry)?;
    let candidate =
        resample(actual, config.samples).map_err(|_| StrokeRejection::InvalidGeometry)?;
    let fit = elastic::compare(&reference, &candidate);
    if fit.mean > config.maximum_average_error || fit.deviation > config.maximum_peak_error {
        if detect_reverse {
            let reversed = Stroke {
                points: actual.points.iter().rev().copied().collect(),
            };
            if match_stroke_inner(expected, &reversed, config, false).is_ok() {
                return Err(StrokeRejection::WrongDirection);
            }
        }
        return Err(StrokeRejection::WrongShape);
    }
    Ok(StrokeScore {
        average_distance: fit.mean,
        peak_distance: fit.deviation,
        start_distance,
        end_distance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn stroke(points: &[(f64, f64)]) -> Stroke {
        Stroke::new(
            points
                .iter()
                .map(|&(x, y)| Point::new(x, y).unwrap())
                .collect(),
        )
        .unwrap()
    }
    #[test]
    fn invalid_direct_geometry_is_rejected_without_panicking() {
        let expected = Stroke { points: vec![] };
        let actual = stroke(&[(0.1, 0.2)]);
        assert_eq!(
            match_stroke(&expected, &actual, MatchConfig::default()),
            Err(StrokeRejection::InvalidGeometry)
        );
        let invalid = Stroke {
            points: vec![Point {
                x: f64::NAN,
                y: 0.0,
            }],
        };
        assert_eq!(
            match_stroke(&invalid, &actual, MatchConfig::default()),
            Err(StrokeRejection::InvalidGeometry)
        );
    }
    #[test]
    fn accepts_imperfect_but_correct_direction() {
        let expected = stroke(&[(0.2, 0.3), (0.5, 0.3), (0.8, 0.3)]);
        let actual = stroke(&[(0.21, 0.32), (0.43, 0.33), (0.62, 0.31), (0.79, 0.32)]);
        assert!(match_stroke(&expected, &actual, MatchConfig::default()).is_ok());
    }
    #[test]
    fn rejects_reverse_and_displaced_strokes() {
        let expected = stroke(&[(0.2, 0.3), (0.8, 0.3)]);
        let reversed = stroke(&[(0.8, 0.3), (0.2, 0.3)]);
        let displaced = stroke(&[(0.2, 0.6), (0.8, 0.6)]);
        assert_eq!(
            match_stroke(&expected, &reversed, MatchConfig::default()),
            Err(StrokeRejection::WrongDirection)
        );
        assert_eq!(
            match_stroke(&expected, &displaced, MatchConfig::default()),
            Err(StrokeRejection::WrongStart)
        );
    }
    #[test]
    fn wrongly_placed_reverse_trace_is_not_mislabelled_as_direction_error() {
        let expected = stroke(&[(0.2, 0.3), (0.8, 0.3)]);
        let distant_reverse = stroke(&[(0.8, 0.7), (0.2, 0.7)]);
        assert_eq!(
            match_stroke(&expected, &distant_reverse, MatchConfig::default()),
            Err(StrokeRejection::WrongStart)
        );
    }

    #[test]
    fn rejects_truncated_and_wrongly_curved_strokes() {
        let expected = stroke(&[(0.2, 0.3), (0.5, 0.3), (0.8, 0.3)]);
        assert_eq!(
            match_stroke(
                &expected,
                &stroke(&[(0.2, 0.3), (0.29, 0.3)]),
                MatchConfig::default()
            ),
            Err(StrokeRejection::TooShort)
        );
        assert_eq!(
            match_stroke(
                &expected,
                &stroke(&[(0.2, 0.3), (0.5, 0.6), (0.8, 0.3)]),
                MatchConfig::default()
            ),
            Err(StrokeRejection::WrongShape)
        );
    }
    #[test]
    fn accepts_local_tap_for_reference_dot_but_not_long_scribble() {
        let dot = stroke(&[(0.3, 0.4)]);
        assert!(match_stroke(&dot, &stroke(&[(0.31, 0.41)]), MatchConfig::default()).is_ok());
        assert_eq!(
            match_stroke(
                &dot,
                &stroke(&[(0.31, 0.41), (0.7, 0.4)]),
                MatchConfig::default()
            ),
            Err(StrokeRejection::WrongShape)
        );
    }
    #[test]
    fn rejects_tap_as_stroke() {
        let expected = stroke(&[(0.2, 0.3), (0.8, 0.3)]);
        assert_eq!(
            match_stroke(&expected, &stroke(&[(0.2, 0.3)]), MatchConfig::default()),
            Err(StrokeRejection::TooShort)
        );
    }
    #[test]
    fn short_hook_is_not_lost_between_uniform_samples() {
        let expected = stroke(&[(0.10, 0.50), (0.86, 0.50), (0.86, 0.28), (0.84, 0.50)]);
        let correct = stroke(&[(0.10, 0.50), (0.86, 0.50), (0.86, 0.28), (0.84, 0.50)]);
        let shortcut = stroke(&[(0.10, 0.50), (0.84, 0.50)]);
        assert!(match_stroke(&expected, &correct, MatchConfig::default()).is_ok());
        assert_eq!(
            match_stroke(&expected, &shortcut, MatchConfig::default()),
            Err(StrokeRejection::WrongShape)
        );
    }

    #[test]
    fn folded_stroke_reversed_path_is_detected() {
        let expected = stroke(&[(0.2, 0.4), (0.55, 0.2), (0.8, 0.5), (0.45, 0.8), (0.2, 0.4)]);
        let reversed = Stroke {
            points: expected.points.iter().rev().copied().collect(),
        };
        assert!(match_stroke(&expected, &expected, MatchConfig::default()).is_ok());
        assert_eq!(
            match_stroke(&expected, &reversed, MatchConfig::default()),
            Err(StrokeRejection::WrongDirection)
        );
    }

    #[test]
    fn config_rejects_invalid_thresholds_without_panicking() {
        let line = stroke(&[(0.2, 0.3), (0.8, 0.3)]);
        for bad in [
            MatchConfig {
                maximum_start_error: f64::NAN,
                ..MatchConfig::default()
            },
            MatchConfig {
                maximum_end_error: f64::INFINITY,
                ..MatchConfig::default()
            },
            MatchConfig {
                minimum_length_ratio: -1.0,
                ..MatchConfig::default()
            },
            MatchConfig {
                maximum_length_ratio: 0.0,
                ..MatchConfig::default()
            },
            MatchConfig {
                minimum_length_ratio: 2.0,
                maximum_length_ratio: 1.0,
                ..MatchConfig::default()
            },
        ] {
            assert_eq!(
                match_stroke(&line, &line, bad),
                Err(StrokeRejection::InvalidGeometry)
            );
        }
    }

    #[test]
    fn extra_hook_in_candidate_is_rejected() {
        let line = stroke(&[(0.10, 0.5), (0.85, 0.5)]);
        let extra_hook = stroke(&[(0.10, 0.5), (0.85, 0.5), (0.85, 0.32), (0.85, 0.5)]);
        assert_eq!(
            match_stroke(&line, &extra_hook, MatchConfig::default()),
            Err(StrokeRejection::WrongShape)
        );
    }
}

#[cfg(test)]
mod handwriting_regressions {
    use super::*;

    fn stroke(points: &[(f64, f64)]) -> Stroke {
        Stroke::new(
            points
                .iter()
                .map(|&(x, y)| Point::new(x, y).unwrap())
                .collect(),
        )
        .unwrap()
    }

    #[test]
    fn natural_variations_of_straight_stroke_are_accepted() {
        let reference = stroke(&[(0.15, 0.4), (0.85, 0.4)]);
        for candidate in [
            stroke(&[(0.17, 0.43), (0.34, 0.44), (0.58, 0.39), (0.83, 0.42)]),
            stroke(&[(0.21, 0.46), (0.48, 0.44), (0.80, 0.45)]),
            stroke(&[(0.12, 0.36), (0.51, 0.39), (0.9, 0.36)]),
        ] {
            assert!(match_stroke(&reference, &candidate, MatchConfig::default()).is_ok());
        }
    }

    #[test]
    fn shifted_turn_and_variable_speed_do_not_require_exact_correspondence() {
        let reference = stroke(&[(0.2, 0.2), (0.72, 0.2), (0.72, 0.79)]);
        let candidate = stroke(&[
            (0.23, 0.23),
            (0.37, 0.23),
            (0.56, 0.22),
            (0.68, 0.24),
            (0.69, 0.39),
            (0.70, 0.63),
            (0.70, 0.76),
        ]);
        assert!(match_stroke(&reference, &candidate, MatchConfig::default()).is_ok());
    }

    #[test]
    fn wrong_direction_and_wrong_shape_remain_rejected() {
        let reference = stroke(&[(0.2, 0.2), (0.8, 0.2), (0.8, 0.75)]);
        let reversed = stroke(&[(0.8, 0.75), (0.8, 0.2), (0.2, 0.2)]);
        let shortcut = stroke(&[(0.2, 0.2), (0.8, 0.75)]);
        assert_eq!(
            match_stroke(&reference, &reversed, MatchConfig::default()),
            Err(StrokeRejection::WrongDirection)
        );
        assert_eq!(
            match_stroke(&reference, &shortcut, MatchConfig::default()),
            Err(StrokeRejection::WrongShape)
        );
    }

    #[test]
    fn generous_tolerance_does_not_accept_a_large_loop() {
        let reference = stroke(&[(0.15, 0.5), (0.85, 0.5)]);
        let looped = stroke(&[
            (0.15, 0.5),
            (0.45, 0.5),
            (0.55, 0.2),
            (0.7, 0.75),
            (0.85, 0.5),
        ]);
        assert_eq!(
            match_stroke(&reference, &looped, MatchConfig::default()),
            Err(StrokeRejection::WrongShape)
        );
    }
}
