use yew::prelude::*;

const BARS: usize = 40;
const VIEW_WIDTH: f32 = 320.0;
const VIEW_HEIGHT: f32 = 72.0;
const MAX_BAR_HEIGHT: f32 = 62.0;

fn bar_height(level: f32) -> f32 {
    if !level.is_finite() {
        return 3.0;
    }
    3.0 + level.clamp(0.0, 1.0) * (MAX_BAR_HEIGHT - 3.0)
}

#[derive(Properties, PartialEq)]
pub struct AudioWaveformProps {
    pub levels: Vec<f32>,
    #[prop_or_default]
    pub recording: bool,
}

#[function_component(AudioWaveform)]
pub fn audio_waveform(props: &AudioWaveformProps) -> Html {
    let count = BARS;
    let stride = VIEW_WIDTH / count as f32;
    let bar_width = stride * 0.56;
    html! {
        <svg
            data-component="audio-waveform"
            data-recording={if props.recording { "true" } else { "false" }}
            viewBox="0 0 320 72"
            role="presentation"
            aria-hidden="true"
            class="h-20 w-full text-blue-500"
            preserveAspectRatio="none"
        >
            {for (0..count).map(|index| {
                let level = props.levels.get(index).copied().unwrap_or(0.0);
                let height = bar_height(level);
                let y = (VIEW_HEIGHT - height) / 2.0;
                html! {
                    <rect
                        key={index}
                        x={(index as f32 * stride + (stride - bar_width) / 2.0).to_string()}
                        y={y.to_string()}
                        width={bar_width.to_string()}
                        height={height.to_string()}
                        rx="2"
                        fill="currentColor"
                        opacity={if props.recording { "1" } else { "0.45" }}
                    />
                }
            })}
        </svg>
    }
}

#[cfg(test)]
mod tests {
    use super::bar_height;

    #[test]
    fn waveform_amplitudes_are_clamped_and_silent_values_remain_visible() {
        assert_eq!(bar_height(-1.0), 3.0);
        assert_eq!(bar_height(0.0), 3.0);
        assert_eq!(bar_height(1.0), 62.0);
        assert_eq!(bar_height(50.0), 62.0);
        assert_eq!(bar_height(f32::NAN), 3.0);
        assert_eq!(bar_height(f32::INFINITY), 3.0);
    }
}
