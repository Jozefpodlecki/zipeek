mod database;
mod error;
mod services;

pub fn generate_handlers() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static
{
    tauri::generate_handler![
        services::start_service,
        services::stop_service,
        services::service_status,
        database::database_open,
        database::database_query,
        database::history_commit,
        database::history_rows,
        database::history_erase,
        database::personal_data_reset,
    ]
}
