use crate::components::SetupChoice;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use zipeek_core::ExerciseVariant;

#[derive(Properties, PartialEq)]
pub struct ActivityPickerProps {
    pub selected: ExerciseVariant,
    pub on_select: Callback<ExerciseVariant>,
}

#[function_component(ActivityPicker)]
pub fn activity_picker(props: &ActivityPickerProps) -> Html {
    html! {
        <fieldset class="min-w-0" data-activity-picker="">
            <legend class="mb-3 text-lg font-semibold">{"Activity"}</legend>
            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                {for [
                    (ExerciseVariant::HanziToMeaning, "Hanzi to meaning"),
                    (ExerciseVariant::MeaningToHanzi, "Meaning to hanzi"),
                    (ExerciseVariant::MeaningToPinyin, "Meaning to pinyin"),
                    (ExerciseVariant::EngPinyin, "English to pinyin"),
                    (ExerciseVariant::PinyinHanzi, "Pinyin to hanzi"),
                    (ExerciseVariant::HanziEng, "Hanzi to English"),
                ].into_iter().map(|(variant,label)| {
                    let select = props.on_select.clone();
                    html! { <SetupChoice label={label} selected={variant == props.selected} value={variant.seed_token()} on_select={Callback::from(move |_| select.emit(variant))}>
                        <span class="flex items-center gap-2 text-sm text-muted"><span aria-hidden="true"><Icon data={if variant.is_mcq() {IconData::LUCIDE_GRADUATION_CAP} else {IconData::LUCIDE_SHUFFLE}} width="18px" height="18px" /></span>{if variant.is_mcq() {"Pick the right answer"} else {"Match four pairs"}}</span>
                    </SetupChoice> }
                })}
            </div>
        </fieldset>
    }
}
