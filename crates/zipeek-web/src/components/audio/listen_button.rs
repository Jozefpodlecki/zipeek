use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::hooks::use_app_environment;

#[derive(Properties, PartialEq)]
pub(crate) struct ListenButtonProps {
    pub hanzi: AttrValue,
    #[prop_or_default]
    pub compact: bool,
}

#[function_component(ListenButton)]
pub(crate) fn listen_button(props: &ListenButtonProps) -> Html {
    let environment = use_app_environment();
    let speech = environment.speech.clone();
    let failed = use_state(|| false);
    let onclick = {
        let hanzi = props.hanzi.clone();
        let failed = failed.clone();
        Callback::from(move |_| {
            let result = crate::services::practice_audio::speak(&speech, &hanzi);
            if let Err(error) = &result {
                log::warn!("Pronunciation unavailable: {error:?}");
            }
            failed.set(result.is_err());
        })
    };
    let audio_failure_notice = if *failed {
        html! {
        <span
            class="text-sm text-muted"
        >
            {"Audio unavailable"}
        </span>}
    } else {
        Html::default()
    };
    html! {
        <span
            class="inline-flex items-center gap-2"
        >
            <button
                type="button"
                data-listen=""
                aria-label="Listen"
                title="Listen"
                class={if props.compact { "grid size-9 shrink-0 place-items-center rounded-lg text-accent hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent" } else { "grid size-11 shrink-0 place-items-center rounded-lg text-accent hover:bg-raised" }}
                {onclick}
            ><span
                    aria-hidden="true"
                ><Icon
                    data={IconData::LUCIDE_VOLUME_2}
                    width="18px"
                    height="18px"
                /></span></button>
            {audio_failure_notice}
        </span>
    }
}
