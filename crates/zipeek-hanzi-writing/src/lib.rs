#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

pub mod characters;
pub mod comparison;
pub mod geometry;
pub mod input;

pub use characters::hanzi_characters;
pub use comparison::{ComparisonError, StrokeComparison, compare_strokes};
pub use geometry::{Bounds, CanvasTransform, GeometryError, Point, Stroke, normalize, resample};
pub use input::{CaptureError, InkCapture};
pub mod reference;
pub use reference::{ReferenceError, character_asset_id, reference_medians, reference_point};

mod elastic;
pub mod matching;
pub mod session;
pub use matching::{MatchConfig, StrokeRejection, StrokeScore, match_stroke};
pub use session::{StrokeOutcome, WritingSession};
