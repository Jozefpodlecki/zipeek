use yew::prelude::*;

use super::activity::{ACTIVITY_WEEKS, WEEKS};

#[derive(Properties, PartialEq)]
pub(super) struct ActivityFallbackProps {
    pub buckets: [u32; WEEKS],
}

#[function_component(ActivityFallback)]
pub(super) fn activity_fallback(props: &ActivityFallbackProps) -> Html {
    let maximum = f64::from(*props.buckets.iter().max().unwrap_or(&1)).max(1.0);
    html! {
        <div class="absolute inset-0 flex items-end justify-around gap-3 px-5 pb-6 pt-2">
            {for props.buckets.iter().enumerate().map(|(index, count)| {
                let height = bar_height(*count, maximum);
                html! {
                    <div key={index} class="flex h-full min-w-0 flex-1 flex-col items-center justify-end gap-1">
                        <div class="w-full max-w-12 rounded-t-md bg-accent/75" style={format!("height: {height:.1}%")} />
                        <span class="text-xs text-muted">{ACTIVITY_WEEKS[index].trim_start_matches("2026-")}</span>
                    </div>
                }
            })}
        </div>
    }
}

fn bar_height(count: u32, maximum: f64) -> f64 {
    if count == 0 {
        0.0
    } else {
        (f64::from(count) / maximum.max(1.0) * 100.0).clamp(8.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::bar_height;

    #[test]
    fn zero_activity_has_no_visible_bar() {
        assert_eq!(bar_height(0, 10.0), 0.0);
        assert_eq!(bar_height(1, 10.0), 10.0);
        assert_eq!(bar_height(10, 10.0), 100.0);
    }
}
