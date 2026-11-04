use crate::auth::{hash_password, require_owner};
use crate::db::AppState;
use crate::error::{AppError, AppResult};
use crate::repos::owner_settings::SettingsPublic;
use crate::repos::OwnerSettingsRepo;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct SetStoreNameInput {
    pub store_display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SetOwnerCredentialsInput {
    pub username: String,
    pub password: String,
    pub password_confirm: String,
}

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> AppResult<SettingsPublic> {
    require_owner(&state)?;
    let conn = state.conn.lock();
    OwnerSettingsRepo::get_public(&conn, &state.store_id)
}

#[tauri::command]
pub fn settings_set_store_name(
    state: State<'_, AppState>,
    input: SetStoreNameInput,
) -> AppResult<()> {
    require_owner(&state)?;
    let conn = state.conn.lock();
    OwnerSettingsRepo::set_store_name(&conn, &state.store_id, &input.store_display_name)
}

#[tauri::command]
pub fn settings_set_owner_credentials(
    state: State<'_, AppState>,
    input: SetOwnerCredentialsInput,
) -> AppResult<()> {
    require_owner(&state)?;

    if input.password != input.password_confirm {
        return Err(AppError::Message("Şifreler eşleşmiyor".into()));
    }
    if input.password.len() < 4 {
        return Err(AppError::Message(
            "Şifre en az 4 karakter olmalı".into(),
        ));
    }

    let password_hash = hash_password(&input.password)?;
    let conn = state.conn.lock();
    OwnerSettingsRepo::set_credentials(
        &conn,
        &state.store_id,
        &input.username,
        &password_hash,
    )
}
