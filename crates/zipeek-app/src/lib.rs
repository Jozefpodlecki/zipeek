#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

mod commands;
mod database;
mod services;
mod setup;
mod shutdown;

pub fn run() {
    tauri::Builder::default()
        .manage(shutdown::Shutdown::default())
        .setup(setup::setup)
        .invoke_handler(commands::generate_handlers())
        .build(tauri::generate_context!())
        .expect("error while building zipeek")
        .run(shutdown::handle_event);
}
