use crate::services::word_sets::UserWordSet;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(super) struct VocabularyPickerProps {
    pub path: &'static str,
    pub sets: Vec<UserWordSet>,
}

#[function_component(VocabularyPicker)]
pub(super) fn vocabulary_picker(props: &VocabularyPickerProps) -> Html {
    html! {
        <nav aria-label="Game vocabulary" class="grid gap-3 rounded-xl border border-line bg-raised p-4 sm:grid-cols-2">
            <a class="flex min-h-11 items-center justify-center rounded-lg border border-line bg-surface px-4 py-2 font-semibold hover:border-accent sm:col-span-2" href={format!("#{}", props.path)}>{"Starter"}</a>
            <details class="rounded-xl border border-line bg-surface p-3" data-vocabulary-hsk="">
                <summary class="cursor-pointer font-semibold">{"HSK levels"}</summary>
                <div class="mt-3 space-y-3">
                    {for [(2_u8, 6_u8), (3_u8, 7_u8)].into_iter().map(|(standard, count)| html! {
                        <div key={standard} class="space-y-2">
                            <p class="text-sm text-muted">{format!("HSK {standard}.0")}</p>
                            <div class="grid grid-cols-3 gap-2">
                                {for (1..=count).map(|level| html! {
                                    <a key={level} href={format!("#{}?source=hsk&standard={standard}&level={level}", props.path)} class="min-h-11 rounded-lg border border-line px-2 py-3 text-center text-sm hover:border-accent">{format!("{level}")}</a>
                                })}
                            </div>
                        </div>
                    })}
                </div>
            </details>
            <details class="rounded-xl border border-line bg-surface p-3" data-vocabulary-sets="">
                <summary class="cursor-pointer font-semibold">{"My Word Sets"}</summary>
                <div class="mt-3 flex max-h-56 flex-col gap-2 overflow-y-auto">
                    if props.sets.is_empty() {
                        <p class="text-sm text-muted">{"Create a Word Set to use your own vocabulary."}</p>
                    }
                    {for props.sets.iter().map(|set| html! {
                        <a key={set.id.as_str().to_owned()} href={format!("#{}?source=set&id={}", props.path, set.id)} class="min-h-11 rounded-lg border border-line px-3 py-3 text-sm hover:border-accent">{set.name.clone()}</a>
                    })}
                </div>
            </details>
        </nav>
    }
}
