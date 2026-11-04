use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use zipeek_themes::{StaticBackgroundProvider, ThemeRequest, resolve_theme};

#[derive(Properties, PartialEq)]
pub struct ThemedBackgroundProps {
    pub categories: Vec<String>,
    #[prop_or_default]
    pub seed: u64,
}

#[function_component(ThemedBackground)]
pub fn themed_background(props: &ThemedBackgroundProps) -> Html {
    let asset = use_state(|| None::<zipeek_themes::BackgroundAsset>);
    {
        let categories = props.categories.clone();
        let seed = props.seed;
        let asset = asset.clone();
        use_effect_with((categories, seed), move |(categories, seed)| {
            let request = ThemeRequest { categories: categories.clone(), seed: *seed };
            let theme = resolve_theme(&request);
            let alive = std::rc::Rc::new(std::cell::Cell::new(true));
            let pending = alive.clone();
            asset.set(None);
            spawn_local(async move {
                let loaded = StaticBackgroundProvider.load(theme).await;
                if pending.get() {
                    asset.set(Some(loaded));
                }
            });
            move || alive.set(false)
        });
    }
    let image = asset.as_ref().map(|asset| asset.path).unwrap_or("public/background.webp");
    let theme = asset.as_ref().map(|asset| asset.theme.code()).unwrap_or("neutral");
    html! {
        <div data-component="themed-background" data-theme={theme} aria-hidden="true" class="pointer-events-none absolute inset-0 -z-10 overflow-hidden rounded-2xl">
            <img src={image} alt="" class="h-full w-full object-cover opacity-15 motion-reduce:transition-none" />
            <div class="absolute inset-0 bg-surface/70" />
        </div>
    }
}
