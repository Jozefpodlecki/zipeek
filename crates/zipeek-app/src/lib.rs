#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

mod commands;
mod database;
mod services;
mod setup;

pub fn run() {
    tauri::Builder::default()
        .setup(setup::setup)
        .invoke_handler(commands::generate_handlers())
        .build(tauri::generate_context!())
        .expect("error while building zipeek")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                use tauri::Manager;
                app.state::<services::NativeServices>().shutdown();
            }
        });
}
