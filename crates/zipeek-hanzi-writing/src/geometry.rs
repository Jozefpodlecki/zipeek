#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Result<Self, GeometryError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        Ok(Self { x, y })
    }

    pub fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub points: Vec<Point>,
}

impl Stroke {
    pub fn new(points: Vec<Point>) -> Result<Self, GeometryError> {
        if points.is_empty() {
            return Err(GeometryError::EmptyStroke);
        }
        if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        Ok(Self { points })
    }

    pub fn length(&self) -> f64 {
        self.points.windows(2).map(|p| p[0].distance(p[1])).sum()
    }

    pub fn bounds(&self) -> Bounds {
        let first = self.points[0];
        self.points.iter().skip(1).fold(
            Bounds {
                min: first,
                max: first,
            },
            |bounds, p| Bounds {
                min: Point {
                    x: bounds.min.x.min(p.x),
                    y: bounds.min.y.min(p.y),
                },
                max: Point {
                    x: bounds.max.x.max(p.x),
                    y: bounds.max.y.max(p.y),
                },
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Point,
    pub max: Point,
}

impl Bounds {
    pub fn width(self) -> f64 {
        self.max.x - self.min.x
    }
    pub fn height(self) -> f64 {
        self.max.y - self.min.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryError {
    NonFiniteCoordinate,
    InvalidDimensions,
    EmptyStroke,
    InvalidSampleCount,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasTransform {
    pub origin: Point,
    pub css_width: f64,
    pub css_height: f64,
    pub backing_width: f64,
    pub backing_height: f64,
}

impl CanvasTransform {
    pub fn new(
        origin: Point,
        css_width: f64,
        css_height: f64,
        backing_width: f64,
        backing_height: f64,
    ) -> Result<Self, GeometryError> {
        if !origin.x.is_finite()
            || !origin.y.is_finite()
            || [css_width, css_height, backing_width, backing_height]
                .iter()
                .any(|n| !n.is_finite() || *n <= 0.0)
        {
            return Err(GeometryError::InvalidDimensions);
        }
        Ok(Self {
            origin,
            css_width,
            css_height,
            backing_width,
            backing_height,
        })
    }

    pub fn client_to_normalized(self, client: Point) -> Point {
        Point {
            x: ((client.x - self.origin.x) / self.css_width).clamp(0.0, 1.0),
            y: ((client.y - self.origin.y) / self.css_height).clamp(0.0, 1.0),
        }
    }

    pub fn normalized_to_backing(self, point: Point) -> Point {
        Point {
            x: point.x * self.backing_width,
            y: point.y * self.backing_height,
        }
    }
}

pub fn resample(stroke: &Stroke, count: usize) -> Result<Stroke, GeometryError> {
    if count < 2 {
        return Err(GeometryError::InvalidSampleCount);
    }
    let total = stroke.length();
    if total <= f64::EPSILON {
        return Stroke::new(vec![stroke.points[0]; count]);
    }
    let mut output = Vec::with_capacity(count);
    let mut segment = 1;
    let mut traversed = 0.0;
    for i in 0..count {
        let target = total * i as f64 / (count - 1) as f64;
        while segment < stroke.points.len() {
            let segment_length = stroke.points[segment - 1].distance(stroke.points[segment]);
            if traversed + segment_length >= target {
                break;
            }
            traversed += segment_length;
            segment += 1;
        }
        if segment == stroke.points.len() {
            output.push(*stroke.points.last().ok_or(GeometryError::EmptyStroke)?);
        } else {
            let a = stroke.points[segment - 1];
            let b = stroke.points[segment];
            let length = a.distance(b);
            let fraction = if length <= f64::EPSILON {
                0.0
            } else {
                ((target - traversed) / length).clamp(0.0, 1.0)
            };
            output.push(Point {
                x: a.x + (b.x - a.x) * fraction,
                y: a.y + (b.y - a.y) * fraction,
            });
        }
    }
    Stroke::new(output)
}

pub fn normalize(stroke: &Stroke) -> Stroke {
    let bounds = stroke.bounds();
    let scale = bounds.width().max(bounds.height());
    let points = stroke
        .points
        .iter()
        .map(|p| {
            if scale <= f64::EPSILON {
                Point { x: 0.5, y: 0.5 }
            } else {
                Point {
                    x: (p.x - bounds.min.x) / scale,
                    y: (p.y - bounds.min.y) / scale,
                }
            }
        })
        .collect();
    Stroke { points }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> Point {
        Point::new(x, y).unwrap()
    }

    #[test]
    fn transform_respects_css_pixels_and_backing_resolution() {
        let transform =
            CanvasTransform::new(point(100.0, 50.0), 200.0, 100.0, 400.0, 200.0).unwrap();
        let normalized = transform.client_to_normalized(point(150.0, 75.0));
        assert_eq!(normalized, point(0.25, 0.25));
        assert_eq!(
            transform.normalized_to_backing(normalized),
            point(100.0, 50.0)
        );
        assert_eq!(
            transform.client_to_normalized(point(20.0, 999.0)),
            point(0.0, 1.0)
        );
    }

    #[test]
    fn invalid_coordinates_and_dimensions_rejected() {
        assert_eq!(
            Point::new(f64::NAN, 0.0),
            Err(GeometryError::NonFiniteCoordinate)
        );
        assert_eq!(
            CanvasTransform::new(point(0.0, 0.0), 0.0, 10.0, 100.0, 100.0),
            Err(GeometryError::InvalidDimensions)
        );
        assert_eq!(Stroke::new(vec![]), Err(GeometryError::EmptyStroke));
    }

    #[test]
    fn resampling_is_uniform_and_preserves_endpoints() {
        let stroke = Stroke::new(vec![point(0.0, 0.0), point(2.0, 0.0), point(10.0, 0.0)]).unwrap();
        let output = resample(&stroke, 6).unwrap();
        assert_eq!(
            output.points,
            (0..6)
                .map(|x| point((x * 2) as f64, 0.0))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn constant_stroke_normalizes_without_nan() {
        let stroke = Stroke::new(vec![point(7.0, 12.0)]).unwrap();
        assert_eq!(
            normalize(&resample(&stroke, 3).unwrap()).points,
            vec![point(0.5, 0.5); 3]
        );
    }

    #[test]
    fn translation_and_scale_preserve_shape() {
        let a = Stroke::new(vec![point(0.0, 0.0), point(2.0, 2.0)]).unwrap();
        let b = Stroke::new(vec![point(10.0, 20.0), point(14.0, 24.0)]).unwrap();
        assert_eq!(normalize(&a), normalize(&b));
    }
}
