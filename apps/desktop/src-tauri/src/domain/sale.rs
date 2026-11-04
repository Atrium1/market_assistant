use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutItemInput {
    pub product_id: String,
    pub barcode: String,
    pub name: String,
    pub unit_price_cents: i64,
    pub qty: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckoutInput {
    pub items: Vec<CheckoutItemInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sale {
    pub id: String,
    pub store_id: String,
    pub total_cents: i64,
    pub status: String,
    pub paid_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaleItem {
    pub id: String,
    pub sale_id: String,
    pub product_id: Option<String>,
    pub barcode: String,
    pub name_snapshot: String,
    pub unit_price_cents: i64,
    pub qty: i64,
    pub line_total_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckoutResult {
    pub sale: Sale,
    pub items: Vec<SaleItem>,
}

pub fn validate_checkout(input: &CheckoutInput) -> Result<i64, String> {
    if input.items.is_empty() {
        return Err("Sepet boş".into());
    }

    let mut total: i64 = 0;
    for item in &input.items {
        if item.qty <= 0 {
            return Err("Adet 1 veya daha büyük olmalı".into());
        }
        if item.unit_price_cents < 0 {
            return Err("Birim fiyat negatif olamaz".into());
        }
        if item.barcode.trim().is_empty() || item.name.trim().is_empty() {
            return Err("Satır bilgisi eksik".into());
        }
        let line = item
            .unit_price_cents
            .checked_mul(item.qty)
            .ok_or_else(|| "Tutar taşması".to_string())?;
        total = total
            .checked_add(line)
            .ok_or_else(|| "Tutar taşması".to_string())?;
    }
    Ok(total)
}
