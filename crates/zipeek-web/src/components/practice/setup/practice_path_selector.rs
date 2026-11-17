use crate::services::practice_profiles::PracticeProfile;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub struct PracticePathSelectorProps {
    pub advanced: bool,
    pub on_select: Callback<bool>,
    pub profiles: Result<Vec<PracticeProfile>, String>,
    pub pending_delete: Option<String>,
    pub on_load_profile: Callback<String>,
    pub on_request_delete: Callback<String>,
    pub on_confirm_delete: Callback<MouseEvent>,
    pub on_cancel_delete: Callback<MouseEvent>,
}

#[function_component(PracticePathSelector)]
pub fn practice_path_selector(props: &PracticePathSelectorProps) -> Html {
    html! {
        <section
            role="tabpanel"
            data-practice-step-panel="path"
            class="space-y-4"
        >
            <PracticePathOptions
                advanced={props.advanced}
                on_select={props.on_select.clone()}
            />

            <SavedPracticeProfiles
                profiles={props.profiles.clone()}
                pending_delete={props.pending_delete.clone()}
                on_load_profile={props.on_load_profile.clone()}
                on_request_delete={props.on_request_delete.clone()}
                on_confirm_delete={props.on_confirm_delete.clone()}
                on_cancel_delete={props.on_cancel_delete.clone()}
            />
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct PracticePathOptionsProps {
    advanced: bool,
    on_select: Callback<bool>,
}

#[function_component(PracticePathOptions)]
fn practice_path_options(props: &PracticePathOptionsProps) -> Html {
    let paths = [
        (
            false,
            "Quick Practice",
            "Matching, multiple choice, pinyin typing, character building, word mosaics, and Character Hunt.",
            IconData::LUCIDE_ZAP,
        ),
        (
            true,
            "Advanced Practice",
            "Configure advanced exercises individually, including translation, speaking and listening.",
            IconData::LUCIDE_WAND_2,
        ),
    ];

    html! {
        <>
            <p class="text-sm text-muted">
                {"Choose a practice path to continue to exercises."}
            </p>

            <div class="grid gap-4 md:grid-cols-2">
                {for paths.into_iter().map(|(is_advanced, title, description, icon)| {
                    let on_select = props.on_select.clone();

                    let on_choose_path = Callback::from(
                        move |_: MouseEvent| on_select.emit(is_advanced)
                    );

                    let element_classes_1 = classes!(
                                "flex",
                                "min-h-32",
                                "flex-col",
                                "items-start",
                                "gap-3",
                                "rounded-2xl",
                                "border-2",
                                "bg-surface",
                                "p-6",
                                "text-left",
                                "transition-[border-color,background-color,box-shadow]",
                                "duration-200",
                                "motion-reduce:transition-none",
                                "focus-visible:outline-2",
                                "focus-visible:outline-accent",
                                "hover:border-accent",
                                if props.advanced == is_advanced {
                                    "border-brand-action bg-accent-soft ring-2 ring-brand-action/30 shadow-sm"
                                } else {
                                    "border-line"
                                },
                            );
                    html! {
                        <button
                            key={title}
                            type="button"
                            data-practice-path={if is_advanced {
                                "advanced"
                            } else {
                                "quick"
                            }}
                            aria-pressed={(props.advanced == is_advanced).to_string()}
                            class={element_classes_1}
                            onclick={on_choose_path}
                        >
                            <span class="flex items-center gap-3">
                                <Icon
                                    data={icon}
                                    width="26px"
                                    height="26px"
                                />
                                <span class="text-lg font-semibold">
                                    {title}
                                </span>
                            </span>

                            <span class="text-sm text-muted">
                                {description}
                            </span>
                        </button>
                    }
                })}
            </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct SavedPracticeProfilesProps {
    profiles: Result<Vec<PracticeProfile>, String>,
    pending_delete: Option<String>,
    on_load_profile: Callback<String>,
    on_request_delete: Callback<String>,
    on_confirm_delete: Callback<MouseEvent>,
    on_cancel_delete: Callback<MouseEvent>,
}

#[function_component(SavedPracticeProfiles)]
fn saved_practice_profiles(props: &SavedPracticeProfilesProps) -> Html {
    let content = match &props.profiles {
        Ok(profiles) if profiles.is_empty() => html! {
            <p class="text-sm text-muted">
                {"No saved profiles. Configure Practice and save a profile on the Summary step."}
            </p>
        },

        Ok(profiles) => html! {
            <ul class="grid gap-3 md:grid-cols-2">
                {for profiles.iter().map(|profile| {
                    let name = profile.name.clone();
                    let load_name = name.clone();
                    let delete_name = name.clone();

                    let on_load = props.on_load_profile.clone();
                    let on_remove = props.on_request_delete.clone();

                    let load = Callback::from(move |_: MouseEvent| {
                        on_load.emit(load_name.clone());
                    });

                    let remove = Callback::from(move |_: MouseEvent| {
                        on_remove.emit(delete_name.clone());
                    });

                    let is_pending_delete =
                        props.pending_delete.as_deref()
                            == Some(name.as_str());

                    let actions = if is_pending_delete {
                        html! {
                            <div class="space-y-2">
                                <p class="text-sm">
                                    {"Remove this saved profile?"}
                                </p>

                                <div class="flex gap-2">
                                    <button
                                        type="button"
                                        onclick={props.on_confirm_delete.clone()}
                                        class="rounded-lg border border-red-500 px-3 py-2"
                                    >
                                        <span class="inline-flex items-center gap-2">
                                            <Icon data={IconData::LUCIDE_X} width="16px" height="16px" />
                                            {"Confirm removal"}
                                        </span>
                                    </button>

                                    <button
                                        type="button"
                                        onclick={props.on_cancel_delete.clone()}
                                        class="rounded-lg border border-line px-3 py-2"
                                    >
                                        {"Cancel"}
                                    </button>
                                </div>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="flex gap-2">
                                <button
                                    type="button"
                                    data-practice-profile-load=""
                                    onclick={load}
                                    class="rounded-lg bg-brand-action px-4 py-2 font-semibold text-white"
                                >
                                    <span class="inline-flex items-center gap-2">
                                        <Icon data={IconData::LUCIDE_ARROW_RIGHT} width="16px" height="16px" />
                                        {"Use profile"}
                                    </span>
                                </button>

                                <button
                                    type="button"
                                    data-practice-profile-remove=""
                                    onclick={remove}
                                    class="rounded-lg border border-line px-4 py-2"
                                >
                                    <span class="inline-flex items-center gap-2">
                                        <Icon data={IconData::LUCIDE_X} width="16px" height="16px" />
                                        {"Remove"}
                                    </span>
                                </button>
                            </div>
                        }
                    };

                    html! {
                        <li
                            key={name.clone()}
                            class="rounded-xl border border-line bg-surface p-4 space-y-2"
                        >
                            <p class="flex items-center gap-2 font-semibold">
                                <Icon data={IconData::LUCIDE_BOOKMARK} width="18px" height="18px" />
                                {name}
                            </p>

                            <p class="text-xs text-muted">
                                {format!(
                                    "{} · {} exercise types",
                                    if profile.advanced {
                                        "Advanced"
                                    } else {
                                        "Quick"
                                    },
                                    profile.exercise_count()
                                )}
                            </p>

                            {actions}
                        </li>
                    }
                })}
            </ul>
        },

        Err(error) => html! {
            <p role="alert" class="text-sm text-red-400">
                {format!("Could not read Practice profiles: {error}")}
            </p>
        },
    };

    html! {
        <div
            class="space-y-3 border-t border-line pt-5"
            data-practice-saved-profiles=""
        >
            <h2 class="text-lg font-semibold">
                {"Saved Practice profiles"}
            </h2>

            {content}
        </div>
    }
}
