use crate::font_preference::{
    CHARACTER_SIZES, HANZI_FONTS, character_size, font_name, stored_font,
};
use crate::hooks::use_app_environment;
use crate::preferences::PreferencesStore;
use yew::prelude::*;

#[function_component(CharacterSettings)]
pub(crate) fn character_settings() -> Html {
    let environment = use_app_environment();
    let initial_storage = environment.local_storage.clone();
    let font = use_state(move || stored_font(&initial_storage));
    let size_storage = environment.local_storage.clone();
    let character_size_value = use_state(move || {
        character_size(&PreferencesStore::new(&size_storage).read().character_size).to_owned()
    });
    let on_character_size = {
        let character_size_value = character_size_value.clone();
        let document = environment.browser_document.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |selected: String| {
            if character_size(&selected) != selected {
                return;
            }
            if let Err(error) = document.set_root_attribute("data-character-size", &selected) {
                log::warn!("Could not apply character size: {error:?}");
                return;
            }
            if let Err(error) = PreferencesStore::new(&storage).update(|settings| {
                settings.character_size = selected.clone();
            }) {
                log::warn!("Could not save character size: {error}");
                return;
            }
            character_size_value.set(selected);
        })
    };
    let on_font = {
        let font = font.clone();
        let browser_document = environment.browser_document.clone();
        let storage = environment.local_storage.clone();
        Callback::from(move |selected: String| {
            if !HANZI_FONTS.iter().any(|option| option.key == selected) {
                return;
            }
            if let Err(error) = browser_document.set_root_attribute("data-hanzi-font", &selected) {
                log::warn!("Could not apply Chinese font: {error:?}");
            }
            if let Err(error) = PreferencesStore::new(&storage).update(|settings| {
                settings.font = selected.clone();
            }) {
                log::warn!("Chinese font preference was not saved: {error}");
                return;
            }
            font.set(selected);
        })
    };
    html! {
    <div
                                               data-settings-characters=""
                                           >
                           <div
                               class="rounded-2xl border border-line bg-surface p-6 shadow-sm sm:p-8"
                           >
                               <h2
                                   class="text-xl font-semibold"
                               >
                                   {"Chinese characters"}
                               </h2>
                               <p
                                   class="mt-2 text-sm text-muted"
                               >
                                   {"Choose a Chinese typeface. Additional fonts require local installation; missing fonts use a readable fallback."}
                               </p>
                               <p
                                   id="hanzi-font-label"
                                   class="mt-6 block text-sm font-semibold"
                               >
                                   {"Character font"}
                               </p>
                               <div
                                   role="group"
                                   aria-labelledby="hanzi-font-label"
                                   data-hanzi-font-options=""
                                   class="mt-2 grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-5"
                               >
                                   {for HANZI_FONTS.iter().map(|option| {
                                       let value = option.key;
                                       let label = option.name;
                                       let sample = "你好";
                                       let selected = font.as_str() == value;
                                       let choose = on_font.clone();
                                       let on_choose_font = Callback::from(move |_| choose.emit(value.to_owned()));
                                       let element_classes_2 = classes!(
                                                       "flex",
                                                       "min-h-20",
                                                       "flex-col",
                                                       "items-center",
                                                       "justify-center",
                                                       "gap-1",
                                                       "rounded-xl",
                                                       "border",
                                                       "px-2",
                                                       "py-2",
                                                       "text-sm",
                                                       if selected {"border-accent bg-accent-soft text-ink ring-1 ring-accent/40"} else {"border-line bg-raised hover:border-accent"},
                                                   );
                                       html! {
                                           <button
                                               key={value}
                                               type="button"
                                               data-hanzi-font-option={value}
                                               aria-pressed={selected.to_string()}
                                               onclick={on_choose_font}
                                               class={element_classes_2}
                                           >
                                           <span
                                               lang="zh-Hans"
                                               data-font-preview={value}
                                               class="text-2xl"
                                           >
                                               {sample}
                                           </span><span
                                                   class="font-semibold"
                                               >
                                               {label}
                                               </span>
                                       </button>}
                                   })}
                               </div>
                               <p
                                   class="mt-2 text-xs text-muted"
                                   role="status"
                               >
                                   {format!("Selected: {}. This choice is saved in this browser.", font_name(&font))}
                               </p>
                               <fieldset
                                   class="mt-5 space-y-2"
                                   data-character-size-settings=""
                               >
                                   <legend class="text-sm font-semibold">
                                       {"Practice character size"}
                                   </legend>
                                   <div class="flex flex-wrap gap-2">
                                       {for CHARACTER_SIZES.iter().map(|(key, label)| {
                                           let selected = *key == character_size_value.as_str();
                                           let value = (*key).to_owned();
                                           let on_character_size = on_character_size.clone();
                                           let on_choose_size = Callback::from(move |_| {
                                               on_character_size.emit(value.clone());
                                           });
                                           let element_classes_3 = classes!(
                                                       "rounded-lg",
                                                       "border",
                                                       "px-4",
                                                       "py-2",
                                                       if selected {
                                                           "border-accent bg-accent-soft"
                                                       } else {
                                                           "border-line bg-raised"
                                                       }
                                                   );
                                           html! {
                                               <button
                                                   key={*key}
                                                   type="button"
                                                   data-character-size-option={*key}
                                                   aria-pressed={selected.to_string()}
                                                   class={element_classes_3}
                                                   onclick={on_choose_size}
                                               >
                                                   {*label}
                                               </button>
                                           }
                                       })}
                                   </div>
                                   <p class="text-xs text-muted">
                                       {"Changes the Hanzi tiles in Character Builder, including their selected slots."}
                                   </p>
                               </fieldset>
                               <div
                                   class="mt-6 rounded-xl border border-line bg-raised p-6 text-center"
                               >
                                   <span
                                       class="block text-xs font-semibold uppercase tracking-widest text-muted"
                                   >
                                       {"Preview"}
                                   </span>
                                   <p
                                       lang="zh-Hans"
                                       class="mt-4 text-5xl font-medium leading-relaxed sm:text-6xl"
                                   >
                                       {"你好世界 · 学习中文"}
                                   </p>
                                   <p
                                       lang="zh-Hant"
                                       class="mt-3 text-2xl text-muted"
                                   >
                                       {"繁體中文"}
                                   </p>
                               </div>
                           </div>
               </div>     }
}
