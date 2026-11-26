use super::super::{data::WordRecords, fixture::EvidenceStatus};
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct ReviewListProps {
    pub words: WordRecords,
    pub on_vocabulary: Callback<MouseEvent>,
}

#[function_component(ReviewList)]
pub(super) fn review_list(props: &ReviewListProps) -> Html {
    let review = props
        .words
        .iter()
        .filter(|record| {
            [
                record.recognition,
                record.listening,
                record.speaking,
                record.writing,
                record.sentence,
            ]
            .contains(&EvidenceStatus::NeedsReview)
        })
        .take(4)
        .collect::<Vec<_>>();
    let open_vocabulary = props.on_vocabulary.clone();
    html! {
                <section class="flex h-full min-w-0 flex-col rounded-xl border border-line bg-surface p-4" aria-label="Items for review">
                    <div class="flex items-center justify-between gap-2">
                        <h2 class="font-semibold">{"To review"}</h2>
                        <button type="button" class="rounded-md p-2 text-accent hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent" aria-label="Browse vocabulary" onclick={open_vocabulary}>
                            <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="18px" height="18px" />
                        </button>
                    </div>
                    <div class="mt-2 flex-1 divide-y divide-line">
                        {for review.iter().map(|record| html! {
                            <div key={record.id.clone()} class="flex items-center justify-between gap-3 py-2">
                                <div><span lang="zh-Hans" class="font-semibold">{record.hanzi.clone()}</span>
                                    <span class="ml-2 text-xs text-muted">{record.pinyin.clone()}</span>
                                </div>
                                <span class="text-xs text-muted">{"Review"}</span>
                            </div>
                        })}
                    </div>
                </section>
    }
}
