use yew::prelude::*;
use zipeek_themes::{ThemeRequest, resolve_theme};

use crate::appearance::Appearance;

#[derive(Properties, PartialEq)]
pub struct ThemedBackgroundProps {
    pub categories: Vec<String>,
    #[prop_or_default]
    pub seed: u64,
}

#[function_component(ThemedBackground)]
pub fn themed_background(props: &ThemedBackgroundProps) -> Html {
    let appearance =
        use_context::<Appearance>().expect("Appearance provider missing from themed background");
    let theme = resolve_theme(&ThemeRequest {
        categories: props.categories.clone(),
        seed: props.seed,
    });
    let image = format!("public/backgrounds/{}", appearance.preset.background());

    html! {
        <div
            data-component="themed-background"
            data-theme={theme.code()}
            aria-hidden="true"
            class="pointer-events-none absolute inset-0 -z-10 overflow-hidden rounded-2xl"
        >
            <img
                src={image}
                alt=""
                class="h-full w-full object-cover opacity-15 motion-reduce:transition-none"
            />
            <div
                class="absolute inset-0 bg-surface/70"
            />
        </div>
    }
}
