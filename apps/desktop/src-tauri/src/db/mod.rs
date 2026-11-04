mod migrate;

use crate::error::{AppError, AppResult};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;

pub const DEFAULT_STORE_ID: &str = "store_default";
pub const DEFAULT_STORE_NAME: &str = "Varsayılan Market";

#[derive(Clone)]
pub struct AppState {
    pub conn: Arc<Mutex<Connection>>,
    pub store_id: String,
    #[allow(dead_code)]
    pub db_path: PathBuf,
}

pub fn resolve_db_path(app_data_dir: PathBuf) -> PathBuf {
    app_data_dir.join("market_assistant.db")
}

pub fn open_and_migrate(db_path: PathBuf) -> AppResult<AppState> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::Message(format!("Uygulama veri klasörü oluşturulamadı: {e}"))
        })?;
    }

    let conn = Connection::open(&db_path)?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;",
    )?;

    migrate::run(&conn)?;
    seed_default_store(&conn)?;

    Ok(AppState {
        conn: Arc::new(Mutex::new(conn)),
        store_id: DEFAULT_STORE_ID.to_string(),
        db_path,
    })
}

fn seed_default_store(conn: &Connection) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO stores (id, name, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![DEFAULT_STORE_ID, DEFAULT_STORE_NAME, now],
    )?;
    Ok(())
}
