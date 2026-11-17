use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_exercises::definition::ExternalService;

#[derive(Properties, PartialEq)]
pub(super) struct ExerciseTileProps {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub icon: IconData,
    pub tab: &'static str,
    pub selected: bool,
    pub services: &'static [ExternalService],
    pub on_toggle: Callback<&'static str>,
}

#[function_component(ExerciseTile)]
pub(super) fn exercise_tile(props: &ExerciseTileProps) -> Html {
    let id = props.id;
    let on_toggle = props.on_toggle.clone();
    let selected = props.selected;

    let on_toggle_exercise = Callback::from(move |_| on_toggle.emit(id));
    let element_classes_1 = classes!(
        "flex",
        "min-h-24",
        "w-full",
        "flex-col",
        "gap-2",
        "rounded-xl",
        "border-2",
        "p-4",
        "text-left",
        "transition-all",
        "duration-200",
        "focus-visible:outline",
        "focus-visible:outline-2",
        "focus-visible:outline-offset-2",
        "focus-visible:outline-accent",
        "motion-reduce:transition-none",
        if selected {
            "border-brand-action bg-accent-soft shadow-md ring-2 ring-brand-action/70"
        } else {
            "border-line bg-[var(--practice-exercise-rest)] hover:border-brand-action hover:bg-[var(--practice-exercise-hover)] motion-safe:hover:-translate-y-0.5"
        },
    );
    let element_classes_2 = classes!(
        "flex",
        "h-8",
        "w-8",
        "shrink-0",
        "items-center",
        "justify-center",
        "rounded-lg",
        "border",
        "transition-colors",
        "motion-reduce:transition-none",
        if selected {
            "border-brand-action bg-brand-action text-white"
        } else {
            "border-line bg-raised text-muted"
        },
    );
    html! {
        <button
            type="button"
            data-practice-exercise={id}
            data-practice-tab={props.tab}
            data-selected={if selected { "true" } else { "false" }}
            data-external-services={match props.services {
                [ExternalService::AiText, ExternalService::ImageGeneration] => "ai,image-generation",
                [ExternalService::AiText] => "ai",
                [ExternalService::ImageGeneration] => "image-generation",
                _ => "none",
            }}
            aria-pressed={selected.to_string()}
            aria-describedby={format!("practice-exercise-description-{id}")}
            aria-label={format!("{}{}", props.title, if selected { ", selected" } else { ", not selected" })}
            class={element_classes_1}
            onclick={on_toggle_exercise}
        >
            <span
                class="flex w-full items-center gap-3 text-sm font-semibold"
            >
                <span
                    aria-hidden="true"
                    class={element_classes_2}
                >
                    <Icon
                        data={props.icon}
                        width="18px"
                        height="18px"
                    />
                </span>
                <span class="flex-1">
                    {props.title}
                </span>
                {if selected { html! { <span data-exercise-selected="" aria-hidden="true" class="flex size-6 items-center justify-center rounded-full bg-brand-action text-white"><Icon data={IconData::LUCIDE_CHECK} width="16px" height="16px" /></span> } } else { Html::default() }}
            </span>
            <span
                id={format!("practice-exercise-description-{id}")}
                class="text-sm text-muted"
            >
                {props.description}
            </span>
            if !props.services.is_empty() {
                <span class="flex flex-wrap gap-1.5" data-exercise-service-labels="">
                    {for props.services.iter().map(|service| {
                        let (label, marker) = match service {
                            ExternalService::AiText => ("AI", "ai"),
                            ExternalService::ImageGeneration => ("Image generation", "image-generation"),
                        };
                        html! {
                            <span
                                key={marker}
                                data-exercise-service={marker}
                                class="rounded-md border border-line bg-raised px-2 py-0.5 text-xs font-medium text-muted"
                            >
                                {label}
                            </span>
                        }
                    })}
                </span>
            }
        </button>
    }
}
