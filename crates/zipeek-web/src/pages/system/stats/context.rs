use yew::prelude::*;

use super::state::DashboardState;

#[derive(Clone, PartialEq)]
pub(super) struct StatsUiContext(pub UseReducerHandle<DashboardState>);

#[hook]
pub(super) fn use_stats_ui() -> StatsUiContext {
    use_context::<StatsUiContext>().expect("Stats UI must be nested under its route provider")
}

use std::rc::Rc;

use super::data::{SectionPayload, StatisticsDataState, StatisticsSource};

#[derive(Clone, PartialEq)]
pub(super) struct StatsDataContext(pub UseReducerHandle<StatisticsDataState>);

#[derive(Clone)]
pub(super) struct StatsSourceContext(pub Rc<dyn StatisticsSource>);

impl PartialEq for StatsSourceContext {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct LoadedSectionContext(pub Rc<SectionPayload>);

#[hook]
pub(super) fn use_loaded_section() -> Rc<SectionPayload> {
    use_context::<LoadedSectionContext>()
        .expect("Statistics section must be rendered beneath a loaded projection provider")
        .0
}
