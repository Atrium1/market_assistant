use crate::auth::require_owner;
use crate::backup::{self, PRE_IMPORT_PREFIX};
use crate::db::AppState;
use crate::error::{AppError, AppResult};
use crate::import_export::classify::RowKind;
use crate::import_export::{self, export, ImportPreview, ImportResult};
use crate::repos::ProductRepo;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn products_import_preview(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<ImportPreview> {
    require_owner(&state)?;
    let path = PathBuf::from(path);
    let rows = {
        let conn = state.conn.lock();
        import_export::analyze(&conn, &state.store_id, &path)?
    };
    Ok(import_export::preview(&path, rows))
}

/// Re-reads the file so the result always matches what is on disk at apply time.
#[tauri::command]
pub async fn products_import_apply(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<ImportResult> {
    require_owner(&state)?;
    let path = PathBuf::from(path);
    let conn = state.conn.lock();
    let rows = import_export::analyze(&conn, &state.store_id, &path)?;
    if !rows
        .iter()
        .any(|r| matches!(r.kind, RowKind::Add | RowKind::Update))
    {
        return Err(AppError::Message(
            "Uygulanacak geçerli satır yok".into(),
        ));
    }
    backup::safety_backup(&conn, &backup::backup_dir(&state.db_path), PRE_IMPORT_PREFIX)?;
    import_export::apply(&conn, &state.store_id, &rows)
}

#[tauri::command]
pub async fn products_export(state: State<'_, AppState>, dest_path: String) -> AppResult<usize> {
    require_owner(&state)?;
    let products = {
        let conn = state.conn.lock();
        ProductRepo::list_active(&conn, &state.store_id)?
    };
    export::write_products(&PathBuf::from(dest_path), &products)?;
    Ok(products.len())
}

#[tauri::command]
pub async fn products_export_template(
    state: State<'_, AppState>,
    dest_path: String,
) -> AppResult<()> {
    require_owner(&state)?;
    export::write_template(&PathBuf::from(dest_path))
}
