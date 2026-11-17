use super::MapRegion;

pub fn zoom_view_box(region: &MapRegion) -> Option<String> {
    let [x, y, width, height] = region.bbox;
    if ![x, y, width, height].iter().all(|value| value.is_finite()) || width <= 0.0 || height <= 0.0
    {
        return None;
    }
    let margin = width.max(height) * 0.08;
    Some(format!(
        "{} {} {} {}",
        x - margin,
        y - margin,
        width + 2.0 * margin,
        height + 2.0 * margin
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::province_quest::RegionKind;

    fn region(bbox: [f64; 4]) -> MapRegion {
        MapRegion {
            id: "1".into(),
            parent_id: None,
            hanzi: "云南".into(),
            pinyin: "Yúnnán".into(),
            english: "Yunnan".into(),
            capital_hanzi: None,
            capital_pinyin: None,
            kind: RegionKind::Province,
            path: "M0,0Z".into(),
            bbox,
        }
    }

    #[test]
    fn zoom_expands_bounds_without_cutting_edges() {
        assert_eq!(
            zoom_view_box(&region([10.0, 20.0, 100.0, 50.0])),
            Some("2 12 116 66".to_owned())
        );
    }

    #[test]
    fn invalid_bbox_cannot_be_used_in_svg() {
        assert_eq!(
            zoom_view_box(&region([0.0, 0.0, f64::INFINITY, 10.0])),
            None
        );
        assert_eq!(zoom_view_box(&region([0.0, 0.0, -3.0, 10.0])), None);
    }
}
