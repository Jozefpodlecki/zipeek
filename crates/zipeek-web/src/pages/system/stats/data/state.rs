use super::super::sections::DashboardTab;
use super::{DataError, SectionPayload};
use std::{collections::BTreeMap, rc::Rc};
use yew::Reducible;

#[derive(Clone, PartialEq, Eq)]
pub(in super::super) enum SectionLoad {
    Loading { generation: u64 },
    Ready(Rc<SectionPayload>),
    Failed { generation: u64, error: DataError },
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(in super::super) struct StatisticsDataState {
    sections: BTreeMap<&'static str, SectionLoad>,
}

impl StatisticsDataState {
    pub fn get(&self, tab: DashboardTab) -> Option<&SectionLoad> {
        self.sections.get(tab.key())
    }
}

pub(in super::super) enum DataAction {
    Start {
        tab: DashboardTab,
        generation: u64,
    },
    Loaded {
        tab: DashboardTab,
        generation: u64,
        payload: SectionPayload,
    },
    Failed {
        tab: DashboardTab,
        generation: u64,
        error: DataError,
    },
    Invalidate {
        tab: DashboardTab,
        generation: u64,
    },
    Reset,
}

impl Reducible for StatisticsDataState {
    type Action = DataAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            DataAction::Reset => next.sections.clear(),
            DataAction::Start { tab, generation } => {
                next.sections
                    .insert(tab.key(), SectionLoad::Loading { generation });
            }
            DataAction::Loaded {
                tab,
                generation,
                payload,
            } => {
                if !matches!(next.get(tab), Some(SectionLoad::Loading { generation: active }) if *active == generation)
                {
                    return self;
                }
                if payload.tab() != tab {
                    next.sections.insert(
                        tab.key(),
                        SectionLoad::Failed {
                            generation,
                            error: DataError::RejectedProjection,
                        },
                    );
                } else {
                    next.sections
                        .insert(tab.key(), SectionLoad::Ready(Rc::new(payload)));
                }
            }
            DataAction::Failed {
                tab,
                generation,
                error,
            } => {
                if !matches!(next.get(tab), Some(SectionLoad::Loading { generation: active }) if *active == generation)
                {
                    return self;
                }
                next.sections
                    .insert(tab.key(), SectionLoad::Failed { generation, error });
            }
            DataAction::Invalidate { tab, generation } => {
                if matches!(next.get(tab), Some(SectionLoad::Loading { generation: active }) if *active == generation)
                {
                    next.sections.remove(tab.key());
                }
            }
        }
        Rc::new(next)
    }
}
