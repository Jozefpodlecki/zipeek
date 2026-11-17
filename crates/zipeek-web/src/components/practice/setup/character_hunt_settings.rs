use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::character_hunt::Difficulty;

#[derive(Properties, PartialEq)]
pub struct CharacterHuntSettingsProps {
    pub selected: Difficulty,
    pub on_change: Callback<Difficulty>,
}

#[function_component(CharacterHuntSettings)]
pub fn character_hunt_settings(props: &CharacterHuntSettingsProps) -> Html {
    html! {
        <section
            data-character-hunt-settings=""
            role="tabpanel"
            class="rounded-2xl border border-line bg-surface p-6 space-y-4"
        >
            <div
                class="space-y-1"
            >
                <h3
                    class="text-lg font-semibold"
                >
                    {"Character Hunt difficulty"}
                </h3>
                <p
                    class="text-sm text-muted"
                >
                    {"Grid size is fixed for each practice session. Larger boards have smaller Hanzi tiles."}
                </p>
            </div>
            <div
                class="grid grid-cols-2 gap-3 sm:grid-cols-4"
                role="group"
                aria-label="Character Hunt difficulty"
            >
                {for Difficulty::ALL.map(|difficulty| {
                    let selected = props.selected == difficulty;
                    let on_change = props.on_change.clone();
                    let on_choose_difficulty = Callback::from(move |_| on_change.emit(difficulty));
                    let element_classes_1 = classes!(
                                    "min-h-20",
                                    "rounded-xl",
                                    "border",
                                    "p-3",
                                    "text-left",
                                    "focus-visible:outline-2",
                                    "focus-visible:outline-accent",
                                    if selected { "border-accent bg-accent-soft ring-1 ring-accent" } else { "border-line bg-raised hover:border-accent" },
                                );
                    html! {
                        <button
                            key={difficulty.key()}
                            type="button"
                            data-hunt-difficulty={difficulty.key()}
                            aria-pressed={selected.to_string()}
                            onclick={on_choose_difficulty}
                            class={element_classes_1}
                        >
                            <span class="flex items-center justify-between gap-2 font-semibold">
                                {difficulty.label()}
                                {selected.then(|| html! {
                                    <Icon data={IconData::LUCIDE_CHECK} width="18px" height="18px" />
                                }).unwrap_or_default()}
                            </span>
                            <span
                                class="block text-sm text-muted"
                            >
                                {format!("{} × {}", difficulty.size(), difficulty.size())}
                            </span>
                        </button>
                    }
                })}
            </div>
        </section>
    }
}
