use crate::db::{known_versions, run_migrations, seed_default_store};
use crate::error::{AppError, AppResult};
use rusqlite::backup::Progress;
use rusqlite::{Connection, DatabaseName, OptionalExtension};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const AUTO_PREFIX: &str = "auto_";
pub const PRE_RESTORE_PREFIX: &str = "pre_restore_";
pub const PRE_IMPORT_PREFIX: &str = "pre_import_";
const AUTO_KEEP: usize = 7;
const SAFETY_KEEP: usize = 5;
const STAGING_NAME: &str = "_staging_restore.db";

#[derive(Debug, Clone, Serialize)]
pub struct BackupFile {
    pub name: String,
    pub size_bytes: u64,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupSummary {
    pub product_count: i64,
    pub sale_count: i64,
    pub last_sale_at: Option<String>,
    pub schema_versions: Vec<String>,
}

pub fn backup_dir(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .map(|p| p.join("backups"))
        .unwrap_or_else(|| PathBuf::from("backups"))
}

/// Copy the live database to `dest` using the SQLite online backup API.
/// Writes to a temporary file first so a failed copy never leaves a half-written backup.
pub fn create_to(conn: &Connection, dest: &Path) -> AppResult<()> {
    let tmp = dest.with_extension("db.tmp");
    let _ = fs::remove_file(&tmp);
    conn.backup(DatabaseName::Main, &tmp, None)
        .map_err(|e| AppError::Message(format!("Yedek alınamadı: {e}")))?;
    if dest.exists() {
        fs::remove_file(dest)?;
    }
    fs::rename(&tmp, dest)?;
    Ok(())
}

/// Validate a backup file without touching the live database.
pub fn inspect(src: &Path, staging_dir: &Path) -> AppResult<BackupSummary> {
    let staged = stage(src, staging_dir)?;
    let result = inspect_file(&staged);
    remove_db_files(&staged);
    result
}

/// Replace the live database contents with `src`.
/// Takes a safety backup of the current data first, then upgrades the restored schema.
pub fn restore_from(conn: &mut Connection, src: &Path, backups: &Path) -> AppResult<BackupSummary> {
    let staged = stage(src, backups)?;
    let result = (|| {
        inspect_file(&staged)?;
        safety_backup(conn, backups, PRE_RESTORE_PREFIX)?;
        conn.restore(DatabaseName::Main, &staged, None::<fn(Progress)>)
            .map_err(|e| AppError::Message(format!("Geri yükleme başarısız: {e}")))?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        run_migrations(conn)?;
        seed_default_store(conn)?;
        summarize(conn)
    })();
    remove_db_files(&staged);
    result
}

fn remove_db_files(path: &Path) {
    let _ = fs::remove_file(path);
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut sidecar = path.as_os_str().to_owned();
        sidecar.push(suffix);
        let _ = fs::remove_file(PathBuf::from(sidecar));
    }
}

pub fn safety_backup(conn: &Connection, backups: &Path, prefix: &str) -> AppResult<PathBuf> {
    fs::create_dir_all(backups)?;
    let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let path = backups.join(format!("{prefix}{stamp}.db"));
    create_to(conn, &path)?;
    prune(backups, prefix, SAFETY_KEEP)?;
    Ok(path)
}

/// Daily automatic copy in the app data folder. Protects against mistakes, not disk failure.
pub fn auto_backup_on_start(conn: &Connection, backups: &Path) -> AppResult<()> {
    fs::create_dir_all(backups)?;
    let today = chrono::Local::now().format("%Y-%m-%d");
    let path = backups.join(format!("{AUTO_PREFIX}{today}.db"));
    if !path.exists() {
        create_to(conn, &path)?;
    }
    prune(backups, AUTO_PREFIX, AUTO_KEEP)
}

/// Keep the newest `keep` files with the given prefix; names sort chronologically.
pub fn prune(dir: &Path, prefix: &str, keep: usize) -> AppResult<()> {
    let mut names: Vec<String> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.starts_with(prefix) && n.ends_with(".db"))
        .collect();
    names.sort();
    names.reverse();
    for old in names.into_iter().skip(keep) {
        let _ = fs::remove_file(dir.join(old));
    }
    Ok(())
}

