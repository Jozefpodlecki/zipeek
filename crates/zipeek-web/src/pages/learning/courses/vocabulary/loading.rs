use yew::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum LoadingPhase {
    Dictionary,
    Vocabulary,
}

impl LoadingPhase {
    fn label(self) -> &'static str {
        match self {
            Self::Dictionary => "Preparing dictionary",
            Self::Vocabulary => "Loading vocabulary",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct LoadingProps {
    pub phase: LoadingPhase,
}

#[function_component(RegionLoading)]
pub fn region_loading(props: &LoadingProps) -> Html {
    html! {
        <div role="status" aria-label={props.phase.label()} data-vocabulary-loading="details"
            class="space-y-4 rounded-xl border border-line bg-raised p-4">
            <div class="flex items-center gap-3">
                <span aria-hidden="true" class="size-5 shrink-0 rounded-full border-2 border-brand-action border-t-transparent motion-safe:animate-spin"></span>
                <p class="text-sm font-semibold text-ink">{props.phase.label()}</p>
            </div>
            <div aria-hidden="true" class="space-y-3 motion-safe:animate-pulse">
                <div class="flex items-center justify-between gap-3">
                    <div class="h-3 w-24 rounded-full bg-line"></div>
                    <div class="h-3 w-20 rounded-full bg-line"></div>
                </div>
                <div class="h-2 w-full rounded-full bg-line"></div>
                <div class="h-10 w-full rounded-xl bg-line/60"></div>
                <div class="grid grid-cols-2 gap-2">
                    {for (0..4).map(|_| html! { <div class="h-10 rounded-lg bg-line/60"></div> })}
                </div>
            </div>
        </div>
    }
}

#[function_component(ExplorerLoading)]
pub fn explorer_loading(props: &LoadingProps) -> Html {
    html! {
        <div role="status" aria-label={props.phase.label()} data-vocabulary-loading="explorer"
            class="mt-3 grid min-w-0 grid-cols-2 gap-2 xl:grid-cols-4">
            <span class="sr-only">{props.phase.label()}</span>
            {for (0..4).map(|_| html! {
                <div aria-hidden="true" class="min-w-0 space-y-2 rounded-xl border border-line bg-raised p-3 motion-safe:animate-pulse">
                    <div class="flex items-center justify-between gap-2">
                        <div class="h-5 w-16 rounded-md bg-line"></div>
                        <div class="size-5 rounded-full bg-line"></div>
                    </div>
                    <div class="h-2.5 w-14 rounded-full bg-line"></div>
                    <div class="h-2.5 w-full rounded-full bg-line"></div>
                    <div class="h-2.5 w-3/4 rounded-full bg-line"></div>
                </div>
            })}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_phases_have_distinct_accessible_labels() {
        assert_eq!(LoadingPhase::Dictionary.label(), "Preparing dictionary");
        assert_eq!(LoadingPhase::Vocabulary.label(), "Loading vocabulary");
    }
}
