use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub store_id: String,
    pub barcode: String,
    pub name: String,
    pub price_cents: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductInput {
    pub barcode: String,
    pub name: String,
    pub price_cents: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductInput {
    pub id: String,
    pub barcode: Option<String>,
    pub name: Option<String>,
    pub price_cents: Option<i64>,
}

pub fn normalize_barcode(barcode: &str) -> String {
    barcode.trim().to_string()
}

pub fn validate_create(input: &CreateProductInput) -> Result<(), String> {
    if normalize_barcode(&input.barcode).is_empty() {
        return Err("Barkod zorunludur".into());
    }
    if input.name.trim().is_empty() {
        return Err("Ürün adı zorunludur".into());
    }
    if input.price_cents < 0 {
        return Err("Fiyat negatif olamaz".into());
    }
    Ok(())
}
