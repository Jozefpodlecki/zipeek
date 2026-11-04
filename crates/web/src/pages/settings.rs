use web_sys::HtmlSelectElement;
use yew::prelude::*;

const KEY: &str = "zipeek.chinese_font";

fn stored_font() -> String {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(KEY).ok().flatten())
        .filter(|value| matches!(value.as_str(), "system" | "sans" | "serif" | "calligraphy"))
        .unwrap_or_else(|| "system".to_owned())
}

#[function_component(SettingsPage)]
pub fn settings_page() -> Html {
    let font = use_state(stored_font);
    let on_font = {
        let font = font.clone();
        Callback::from(move |event: Event| {
            let selected = event.target_unchecked_into::<HtmlSelectElement>().value();
            if !matches!(selected.as_str(), "system" | "sans" | "serif" | "calligraphy") { return; }
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(root) = document.document_element() {
                        let _ = root.set_attribute("data-hanzi-font", &selected);
                    }
                }
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Err(error) = storage.set_item(KEY, &selected) {
                        log::warn!("Chinese font preference was not saved: {error:?}");
                    }
                }
            }
            font.set(selected);
        })
    };
    html! {
        <section data-component="settings-page" class="mx-auto flex w-full max-w-3xl flex-col gap-6 py-4">
            <header class="space-y-2">
                <h1 class="text-3xl font-bold tracking-tight">{"Settings"}</h1>
                <p class="text-sm text-muted">{"Personalize your reading experience. Changes apply across the app."}</p>
            </header>
            <div class="rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8">
                <h2 class="text-xl font-semibold">{"Chinese characters"}</h2>
                <p class="mt-2 text-sm text-muted">{"Choose a font for hanzi. Installed font availability depends on your device."}</p>
                <label for="hanzi-font" class="mt-6 block text-sm font-semibold">{"Character font"}</label>
                <select id="hanzi-font" data-hanzi-font-select="" value={(*font).clone()} onchange={on_font}
                    class="mt-2 min-h-11 w-full rounded-lg border border-line bg-raised px-3 text-ink">
                    <option value="system">{"System default"}</option>
                    <option value="sans">{"Sans-serif (modern)"}</option>
                    <option value="serif">{"Serif (traditional print)"}</option>
                    <option value="calligraphy">{"Calligraphy (Ma Shan Zheng)"}</option>
                </select>
                <div class="mt-6 rounded-xl border border-line bg-raised p-6 text-center">
                    <span class="block text-xs font-semibold uppercase tracking-widest text-muted">{"Preview"}</span>
                    <p lang="zh-Hans" class="mt-4 text-5xl font-medium leading-relaxed sm:text-6xl">{"你好世界 · 学习中文"}</p>
                    <p lang="zh-Hant" class="mt-3 text-2xl text-muted">{"繁體中文"}</p>
                </div>
            </div>
        </section>
    }
}
