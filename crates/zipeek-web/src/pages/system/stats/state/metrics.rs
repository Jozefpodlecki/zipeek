#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MetricId {
    Words,
    Encounters,
    Sessions,
    Coverage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MetricSelection {
    pub words: bool,
    pub encounters: bool,
    pub sessions: bool,
    pub coverage: bool,
}

impl Default for MetricSelection {
    fn default() -> Self {
        Self {
            words: true,
            encounters: true,
            sessions: false,
            coverage: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DashboardPreset {
    Focused,
    Activity,
    Detailed,
}

impl DashboardPreset {
    pub const fn metrics(self) -> MetricSelection {
        match self {
            Self::Focused => MetricSelection {
                words: true,
                encounters: true,
                sessions: false,
                coverage: false,
            },
            Self::Activity => MetricSelection {
                words: true,
                encounters: false,
                sessions: true,
                coverage: false,
            },
            Self::Detailed => MetricSelection {
                words: true,
                encounters: true,
                sessions: true,
                coverage: true,
            },
        }
    }
}

pub(crate) const PAGE_SIZES: [usize; 4] = [10, 25, 50, 100];
pub(crate) const MAX_FILTERED_SELECTION: usize = 250;

impl MetricSelection {
    pub fn is_enabled(self, metric: MetricId) -> bool {
        match metric {
            MetricId::Words => self.words,
            MetricId::Encounters => self.encounters,
            MetricId::Sessions => self.sessions,
            MetricId::Coverage => self.coverage,
        }
    }

    pub(crate) fn toggle(&mut self, metric: MetricId) {
        let value = match metric {
            MetricId::Words => &mut self.words,
            MetricId::Encounters => &mut self.encounters,
            MetricId::Sessions => &mut self.sessions,
            MetricId::Coverage => &mut self.coverage,
        };
        *value = !*value;
    }
}
