use animato::{Easing, Tween, Update};
use gloo::timers::callback::Interval;
use yew::prelude::*;

use crate::{
    appearance::{Appearance, Preset},
    hooks::use_app_environment,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CreatureOrientation {
    Vertical,
    Horizontal,
}

impl CreatureOrientation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

pub(super) fn creature_orientation(preset: Preset) -> CreatureOrientation {
    match preset {
        Preset::Midnight | Preset::Maple => CreatureOrientation::Horizontal,
        Preset::Lavender | Preset::Jade | Preset::Sunset | Preset::Ink => {
            CreatureOrientation::Vertical
        }
    }
}

fn creature_for(preset: Preset) -> (&'static str, &'static str) {
    match preset {
        Preset::Lavender => ("dragon", "dragon.webp"),
        Preset::Jade => ("qilin", "qilin.webp"),
        Preset::Sunset => ("phoenix", "phoenix.webp"),
        Preset::Midnight => ("white-tiger", "white-tiger.webp"),
        Preset::Ink => ("crane", "crane.webp"),
        Preset::Maple => ("deer", "deer.webp"),
    }
}

fn creature_asset_path(preset: Preset, hosted: bool) -> String {
    let (_, file) = creature_for(preset);
    let prefix = if hosted { "/zipeek" } else { "" };
    format!("{prefix}/practice/{file}")
}

#[derive(Properties, PartialEq)]
pub(super) struct CreatureProgressProps {
    pub completed: usize,
    pub total: usize,
    pub endless: bool,
}

fn illumination_position(progress: f32) -> f32 {
    if progress <= 0.0 {
        -6.0
    } else if progress >= 100.0 {
        106.0
    } else {
        progress
    }
}

fn completed_percent(completed: usize, total: usize, endless: bool) -> f32 {
    if endless || total == 0 {
        0.0
    } else {
        (completed.min(total) as f32 / total as f32) * 100.0
    }
}

#[function_component(CreatureProgress)]
pub(super) fn creature_progress(props: &CreatureProgressProps) -> Html {
    let environment = use_app_environment();
    let appearance = use_context::<Appearance>()
        .expect("Appearance provider missing from Practice creature progress");
    let (creature, artwork) = creature_for(appearance.preset);
    let orientation = creature_orientation(appearance.preset);
    let hosted = environment
        .window
        .location()
        .pathname()
        .ok()
        .is_some_and(|path| path == "/zipeek" || path.starts_with("/zipeek/"));
    let image_url = creature_asset_path(appearance.preset, hosted);
    let target = completed_percent(props.completed, props.total, props.endless);
    let reveal = use_node_ref();
    let current = use_mut_ref(|| 0.0_f32);
    {
        let reveal = reveal.clone();
        let current = current.clone();
        use_effect_with((target.to_bits(), appearance.preset.key()), move |_| {
            let initial = *current.borrow();
            if let Some(element) = reveal.cast::<web_sys::Element>() {
                let _ = element.remove_attribute("data-illumination-complete");
                let _ = element.set_attribute(
                    "style",
                    &format!(
                        "--creature-progress: {:.3}%;",
                        illumination_position(initial)
                    ),
                );
            }
            let mut tween = Tween::new(initial, target)
                .duration(0.35)
                .easing(Easing::EaseOutCubic)
                .build();
            let interval = Interval::new(16, move || {
                if tween.is_complete() {
                    return;
                }
                tween.update(0.016);
                let value = tween.value();
                *current.borrow_mut() = value;
                let illumination = illumination_position(value);
                if let Some(element) = reveal.cast::<web_sys::Element>() {
                    let _ = element.set_attribute(
                        "style",
                        &format!("--creature-progress: {illumination:.3}%;"),
                    );
                    if tween.is_complete() && target >= 100.0 {
                        let _ = element.set_attribute("data-illumination-complete", "");
                    }
                }
            });
            move || drop(interval)
        });
    }

    let description = if props.endless {
        format!("Endless practice: {} exercises finished", props.completed)
    } else {
        format!("{} of {} exercises finished", props.completed, props.total)
    };
    let target_style = format!("--creature-target: {:.3}%;", illumination_position(target));
    let rail_classes = match orientation {
        CreatureOrientation::Vertical => {
            "relative flex h-32 w-full shrink-0 items-center justify-center"
        }
        CreatureOrientation::Horizontal => {
            "relative flex h-28 w-full shrink-0 items-center justify-center"
        }
    };
    let frame_classes = match orientation {
        CreatureOrientation::Vertical => "practice-creature-frame relative h-full aspect-[1/3]",
        CreatureOrientation::Horizontal => {
            "practice-creature-frame relative h-full aspect-[3/1] max-w-[28rem] shrink-0"
        }
    };
    html! {
        <aside data-practice-step-progress="" data-creature-progress="" data-progress-creature={creature} data-creature-orientation={orientation.as_str()} data-animation-engine="animato"
            class={rail_classes}
            aria-label={format!("Practice progress: {creature}")}>
            <div class={frame_classes} style={target_style}>
                <img class="practice-creature-underlay absolute inset-0 h-full w-full object-contain"
                    src={image_url.clone()} data-creature-artwork={artwork} alt="" aria-hidden="true" />
                <div ref={reveal} class="practice-creature-reveal absolute inset-0"
                    data-creature-reveal="" data-reveal-treatment="feathered" style="--creature-progress: -6%;">
                    <img class="practice-creature-illuminated h-full w-full object-contain"
                        src={image_url.clone()} data-creature-artwork={artwork} alt="" aria-hidden="true" />
                </div>
            </div>
            <div class="sr-only" role="progressbar" aria-label="Practice exercises finished"
                aria-valuemin="0" aria-valuemax="100" aria-valuenow={target.round().to_string()}>
                {description}
            </div>
        </aside>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CreatureOrientation, completed_percent, creature_asset_path, creature_for,
        creature_orientation, illumination_position,
    };
    use crate::appearance::Preset;
    use animato::{Easing, Tween, Update};

    #[test]
    fn each_theme_has_a_distinct_creature_and_asset() {
        let mut names = std::collections::HashSet::new();
        let mut files = std::collections::HashSet::new();
        for preset in Preset::ALL {
            let (creature, file) = creature_for(preset);
            assert!(names.insert(creature));
            assert!(files.insert(file));
            assert!(file.ends_with(".webp") && !file.contains('/'));
            assert_eq!(
                creature_asset_path(preset, false),
                format!("/practice/{file}")
            );
            assert_eq!(
                creature_asset_path(preset, true),
                format!("/zipeek/practice/{file}")
            );
        }
    }

    #[test]
    fn horizontal_creatures_only_for_midnight_and_maple() {
        for preset in Preset::ALL {
            let expected = matches!(preset, Preset::Midnight | Preset::Maple);
            assert_eq!(
                creature_orientation(preset) == CreatureOrientation::Horizontal,
                expected
            );
        }
    }

    #[test]
    fn progress_is_bounded_by_total() {
        assert_eq!(completed_percent(0, 5, false), 0.0);
        assert_eq!(completed_percent(2, 5, false), 40.0);
        assert_eq!(completed_percent(5, 5, false), 100.0);
        assert_eq!(completed_percent(6, 5, false), 100.0);
    }

    #[test]
    fn endless_and_empty_sessions_do_not_invent_progress() {
        assert_eq!(completed_percent(3, 5, true), 0.0);
        assert_eq!(completed_percent(0, 0, false), 0.0);
    }

    #[test]
    fn feathered_reveal_reaches_both_ends_without_a_visible_cutoff() {
        assert_eq!(illumination_position(0.0), -6.0);
        assert_eq!(illumination_position(50.0), 50.0);
        assert_eq!(illumination_position(100.0), 106.0);
        assert_eq!(illumination_position(-5.0), -6.0);
        assert_eq!(illumination_position(120.0), 106.0);
    }

    #[test]
    fn animato_tween_interpolates_without_jumping() {
        let mut tween = Tween::new(0.0_f32, 40.0_f32)
            .duration(0.35)
            .easing(Easing::EaseOutCubic)
            .build();
        tween.update(0.4);
        assert!(tween.value() > 0.0 && tween.value() < 40.0);
        tween.update(1.0);
        assert_eq!(tween.value(), 40.0);
    }
}
