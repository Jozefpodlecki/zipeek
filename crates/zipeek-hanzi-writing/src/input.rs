use crate::geometry::{GeometryError, Point, Stroke};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    AlreadyDrawing,
    NotDrawing,
    WrongPointer,
    InvalidPoint(GeometryError),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct InkCapture {
    completed: Vec<Stroke>,
    active: Option<(i64, Vec<Point>)>,
}

impl InkCapture {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn strokes(&self) -> &[Stroke] {
        &self.completed
    }
    pub fn active_points(&self) -> Option<&[Point]> {
        self.active.as_ref().map(|(_, points)| points.as_slice())
    }
    pub fn is_drawing(&self) -> bool {
        self.active.is_some()
    }

    pub fn begin(&mut self, pointer: i64, point: Point) -> Result<(), CaptureError> {
        Self::validate_point(point)?;
        if self.active.is_some() {
            return Err(CaptureError::AlreadyDrawing);
        }
        self.active = Some((pointer, vec![point]));
        Ok(())
    }

    pub fn append(&mut self, pointer: i64, point: Point) -> Result<bool, CaptureError> {
        Self::validate_point(point)?;
        let (active_pointer, points) = self.active.as_mut().ok_or(CaptureError::NotDrawing)?;
        if *active_pointer != pointer {
            return Err(CaptureError::WrongPointer);
        }
        if points.last() == Some(&point) {
            return Ok(false);
        }
        points.push(point);
        Ok(true)
    }

    pub fn finish(&mut self, pointer: i64) -> Result<(), CaptureError> {
        let (active_pointer, _) = self.active.as_ref().ok_or(CaptureError::NotDrawing)?;
        if *active_pointer != pointer {
            return Err(CaptureError::WrongPointer);
        }
        if let Some((_, points)) = self.active.take() {
            self.completed.push(Stroke { points });
        }
        Ok(())
    }

    pub fn cancel(&mut self, pointer: i64) -> Result<(), CaptureError> {
        let (active_pointer, _) = self.active.as_ref().ok_or(CaptureError::NotDrawing)?;
        if *active_pointer != pointer {
            return Err(CaptureError::WrongPointer);
        }
        self.active = None;
        Ok(())
    }

    pub fn undo(&mut self) -> bool {
        self.completed.pop().is_some()
    }

    pub fn clear(&mut self) {
        self.active = None;
        self.completed.clear();
    }

    fn validate_point(point: Point) -> Result<(), CaptureError> {
        if point.x.is_finite() && point.y.is_finite() {
            Ok(())
        } else {
            Err(CaptureError::InvalidPoint(
                GeometryError::NonFiniteCoordinate,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y).unwrap()
    }

    #[test]
    fn complete_undo_clear() {
        let mut ink = InkCapture::new();
        ink.begin(4, p(0.0, 0.0)).unwrap();
        assert!(ink.is_drawing());
        assert!(ink.append(4, p(1.0, 2.0)).unwrap());
        assert!(!ink.append(4, p(1.0, 2.0)).unwrap());
        ink.finish(4).unwrap();
        assert_eq!(ink.strokes()[0].points.len(), 2);
        assert!(ink.undo());
        assert!(!ink.undo());
        ink.begin(5, p(1.0, 1.0)).unwrap();
        ink.clear();
        assert!(!ink.is_drawing());
    }

    #[test]
    fn mismatched_pointer_does_not_modify_state() {
        let mut ink = InkCapture::new();
        ink.begin(8, p(0.0, 0.0)).unwrap();
        let snapshot = ink.clone();
        assert_eq!(ink.append(9, p(1.0, 1.0)), Err(CaptureError::WrongPointer));
        assert_eq!(ink.finish(9), Err(CaptureError::WrongPointer));
        assert_eq!(ink.cancel(9), Err(CaptureError::WrongPointer));
        assert_eq!(ink.begin(9, p(2.0, 2.0)), Err(CaptureError::AlreadyDrawing));
        assert_eq!(ink, snapshot);
        ink.cancel(8).unwrap();
        assert!(ink.strokes().is_empty());
    }

    #[test]
    fn missing_active_stroke_is_rejected() {
        let mut ink = InkCapture::new();
        assert_eq!(ink.append(1, p(0.0, 0.0)), Err(CaptureError::NotDrawing));
        assert_eq!(ink.finish(1), Err(CaptureError::NotDrawing));
        assert_eq!(ink.cancel(1), Err(CaptureError::NotDrawing));
    }

    #[test]
    fn invalid_input_is_rejected_without_losing_active_stroke() {
        let mut ink = InkCapture::new();
        ink.begin(1, p(0.0, 0.0)).unwrap();
        assert_eq!(
            ink.append(
                1,
                Point {
                    x: f64::INFINITY,
                    y: 0.0
                }
            ),
            Err(CaptureError::InvalidPoint(
                GeometryError::NonFiniteCoordinate
            ))
        );
        assert_eq!(ink.active_points(), Some([p(0.0, 0.0)].as_slice()));
    }
}
