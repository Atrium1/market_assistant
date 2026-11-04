use crate::domain::{CreateProductInput, Product, UpdateProductInput, normalize_barcode};
use crate::error::{AppError, AppResult};
use rusqlite::{Connection, OptionalExtension, params};

pub struct ProductRepo;

impl ProductRepo {
    pub fn list_active(conn: &Connection, store_id: &str) -> AppResult<Vec<Product>> {
        let mut stmt = conn.prepare(
            "SELECT id, store_id, barcode, name, price_cents, is_active, created_at, updated_at
             FROM products
             WHERE store_id = ?1 AND is_active = 1
             ORDER BY name COLLATE NOCASE",
        )?;

        let rows = stmt.query_map([store_id], map_product)?;
        let mut products = Vec::new();
        for row in rows {
            products.push(row?);
        }
        Ok(products)
    }

    pub fn get_by_barcode(
        conn: &Connection,
        store_id: &str,
        barcode: &str,
    ) -> AppResult<Option<Product>> {
        let barcode = normalize_barcode(barcode);
        conn.query_row(
            "SELECT id, store_id, barcode, name, price_cents, is_active, created_at, updated_at
             FROM products
             WHERE store_id = ?1 AND barcode = ?2 AND is_active = 1",
            params![store_id, barcode],
            map_product,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn create(
        conn: &Connection,
        store_id: &str,
        input: &CreateProductInput,
    ) -> AppResult<Product> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let barcode = normalize_barcode(&input.barcode);

        conn.execute(
            "INSERT INTO products (id, store_id, barcode, name, price_cents, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6)",
            params![
                id,
                store_id,
                barcode,
                input.name.trim(),
                input.price_cents,
                now
            ],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(info, _)
                if info.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                AppError::Message("Bu barkod zaten kayıtlı".into())
            }
            other => AppError::from(other),
        })?;

        // Ensure inventory row exists for future stock tracking
        conn.execute(
            "INSERT OR IGNORE INTO inventory (product_id, quantity, low_stock_threshold, updated_at)
             VALUES (?1, 0, NULL, ?2)",
            params![id, now],
        )?;

        Self::get_by_id(conn, &id)?.ok_or_else(|| AppError::Message("Ürün oluşturulamadı".into()))
    }

    pub fn update(conn: &Connection, store_id: &str, input: &UpdateProductInput) -> AppResult<Product> {
        let existing = Self::get_by_id(conn, &input.id)?
            .ok_or(AppError::NotFound)?;
        if existing.store_id != store_id {
            return Err(AppError::NotFound);
        }

        let barcode = input
            .barcode
            .as_ref()
            .map(|b| normalize_barcode(b))
            .unwrap_or(existing.barcode.clone());
        let name = input
            .name
            .as_ref()
            .map(|n| n.trim().to_string())
            .unwrap_or(existing.name.clone());
        let price_cents = input.price_cents.unwrap_or(existing.price_cents);

        if barcode.is_empty() {
            return Err(AppError::Message("Barkod zorunludur".into()));
        }
        if name.is_empty() {
            return Err(AppError::Message("Ürün adı zorunludur".into()));
        }
        if price_cents < 0 {
            return Err(AppError::Message("Fiyat negatif olamaz".into()));
        }

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE products
             SET barcode = ?1, name = ?2, price_cents = ?3, updated_at = ?4
             WHERE id = ?5 AND store_id = ?6 AND is_active = 1",
            params![barcode, name, price_cents, now, input.id, store_id],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(info, _)
                if info.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                AppError::Message("Bu barkod zaten kayıtlı".into())
            }
            other => AppError::from(other),
        })?;

        Self::get_by_id(conn, &input.id)?.ok_or(AppError::NotFound)
    }

    pub fn deactivate(conn: &Connection, store_id: &str, id: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let changed = conn.execute(
            "UPDATE products SET is_active = 0, updated_at = ?1
             WHERE id = ?2 AND store_id = ?3 AND is_active = 1",
            params![now, id, store_id],
        )?;
        if changed == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    fn get_by_id(conn: &Connection, id: &str) -> AppResult<Option<Product>> {
        conn.query_row(
            "SELECT id, store_id, barcode, name, price_cents, is_active, created_at, updated_at
             FROM products WHERE id = ?1",
            [id],
            map_product,
        )
        .optional()
        .map_err(AppError::from)
    }
}

fn map_product(row: &rusqlite::Row<'_>) -> rusqlite::Result<Product> {
    let is_active: i64 = row.get(5)?;
    Ok(Product {
        id: row.get(0)?,
        store_id: row.get(1)?,
        barcode: row.get(2)?,
        name: row.get(3)?,
        price_cents: row.get(4)?,
        is_active: is_active == 1,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}
