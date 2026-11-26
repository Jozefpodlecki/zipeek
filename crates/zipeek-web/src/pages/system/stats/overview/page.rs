use super::super::{
    context::use_loaded_section,
    data::SectionPayload,
    sections::OverviewCards,
    state::MetricSelection,
    tiles::{ExtraTiles, TileSelection},
};
use super::chart::ActivityChart;
use super::review::ReviewList;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct OverviewProps {
    pub metrics: MetricSelection,
    pub tiles: TileSelection,
    pub on_vocabulary: Callback<()>,
}

#[function_component(Overview)]
pub(crate) fn overview(props: &OverviewProps) -> Html {
    let on_vocabulary = props.on_vocabulary.clone();
    let open_vocabulary = Callback::from(move |_: MouseEvent| on_vocabulary.emit(()));
    let loaded = use_loaded_section();
    let SectionPayload::Overview { words, .. } = loaded.as_ref() else {
        unreachable!("Overview projection expected")
    };
    html! {
        <div class="space-y-3" data-stats-overview="">
            <OverviewCards words={props.metrics.words} encounters={props.metrics.encounters} sessions={props.metrics.sessions} assessment={props.metrics.coverage} />
            <div class="grid items-stretch gap-3 lg:grid-cols-[minmax(0,3fr)_minmax(16rem,2fr)]">
                <ActivityChart />
                <ReviewList words={words.clone()} on_vocabulary={open_vocabulary} />
            </div>
            <ExtraTiles selection={props.tiles.clone()} />
        </div>
    }
}
