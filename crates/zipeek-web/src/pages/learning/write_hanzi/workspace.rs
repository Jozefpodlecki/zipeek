use crate::{components::ListenButton, pages::hanzi_writing::HanziWriting};
use yew::prelude::*;
use zipeek_core::Word;
use zipeek_hanzi_writing::hanzi_characters;

#[derive(Properties, PartialEq)]
pub struct WritingWorkspaceProps {
    pub word: Option<Word>,
    pub character_index: usize,
    pub on_character: Callback<usize>,
}

#[function_component(WritingWorkspace)]
pub fn writing_workspace(props: &WritingWorkspaceProps) -> Html {
    let selected_character = props.word.as_ref().and_then(|word| {
        hanzi_characters(&word.simplified)
            .get(props.character_index)
            .copied()
            .map(|hanzi| {
                let pronunciation = if hanzi_characters(&word.simplified).len() == 1 {
                    word.pinyin.clone()
                } else {
                    format!("Word reading: {}", word.pinyin)
                };
                (
                    hanzi.to_string(),
                    pronunciation,
                    word.english.clone().unwrap_or_default().replace('|', " · "),
                )
            })
    });
    let writing_workspace_content = {
        if let Some(word) = &props.word {
            let empty_word_notice = {
                if hanzi_characters(&word.simplified).is_empty() {
                    html! {
                    <p
                        role="status"
                        class="text-sm text-muted"
                    >
                        {"This entry contains no supported Hanzi characters."}
                    </p>}
                } else {
                    Html::default()
                }
            };
            html! {
                <div
                    class="min-h-0 space-y-2"
                >
                    <div
                        class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-line bg-surface px-4 py-3"
                        data-write-hanzi-selectable-header=""
                    >
                        <div
                            class="min-w-0 flex-1 space-y-1"
                        >
                            <div
                                class="flex flex-wrap items-center gap-2"
                            >
                                <div
                                    class="flex items-center gap-0.5"
                                    aria-label="Select character from word"
                                >
                                    {for hanzi_characters(&word.simplified).into_iter().enumerate().map(|(index, hanzi)| {
                                        let on_character = props.on_character.clone();
                                        let on_select_character = Callback::from(move |_| on_character.emit(index));
                                        let element_classes_1 = classes!(
                                                        "rounded-md",
                                                        "px-1",
                                                        "py-0.5",
                                                        "text-3xl",
                                                        "transition-colors",
                                                        "focus-visible:outline-none",
                                                        "focus-visible:ring-1",
                                                        "focus-visible:ring-blue-400",
                                                        if props.character_index == index {"bg-blue-600/20 text-ink ring-1 ring-blue-400/80"} else {"text-ink hover:bg-raised"},
                                                    );
                                        html! {
                                            <button
                                                key={index}
                                                type="button"
                                                data-write-hanzi-character=""
                                                aria-label={format!("Practice character {}: {}", index + 1, hanzi)}
                                                aria-pressed={(props.character_index == index).to_string()}
                                                class={element_classes_1}
                                                onclick={on_select_character}
                                            >
                                                {hanzi}
                                            </button>}
                                    })}
                                </div>
                                <span
                                    class="text-sm font-medium text-accent"
                                >
                                    {&word.pinyin}
                                </span>
                                <ListenButton
                                    hanzi={word.simplified.clone()}
                                    compact={true}
                                />
                            </div>
                            <p
                                class="text-xs text-muted"
                            >
                                {word.english.as_deref().unwrap_or("Meaning unavailable").replace('|', " · ")}
                            </p>
                        </div>
                        <span
                            class="shrink-0 rounded-lg border border-line bg-raised px-2 py-1 text-xs text-muted"
                        >
                            {format!("Character {} of {}", props.character_index + 1, hanzi_characters(&word.simplified).len())}
                        </span>
                    </div>
                    {empty_word_notice}
                    {if let Some(character) = selected_character {html! {
                        <HanziWriting
                            key={format!("{}:{}",word.id.as_i32(),props.character_index)}
                            selected={Some(character)}
                        />} } else {Html::default()}}
                </div>
            }
        } else {
            html! {
                <div
                    class="flex min-h-96 flex-col items-center justify-center gap-3 rounded-2xl border border-line bg-surface p-8 text-center"
                >
                    <span
                        aria-hidden="true"
                        class="text-5xl text-accent"
                    >
                        {"写"}
                    </span>
                    <h2
                        class="text-xl font-semibold"
                    >
                        {"Choose a Chinese word to begin"}
                    </h2>
                    <p
                        class="max-w-sm text-sm text-muted"
                    >
                        {"Select a saved word or search its English meaning. The writing canvas will appear here."}
                    </p>
                </div>
            }
        }
    };
    html! {
        <section
            class="min-h-0 min-w-0 space-y-2 writing-desktop:overflow-y-auto writing-desktop:overscroll-contain"
            aria-label="Hanzi writing workspace"
        >
            {writing_workspace_content}
        </section>
    }
}