pub fn list_recent(dir: &Path) -> AppResult<Vec<BackupFile>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files: Vec<(std::time::SystemTime, BackupFile)> = Vec::new();
    for entry in fs::read_dir(dir)?.filter_map(|e| e.ok()) {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if !name.ends_with(".db") || name.starts_with('_') {
            continue;
        }
        let meta = entry.metadata()?;
        let modified = meta.modified().ok();
        files.push((
            modified.unwrap_or(std::time::UNIX_EPOCH),
            BackupFile {
                name,
                size_bytes: meta.len(),
                modified_at: modified
                    .map(|t| chrono::DateTime::<chrono::Local>::from(t).to_rfc3339()),
            },
        ));
    }
    files.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(files.into_iter().map(|(_, f)| f).collect())
}

/// Copy the source next to the live DB so validation and restore read a stable local file
/// (the user may pull the USB stick, and SQLite may need to create sidecar files).
fn stage(src: &Path, staging_dir: &Path) -> AppResult<PathBuf> {
    if !src.is_file() {
        return Err(AppError::Message("Yedek dosyası bulunamadı".into()));
    }
    fs::create_dir_all(staging_dir)?;
    let staged = staging_dir.join(STAGING_NAME);
    remove_db_files(&staged);
    fs::copy(src, &staged)?;
    Ok(staged)
}

fn not_a_backup() -> AppError {
    AppError::Message("Bu dosya geçerli bir Market Asistanı yedeği değil".into())
}

fn inspect_file(path: &Path) -> AppResult<BackupSummary> {
    let conn = Connection::open(path).map_err(|_| not_a_backup())?;

    let check: String = conn
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .map_err(|_| not_a_backup())?;
    if check != "ok" {
        return Err(AppError::Message("Yedek dosyası bozuk görünüyor".into()));
    }

    for table in ["schema_migrations", "products", "sales"] {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                [table],
                |r| r.get(0),
            )
            .map_err(|_| not_a_backup())?;
        if !exists {
            return Err(not_a_backup());
        }
    }

    let summary = summarize(&conn)?;
    let known = known_versions();
    if summary
        .schema_versions
        .iter()
        .any(|v| !known.contains(&v.as_str()))
    {
        return Err(AppError::Message(
            "Bu yedek daha yeni bir sürümden; önce uygulamayı güncelleyin".into(),
        ));
    }
    Ok(summary)
}

