use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HskSystem {
    Hsk2,
    Hsk3,
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
    fn level_carries_its_system_without_conversion() {
        let old = HskLevel::Hsk2(Hsk2Level::Two);
        let new = HskLevel::Hsk3(Hsk3Level::Three);
        assert_eq!((old.system(), old.as_i32()), (HskSystem::Hsk2, 2));
        assert_eq!((new.system(), new.as_i32()), (HskSystem::Hsk3, 3));
    }
}
