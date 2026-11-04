mod commands;
mod db;
mod domain;
mod error;
mod repos;

use db::{open_and_migrate, resolve_db_path};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Uygulama veri yolu alınamadı: {e}"))?;
            let db_path = resolve_db_path(app_data_dir);
            let state = open_and_migrate(db_path).map_err(|e| e.to_string())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::products_list,
            commands::products_get_by_barcode,
            commands::products_create,
            commands::products_update,
            commands::products_deactivate,
            commands::sales_checkout,
            commands::returns_create,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
