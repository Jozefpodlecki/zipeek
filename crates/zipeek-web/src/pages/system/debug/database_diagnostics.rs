use std::{cell::Cell, rc::Rc};
use yew::prelude::*;

use crate::{
    contexts::database::DatabaseStatus,
    hooks::{use_app_environment, use_database_status, use_dictionary},
    services::database::StorageStatistics,
};

#[function_component(DatabaseDiagnostics)]
pub(super) fn database_diagnostics() -> Html {
    let dictionary = use_dictionary();
    let environment = use_app_environment();
    let probe = use_state(|| None::<Result<zipeek_db::wasm::ProbeReport, String>>);
    let actual_tags = use_state(|| None::<Result<String, String>>);
    {
        let probe = probe.clone();
        let document = environment.document.clone();
        let is_tauri = environment.is_tauri;
        use_effect_with((), move |_| {
            let active = Rc::new(Cell::new(true));
            if !is_tauri {
                let pending = active.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = zipeek_db::wasm::PersonalDbProbe::run(&document)
                        .await
                        .map_err(|error| error.to_string());
                    if pending.get() {
                        probe.set(Some(result));
                    }
                });
            }
            move || active.set(false)
        });
    }
    let status = use_database_status();
    let statistics = use_state(|| None::<Result<StorageStatistics, String>>);

    {
        let statistics = statistics.clone();
        let actual_tags = actual_tags.clone();
        use_effect_with(status.clone(), move |status| {
            let active = Rc::new(Cell::new(true));
            let pending = active.clone();
            match status {
                DatabaseStatus::Loading => {
                    statistics.set(None);
                    actual_tags.set(None);
                }
                DatabaseStatus::Failed(message) => {
                    statistics.set(Some(Err(format!("Dictionary unavailable: {message}"))));
                    actual_tags.set(None);
                }
                DatabaseStatus::Ready(_) => {
                    statistics.set(None);
                    actual_tags.set(None);
                    wasm_bindgen_futures::spawn_local(async move {
                        let (result, tags) = match dictionary.database() {
                            Ok(database) => (
                                database
                                    .storage_statistics()
                                    .await
                                    .map_err(|error| error.to_string()),
                                database
                                    .actual_storage_tags()
                                    .await
                                    .map_err(|error| error.to_string()),
                            ),
                            Err(error) => (Err(error.to_string()), Err(error.to_string())),
                        };
                        if pending.get() {
                            statistics.set(Some(result));
                            actual_tags.set(Some(tags));
                        }
                    });
                }
            }
            move || active.set(false)
        });
    }

    let manifest_details = match &status {
        DatabaseStatus::Ready(info) => html! {
            <dl class="mt-3 grid grid-cols-2 gap-2 text-sm">
                <dt>{if environment.is_tauri { "Native DuckDB engine" } else { "Browser DuckDB engine" }}</dt>
                <dd>{&info.engine}</dd>
                <dt>{"Generator DuckDB engine"}</dt>
                <dd>{&info.native_engine}</dd>
                <dt>{"Dictionary storage version"}</dt>
                <dd>{match &*actual_tags {
                    Some(Ok(tags)) => extract_storage_version(tags).unwrap_or("Unknown"),
                    Some(Err(_)) => "Unavailable",
                    None => "Loading...",
                }}</dd>
                <dt>{if environment.is_tauri { "Native storage version" } else { "WASM supported storage version" }}</dt>
                <dd role="status">{if environment.is_tauri {
                    info.storage_compatibility.as_str()
                } else {
                    match &*probe {
                        None => "Checking...",
                        Some(Ok(report)) => extract_storage_version(&report.storage_tags).unwrap_or("Unknown"),
                        Some(Err(_)) => "Check failed",
                    }
                }}</dd>
                <dt>{"Dictionary words"}</dt>
                <dd>{info.row_count}</dd>
            </dl>
        },
        _ => Html::default(),
    };

    let storage_details = match &*statistics {
        None => html! { <p role="status">{"Reading database metadata..."}</p> },
        Some(Err(error)) => html! {
            <p role="status" class="break-all">
                {format!("DuckDB storage metadata unavailable: {error}")}
            </p>
        },
        Some(Ok(info)) => html! {
            <dl data-duckdb-statistics="" class="mt-3 grid grid-cols-2 gap-2 text-sm">
                <dt>{"Database size"}</dt><dd>{&info.database_size}</dd>
                <dt>{"Block size (bytes)"}</dt><dd>{&info.block_size}</dd>
                <dt>{"Total blocks"}</dt><dd>{&info.total_blocks}</dd>
                <dt>{"Used blocks"}</dt><dd>{&info.used_blocks}</dd>
                <dt>{"Free blocks"}</dt><dd>{&info.free_blocks}</dd>
                <dt>{"WAL size"}</dt><dd>{&info.wal_size}</dd>
            </dl>
        },
    };

    html! {
        <section class="rounded-xl border border-line bg-raised p-4" data-duckdb-diagnostics="">
            <h3 class="font-semibold">{"Loaded dictionary storage"}</h3>
            <p class="mt-1 text-xs text-muted">
                {if environment.is_tauri { "Live native DuckDB metadata from the desktop dictionary." } else { "Live storage metadata from the loaded dictionary and the browser DuckDB runtime." }}
            </p>
            {manifest_details}
            {storage_details}
            {if let Some(Err(error)) = &*actual_tags { html! {
                <p role="alert" class="mt-2 break-words text-sm text-red-300">{format!("Unable to query actual dictionary storage version: {error}")}</p>
            }} else { Html::default() }}
            {if let Some(Err(error)) = &*probe { html! {
                <p role="alert" class="mt-2 break-words text-sm text-red-300">{format!("WASM storage check failed: {error}")}</p>
            }} else { Html::default() }}
        </section>
    }
}

fn extract_storage_version(tags: &str) -> Option<&str> {
    let index = tags.find("storage_version")?;
    let tail = &tags[index + "storage_version".len()..];
    let start = tail.find('v')?;
    let version = &tail[start..];
    let len = version
        .bytes()
        .take_while(|byte| byte.is_ascii_digit() || *byte == b'v' || *byte == b'.' || *byte == b'+')
        .count();
    (len > 1).then_some(&version[..len])
}

#[cfg(test)]
mod tests {
    use super::extract_storage_version;

    #[test]
    fn duckdb_storage_tags_display_only_the_version() {
        assert_eq!(
            extract_storage_version("{storage_version=v2.0.0+}"),
            Some("v2.0.0+")
        );
        assert_eq!(
            extract_storage_version("{'storage_version': 'v1.4.0+'}"),
            Some("v1.4.0+")
        );
        assert_eq!(extract_storage_version("{}"), None);
    }
}
