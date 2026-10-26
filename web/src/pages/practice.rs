use yew::prelude::*;

use crate::components::Loader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    EngPinyin,
    PinyinHanzi,
    HanziEng,
}

#[derive(Properties, PartialEq)]
pub struct PracticeProps {
    pub variant: Variant,
}

#[function_component(Practice)]
pub fn practice(props: &PracticeProps) -> Html {
    let _ = props.variant;
    html! {
        <Loader />
    }
}
