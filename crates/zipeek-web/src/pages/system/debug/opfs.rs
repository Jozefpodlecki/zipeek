use js_sys::{Array, Function, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::Navigator;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::hooks::use_app_environment;

const MAX_ENTRIES: usize = 500;

#[derive(Clone, PartialEq)]
struct Entry {
    name: String,
    kind: String,
}

#[derive(Clone, PartialEq)]
enum Listing {
    Idle,
    Loading,
    Ready(Vec<Entry>, bool),
    Failed(String),
}

fn property(value: &JsValue, name: &str) -> Result<JsValue, String> {
    Reflect::get(value, &JsValue::from_str(name))
        .map_err(|error| format!("Unable to access {name}: {error:?}"))
}

fn call(value: &JsValue, name: &str) -> Result<JsValue, String> {
    let function: Function = property(value, name)?
        .dyn_into()
        .map_err(|_| format!("Browser does not support {name}"))?;
    function
        .apply(value, &Array::new())
        .map_err(|error| format!("{name} failed: {error:?}"))
}

async fn invoke(value: &JsValue, name: &str) -> Result<JsValue, String> {
    let response = call(value, name)?;
    JsFuture::from(Promise::resolve(&response))
        .await
        .map_err(|error| format!("{name} failed: {error:?}"))
}

async fn list_root(navigator: &Navigator) -> Result<(Vec<Entry>, bool), String> {
    let storage = property(navigator.as_ref(), "storage")?;
    if storage.is_null() || storage.is_undefined() {
        return Err("Origin-private file storage is unavailable".into());
    }
    let root = invoke(&storage, "getDirectory").await?;
    let iterator = call(&root, "entries")?;
    let mut entries = Vec::new();
    loop {
        let result = invoke(&iterator, "next").await?;
        if property(&result, "done")?.as_bool() == Some(true) {
            entries.sort_by(|a: &Entry, b: &Entry| a.name.cmp(&b.name));
            return Ok((entries, false));
        }
        if entries.len() == MAX_ENTRIES {
            entries.sort_by(|a: &Entry, b: &Entry| a.name.cmp(&b.name));
            return Ok((entries, true));
        }
        let pair = Array::from(&property(&result, "value")?);
        if pair.length() != 2 {
            return Err("Unexpected OPFS directory entry".into());
        }
        let name = pair.get(0).as_string().ok_or("Invalid OPFS filename")?;
        let kind = property(&pair.get(1), "kind")?
            .as_string()
            .ok_or("Invalid OPFS entry kind")?;
        entries.push(Entry { name, kind });
    }
}

#[function_component(OpfsInspector)]
pub(super) fn opfs_inspector() -> Html {
    let environment = use_app_environment();
    let listing = use_state(|| Listing::Idle);
    let request = use_mut_ref(|| 0_u64);
    {
        let listing = listing.clone();
        let request = request.clone();
        let navigator = environment.navigator.clone();
        use_effect_with((), move |_| {
            let id = {
                let mut current = request.borrow_mut();
                *current += 1;
                *current
            };
            listing.set(Listing::Loading);
            let pending = request.clone();
            let state = listing.clone();
            spawn_local(async move {
                let result = list_root(&navigator).await;
                if *pending.borrow() == id {
                    state.set(match result {
                        Ok((entries, truncated)) => Listing::Ready(entries, truncated),
                        Err(message) => Listing::Failed(message),
                    });
                }
            });
            move || *request.borrow_mut() += 1
        });
    }
    let refresh = {
        let navigator = environment.navigator.clone();
        let listing = listing.clone();
        let request = request.clone();
        Callback::from(move |_: MouseEvent| {
            let id = {
                let mut current = request.borrow_mut();
                *current += 1;
                *current
            };
            listing.set(Listing::Loading);
            let state = listing.clone();
            let pending = request.clone();
            let navigator = navigator.clone();
            spawn_local(async move {
                let result = list_root(&navigator).await;
                if *pending.borrow() == id {
                    state.set(match result {
                        Ok((entries, truncated)) => Listing::Ready(entries, truncated),
                        Err(message) => Listing::Failed(message),
                    });
                }
            });
        })
    };
    let loading = matches!(&*listing, Listing::Loading);
    html! {
        <section data-debug-opfs="" class="space-y-4 rounded-xl border border-line bg-raised p-4">
            <div class="flex items-center justify-between gap-3">
                <h3 class="font-semibold">{"Origin-private files"}</h3>
                <button type="button" aria-label="Refresh OPFS files" title="Refresh" onclick={refresh} disabled={loading}
                    class="inline-flex size-10 items-center justify-center rounded-lg border border-line focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50">
                    <Icon data={IconData::LUCIDE_REFRESH_CW} width="18" height="18" class={if loading {"animate-spin motion-reduce:animate-none"} else {""}} />
                </button>
            </div>
            {match &*listing {
                Listing::Idle | Listing::Loading => html! { <p role="status" class="text-sm text-muted">{"Reading files…"}</p> },
                Listing::Failed(message) => html! { <p role="alert" class="text-sm text-red-400">{message}</p> },
                Listing::Ready(entries, _) if entries.is_empty() => html! { <p role="status" class="text-sm text-muted">{"No files or folders in the OPFS root."}</p> },
                Listing::Ready(entries, truncated) => html! {
                    <div class="space-y-2">
                        <p role="status" class="text-xs text-muted">{format!("{} entries{}", entries.len(), if *truncated {" (first 500)"} else {""})}</p>
                        <div class="max-h-[28rem] overflow-auto rounded-lg border border-line">
                            <table class="w-full text-left text-sm">
                                <thead class="sticky top-0 bg-raised"><tr><th class="p-3">{"Name"}</th><th class="p-3">{"Type"}</th></tr></thead>
                                <tbody>{for entries.iter().map(|entry| html! {
                                    <tr key={entry.name.clone()} class="border-t border-line">
                                        <td class="break-all p-3 font-mono">{&entry.name}</td>
                                        <td class="p-3 text-muted">{if entry.kind == "directory" {"Folder"} else {"File"}}</td>
                                    </tr>
                                })}</tbody>
                            </table>
                        </div>
                    </div>
                },
            }}
        </section>
    }
}
