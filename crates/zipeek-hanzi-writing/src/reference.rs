use crate::geometry::{GeometryError, Point, Stroke};

pub fn reference_point(x: f64, y: f64) -> Result<Point, GeometryError> {
    Point::new(x / 1024.0, (900.0 - y) / 1024.0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceError {
    MissingStrokes,
    StrokeCountMismatch,
    EmptyMedian,
    InvalidCoordinate,
}

pub fn reference_medians(
    medians: &[Vec<[f64; 2]>],
    path_count: usize,
) -> Result<Vec<Stroke>, ReferenceError> {
    if path_count == 0 {
        return Err(ReferenceError::MissingStrokes);
    }
    if medians.len() != path_count {
        return Err(ReferenceError::StrokeCountMismatch);
    }
    medians
        .iter()
        .map(|median| {
            if median.is_empty() {
                return Err(ReferenceError::EmptyMedian);
            }
            let points = median
                .iter()
                .map(|pair| {
                    if !pair[0].is_finite()
                        || !pair[1].is_finite()
                        || !(0.0..=1024.0).contains(&pair[0])
                        || !(-124.0..=900.0).contains(&pair[1])
                    {
                        return Err(ReferenceError::InvalidCoordinate);
                    }
                    reference_point(pair[0], pair[1]).map_err(|_| ReferenceError::InvalidCoordinate)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Stroke { points })
        })
        .collect()
}

pub fn character_asset_id(character: char) -> String {
    format!("{:x}", u32::from(character))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_axes_are_not_inverted() {
        assert_eq!(
            reference_point(0.0, 900.0).unwrap(),
            Point::new(0.0, 0.0).unwrap()
        );
        assert_eq!(
            reference_point(1024.0, -124.0).unwrap(),
            Point::new(1.0, 1.0).unwrap()
        );
    }

    #[test]
    fn order_and_positions_are_preserved() {
        let strokes = reference_medians(
            &[vec![[0.0, 900.0], [1024.0, -124.0]], vec![[512.0, 388.0]]],
            2,
        )
        .unwrap();
        assert_eq!(strokes.len(), 2);
        assert_eq!(strokes[0].points[0].x, 0.0);
        assert_eq!(strokes[0].points[1].y, 1.0);
        assert_eq!(strokes[1].points[0].x, 0.5);
    }

    #[test]
    fn malformed_references_fail_closed() {
        assert_eq!(
            reference_medians(&[], 0),
            Err(ReferenceError::MissingStrokes)
        );
        assert_eq!(
            reference_medians(&[], 1),
            Err(ReferenceError::StrokeCountMismatch)
        );
        assert_eq!(
            reference_medians(&[vec![]], 1),
            Err(ReferenceError::EmptyMedian)
        );
        assert_eq!(
            reference_medians(&[vec![[f64::NAN, 0.0]]], 1),
            Err(ReferenceError::InvalidCoordinate)
        );
        assert_eq!(
            reference_medians(&[vec![[2048.0, 0.0]]], 1),
            Err(ReferenceError::InvalidCoordinate)
        );
    }

    #[test]
    fn asset_ids_use_unicode_scalar_values() {
        assert_eq!(character_asset_id('人'), "4eba");
    }
}
