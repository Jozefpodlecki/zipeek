use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HskSystem {
    Hsk2,
    Hsk3,
}

impl HskSystem {
    pub const ALL: [Self; 2] = [Self::Hsk2, Self::Hsk3];

    pub const fn number(self) -> u8 {
        match self {
            Self::Hsk2 => 2,
            Self::Hsk3 => 3,
        }
    }

    pub const fn from_number(number: u8) -> Option<Self> {
        match number {
            2 => Some(Self::Hsk2),
            3 => Some(Self::Hsk3),
            _ => None,
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Hsk2 => "2",
            Self::Hsk3 => "3",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "2" => Some(Self::Hsk2),
            "3" => Some(Self::Hsk3),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Hsk2 => "HSK 2.0",
            Self::Hsk3 => "HSK 3.0",
        }
    }

    pub const fn levels(self) -> &'static [HskLevel] {
        match self {
            Self::Hsk2 => &[
                HskLevel::Hsk2(Hsk2Level::One),
                HskLevel::Hsk2(Hsk2Level::Two),
                HskLevel::Hsk2(Hsk2Level::Three),
                HskLevel::Hsk2(Hsk2Level::Four),
                HskLevel::Hsk2(Hsk2Level::Five),
                HskLevel::Hsk2(Hsk2Level::Six),
            ],
            Self::Hsk3 => &[
                HskLevel::Hsk3(Hsk3Level::One),
                HskLevel::Hsk3(Hsk3Level::Two),
                HskLevel::Hsk3(Hsk3Level::Three),
                HskLevel::Hsk3(Hsk3Level::Four),
                HskLevel::Hsk3(Hsk3Level::Five),
                HskLevel::Hsk3(Hsk3Level::Six),
                HskLevel::Hsk3(Hsk3Level::SevenToNine),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HskLevelError {
    pub system: HskSystem,
    pub value: i32,
}

impl fmt::Display for HskLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid {:?} level: {}", self.system, self.value)
    }
}
impl Error for HskLevelError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum Hsk2Level {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
}
impl Hsk2Level {
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}
impl TryFrom<i32> for Hsk2Level {
    type Error = HskLevelError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            _ => Err(HskLevelError {
                system: HskSystem::Hsk2,
                value,
            }),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum Hsk3Level {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    SevenToNine = 7,
}
impl Hsk3Level {
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}
impl TryFrom<i32> for Hsk3Level {
    type Error = HskLevelError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::SevenToNine),
            _ => Err(HskLevelError {
                system: HskSystem::Hsk3,
                value,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HskLevel {
    Hsk2(Hsk2Level),
    Hsk3(Hsk3Level),
}
impl HskLevel {
    pub const fn system(self) -> HskSystem {
        match self {
            Self::Hsk2(_) => HskSystem::Hsk2,
            Self::Hsk3(_) => HskSystem::Hsk3,
        }
    }
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Hsk2(level) => level.as_i32(),
            Self::Hsk3(level) => level.as_i32(),
        }
    }

    pub const fn band_label(self) -> &'static str {
        match self {
            Self::Hsk2(Hsk2Level::One) | Self::Hsk3(Hsk3Level::One) => "1",
            Self::Hsk2(Hsk2Level::Two) | Self::Hsk3(Hsk3Level::Two) => "2",
            Self::Hsk2(Hsk2Level::Three) | Self::Hsk3(Hsk3Level::Three) => "3",
            Self::Hsk2(Hsk2Level::Four) | Self::Hsk3(Hsk3Level::Four) => "4",
            Self::Hsk2(Hsk2Level::Five) | Self::Hsk3(Hsk3Level::Five) => "5",
            Self::Hsk2(Hsk2Level::Six) | Self::Hsk3(Hsk3Level::Six) => "6",
            Self::Hsk3(Hsk3Level::SevenToNine) => "7–9",
        }
    }

    pub fn label(self) -> String {
        format!("HSK {}", self.band_label())
    }

    pub fn badge_label(self) -> String {
        format!("{} · {}", self.system().label(), self.band_label())
    }
}
impl TryFrom<(HskSystem, i32)> for HskLevel {
    type Error = HskLevelError;

    fn try_from((system, value): (HskSystem, i32)) -> Result<Self, Self::Error> {
        match system {
            HskSystem::Hsk2 => Hsk2Level::try_from(value).map(Self::Hsk2),
            HskSystem::Hsk3 => Hsk3Level::try_from(value).map(Self::Hsk3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_documented_levels_are_representable() {
        for value in -1..=10 {
            assert_eq!(Hsk2Level::try_from(value).is_ok(), (1..=6).contains(&value));
            assert_eq!(Hsk3Level::try_from(value).is_ok(), (1..=7).contains(&value));
            if let Ok(level) = Hsk3Level::try_from(value) {
                assert_eq!(level.as_i32(), value);
            }
            if let Ok(level) = Hsk2Level::try_from(value) {
                assert_eq!(level.as_i32(), value);
            }
        }
        assert_eq!(Hsk3Level::try_from(7), Ok(Hsk3Level::SevenToNine));
    }
    #[test]
    fn system_metadata_is_consistent_with_level_domains() {
        assert_eq!(HskSystem::ALL, [HskSystem::Hsk2, HskSystem::Hsk3]);
        assert_eq!(HskSystem::Hsk2.label(), "HSK 2.0");
        assert_eq!(HskSystem::Hsk3.label(), "HSK 3.0");
        for system in HskSystem::ALL {
            assert_eq!(HskSystem::from_code(system.code()), Some(system));
            assert_eq!(HskSystem::from_number(system.number()), Some(system));
            let levels = system.levels();
            assert_eq!(levels.len(), if system == HskSystem::Hsk2 { 6 } else { 7 });
            for (index, level) in levels.iter().enumerate() {
                assert_eq!(level.system(), system);
                assert_eq!(level.as_i32(), index as i32 + 1);
                assert_eq!(HskLevel::try_from((system, index as i32 + 1)), Ok(*level));
                assert!(level.badge_label().starts_with(system.label()));
            }
        }
        assert_eq!(HskSystem::from_code("4"), None);
        assert_eq!(HskSystem::from_number(4), None);
        assert!(HskLevel::try_from((HskSystem::Hsk2, 7)).is_err());
        assert_eq!(HskSystem::Hsk3.levels()[6].label(), "HSK 7–9");
        assert_eq!(HskSystem::Hsk3.levels()[6].badge_label(), "HSK 3.0 · 7–9");
    }

    #[test]
    fn level_carries_its_system_without_conversion() {
        let old = HskLevel::Hsk2(Hsk2Level::Two);
        let new = HskLevel::Hsk3(Hsk3Level::Three);
        assert_eq!((old.system(), old.as_i32()), (HskSystem::Hsk2, 2));
        assert_eq!((new.system(), new.as_i32()), (HskSystem::Hsk3, 3));
    }
}
