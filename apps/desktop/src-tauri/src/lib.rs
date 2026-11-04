mod auth;
mod backup;
mod commands;
mod db;
mod domain;
mod error;
mod import_export;
mod repos;

use db::{open_and_migrate, resolve_db_path};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Uygulama veri yolu alınamadı: {e}"))?;
            let db_path = resolve_db_path(app_data_dir);
            let state = open_and_migrate(db_path).map_err(|e| e.to_string())?;
            {
                let conn = state.conn.lock();
                let dir = backup::backup_dir(&state.db_path);
                if let Err(e) = backup::auto_backup_on_start(&conn, &dir) {
                    eprintln!("Otomatik yedek alınamadı: {e}");
                }
            }
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
            commands::sales_list,
            commands::sales_get,
            commands::sales_void,
            commands::reports_daily,
            commands::reports_range,
            commands::reports_today_summary,
            commands::returns_create,
            commands::auth_login,
            commands::auth_logout,
            commands::auth_session_status,
            commands::settings_get,
            commands::settings_set_store_name,
            commands::settings_set_owner_credentials,
            commands::backup_info,
            commands::backup_create,
            commands::backup_inspect,
            commands::backup_restore,
            commands::products_import_preview,
            commands::products_import_apply,
            commands::products_export,
            commands::products_export_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
