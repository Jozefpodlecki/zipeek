use crate::{Result, geojson::Geometry};
use serde_json::Value;

const WIDTH: f64 = 1000.0;
const HEIGHT: f64 = 680.0;
const LON_MIN: f64 = 72.0;
const LON_MAX: f64 = 136.0;
const LAT_MIN: f64 = 16.0;
const LAT_MAX: f64 = 55.0;

pub(crate) fn projected(point: &Value) -> Result<(f64, f64)> {
    let coordinates = point
        .as_array()
        .ok_or_else(|| format!("Invalid geographic coordinate {point}"))?;
    let lon = coordinates.first().and_then(Value::as_f64);
    let lat = coordinates.get(1).and_then(Value::as_f64);
    let (Some(lon), Some(lat)) = (lon, lat) else {
        return Err(format!("Invalid geographic coordinate {point}"));
    };
    if !lon.is_finite()
        || !lat.is_finite()
        || !(-180.0..=180.0).contains(&lon)
        || !(-90.0..=90.0).contains(&lat)
    {
        return Err(format!("Invalid geographic coordinate {point}"));
    }
    Ok((
        (lon - LON_MIN) / (LON_MAX - LON_MIN) * WIDTH,
        (LAT_MAX - lat) / (LAT_MAX - LAT_MIN) * HEIGHT,
    ))
}

pub(crate) fn geometry_path(geometry: &Geometry) -> Result<(String, [f64; 4])> {
    let polygons = match geometry.kind.as_str() {
        "Polygon" => vec![&geometry.coordinates],
        "MultiPolygon" => geometry
            .coordinates
            .as_array()
            .ok_or("Invalid MultiPolygon coordinates")?
            .iter()
            .collect(),
        kind => return Err(format!("Unsupported geometry type {kind}")),
    };
    let mut path = String::new();
    let mut bounds = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    let mut points = 0;
    for polygon in polygons {
        let rings = polygon.as_array().ok_or("Invalid polygon rings")?;
        for ring in rings {
            let coordinates = ring.as_array().ok_or("Invalid boundary ring")?;
            if coordinates.len() < 4 {
                return Err("A boundary ring requires at least four points".to_owned());
            }
            path.push('M');
            for (index, point) in coordinates.iter().enumerate() {
                let (x, y) = projected(point)?;
                if index > 0 {
                    path.push('L');
                }
                path.push_str(&format!("{x:.2},{y:.2}"));
                bounds[0] = bounds[0].min(x);
                bounds[1] = bounds[1].min(y);
                bounds[2] = bounds[2].max(x);
                bounds[3] = bounds[3].max(y);
                points += 1;
            }
            path.push('Z');
        }
    }
    if points == 0 {
        return Err("Boundary geometry has no points".to_owned());
    }
    Ok((
        path,
        [
            round2(bounds[0]),
            round2(bounds[1]),
            round2(bounds[2] - bounds[0]),
            round2(bounds[3] - bounds[1]),
        ],
    ))
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
