use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VocabularyTab {
    Source,
    Filters,
}

#[derive(Properties, PartialEq)]
pub(super) struct VocabularyTabsProps {
    pub selected: VocabularyTab,
    pub on_select: Callback<VocabularyTab>,
}

#[function_component(VocabularyTabs)]
pub(super) fn vocabulary_tabs(props: &VocabularyTabsProps) -> Html {
    html! {
        <div
            role="group"
            aria-label="Vocabulary configuration"
            data-vocabulary-subtabs=""
            class="inline-flex max-w-full flex-wrap gap-1 rounded-lg border border-line bg-raised/55 p-0.5"
        >
            {for [
                (VocabularyTab::Source, "Source & HSK", IconData::LUCIDE_BOOK_OPEN),
                (VocabularyTab::Filters, "Categories & filters", IconData::LUCIDE_SLIDERS_HORIZONTAL),
            ].into_iter().map(|(tab, label, icon)| {
                let selected = props.selected == tab;
                let on_select = props.on_select.clone();
                let key = if tab == VocabularyTab::Source { "source" } else { "filters" };
                let on_select_tab = Callback::from(move |_| on_select.emit(tab));
                let element_classes_1 = classes!(
                                "inline-flex",
                                "min-h-9",
                                "items-center",
                                "gap-1.5",
                                "rounded-md",
                                "px-3",
                                "font-semibold",
                                "text-xs",
                                "focus-visible:outline-2",
                                "focus-visible:outline-offset-2",
                                "focus-visible:outline-brand-action",
                                if selected { "bg-surface text-ink shadow-sm ring-1 ring-line" } else { "text-muted hover:bg-surface/60 hover:text-ink" },
                            );
                html! {
                    <button
                        key={key}
                        type="button"
                        data-vocabulary-subtab={key}
                        aria-pressed={selected.to_string()}
                        onclick={on_select_tab}
                        class={element_classes_1}
                    >
                        <Icon
                            data={icon}
                            width="15px"
                            height="15px"
                        />
                        {label}
                    </button>
                }
            })}
        </div>
    }
}
