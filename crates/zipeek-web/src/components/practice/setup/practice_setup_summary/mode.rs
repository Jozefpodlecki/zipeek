use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::practice_draft::SessionLength;

use super::PracticeSetupSummaryProps;
use crate::components::SetupChoice;

pub(super) fn render(props: &PracticeSetupSummaryProps) -> Html {
    let mode = if props.draft.translation_session_only() {
        translation_modes(props)
    } else if props.draft.supports_local_vocabulary_session() {
        fixed_session_modes(props)
    } else {
        html! { <p class="text-sm text-muted">{"Select supported exercises and a valid session length."}</p> }
    };
    html! {
        <section
            class="rounded-xl border border-line bg-raised p-4 space-y-3"
            data-mode-picker=""
        >
            <h3 class="flex items-center gap-2 font-semibold">
                <Icon data={IconData::LUCIDE_CLOCK} width="19px" height="19px" />
                {"Session mode"}
            </h3>
            {mode}
        </section>
    }
}

fn translation_modes(props: &PracticeSetupSummaryProps) -> Html {
    html! {
        <div class="flex flex-wrap gap-2" data-translation-setup-count="">
            {for [(0_u16, "∞ Endless"), (5, "5 sentences"), (10, "10 sentences"), (20, "20 sentences")]
                .into_iter()
                .map(|(count, label)| {
                    let selected = match (&props.draft.length, count) {
                        (SessionLength::Endless, 0) => true,
                        (SessionLength::Fixed(current), target) => *current == target,
                        _ => false,
                    };
                    let on_count = props.on_count.clone();
                    let on_select_count = Callback::from(move |_| on_count.emit(count));
                    html! {
                        <SetupChoice
                            label={label}
                            value={label}
                            selected={selected}
                            on_select={on_select_count}
                        />
                    }
                })}
        </div>
    }
}

fn fixed_session_modes(props: &PracticeSetupSummaryProps) -> Html {
    html! {
            <div
                class="space-y-3"
            >
                <div
                    class="flex flex-wrap gap-2"
                >
                    {for [(Some(5_u16), "Fixed 5"), (Some(10), "Fixed 10"), (Some(20), "Fixed 20"), (None, "∞ Endless")]
                        .into_iter()
                        .map(|(count, label)| {
                            let selected = match (&props.draft.length, count) {
                                (SessionLength::Endless, None) => true,
                                (SessionLength::Fixed(current), Some(target)) => *current == target,
                                _ => false,
                            };
                            let on_length = props.on_length.clone();
                            let on_select_length = Callback::from(move |_| {
                                on_length.emit(count.map_or(SessionLength::Endless, SessionLength::Fixed));
                            });

                            html! {
                                <SetupChoice
                                    label={label}
                                    value={label}
                                    selected={selected}
                                    on_select={on_select_length}
                                />
                            }
                        })}
                </div>
                {props.draft.has_exercise("spot-mistake").then(|| html! {
                    <p class="text-sm text-muted" data-spot-mistake-practice-only="">
                        {"Spot the Mistake separates finding the error from repairing it. Example results do not count toward mastery."}
                    </p>
                }).unwrap_or_default()}
                {props.plan_rounds.map(|rounds| html! {
                    <p
                        class="text-sm text-muted"
                        data-practice-plan-preview=""
                    >
                        {format!("{rounds} exercise rounds planned; question count depends on exercise type")}
                    </p>
                }).unwrap_or_default()}
            </div>
    }
}
