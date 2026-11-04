use crate::auth::{clear_session, require_owner};
use crate::backup::{self, BackupFile, BackupSummary};
use crate::db::AppState;
use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct BackupInfo {
    pub db_path: String,
    pub backup_dir: String,
    pub recent: Vec<BackupFile>,
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => a == b,
    }
}

#[tauri::command]
pub async fn backup_info(state: State<'_, AppState>) -> AppResult<BackupInfo> {
    require_owner(&state)?;
    let dir = backup::backup_dir(&state.db_path);
    Ok(BackupInfo {
        db_path: state.db_path.display().to_string(),
        backup_dir: dir.display().to_string(),
        recent: backup::list_recent(&dir)?,
    })
}

#[tauri::command]
pub async fn backup_create(state: State<'_, AppState>, dest_path: String) -> AppResult<()> {
    require_owner(&state)?;
    let dest = PathBuf::from(dest_path);
    if same_file(&dest, &state.db_path) {
        return Err(AppError::Message(
            "Yedek, kullanılan veritabanının üzerine yazılamaz".into(),
        ));
    }
    let conn = state.conn.lock();
    backup::create_to(&conn, &dest)
}

#[tauri::command]
pub async fn backup_inspect(
    state: State<'_, AppState>,
    src_path: String,
) -> AppResult<BackupSummary> {
    require_owner(&state)?;
    let src = PathBuf::from(src_path);
    if same_file(&src, &state.db_path) {
        return Err(AppError::Message(
            "Kullanılan veritabanı geri yükleme için seçilemez".into(),
        ));
    }
    backup::inspect(&src, &backup::backup_dir(&state.db_path))
}

#[tauri::command]
pub async fn backup_restore(
    state: State<'_, AppState>,
    src_path: String,
) -> AppResult<BackupSummary> {
    require_owner(&state)?;
    let src = PathBuf::from(src_path);
    if same_file(&src, &state.db_path) {
        return Err(AppError::Message(
            "Kullanılan veritabanı geri yükleme için seçilemez".into(),
        ));
    }
    let summary = {
        let mut conn = state.conn.lock();
        backup::restore_from(&mut conn, &src, &backup::backup_dir(&state.db_path))?
    };
    // Restored owner_settings may hold different shop credentials.
    clear_session(&state);
    Ok(summary)
}
