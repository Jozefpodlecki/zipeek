use super::super::tessellation::{self, Point};

pub const COLUMNS: usize = 10;
pub const ROWS: usize = 10;
pub const GRAMMAR_COUNT: usize = COLUMNS * ROWS;
pub const VIEW_WIDTH: f64 = tessellation::WIDTH;
pub const VIEW_HEIGHT: f64 = tessellation::HEIGHT;

#[derive(Debug, Clone)]
pub struct Region {
    pub id: String,
    pub path: String,
    pub order: usize,
}

fn mix(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}

fn jitter(row: usize, col: usize, salt: u32) -> f64 {
    (mix(row as u32 * 131 + col as u32 * 17 + salt) % 2001) as f64 / 1000.0 - 1.0
}

fn sites() -> Vec<Point> {
    let mut points = Vec::with_capacity(GRAMMAR_COUNT);
    points.push(Point {
        x: VIEW_WIDTH / 2.0,
        y: VIEW_HEIGHT / 2.0,
    });
    for row in 0..ROWS {
        for col in 0..COLUMNS {
            if row == 4 && col == 4 {
                continue;
            }
            points.push(Point {
                x: (col as f64 + 0.5) * VIEW_WIDTH / COLUMNS as f64
                    + 11.0 * jitter(row, col, 7),
                y: (row as f64 + 0.5) * VIEW_HEIGHT / ROWS as f64
                    + 7.0 * jitter(row, col, 11),
            });
        }
    }
    points
}

pub fn regions() -> Vec<Region> {
    tessellation::voronoi(&sites())
        .iter()
        .enumerate()
        .map(|(index, polygon)| Region {
            id: format!("grammar-{}", index + 1),
            order: index + 1,
            path: tessellation::curved_path(polygon),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn area(polygon: &[Point]) -> f64 {
        polygon
            .iter()
            .zip(polygon.iter().cycle().skip(1))
            .take(polygon.len())
            .map(|(a, b)| a.x * b.y - b.x * a.y)
            .sum::<f64>()
            .abs()
            / 2.0
    }

    #[test]
    fn exactly_one_hundred_grammar_only_regions() {
        let items = regions();
        assert_eq!(items.len(), GRAMMAR_COUNT);
        assert_eq!(items[0].id, "grammar-1");
        assert_eq!(items.iter().map(|r| &r.id).collect::<BTreeSet<_>>().len(), 100);
        assert!(items.iter().all(|r| r.path.contains(" Q ") && r.path.ends_with(" Z")));
    }

    #[test]
    fn regions_are_distinct_and_cover_canvas_without_gaps() {
        let polygons = tessellation::voronoi(&sites());
        assert_eq!(polygons.len(), GRAMMAR_COUNT);
        assert!(polygons.iter().all(|polygon| polygon.len() >= 3 && area(polygon) > 100.0));
        let sum = polygons.iter().map(|polygon| area(polygon)).sum::<f64>();
        assert!((sum - VIEW_WIDTH * VIEW_HEIGHT).abs() < 1e-4);
    }

    #[test]
    fn center_course_stays_at_center() {
        let points = sites();
        assert_eq!(points[0], Point { x: 500.0, y: 300.0 });
        let center_polygon = &tessellation::voronoi(&points)[0];
        assert!(center_polygon.iter().any(|p| p.x < 500.0 && p.y < 300.0));
        assert!(center_polygon.iter().any(|p| p.x > 500.0 && p.y > 300.0));
    }

    #[test]
    fn deterministic_geometry() {
        assert_eq!(
            regions().into_iter().map(|r| r.path).collect::<Vec<_>>(),
            regions().into_iter().map(|r| r.path).collect::<Vec<_>>()
        );
    }
}
