mod ai;
mod database;
mod error;
mod image;
mod native_http;
mod services;
mod speech;

pub fn generate_handlers() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static
{
    tauri::generate_handler![
        image::image_generate,
        image::image_health,
        speech::speech_request,
        ai::ai_models,
        ai::ai_complete,
        services::start_service,
        services::stop_service,
        services::service_status,
        database::database_open,
        database::personal_storage_info,
        database::personal_storage_reveal,
        database::database_query,
        database::history_commit,
        database::history_rows,
        database::history_erase,
        database::personal_data_reset,
    ]
}
