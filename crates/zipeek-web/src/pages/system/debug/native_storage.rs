use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconData};

use crate::services::tauri_database::{self, PersonalStorageInfo};

#[derive(Clone, PartialEq)]
enum NativeStorageState {
    Loading,
    Ready(PersonalStorageInfo),
    Failed(String),
}

#[function_component(NativeStorageInspector)]
pub(super) fn native_storage_inspector() -> Html {
    let state = use_state(|| NativeStorageState::Loading);
    let action_error = use_state(|| None::<String>);
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                state.set(match tauri_database::personal_storage_info().await {
                    Ok(info) => NativeStorageState::Ready(info),
                    Err(error) => NativeStorageState::Failed(error),
                });
            });
            || ()
        });
    }
    let refresh = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.set(NativeStorageState::Loading);
            let state = state.clone();
            spawn_local(async move {
                state.set(match tauri_database::personal_storage_info().await {
                    Ok(info) => NativeStorageState::Ready(info),
                    Err(error) => NativeStorageState::Failed(error),
                });
            });
        })
    };
    let reveal = {
        let action_error = action_error.clone();
        Callback::from(move |_: MouseEvent| {
            let action_error = action_error.clone();
            action_error.set(None);
            spawn_local(async move {
                if let Err(error) = tauri_database::personal_storage_reveal().await {
                    action_error.set(Some(error));
                }
            });
        })
    };
    html! {
        <section data-debug-native-storage="" class="space-y-3 rounded-xl border border-line bg-raised p-4">
            <div class="flex items-center justify-between gap-3">
                <h3 class="font-semibold">{"Personal database (desktop)"}</h3>
                <button type="button" onclick={refresh} aria-label="Refresh native database details"
                    class="inline-flex size-10 items-center justify-center rounded-lg border border-line">
                    <Icon data={IconData::LUCIDE_REFRESH_CW} width="18" height="18" />
                </button>
            </div>
            {match &*state {
                NativeStorageState::Loading => html! { <p role="status" class="text-sm text-muted">{"Reading database location…"}</p> },
                NativeStorageState::Failed(error) => html! { <p role="alert" class="text-sm text-red-400">{error}</p> },
                NativeStorageState::Ready(info) => html! {
                    <div class="space-y-3 text-sm">
                        <dl class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-2">
                            <dt class="text-muted">{"Path"}</dt>
                            <dd class="break-all font-mono text-xs" data-personal-db-path="">{&info.path}</dd>
                            <dt class="text-muted">{"Status"}</dt>
                            <dd>{if info.exists { "Present" } else { "Not created yet" }}</dd>
                            <dt class="text-muted">{"Size"}</dt>
                            <dd data-personal-db-size="">{info.size_bytes.map(|bytes| format!("{bytes} bytes ({:.2} MiB)", bytes as f64 / 1_048_576.0)).unwrap_or_else(|| "Not available".into())}</dd>
                        </dl>
                        <button type="button" onclick={reveal}
                            class="inline-flex min-h-10 items-center gap-2 rounded-lg border border-line px-3 font-semibold hover:bg-surface">
                            <Icon data={IconData::LUCIDE_FOLDER_OPEN} width="18" height="18" />
                            {"Show in file manager"}
                        </button>
                    </div>
                },
            }}
            {if let Some(error) = &*action_error {
                html! { <p role="alert" class="text-sm text-red-400">{error}</p> }
            } else { Html::default() }}
        </section>
    }
}
