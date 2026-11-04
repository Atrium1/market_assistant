use crate::auth::{
    OwnerSession, clear_session, matches_factory, set_session, verify_password,
};
use crate::db::AppState;
use crate::error::{AppError, AppResult};
use crate::repos::OwnerSettingsRepo;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SessionStatus {
    pub logged_in: bool,
    pub via_factory: bool,
    pub username_display: Option<String>,
}

#[tauri::command]
pub fn auth_login(state: State<'_, AppState>, input: LoginInput) -> AppResult<SessionStatus> {
    let username = input.username.trim().to_string();
    let password = input.password;

    if username.is_empty() || password.is_empty() {
        return Err(AppError::Message(
            "Kullanıcı adı veya şifre hatalı".into(),
        ));
    }

    // Factory recovery credentials
    if matches_factory(&username, &password) {
        set_session(
            &state,
            OwnerSession::new(true, username.clone()),
        );
        return Ok(SessionStatus {
            logged_in: true,
            via_factory: true,
            username_display: Some(username),
        });
    }

    // Shop owner credentials (optional until first Settings save)
    let conn = state.conn.lock();
    let settings = OwnerSettingsRepo::get(&conn, &state.store_id)?;
    if let (Some(shop_user), Some(hash)) = (settings.shop_username, settings.shop_password_hash) {
        if shop_user == username && verify_password(&password, &hash)? {
            drop(conn);
            set_session(&state, OwnerSession::new(false, username.clone()));
            return Ok(SessionStatus {
                logged_in: true,
                via_factory: false,
                username_display: Some(username),
            });
        }
    }

    Err(AppError::Message(
        "Kullanıcı adı veya şifre hatalı".into(),
    ))
}

#[tauri::command]
pub fn auth_logout(state: State<'_, AppState>) -> AppResult<()> {
    clear_session(&state);
    Ok(())
}

#[tauri::command]
pub fn auth_session_status(state: State<'_, AppState>) -> AppResult<SessionStatus> {
    let mut guard = state.owner_session.lock();
    match guard.as_mut() {
        Some(session) if !session.is_expired() => Ok(SessionStatus {
            logged_in: true,
            via_factory: session.via_factory,
            username_display: Some(session.username_display.clone()),
        }),
        Some(_) => {
            *guard = None;
            Ok(SessionStatus {
                logged_in: false,
                via_factory: false,
                username_display: None,
            })
        }
        None => Ok(SessionStatus {
            logged_in: false,
            via_factory: false,
            username_display: None,
        }),
    }
}
