use crate::components::ListenButton;
use super::state::RegionLoad;
use super::loading::{ExplorerLoading, LoadingPhase};
use yew::prelude::*;

pub const PAGE_SIZE: usize = 4;

#[derive(Properties, PartialEq)]
pub struct ExplorerProps {
    pub load: RegionLoad,
    pub page: usize,
    pub on_previous: Callback<MouseEvent>,
    pub on_next: Callback<MouseEvent>,
}

#[function_component(WordExplorer)]
pub fn word_explorer(props: &ExplorerProps) -> Html {
    let (pages, current) = match &props.load {
        RegionLoad::Ready(words) => {
            let pages = words.len().div_ceil(PAGE_SIZE).max(1);
            (pages, props.page.min(pages - 1))
        }
        _ => (1, 0),
    };
    html! {
        <section data-vocabulary-explorer="" aria-label="Vocabulary Explorer"
            class="min-w-0 h-[166px] overflow-hidden rounded-2xl border border-line bg-surface p-3 shadow-lg">
            <div class="flex items-center justify-between gap-2">
                <h3 class="text-sm font-bold text-ink">{"Vocabulary Explorer"}</h3>
                <div class="flex items-center gap-2">
                    <span class="text-xs tabular-nums text-muted">{format!("{}/{}", current + 1, pages)}</span>
                    <button type="button" aria-label="Previous vocabulary page" disabled={current == 0}
                        onclick={props.on_previous.clone()} class="rounded-lg border border-line px-2 py-1 text-sm text-ink disabled:opacity-40">{"‹"}</button>
                    <button type="button" aria-label="Next vocabulary page" disabled={current + 1 >= pages}
                        onclick={props.on_next.clone()} class="rounded-lg border border-line px-2 py-1 text-sm text-ink disabled:opacity-40">{"›"}</button>
                </div>
            </div>
            {match &props.load {
                RegionLoad::WaitingForDictionary => html! { <ExplorerLoading phase={LoadingPhase::Dictionary} /> },
                RegionLoad::Loading => html! { <ExplorerLoading phase={LoadingPhase::Vocabulary} /> },
                RegionLoad::DictionaryUnavailable(_) => html! { <p class="mt-3 flex min-h-[110px] items-center text-sm text-muted">{"Dictionary initialization failed. Retry in the region panel."}</p> },
                RegionLoad::Failed(_) => html! { <p class="mt-3 flex min-h-[110px] items-center text-sm text-muted">{"Words will appear when this region loads. Retry in the region panel."}</p> },
                RegionLoad::Ready(words) if words.is_empty() => html! { <p class="mt-3 flex min-h-[110px] items-center text-sm text-muted">{"No words available."}</p> },
                RegionLoad::Ready(words) => html! {
                    <div class="mt-3 grid h-[106px] min-w-0 grid-cols-2 gap-2 xl:grid-cols-4">
                        {for words.iter().skip(current * PAGE_SIZE).take(PAGE_SIZE).map(|word| html! {
                            <article key={word.id.as_i32()} class="min-h-0 min-w-0 overflow-hidden rounded-xl border border-line bg-raised p-2">
                                <div class="flex items-start justify-between gap-1">
                                    <p class="min-w-0 truncate text-lg font-semibold text-ink" title={word.simplified.clone()}>{word.simplified.clone()}</p>
                                    <ListenButton hanzi={word.simplified.clone()} compact=true />
                                </div>
                                <p class="truncate text-xs text-muted" title={word.pinyin.clone()}>{word.pinyin.clone()}</p>
                                <p class="line-clamp-2 min-h-8 break-words text-xs leading-4 text-muted" title={word.english.clone().unwrap_or_default()}>{word.english.clone().unwrap_or_else(|| "Meaning unavailable".to_owned())}</p>
                            </article>
                        })}
                    </div>
                },
            }}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explorer_page_size_is_bounded() {
        assert_eq!(PAGE_SIZE, 4);
    }
}
