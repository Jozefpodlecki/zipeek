use crate::components::SetupChoice;
use yew::prelude::*;
use zipeek_core::{Hsk2Level, Hsk3Level, HskLevel, HskSystem};

#[derive(Properties, PartialEq)]
pub struct HskPickerProps {
    pub level: HskLevel,
    pub on_system: Callback<HskSystem>,
    pub on_level: Callback<HskLevel>,
}

#[function_component(HskPicker)]
pub fn hsk_picker(props: &HskPickerProps) -> Html {
    let levels: Vec<HskLevel> = match props.level.system() {
        HskSystem::Hsk2 => [
            Hsk2Level::One,
            Hsk2Level::Two,
            Hsk2Level::Three,
            Hsk2Level::Four,
            Hsk2Level::Five,
            Hsk2Level::Six,
        ]
        .into_iter()
        .map(HskLevel::Hsk2)
        .collect(),
        HskSystem::Hsk3 => [
            Hsk3Level::One,
            Hsk3Level::Two,
            Hsk3Level::Three,
            Hsk3Level::Four,
            Hsk3Level::Five,
            Hsk3Level::Six,
            Hsk3Level::SevenToNine,
        ]
        .into_iter()
        .map(HskLevel::Hsk3)
        .collect(),
    };
    html! {
        <fieldset class="min-w-0" data-hsk-picker="">
            <legend class="mb-3 text-lg font-semibold">{"HSK level"}</legend>
            <div class="mb-3 flex flex-wrap gap-2">
                {for [(HskSystem::Hsk2,"HSK 2.0"),(HskSystem::Hsk3,"HSK 3.0")].into_iter().map(|(system,label)| {
                    let select=props.on_system.clone();
                    html! { <SetupChoice label={label} value={label} selected={system==props.level.system()} on_select={Callback::from(move |_| select.emit(system))} /> }
                })}
            </div>
            <div class="flex flex-wrap gap-2">
                {for levels.into_iter().map(|level| {
                    let label=if level.as_i32()==7 {"HSK 7–9".to_owned()} else {format!("HSK {}",level.as_i32())};
                    let select=props.on_level.clone();
                    html! { <SetupChoice label={label} value={format!("level-{}",level.as_i32())} selected={level==props.level} on_select={Callback::from(move |_| select.emit(level))} /> }
                })}
            </div>
        </fieldset>
    }
}
