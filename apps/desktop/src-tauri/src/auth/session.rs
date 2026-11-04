use crate::db::AppState;
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Duration, Utc};

pub const SESSION_IDLE_MINUTES: i64 = 15;

#[derive(Debug, Clone)]
pub struct OwnerSession {
    #[allow(dead_code)]
    pub logged_in_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub via_factory: bool,
    pub username_display: String,
}

impl OwnerSession {
    pub fn new(via_factory: bool, username_display: String) -> Self {
        let now = Utc::now();
        Self {
            logged_in_at: now,
            last_active_at: now,
            via_factory,
            username_display,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() - self.last_active_at > Duration::minutes(SESSION_IDLE_MINUTES)
    }

    pub fn touch(&mut self) {
        self.last_active_at = Utc::now();
    }
}

/// Ensure an active owner session; refresh last_active on success.
pub fn require_owner(state: &AppState) -> AppResult<()> {
    let mut guard = state.owner_session.lock();
    match guard.as_mut() {
        Some(session) if !session.is_expired() => {
            session.touch();
            Ok(())
        }
        Some(_) => {
            *guard = None;
            Err(AppError::Message(
                "Oturum süresi doldu. Lütfen tekrar giriş yapın.".into(),
            ))
        }
        None => Err(AppError::Message(
            "Bu işlem için sahip girişi gerekli.".into(),
        )),
    }
}

pub fn clear_session(state: &AppState) {
    *state.owner_session.lock() = None;
}

pub fn set_session(state: &AppState, session: OwnerSession) {
    *state.owner_session.lock() = Some(session);
}
