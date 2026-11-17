use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullRunKind {
    Standard,
    Pedagogical,
}

impl FullRunKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Pedagogical => "pedagogical",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JourneyStep {
    pub name: &'static str,
    pub route: Option<&'static str>,
    pub selector: &'static str,
}

const STANDARD: &[JourneyStep] = &[
    JourneyStep {
        name: "landing",
        route: None,
        selector: "[data-component='welcome-landing']",
    },
    JourneyStep {
        name: "onboarding",
        route: None,
        selector: "[data-onboarding-form]",
    },
    JourneyStep {
        name: "dashboard",
        route: Some("/"),
        selector: "[data-component='home-page']",
    },
    JourneyStep {
        name: "practice-first",
        route: Some("/practice"),
        selector: "[data-practice-setup]",
    },
    JourneyStep {
        name: "wordsets",
        route: Some("/word-sets"),
        selector: "[data-component='main-content']",
    },
    JourneyStep {
        name: "dictionary",
        route: Some("/browse"),
        selector: "[data-component='main-content']",
    },
    JourneyStep {
        name: "hanzi-writer",
        route: Some("/write-hanzi"),
        selector: "[data-component='write-hanzi-page']",
    },
    JourneyStep {
        name: "games",
        route: Some("/games"),
        selector: "[data-component='games-gallery']",
    },
    JourneyStep {
        name: "settings",
        route: Some("/settings"),
        selector: "[data-component='settings-page']",
    },
    JourneyStep {
        name: "courses",
        route: Some("/courses"),
        selector: "[data-component='courses-page']",
    },
    JourneyStep {
        name: "practice-final",
        route: Some("/practice"),
        selector: "[data-practice-setup]",
    },
];

const PEDAGOGICAL: &[JourneyStep] = &[
    JourneyStep {
        name: "landing",
        route: None,
        selector: "[data-component='welcome-landing']",
    },
    JourneyStep {
        name: "onboarding",
        route: None,
        selector: "[data-onboarding-form]",
    },
    JourneyStep {
        name: "dashboard",
        route: Some("/"),
        selector: "[data-component='home-page']",
    },
    JourneyStep {
        name: "courses",
        route: Some("/courses"),
        selector: "[data-component='courses-page']",
    },
    JourneyStep {
        name: "practice",
        route: Some("/practice"),
        selector: "[data-practice-setup]",
    },
    JourneyStep {
        name: "hanzi-writer",
        route: Some("/write-hanzi"),
        selector: "[data-component='write-hanzi-page']",
    },
    JourneyStep {
        name: "games",
        route: Some("/games"),
        selector: "[data-component='games-gallery']",
    },
    JourneyStep {
        name: "stats",
        route: Some("/stats"),
        selector: "[data-stats-page]",
    },
];

#[derive(Debug, Clone)]
pub struct FullRunConfig {
    pub kind: FullRunKind,
    pub seed: u64,
    pub max_duration: Duration,
}

impl FullRunConfig {
    pub fn from_environment(kind: FullRunKind) -> Result<Self, String> {
        let seed = std::env::var("ZIPEEK_E2E_SEED")
            .unwrap_or_else(|_| "20261112".to_owned())
            .parse::<u64>()
            .map_err(|_| "ZIPEEK_E2E_SEED must be an unsigned integer")?;
        let minutes = std::env::var("ZIPEEK_E2E_MAX_MINUTES")
            .unwrap_or_else(|_| "60".to_owned())
            .parse::<u64>()
            .map_err(|_| "ZIPEEK_E2E_MAX_MINUTES must be an integer from 1 to 120")?;
        if !(1..=120).contains(&minutes) {
            return Err("ZIPEEK_E2E_MAX_MINUTES must be between 1 and 120".to_owned());
        }
        Ok(Self {
            kind,
            seed,
            max_duration: Duration::from_secs(minutes * 60),
        })
    }

    pub fn theme_index(&self, available: usize) -> usize {
        assert!(available > 0);
        let mut value = self.seed ^ 0x9E37_79B9_7F4A_7C15;
        value ^= value >> 30;
        value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value ^= value >> 27;
        value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^= value >> 31;
        (value % available as u64) as usize
    }
}

pub struct FullRunPlan;
impl FullRunPlan {
    pub const fn steps(kind: FullRunKind) -> &'static [JourneyStep] {
        match kind {
            FullRunKind::Standard => STANDARD,
            FullRunKind::Pedagogical => PEDAGOGICAL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_variants_start_with_onboarding_and_diverge() {
        for kind in [FullRunKind::Standard, FullRunKind::Pedagogical] {
            let steps = FullRunPlan::steps(kind);
            assert_eq!(steps[0].name, "landing");
            assert_eq!(steps[1].name, "onboarding");
            assert_eq!(steps[2].name, "dashboard");
            assert!(steps.iter().all(|step| !step.selector.is_empty()));
        }
        assert_eq!(
            FullRunPlan::steps(FullRunKind::Standard)
                .last()
                .unwrap()
                .name,
            "practice-final"
        );
        assert_eq!(
            FullRunPlan::steps(FullRunKind::Pedagogical)
                .last()
                .unwrap()
                .name,
            "stats"
        );
    }
    #[test]
    fn theme_seed_is_reproducible() {
        let config = FullRunConfig {
            kind: FullRunKind::Standard,
            seed: 42,
            max_duration: Duration::from_secs(60),
        };
        assert_eq!(config.theme_index(6), config.theme_index(6));
        assert!(config.theme_index(6) < 6);
    }
}
