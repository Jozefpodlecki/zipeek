use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::theme::ThemeContext;

#[function_component(ThemeToggle)]
pub fn theme_toggle() -> Html {
    let Some(theme) = use_context::<ThemeContext>() else {
        return Html::default();
    };
    let dark = theme.current().is_dark();
    let onclick = Callback::from(move |_| theme.toggle());
    html! {
        <button type="button" class="theme-toggle" data-theme-toggle="" role="switch"
            aria-label="Dark mode" aria-checked={dark.to_string()} {onclick}>
            <span aria-hidden="true"><Icon data={if dark { IconData::LUCIDE_MOON } else { IconData::LUCIDE_SUN }} width="18px" height="18px" /></span>
            <span class="theme-label">{if dark { "Dark" } else { "Light" }}</span>
            <span class={classes!("theme-switch", dark.then_some("is-dark"))} aria-hidden="true"><span /></span>
        </button>
    }
}
