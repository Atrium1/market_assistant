use crate::error::{AppError, AppResult};

/// Stub for future returns workflow. UI can call this and show "Yakında".
#[tauri::command]
pub fn returns_create() -> AppResult<()> {
    Err(AppError::NotImplemented)
}
