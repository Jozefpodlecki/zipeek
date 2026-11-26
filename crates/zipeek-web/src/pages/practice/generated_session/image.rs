use crate::services::practice_generated::GeneratedImageChoice;
use std::collections::BTreeSet;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ImageLoadStatus {
    Loading,
    Ready,
    Failed,
}

#[derive(Properties, PartialEq)]
pub(super) struct ImagePracticeProps {
    pub choices: Vec<GeneratedImageChoice>,
    pub correct_ids: BTreeSet<i32>,
    pub image_url: String,
    pub scene: String,
    pub on_completed: Callback<()>,
}

#[function_component(ImagePractice)]
pub(super) fn image_practice(props: &ImagePracticeProps) -> Html {
    let selected = use_state(Vec::<usize>::new);
    let checked = use_state(|| false);
    let image_status = use_state(|| ImageLoadStatus::Loading);
    let choices = &props.choices;
    let correct_ids = &props.correct_ids;
    let image_url = props.image_url.clone();
    let scene = props.scene.clone();
    let on_continue = {
        let completed = props.on_completed.clone();
        Callback::from(move |_: MouseEvent| completed.emit(()))
    };
    let on_image_load = {
        let image_status = image_status.clone();
        Callback::from(move |_: Event| {
            image_status.set(ImageLoadStatus::Ready);
        })
    };
    let on_image_error = {
        let image_status = image_status.clone();
        Callback::from(move |_: Event| {
            image_status.set(ImageLoadStatus::Failed);
        })
    };
    let toggle = {
        let selected = selected.clone();
        let selection_limit = correct_ids.len();
        Callback::from(move |index: usize| {
            let mut next = (*selected).clone();
            if let Some(position) = next.iter().position(|item| *item == index) {
                next.remove(position);
            } else if next.len() < selection_limit {
                next.push(index);
            }
            selected.set(next);
        })
    };
    let check = {
        let checked = checked.clone();
        Callback::from(move |_: MouseEvent| checked.set(true))
    };
    let selected_ids = selected
        .iter()
        .filter_map(|index| choices.get(*index))
        .map(|choice| choice.id)
        .collect::<BTreeSet<_>>();
    let correct = selected_ids == *correct_ids;
    html! {
        <div
                    class="space-y-4"
                    data-generated-image="">
            <img
                src={image_url}
                alt="Generated illustration for Mandarin vocabulary practice"
                class="mx-auto max-h-96 rounded-xl object-contain"
                onload={on_image_load}
                onerror={on_image_error}
            />
            {
                if *image_status == ImageLoadStatus::Failed {
                    html! { <p role="alert" class="rounded-lg border border-line bg-surface p-3">{"The illustration did not load. Return to Practice to regenerate it."}</p> }
                } else if *image_status != ImageLoadStatus::Ready {
                    html! { <p role="status">{"Loading illustration…"}</p> }
                } else {
                    Html::default()
                }
            }
            <div class="mx-auto w-full max-w-2xl space-y-3 rounded-xl border border-line bg-surface p-4 shadow-lg">
                <div class="space-y-1">
                    <h2 class="text-base font-semibold">{"Which Chinese words match the picture?"}</h2>
                    <p class="text-sm text-muted">
                        {if correct_ids.len() == 1 {
                            "Choose the one word depicted in the image."
                        } else {
                            "Choose both words depicted in the image. The other words are distractors."
                        }}
                    </p>
                </div>
                <div role="group" aria-label="Choose Chinese words matching the picture"
                     class="grid grid-cols-2 gap-2 sm:grid-cols-3">
                    {for choices.iter().enumerate().map(|(index, choice)| {
                        let toggle = toggle.clone();
                        let choose = Callback::from(move |_: MouseEvent| toggle.emit(index));
                        let is_selected = selected.contains(&index);
                        html! { <button
                            key={choice.id}
                            type="button"
                            class={classes!("min-h-14", "rounded-xl", "border", "px-3", "py-3", "text-lg", "font-medium", "transition-colors", "focus-visible:outline-2", "focus-visible:outline-offset-2",
                                if is_selected {"border-brand-action bg-brand-action text-white"} else {"border-line bg-surface hover:border-brand-action"})}
                            aria-pressed={is_selected.to_string()}
                            disabled={*checked || *image_status != ImageLoadStatus::Ready}
                            onclick={choose}>{&choice.hanzi}</button> }
                    })}
                </div>
                {if *checked {
                    html! { <div role="status" class="space-y-3">
                        <p class="font-semibold">{if correct {"Correct!"} else {"Not quite — review the expected words."}}</p>
                        <p class="text-sm">{format!("Expected: {}", choices.iter()
                            .filter(|choice| correct_ids.contains(&choice.id))
                            .map(|choice| choice.hanzi.as_str()).collect::<Vec<_>>().join(", "))}</p>
                        <button type="button" class="rounded-lg bg-brand-action px-5 py-3 text-white"
                            onclick={on_continue.clone()}>{"Continue"}</button>
                    </div> }
                } else {
                    html! { <button type="button"
                        class="rounded-lg bg-brand-action px-5 py-3 text-white"
                        disabled={selected.is_empty() || *image_status != ImageLoadStatus::Ready}
                        onclick={check}>{"Check answer"}</button> }
                }}
                <details class="text-sm text-muted">
                    <summary class="cursor-pointer">{"Image quality and generation details"}</summary>
                    <p class="mt-2">{"If the illustration does not match the intended subjects, return to Practice and generate another. An unsuitable image should not be treated as a knowledge assessment."}</p>
                    <p class="mt-2">{scene}</p>
                </details>
            </div>
        </div>
    }
}
