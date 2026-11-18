use std::rc::Rc;

use yew::prelude::*;
use zipeek_exercises::province_quest::{GeoDataset, LearningSupport};

#[derive(Properties)]
pub struct HanziChoicesProps {
    pub map: Rc<GeoDataset>,
    pub options: Vec<String>,
    pub selected_id: Option<String>,
    pub correct_id: Option<String>,
    pub disabled: bool,
    pub support: LearningSupport,
    pub revealed: bool,
    pub on_select: Callback<String>,
}

impl PartialEq for HanziChoicesProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.map, &other.map)
            && self.options == other.options
            && self.selected_id == other.selected_id
            && self.correct_id == other.correct_id
            && self.disabled == other.disabled
            && self.support == other.support
            && self.revealed == other.revealed
            && self.on_select == other.on_select
    }
}

#[function_component(HanziChoices)]
pub fn hanzi_choices(props: &HanziChoicesProps) -> Html {
    let buttons = props.options.iter().filter_map(|id| {
        let region = props.map.regions.iter().find(|region| region.id == *id)?;
        let is_correct = props.correct_id.as_ref() == Some(id);
        let is_wrong = props.selected_id.as_ref() == Some(id) && !is_correct;
        let status_class = if is_correct {
            "border-[var(--province-map-correct)] bg-[var(--province-choice-correct)]"
        } else if is_wrong {
            "border-[var(--province-map-wrong)] bg-[var(--province-choice-wrong)]"
        } else {
            "border-line bg-raised hover:border-accent"
        };
        let onclick = {
            let id = id.clone();
            props.on_select.reform(move |_: MouseEvent| id.clone())
        };
        Some(html! {
            <button
                key={id.clone()}
                type="button"
                disabled={props.disabled}
                data-province-choice={id.clone()}
                class={format!("rounded-xl border px-4 py-4 text-xl font-semibold transition-colors disabled:cursor-default {status_class}")}
                onclick={onclick}
            >
                <span class="block">{&region.hanzi}</span>
                if props.disabled || props.revealed || props.support == LearningSupport::Guided {
                    <span class="mt-1 block text-sm font-normal text-muted">{&region.pinyin}</span>
                }
            </button>
        })
    }).collect::<Html>();
    html! {
        <fieldset class="space-y-3" data-province-choices="">
            <legend class="mb-2 text-base font-semibold">{"Which Hanzi names the highlighted region?"}</legend>
            <div class="grid gap-3 sm:grid-cols-3">{buttons}</div>
        </fieldset>
    }
}