fn summarize(conn: &Connection) -> AppResult<BackupSummary> {
    let product_count = conn.query_row(
        "SELECT COUNT(*) FROM products WHERE is_active = 1",
        [],
        |r| r.get(0),
    )?;
    let sale_count = conn.query_row("SELECT COUNT(*) FROM sales", [], |r| r.get(0))?;
    let last_sale_at = conn
        .query_row("SELECT MAX(paid_at) FROM sales", [], |r| r.get::<_, Option<String>>(0))
        .optional()?
        .flatten();
    let mut stmt = conn.prepare("SELECT version FROM schema_migrations ORDER BY version")?;
    let schema_versions = stmt
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(BackupSummary {
        product_count,
        sale_count,
        last_sale_at,
        schema_versions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_and_migrate;
    use crate::domain::CreateProductInput;
    use crate::repos::ProductRepo;

    fn add_product(conn: &Connection, barcode: &str) {
        ProductRepo::create(
            conn,
            "store_default",
            &CreateProductInput {
                barcode: barcode.into(),
                name: format!("Ürün {barcode}"),
                price_cents: 1250,
            },
        )
        .unwrap();
    }

    #[test]
    fn backup_then_restore_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let state = open_and_migrate(dir.path().join("live.db")).unwrap();
        let backups = dir.path().join("backups");
        let target = dir.path().join("yedek.db");

        {
            let conn = state.conn.lock();
            add_product(&conn, "111");
            create_to(&conn, &target).unwrap();
            add_product(&conn, "222");
        }

        let summary = inspect(&target, &backups).unwrap();
        assert_eq!(summary.product_count, 1);

        let mut conn = state.conn.lock();
        let restored = restore_from(&mut conn, &target, &backups).unwrap();
        assert_eq!(restored.product_count, 1);
        assert!(ProductRepo::get_by_barcode(&conn, "store_default", "222")
            .unwrap()
            .is_none());

        let safety: Vec<_> = list_recent(&backups)
            .unwrap()
            .into_iter()
            .filter(|f| f.name.starts_with(PRE_RESTORE_PREFIX))
            .collect();
        assert_eq!(safety.len(), 1);
    }

    #[test]
    fn older_schema_backup_is_upgraded() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("old.db");
        {
            let conn = Connection::open(&old).unwrap();
            conn.execute_batch(include_str!("../db/migrations/001_init.sql"))
                .unwrap();
            conn.execute_batch(
                "CREATE TABLE schema_migrations (version TEXT PRIMARY KEY, applied_at TEXT NOT NULL);
                 INSERT INTO schema_migrations VALUES ('001_init', '2026-01-01');
                 INSERT INTO stores VALUES ('store_default', 'Eski Market', '2026-01-01');
                 INSERT INTO sales (id, store_id, total_cents, status, paid_at, created_at) VALUES
                   ('s1', 'store_default', 1500, 'completed', '2026-03-10T08:00:00+00:00', 'x'),
                   ('s2', 'store_default', 700, 'completed', '2026-03-10T09:00:00+00:00', 'x');",
            )
            .unwrap();
        }

        let state = open_and_migrate(dir.path().join("live.db")).unwrap();
        let mut conn = state.conn.lock();
        let summary = restore_from(&mut conn, &old, &dir.path().join("backups")).unwrap();
        assert!(summary.schema_versions.contains(&"002_owner_settings".to_string()));
        assert!(summary.schema_versions.contains(&"003_sales_reporting".to_string()));
        assert_eq!(summary.sale_count, 2);
        let has_settings: i64 = conn
            .query_row("SELECT COUNT(*) FROM owner_settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(has_settings, 1);

        let expected_day = chrono::DateTime::parse_from_rfc3339("2026-03-10T09:00:00+00:00")
            .unwrap()
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string();
        let (method, day, receipt): (String, String, i64) = conn
            .query_row(
                "SELECT payment_method, business_day, receipt_no FROM sales WHERE id = 's2'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(method, "cash");
        assert_eq!(day, expected_day);
        assert_eq!(receipt, 2);
    }

    #[test]
    fn rejects_newer_and_garbage_files() {
        let dir = tempfile::tempdir().unwrap();
        let backups = dir.path().join("backups");

        let state = open_and_migrate(dir.path().join("live.db")).unwrap();
        let newer = dir.path().join("newer.db");
        {
            let conn = state.conn.lock();
            create_to(&conn, &newer).unwrap();
        }
        Connection::open(&newer)
            .unwrap()
            .execute("INSERT INTO schema_migrations VALUES ('999_future', 'x')", [])
            .unwrap();
        let err = inspect(&newer, &backups).unwrap_err().to_string();
        assert!(err.contains("daha yeni"), "{err}");

        let garbage = dir.path().join("resim.db");
        fs::write(&garbage, b"bu bir veritabani degil").unwrap();
        let err = inspect(&garbage, &backups).unwrap_err().to_string();
        assert!(err.contains("geçerli bir Market"), "{err}");
    }

    #[test]
    fn prune_keeps_newest() {
        let dir = tempfile::tempdir().unwrap();
        for day in 1..=9 {
            fs::write(dir.path().join(format!("auto_2026-11-0{day}.db")), b"x").unwrap();
        }
        prune(dir.path(), AUTO_PREFIX, 7).unwrap();
        let mut left: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        left.sort();
        assert_eq!(left.len(), 7);
        assert_eq!(left[0], "auto_2026-11-03.db");
    }
}
