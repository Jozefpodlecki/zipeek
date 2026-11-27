use super::super::tessellation::{self, Point};

const VIEW_WIDTH: f64 = tessellation::WIDTH;
const VIEW_HEIGHT: f64 = tessellation::HEIGHT;
const REGION_COUNT: usize = 100;
const MIN_EDGE: f64 = 36.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn partition(indices: &[usize], weights: &[u64], bounds: Rect, result: &mut [Rect]) {
    if indices.len() == 1 {
        result[indices[0]] = bounds;
        return;
    }

    let total: u64 = indices.iter().map(|&index| weights[index]).sum();
    let mut accumulated = 0_u64;
    let mut split = 1;
    let mut closest = u64::MAX;
    for position in 1..indices.len() {
        accumulated += weights[indices[position - 1]];
        let distance = accumulated.abs_diff(total - accumulated);
        if distance < closest {
            closest = distance;
            split = position;
        }
    }

    let (first, second) = indices.split_at(split);
    let first_weight: u64 = first.iter().map(|&index| weights[index]).sum();
    let fraction = first_weight as f64 / total as f64;
    if bounds.width >= bounds.height {
        let width = bounds.width * fraction;
        partition(first, weights, Rect { width, ..bounds }, result);
        partition(
            second,
            weights,
            Rect {
                x: bounds.x + width,
                width: bounds.width - width,
                ..bounds
            },
            result,
        );
    } else {
        let height = bounds.height * fraction;
        partition(first, weights, Rect { height, ..bounds }, result);
        partition(
            second,
            weights,
            Rect {
                y: bounds.y + height,
                height: bounds.height - height,
                ..bounds
            },
            result,
        );
    }
}

fn scattered_order() -> Vec<usize> {
    let mut indices: Vec<usize> = (0..REGION_COUNT).collect();
    indices.sort_by_key(|&index| {
        let value = index as u32 + 1;
        (
            value.wrapping_mul(0x9e37_79b1) ^ (index as u32).wrapping_mul(0x85eb_ca6b),
            index,
        )
    });
    indices
}

fn base_layout(reading_counts: &[usize; REGION_COUNT]) -> [Rect; REGION_COUNT] {
    assert!(reading_counts.iter().all(|&count| count > 0));
    let indices = scattered_order();
    for floor in (110_u64..=400).step_by(10) {
        let weights =
            reading_counts.map(|count| (((count as f64).sqrt() * 12.0).round() as u64).max(floor));
        let mut result = [Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }; REGION_COUNT];
        partition(
            &indices,
            &weights,
            Rect {
                x: 0.0,
                y: 0.0,
                width: VIEW_WIDTH,
                height: VIEW_HEIGHT,
            },
            &mut result,
        );
        if result
            .iter()
            .all(|rect| rect.width >= MIN_EDGE && rect.height >= MIN_EDGE)
        {
            return result;
        }
    }
    panic!("Vocabulary region layout cannot satisfy minimum region size");
}

fn sites(rectangles: &[Rect; REGION_COUNT]) -> [Point; REGION_COUNT] {
    std::array::from_fn(|index| {
        let rect = rectangles[index];
        let phase = index as f64 + 1.0;
        let dx = ((phase * 12.9898).sin() * 43758.5453).rem_euclid(1.0) - 0.5;
        let dy = ((phase * 78.233).sin() * 13489.213).rem_euclid(1.0) - 0.5;
        Point {
            x: rect.x + rect.width * (0.5 + dx * 0.25),
            y: rect.y + rect.height * (0.5 + dy * 0.25),
        }
    })
}

fn polygons(reading_counts: &[usize; REGION_COUNT]) -> Vec<Vec<Point>> {
    let seeds = sites(&base_layout(reading_counts));
    tessellation::voronoi(&seeds)
}

pub(super) fn region_paths(reading_counts: &[usize; REGION_COUNT]) -> Vec<String> {
    polygons(reading_counts)
        .iter()
        .map(|polygon| tessellation::curved_path(polygon))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use zipeek_pedagogy::curriculum::VOCABULARY_REGIONS;

    fn counts() -> [usize; REGION_COUNT] {
        VOCABULARY_REGIONS.map(|region| region.readings.len())
    }

    fn polygon_area(polygon: &[Point]) -> f64 {
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
    fn every_region_is_selectable_and_has_a_minimum_bounding_size() {
        for polygon in polygons(&counts()) {
            assert!(polygon.len() >= 3);
            let min_x = polygon
                .iter()
                .map(|point| point.x)
                .fold(f64::INFINITY, f64::min);
            let max_x = polygon
                .iter()
                .map(|point| point.x)
                .fold(f64::NEG_INFINITY, f64::max);
            let min_y = polygon
                .iter()
                .map(|point| point.y)
                .fold(f64::INFINITY, f64::min);
            let max_y = polygon
                .iter()
                .map(|point| point.y)
                .fold(f64::NEG_INFINITY, f64::max);
            assert!(max_x - min_x >= MIN_EDGE);
            assert!(max_y - min_y >= MIN_EDGE);
        }
    }

    #[test]
    fn region_area_reflects_reading_count_without_extreme_sizes() {
        let counts = counts();
        let regions = polygons(&counts);
        let small = counts.iter().position(|&count| count == 11).unwrap();
        let large = counts.iter().position(|&count| count == 2200).unwrap();
        assert!(polygon_area(&regions[small]) < polygon_area(&regions[large]));
        assert!(polygon_area(&regions[large]) / polygon_area(&regions[small]) > 3.0);
    }

    #[test]
    fn larger_regions_are_scattered_across_the_canvas() {
        let counts = counts();
        let seeds = sites(&base_layout(&counts));
        let large: Vec<_> = seeds
            .iter()
            .zip(counts.iter())
            .filter(|&(_, &count)| count >= 200)
            .map(|(site, _)| site.x)
            .collect();
        assert!(large.iter().any(|&x| x < VIEW_WIDTH / 3.0));
        assert!(large.iter().any(|&x| x > VIEW_WIDTH * 2.0 / 3.0));
    }

    #[test]
    fn irregular_regions_cover_the_canvas() {
        let regions = polygons(&counts());
        let area: f64 = regions.iter().map(|polygon| polygon_area(polygon)).sum();
        assert!((area - VIEW_WIDTH * VIEW_HEIGHT).abs() < 1e-4);
        assert!(regions.iter().all(|polygon| {
            polygon.iter().all(|point| {
                point.x >= -1e-5
                    && point.x <= VIEW_WIDTH + 1e-5
                    && point.y >= -1e-5
                    && point.y <= VIEW_HEIGHT + 1e-5
            })
        }));
        assert!(regions.iter().any(|polygon| polygon.len() >= 5));
    }

    #[test]
    fn paths_are_deterministic_and_keep_region_identity() {
        let counts = counts();
        let first = region_paths(&counts);
        assert_eq!(first, region_paths(&counts));
        assert_eq!(first.len(), REGION_COUNT);
        assert_eq!(
            first
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            REGION_COUNT
        );
        assert!(
            first
                .iter()
                .all(|path| path.starts_with("M ") && path.ends_with(" Z"))
        );
    }
}
