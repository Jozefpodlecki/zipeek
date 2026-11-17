use std::cell::Cell;
use std::rc::Rc;

use gloo::net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use zipeek_docs::{DocumentEntry, is_safe_path};

use super::super::navigation::asset_url;

#[derive(Clone, PartialEq)]
pub(in crate::pages::system::docs) enum LoadState<T> {
    Loading,
    Ready(T),
    Error(String),
}

async fn load_index() -> Result<Vec<DocumentEntry>, String> {
    let url = asset_url("docs/index.json")?;
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.ok() {
        return Err(format!("Documentation index HTTP {}", response.status()));
    }
    let items = response
        .json::<Vec<DocumentEntry>>()
        .await
        .map_err(|error| error.to_string())?;
    Ok(items
        .into_iter()
        .filter(|item| is_safe_path(&item.path))
        .collect())
}

async fn load_document(path: &str) -> Result<String, String> {
    if !is_safe_path(path) || !path.ends_with(".md") {
        return Err("Select a Markdown document from the explorer.".into());
    }
    let url = asset_url(&format!("docs/{path}"))?;
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.ok() {
        return Err(format!("Document HTTP {}", response.status()));
    }
    response.text().await.map_err(|error| error.to_string())
}

#[hook]
pub(in crate::pages::system::docs) fn use_index() -> UseStateHandle<LoadState<Vec<DocumentEntry>>> {
    let state = use_state(|| LoadState::Loading);
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            let active = Rc::new(Cell::new(true));
            let task_active = active.clone();
            spawn_local(async move {
                let result = load_index().await;
                if task_active.get() {
                    state.set(match result {
                        Ok(items) => LoadState::Ready(items),
                        Err(error) => LoadState::Error(error),
                    });
                }
            });
            move || active.set(false)
        });
    }
    state
}

#[hook]
pub(in crate::pages::system::docs) fn use_document(
    path: String,
) -> UseStateHandle<LoadState<String>> {
    let state = use_state(|| LoadState::Loading);
    {
        let state = state.clone();
        use_effect_with(path, move |path| {
            state.set(LoadState::Loading);
            let active = Rc::new(Cell::new(true));
            let task_active = active.clone();
            let path = path.clone();
            spawn_local(async move {
                let result = load_document(&path).await;
                if task_active.get() {
                    state.set(match result {
                        Ok(text) => LoadState::Ready(text),
                        Err(error) => LoadState::Error(error),
                    });
                }
            });
            move || active.set(false)
        });
    }
    state
}
