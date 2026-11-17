use crate::components::SetupChoice;
use yew::prelude::*;
use zipeek_core::{HskLevel, HskSystem};

#[derive(Properties, PartialEq)]
pub struct HskPickerProps {
    pub level: HskLevel,
    pub on_system: Callback<HskSystem>,
    pub on_level: Callback<HskLevel>,
}

#[function_component(HskPicker)]
pub fn hsk_picker(props: &HskPickerProps) -> Html {
    let levels = props.level.system().levels();
    html! {
        <fieldset
            class="min-w-0"
            data-hsk-picker=""
        >
            <legend
                class="mb-3 text-lg font-semibold"
            >
                {"HSK level"}
            </legend>
            <div
                class="mb-3 flex flex-wrap gap-2"
            >
                {for HskSystem::ALL.into_iter().map(|system| {
                    let label = system.label();
                    let select=props.on_system.clone();
                    html! { <SetupChoice
                        label={label}
                        value={label}
                        selected={system==props.level.system()}
                        on_select={Callback::from(move |_| select.emit(system))}
                    /> }
                })}
            </div>
            <div
                class="flex flex-wrap gap-2"
            >
                {for levels.iter().copied().map(|level| {
                    let label = level.label();
                    let select=props.on_level.clone();
                    html! { <SetupChoice
                        label={label}
                        value={format!("level-{}",level.as_i32())}
                        selected={level==props.level}
                        on_select={Callback::from(move |_| select.emit(level))}
                    /> }
                })}
            </div>
        </fieldset>
    }
}
