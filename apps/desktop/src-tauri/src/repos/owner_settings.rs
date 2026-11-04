use crate::error::{AppError, AppResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct OwnerSettings {
    pub store_id: String,
    pub shop_username: Option<String>,
    pub shop_password_hash: Option<String>,
    pub store_display_name: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SettingsPublic {
    pub store_display_name: Option<String>,
    pub shop_username: Option<String>,
    pub shop_credentials_set: bool,
}

pub struct OwnerSettingsRepo;

impl OwnerSettingsRepo {
    pub fn get(conn: &Connection, store_id: &str) -> AppResult<OwnerSettings> {
        conn.query_row(
            "SELECT store_id, shop_username, shop_password_hash, store_display_name, updated_at
             FROM owner_settings WHERE store_id = ?1",
            [store_id],
            map_settings,
        )
        .optional()?
        .ok_or_else(|| AppError::Message("Mağaza ayarları bulunamadı".into()))
    }

    pub fn get_public(conn: &Connection, store_id: &str) -> AppResult<SettingsPublic> {
        let s = Self::get(conn, store_id)?;
        Ok(SettingsPublic {
            store_display_name: s.store_display_name,
            shop_username: s.shop_username.clone(),
            shop_credentials_set: s.shop_username.is_some() && s.shop_password_hash.is_some(),
        })
    }

    pub fn set_store_name(conn: &Connection, store_id: &str, name: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::Message("Mağaza adı boş olamaz".into()));
        }
        let changed = conn.execute(
            "UPDATE owner_settings SET store_display_name = ?1, updated_at = ?2 WHERE store_id = ?3",
            params![name, now, store_id],
        )?;
        if changed == 0 {
            return Err(AppError::Message("Mağaza ayarları bulunamadı".into()));
        }
        // Keep stores.name in sync for consistency
        conn.execute(
            "UPDATE stores SET name = ?1 WHERE id = ?2",
            params![name, store_id],
        )?;
        Ok(())
    }

    pub fn set_credentials(
        conn: &Connection,
        store_id: &str,
        username: &str,
        password_hash: &str,
    ) -> AppResult<()> {
        let username = username.trim();
        if username.is_empty() {
            return Err(AppError::Message("Kullanıcı adı zorunludur".into()));
        }
        if username.len() < 3 {
            return Err(AppError::Message(
                "Kullanıcı adı en az 3 karakter olmalı".into(),
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();
        let changed = conn.execute(
            "UPDATE owner_settings
             SET shop_username = ?1, shop_password_hash = ?2, updated_at = ?3
             WHERE store_id = ?4",
            params![username, password_hash, now, store_id],
        )?;
        if changed == 0 {
            return Err(AppError::Message("Mağaza ayarları bulunamadı".into()));
        }
        Ok(())
    }
}

fn map_settings(row: &rusqlite::Row<'_>) -> rusqlite::Result<OwnerSettings> {
    Ok(OwnerSettings {
        store_id: row.get(0)?,
        shop_username: row.get(1)?,
        shop_password_hash: row.get(2)?,
        store_display_name: row.get(3)?,
        updated_at: row.get(4)?,
    })
}
