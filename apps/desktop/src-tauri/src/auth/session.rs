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
            Err(AppError::SessionExpired)
        }
        None => Err(AppError::OwnerRequired),
    }
}

pub fn clear_session(state: &AppState) {
    *state.owner_session.lock() = None;
}

pub fn set_session(state: &AppState, session: OwnerSession) {
    *state.owner_session.lock() = Some(session);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_and_migrate;
    use crate::domain::PaymentMethod;
    use crate::repos::SaleRepo;
    use crate::repos::sale::tests::{add_product, sell, tr};

    #[test]
    fn owner_gate_codes_and_void_flow() {
        let dir = tempfile::tempdir().unwrap();
        let state = open_and_migrate(dir.path().join("live.db")).unwrap();

        let err = require_owner(&state).unwrap_err();
        assert_eq!(err.code(), "owner_required");

        let mut stale = OwnerSession::new(false, "magaza".into());
        stale.last_active_at = Utc::now() - Duration::minutes(SESSION_IDLE_MINUTES + 1);
        set_session(&state, stale);
        assert_eq!(require_owner(&state).unwrap_err().code(), "session_expired");
        assert!(state.owner_session.lock().is_none());

        set_session(&state, OwnerSession::new(false, "magaza".into()));
        require_owner(&state).unwrap();
        let conn = state.conn.lock();
        let p = add_product(&conn, "111", 100);
        let sale = sell(&conn, &[(&p, 1, None)], PaymentMethod::Cash, tr(2026, 11, 4, 9, 0)).sale;
        SaleRepo::void(&conn, &state.store_id, &sale.id, "test", Utc::now()).unwrap();
    }
}
