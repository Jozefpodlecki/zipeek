use web_sys::SpeechSynthesisUtterance;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

#[derive(Properties, PartialEq)]
pub(crate) struct ListenButtonProps {
    pub hanzi: AttrValue,
}

#[function_component(ListenButton)]
pub(crate) fn listen_button(props: &ListenButtonProps) -> Html {
    let failed = use_state(|| false);
    let onclick = {
        let hanzi = props.hanzi.clone();
        let failed = failed.clone();
        Callback::from(move |_| {
            let result = web_sys::window()
                .ok_or_else(|| wasm_bindgen::JsValue::from_str("Window unavailable"))
                .and_then(|window| window.speech_synthesis())
                .and_then(|speech| {
                    let utterance = SpeechSynthesisUtterance::new_with_text(&hanzi)?;
                    utterance.set_lang("zh-CN");
                    speech.cancel();
                    speech.speak(&utterance);
                    Ok(())
                });
            if let Err(error) = &result {
                log::warn!("Pronunciation unavailable: {error:?}");
            }
            failed.set(result.is_err());
        })
    };
    html! {
        <span class="inline-flex items-center gap-2">
            <button type="button" data-listen="" aria-label="Listen" title="Listen" class="grid size-11 shrink-0 place-items-center rounded-lg text-accent hover:bg-raised" {onclick}><span aria-hidden="true"><Icon data={IconData::LUCIDE_VOLUME_2} width="20px" height="20px" /></span></button>
            {if *failed {html!{<span class="text-sm text-muted">{"Audio unavailable"}</span>}} else {Html::default()}}
        </span>
    }
}
