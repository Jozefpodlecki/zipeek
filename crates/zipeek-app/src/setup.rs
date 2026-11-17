use tauri::Manager;

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(crate::services::NativeServices::new());
    app.manage(crate::database::NativeDatabase::new(&app.handle()));
    Ok(())
}
