use serde::{Deserialize, Serialize};

pub const MAX_QTY_PER_LINE: i64 = 9999;
pub const MAX_CART_LINES: usize = 500;
pub const MAX_VOID_REASON_CHARS: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    #[default]
    Cash,
    Card,
}

impl PaymentMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            PaymentMethod::Cash => "cash",
            PaymentMethod::Card => "card",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "card" => PaymentMethod::Card,
            _ => PaymentMethod::Cash,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaleStatus {
    Completed,
    Voided,
}

impl SaleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SaleStatus::Completed => "completed",
            SaleStatus::Voided => "voided",
        }
    }
}

/// Only `product_id` and `qty` are trusted. `expected_unit_price_cents` is the price the
/// cashier saw; older clients send it as `unit_price_cents` together with `barcode`/`name`.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckoutItemInput {
    pub product_id: String,
    pub qty: i64,
    #[serde(default, alias = "unit_price_cents")]
    pub expected_unit_price_cents: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub barcode: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckoutInput {
    pub items: Vec<CheckoutItemInput>,
    #[serde(default)]
    pub payment_method: PaymentMethod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedLine {
    pub product_id: String,
    pub qty: i64,
    pub expected_unit_price_cents: Option<i64>,
    pub name_hint: Option<String>,
}

/// Validate and merge duplicate product lines, keeping first-seen order.
pub fn normalize_lines(items: &[CheckoutItemInput]) -> Result<Vec<NormalizedLine>, String> {
    if items.is_empty() {
        return Err("Sepet boş".into());
    }

    let mut lines: Vec<NormalizedLine> = Vec::new();
    for item in items {
        let product_id = item.product_id.trim();
        if product_id.is_empty() {
            return Err("Satır bilgisi eksik".into());
        }
        if item.qty <= 0 {
            return Err("Adet 1 veya daha büyük olmalı".into());
        }
        if item.expected_unit_price_cents.is_some_and(|p| p < 0) {
            return Err("Birim fiyat negatif olamaz".into());
        }

        if let Some(line) = lines.iter_mut().find(|l| l.product_id == product_id) {
            line.qty = line
                .qty
                .checked_add(item.qty)
                .ok_or_else(|| "Tutar taşması".to_string())?;
            if line.expected_unit_price_cents.is_none() {
                line.expected_unit_price_cents = item.expected_unit_price_cents;
            }
        } else {
            lines.push(NormalizedLine {
                product_id: product_id.to_string(),
                qty: item.qty,
                expected_unit_price_cents: item.expected_unit_price_cents,
                name_hint: item.name.as_ref().map(|n| n.trim().to_string()),
            });
        }
    }

    if lines.len() > MAX_CART_LINES {
        return Err(format!("Sepette en fazla {MAX_CART_LINES} farklı ürün olabilir"));
    }
    if lines.iter().any(|l| l.qty > MAX_QTY_PER_LINE) {
        return Err(format!("Bir üründen en fazla {MAX_QTY_PER_LINE} adet satılabilir"));
    }
    Ok(lines)
}

pub fn line_total(unit_price_cents: i64, qty: i64) -> Result<i64, String> {
    unit_price_cents
        .checked_mul(qty)
        .ok_or_else(|| "Tutar taşması".to_string())
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CartLineOut {
    pub product_id: String,
    pub barcode: String,
    pub name: String,
    pub unit_price_cents: i64,
    pub qty: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PriceChange {
    pub product_id: String,
    pub name: String,
    pub old_unit_price_cents: i64,
    pub new_unit_price_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RemovedLine {
    pub product_id: String,
    pub name: Option<String>,
}

/// Returned with `cart_changed`: `current` is the cart rebuilt from DB prices.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CartChangedDetails {
    pub changed: Vec<PriceChange>,
    pub removed: Vec<RemovedLine>,
    pub current: Vec<CartLineOut>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sale {
    pub id: String,
    pub store_id: String,
    pub total_cents: i64,
    pub status: String,
    pub payment_method: PaymentMethod,
    pub business_day: String,
    pub receipt_no: i64,
    pub paid_at: String,
    pub created_at: String,
    pub voided_at: Option<String>,
    pub void_reason: Option<String>,
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

pub type SaleDetail = CheckoutResult;

#[derive(Debug, Clone, Deserialize)]
pub struct SalesListQuery {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub status: Option<SaleStatus>,
    #[serde(default)]
    pub payment_method: Option<PaymentMethod>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaleListItem {
    pub id: String,
    pub business_day: String,
    pub receipt_no: i64,
    pub paid_at: String,
    pub total_cents: i64,
    pub payment_method: PaymentMethod,
    pub item_count: i64,
    pub status: String,
    pub void_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SalesPage {
    pub items: Vec<SaleListItem>,
    pub next_cursor: Option<String>,
}

pub fn normalize_void_reason(reason: &str) -> Result<String, String> {
    let reason = reason.trim();
    if reason.is_empty() {
        return Err("İptal nedeni yazın".into());
    }
    if reason.chars().count() > MAX_VOID_REASON_CHARS {
        return Err(format!(
            "İptal nedeni en fazla {MAX_VOID_REASON_CHARS} karakter olabilir"
        ));
    }
    Ok(reason.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, qty: i64, price: Option<i64>) -> CheckoutItemInput {
        CheckoutItemInput {
            product_id: id.into(),
            qty,
            expected_unit_price_cents: price,
            barcode: None,
            name: None,
        }
    }

    #[test]
    fn merges_duplicate_lines_in_order() {
        let lines =
            normalize_lines(&[item("a", 1, Some(100)), item("b", 2, None), item("a", 3, None)])
                .unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].product_id, "a");
        assert_eq!(lines[0].qty, 4);
        assert_eq!(lines[0].expected_unit_price_cents, Some(100));
        assert_eq!(lines[1].qty, 2);
    }

    #[test]
    fn rejects_bad_quantities_and_sizes() {
        assert!(normalize_lines(&[]).is_err());
        assert!(normalize_lines(&[item("a", 0, None)]).is_err());
        assert!(normalize_lines(&[item("a", 10_000, None)]).is_err());
        assert!(normalize_lines(&[item("a", 9_999, None)]).is_ok());
        assert!(normalize_lines(&[item("a", 5_000, None), item("a", 5_000, None)]).is_err());
        assert!(normalize_lines(&[item("a", i64::MAX, None), item("a", 1, None)]).is_err());
        assert!(normalize_lines(&[item(" ", 1, None)]).is_err());
        assert!(normalize_lines(&[item("a", 1, Some(-1))]).is_err());
        let many: Vec<_> = (0..=MAX_CART_LINES).map(|i| item(&i.to_string(), 1, None)).collect();
        assert!(normalize_lines(&many).is_err());
    }

    #[test]
    fn legacy_payload_is_accepted() {
        let input: CheckoutInput = serde_json::from_str(
            r#"{"items":[{"product_id":"p","barcode":"1","name":"Su","unit_price_cents":1500,"qty":2}]}"#,
        )
        .unwrap();
        assert_eq!(input.payment_method, PaymentMethod::Cash);
        assert_eq!(input.items[0].expected_unit_price_cents, Some(1500));
    }

    #[test]
    fn void_reason_rules() {
        assert!(normalize_void_reason("  ").is_err());
        assert_eq!(normalize_void_reason(" yanlış ürün ").unwrap(), "yanlış ürün");
        assert!(normalize_void_reason(&"ş".repeat(201)).is_err());
    }
}
