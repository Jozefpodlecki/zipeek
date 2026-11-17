use super::stroke_loading::StrokeStudy;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(super) struct WritingControlsProps {
    pub study: Option<Result<StrokeStudy, String>>,
    pub drawing: bool,
    pub guide: bool,
    pub replaying: bool,
    pub show_next: bool,
    pub allow_animation: bool,
    pub allow_guide_toggle: bool,
    pub on_replay: Callback<MouseEvent>,
    pub on_undo: Callback<MouseEvent>,
    pub on_clear: Callback<MouseEvent>,
    pub on_toggle_guide: Callback<MouseEvent>,
    pub on_next_character: Callback<MouseEvent>,
}

#[function_component(WritingControls)]
pub(super) fn writing_controls(props: &WritingControlsProps) -> Html {
    let has_progress = matches!(&props.study, Some(Ok(current)) if current.session.completed() > 0);
    let has_attempts = matches!(&props.study, Some(Ok(current)) if current.session.completed() > 0 || current.session.mistakes() > 0);
    let has_reference = matches!(&props.study, Some(Ok(_)));
    html! {
        <div class="mx-auto flex w-full max-w-md flex-wrap items-center justify-center gap-2" data-hanzi-writing-controls="" data-hanzi-watch-enabled={props.allow_animation.to_string()}>
            <button type="button" data-action="undo-stroke" onclick={props.on_undo.clone()}
                disabled={!has_progress || props.replaying}
                aria-label="Undo stroke" title="Undo stroke"
                class="inline-flex size-11 items-center justify-center rounded-lg border border-line bg-raised text-ink transition-colors hover:bg-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action disabled:opacity-40">
                <Icon data={IconData::LUCIDE_UNDO_2} width="16" height="16" />
            </button>
            <button type="button" data-action="reset-character" onclick={props.on_clear.clone()}
                disabled={!has_attempts && !props.drawing && !props.replaying}
                aria-label="Reset character" title="Reset character"
                class="inline-flex size-11 items-center justify-center rounded-lg border border-line bg-raised text-ink transition-colors hover:bg-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action disabled:opacity-40">
                <Icon data={IconData::LUCIDE_ROTATE_CCW} width="16" height="16" />
            </button>
            if props.allow_animation {
            <button type="button" data-action="replay-hanzi-strokes" onclick={props.on_replay.clone()}
                disabled={!has_reference}
                aria-label="Watch character stroke animation" title="Watch character stroke animation"
                class="inline-flex size-11 items-center justify-center rounded-lg border border-line bg-raised text-ink transition-colors hover:bg-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action disabled:opacity-40">
                <Icon data={IconData::LUCIDE_PLAY} width="16" height="16" />
            </button>
            }
            if props.allow_guide_toggle {
            <button type="button" data-action="toggle-guide" aria-pressed={props.guide.to_string()}
                aria-label={if props.guide { "Hide guide" } else { "Show guide" }}
                title={if props.guide { "Hide guide" } else { "Show guide" }}
                onclick={props.on_toggle_guide.clone()}
                class="inline-flex size-11 items-center justify-center rounded-lg border border-line bg-raised text-ink transition-colors hover:bg-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action">
                <Icon data={IconData::LUCIDE_PENCIL} width="16" height="16" />
            </button>
            }
            if props.show_next {
                <button type="button" data-action="next-character" onclick={props.on_next_character.clone()}
                    aria-label="Next character" title="Next character"
                    class="inline-flex size-11 items-center justify-center rounded-lg bg-brand-action text-white hover:bg-blue-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-action">
                    <Icon data={IconData::LUCIDE_CHEVRON_RIGHT} width="16" height="16" />
                </button>
            }
        </div>
    }
}
