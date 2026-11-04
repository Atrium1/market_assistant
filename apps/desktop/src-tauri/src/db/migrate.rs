use crate::error::AppResult;
use rusqlite::Connection;

struct Migration {
    version: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: "001_init",
        sql: include_str!("migrations/001_init.sql"),
    },
    Migration {
        version: "002_owner_settings",
        sql: include_str!("migrations/002_owner_settings.sql"),
    },
];

pub fn run(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    for migration in MIGRATIONS {
        let already: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            [migration.version],
            |row| row.get(0),
        )?;

        if already {
            continue;
        }

        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(migration.sql)?;
        let now = chrono::Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![migration.version, now],
        )?;
        tx.commit()?;
    }

    Ok(())
}
