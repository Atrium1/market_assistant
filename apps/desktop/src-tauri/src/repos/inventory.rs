use crate::error::AppResult;
use rusqlite::{Connection, params};

/// Inventory repository — ready for stock tracking expansions.
/// V1 does not enforce quantities on sale.
#[allow(dead_code)]
pub struct InventoryRepo;

impl InventoryRepo {
    #[allow(dead_code)]
    pub fn get_quantity(conn: &Connection, product_id: &str) -> AppResult<i64> {
        let qty = conn.query_row(
            "SELECT quantity FROM inventory WHERE product_id = ?1",
            [product_id],
            |row| row.get(0),
        )?;
        Ok(qty)
    }

    #[allow(dead_code)]
    pub fn adjust(conn: &Connection, product_id: &str, delta: i64) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE inventory
             SET quantity = quantity + ?1, updated_at = ?2
             WHERE product_id = ?3",
            params![delta, now, product_id],
        )?;
        Ok(())
    }
}
