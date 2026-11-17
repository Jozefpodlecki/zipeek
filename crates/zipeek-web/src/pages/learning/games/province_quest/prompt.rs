use crate::components::audio::listen_button::ListenButton;
use yew::prelude::*;
use zipeek_exercises::province_quest::{
    AnswerMode, LearningSupport, MapRegion, QuestionKind, reading,
};

#[derive(Properties, PartialEq)]
pub struct ProvincePromptProps {
    pub region: MapRegion,
    pub kind: QuestionKind,
    pub answer_mode: AnswerMode,
    pub support: LearningSupport,
    pub revealed: bool,
    pub reviewing: bool,
    pub on_reveal: Callback<MouseEvent>,
}

#[function_component(ProvincePrompt)]
pub fn province_prompt(props: &ProvincePromptProps) -> Html {
    let prompt = reading(
        &props.region,
        props.kind,
        props.support,
        props.revealed,
        props.reviewing,
    );
    let choice_round = props.answer_mode == AnswerMode::SelectHanzi;
    let outline = props.kind == QuestionKind::FindOutline && !choice_round;
    let [x, y, width, height] = props.region.bbox;
    let margin = width.max(height) * 0.1;
    let outline_view_box = format!(
        "{} {} {} {}",
        x - margin,
        y - margin,
        width + 2.0 * margin,
        height + 2.0 * margin
    );
    html! {
        <div class="space-y-1.5 rounded-xl bg-raised px-3 py-3 text-center" data-province-prompt="">
            <p class="text-sm text-muted">{if choice_round { "Identify the highlighted region" } else { prompt.title }}</p>
            if outline {
                <svg viewBox={outline_view_box} class="mx-auto h-24 w-full max-w-xs" role="img" aria-label="Silhouette of the region to identify">
                    <path d={props.region.path.clone()} fill="var(--province-map-fill)" stroke="var(--province-map-stroke)" stroke-width="1.5"/>
                </svg>
            }
            if !choice_round && prompt.show_hanzi {
                <div class="flex items-center justify-center gap-2"><h2 class="text-2xl font-bold" lang="zh-Hans">{prompt.hanzi}</h2>
                    <ListenButton hanzi={AttrValue::from(prompt.hanzi.to_owned())} compact=true />
                </div>
            }
            if !choice_round && prompt.show_pinyin && prompt.show_hanzi {
                <p class="text-accent">{prompt.pinyin}</p>
            }
            if let Some(english) = prompt.english.filter(|_| !choice_round && prompt.show_hanzi) {
                <p class="text-sm text-muted">{english}</p>
            }
            if !props.reviewing && ((!choice_round && (!prompt.show_pinyin || !prompt.show_hanzi))
                || (choice_round && props.support != LearningSupport::Guided && !props.revealed)) {
                <button type="button" class="text-sm font-medium text-accent underline" onclick={props.on_reveal.clone()}> {"Reveal help"} </button>
            }
        </div>
    }
}
