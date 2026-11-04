use crate::domain::{
    CheckoutInput, CheckoutResult, Sale, SaleItem, validate_checkout,
};
use crate::error::{AppError, AppResult};
use rusqlite::{Connection, params};

pub struct SaleRepo;

impl SaleRepo {
    pub fn checkout(
        conn: &Connection,
        store_id: &str,
        input: &CheckoutInput,
    ) -> AppResult<CheckoutResult> {
        let total_cents =
            validate_checkout(input).map_err(AppError::Message)?;

        let sale_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let tx = conn.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO sales (id, store_id, total_cents, status, paid_at, created_at)
             VALUES (?1, ?2, ?3, 'completed', ?4, ?4)",
            params![sale_id, store_id, total_cents, now],
        )?;

        let mut items = Vec::with_capacity(input.items.len());
        for line in &input.items {
            let item_id = uuid::Uuid::new_v4().to_string();
            let line_total = line.unit_price_cents * line.qty;
            tx.execute(
                "INSERT INTO sale_items (
                    id, sale_id, product_id, barcode, name_snapshot,
                    unit_price_cents, qty, line_total_cents
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    item_id,
                    sale_id,
                    line.product_id,
                    line.barcode.trim(),
                    line.name.trim(),
                    line.unit_price_cents,
                    line.qty,
                    line_total
                ],
            )?;

            items.push(SaleItem {
                id: item_id,
                sale_id: sale_id.clone(),
                product_id: Some(line.product_id.clone()),
                barcode: line.barcode.trim().to_string(),
                name_snapshot: line.name.trim().to_string(),
                unit_price_cents: line.unit_price_cents,
                qty: line.qty,
                line_total_cents: line_total,
            });
        }

        tx.commit()?;

        Ok(CheckoutResult {
            sale: Sale {
                id: sale_id,
                store_id: store_id.to_string(),
                total_cents,
                status: "completed".into(),
                paid_at: now.clone(),
                created_at: now,
            },
            items,
        })
    }
}
