use std::{fmt, fs, path::PathBuf};
use anyhow::{Context, Result};
use hashbrown::HashSet;
use serde::Deserialize;

use crate::ReferenceStandard;

#[derive(Debug, Deserialize, Clone)]
pub struct Entry {
    #[serde(rename = "s")]
    pub simplified: Box<str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hsk20Level {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

impl Hsk20Level {
    pub fn to_standard(self) -> ReferenceStandard {
        ReferenceStandard {
            name: "hsk20".into(),
            kind: "level".into(),
            value: self.to_string().into(),
        }
    }
}

impl fmt::Display for Hsk20Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::One => write!(f, "1"),
            Self::Two => write!(f, "2"),
            Self::Three => write!(f, "3"),
            Self::Four => write!(f, "4"),
            Self::Five => write!(f, "5"),
            Self::Six => write!(f, "6"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hsk30Level {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    SevenEightNine,
}

impl fmt::Display for Hsk30Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::One => write!(f, "1"),
            Self::Two => write!(f, "2"),
            Self::Three => write!(f, "3"),
            Self::Four => write!(f, "4"),
            Self::Five => write!(f, "5"),
            Self::Six => write!(f, "6"),
            Self::SevenEightNine => write!(f, "7-9"),
        }
    }
}

impl Hsk30Level {
    pub fn to_standard(self) -> ReferenceStandard {
        ReferenceStandard {
            name: "hsk30".into(),
            kind: "level".into(),
            value: self.to_string().into(),
        }
    }
}

pub struct Hsk20 {
    one: HashSet<Box<str>>,
    two: HashSet<Box<str>>,
    three: HashSet<Box<str>>,
    four: HashSet<Box<str>>,
    five: HashSet<Box<str>>,
    six: HashSet<Box<str>>,
}

impl Hsk20 {
    pub fn new() -> Result<Self> {
        let base_path = r#"C:\repos\zipeek\crates\dict-builder\data\hsk2.0"#;
        
        Ok(Self {
            one: load_level(base_path, "1")?,
            two: load_level(base_path, "2")?,
            three: load_level(base_path, "3")?,
            four: load_level(base_path, "4")?,
            five: load_level(base_path, "5")?,
            six: load_level(base_path, "6")?,
        })
    }
    
    pub fn get_level(&self, simplified: &str) -> Option<Hsk20Level> {
        if self.one.contains(simplified) {
            return Some(Hsk20Level::One);
        }
        if self.two.contains(simplified) {
            return Some(Hsk20Level::Two);
        }
        if self.three.contains(simplified) {
            return Some(Hsk20Level::Three);
        }
        if self.four.contains(simplified) {
            return Some(Hsk20Level::Four);
        }
        if self.five.contains(simplified) {
            return Some(Hsk20Level::Five);
        }
        if self.six.contains(simplified) {
            return Some(Hsk20Level::Six);
        }
        None
    }
}

pub struct Hsk30 {
    one: HashSet<Box<str>>,
    two: HashSet<Box<str>>,
    three: HashSet<Box<str>>,
    four: HashSet<Box<str>>,
    five: HashSet<Box<str>>,
    six: HashSet<Box<str>>,
    seven_eight_nine: HashSet<Box<str>>,
}

impl Hsk30 {
    pub fn new() -> Result<Self> {
        let base_path = r#"C:\repos\zipeek\crates\dict-builder\data\hsk3.0"#;
        
        Ok(Self {
            one: load_level(base_path, "1")?,
            two: load_level(base_path, "2")?,
            three: load_level(base_path, "3")?,
            four: load_level(base_path, "4")?,
            five: load_level(base_path, "5")?,
            six: load_level(base_path, "6")?,
            seven_eight_nine: load_level(base_path, "7")?,
        })
    }

    pub fn get_level(&self, simplified: &str) -> Option<Hsk30Level> {
        if self.one.contains(simplified) {
            return Some(Hsk30Level::One);
        }
        if self.two.contains(simplified) {
            return Some(Hsk30Level::Two);
        }
        if self.three.contains(simplified) {
            return Some(Hsk30Level::Three);
        }
        if self.four.contains(simplified) {
            return Some(Hsk30Level::Four);
        }
        if self.five.contains(simplified) {
            return Some(Hsk30Level::Five);
        }
        if self.six.contains(simplified) {
            return Some(Hsk30Level::Six);
        }
        if self.seven_eight_nine.contains(simplified) {
            return Some(Hsk30Level::SevenEightNine);
        }
        None
    }
}

fn load_level(base_path: &str, level: &str) -> Result<HashSet<Box<str>>> {
    let path = PathBuf::from(base_path).join(format!("{}.min.json", level));
    let buffer = fs::read(&path)
        .with_context(|| format!("Failed to read file: {:?}", path))?;
    let entries: Vec<Entry> = serde_json::from_slice(&buffer)
        .with_context(|| format!("Failed to parse JSON from: {:?}", path))?;
    Ok(entries.into_iter().map(|entry| entry.simplified).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_load_hsk20() {
        let set = Hsk20::new().unwrap();

        assert!(set.get_level("的").is_some());
    }
    
    #[test]
    fn should_load_hsk30() {
        let set = Hsk30::new().unwrap();

        assert!(set.get_level("的").is_some());
    }
}