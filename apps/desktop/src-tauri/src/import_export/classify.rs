use super::normalize::{
    barcode_warning, find_header, parse_barcode, parse_name, parse_price,
};
use super::read::Cell;
use crate::domain::Product;
use serde::Serialize;
use std::collections::HashMap;

pub const MAX_ROWS: usize = 20_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RowKind {
    Add,
    Update,
    Unchanged,
    Invalid,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassifiedRow {
    /// 1-based row number as the owner sees it in Excel.
    pub row_number: usize,
    pub kind: RowKind,
    pub barcode: String,
    pub name: String,
    pub price_cents: Option<i64>,
    pub old_name: Option<String>,
    pub old_price_cents: Option<i64>,
    pub reactivate: bool,
    pub reason: Option<String>,
    pub warning: Option<String>,
}

pub fn classify(rows: &[Vec<Cell>], existing: &[Product]) -> Result<Vec<ClassifiedRow>, String> {
    let (header_idx, cols) = find_header(rows).ok_or_else(|| {
        "Başlık satırı bulunamadı. Sütunlar şunlar olmalı: Barkod, Ürün adı, Fiyat".to_string()
    })?;

    let empty = Cell::Empty;
    let mut parsed: Vec<ClassifiedRow> = Vec::new();
    for (idx, row) in rows.iter().enumerate().skip(header_idx + 1) {
        let get = |i: usize| row.get(i).unwrap_or(&empty);
        if row.iter().all(Cell::is_empty) {
            continue;
        }
        if parsed.len() >= MAX_ROWS {
            return Err(format!("Dosyada çok fazla satır var (en fazla {MAX_ROWS})"));
        }

        let barcode = parse_barcode(get(cols.barcode));
        let name = parse_name(get(cols.name));
        let price = parse_price(get(cols.price));
        let reason = [&barcode.as_ref().err(), &name.as_ref().err(), &price.as_ref().err()]
            .into_iter()
            .flatten()
            .map(|e| e.to_string())
            .collect::<Vec<_>>();

        let barcode_text = barcode.clone().unwrap_or_else(|_| get(cols.barcode).as_text());
        parsed.push(ClassifiedRow {
            row_number: idx + 1,
            kind: if reason.is_empty() { RowKind::Add } else { RowKind::Invalid },
            warning: barcode.as_ref().ok().and_then(|b| barcode_warning(b)),
            barcode: barcode_text,
            name: name.unwrap_or_else(|_| get(cols.name).as_text()),
            price_cents: price.ok(),
            old_name: None,
            old_price_cents: None,
            reactivate: false,
            reason: (!reason.is_empty()).then(|| reason.join("; ")),
        });
    }

    let mut occurrences: HashMap<String, Vec<usize>> = HashMap::new();
    for row in &parsed {
        if !row.barcode.is_empty() {
            occurrences.entry(row.barcode.clone()).or_default().push(row.row_number);
        }
    }

    let by_barcode: HashMap<&str, &Product> =
        existing.iter().map(|p| (p.barcode.as_str(), p)).collect();

    for row in parsed.iter_mut() {
        if let Some(lines) = occurrences.get(&row.barcode).filter(|l| l.len() > 1) {
            let list = lines.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
            let dup = format!("Dosyada tekrar eden barkod (satır {list})");
            row.reason = Some(match row.reason.take() {
                Some(r) => format!("{r}; {dup}"),
                None => dup,
            });
            row.kind = RowKind::Invalid;
            continue;
        }
        if row.kind == RowKind::Invalid {
            continue;
        }
        let Some(product) = by_barcode.get(row.barcode.as_str()) else {
            continue;
        };
        let price = row.price_cents.unwrap_or_default();
        row.old_name = Some(product.name.clone());
        row.old_price_cents = Some(product.price_cents);
        row.reactivate = !product.is_active;
        row.kind = if product.is_active && product.name == row.name && product.price_cents == price {
            RowKind::Unchanged
        } else {
            RowKind::Update
        };
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Cell {
        Cell::Text(s.into())
    }

    fn product(barcode: &str, name: &str, price: i64, active: bool) -> Product {
        Product {
            id: barcode.into(),
            store_id: "store_default".into(),
            barcode: barcode.into(),
            name: name.into(),
            price_cents: price,
            is_active: active,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn classifies_every_case() {
        let rows = vec![
            vec![text("Barkod"), text("Ürün adı"), text("Fiyat")],
            vec![text("1"), text("Yeni ürün"), text("5,00")],
            vec![text("2"), text("Süt"), text("15,00")],
            vec![text("3"), text("Ekmek"), text("10,00")],
            vec![text("4"), text("Silinmiş"), text("3,00")],
            vec![text(""), text("Barkodsuz"), text("1")],
            vec![text("5"), text("Çay"), text("abc")],
            vec![text("6"), text("İkiz A"), text("1")],
            vec![text("6"), text("İkiz B"), text("2")],
            vec![Cell::Empty, Cell::Empty, Cell::Empty],
        ];
        let existing = vec![
            product("2", "Süt", 1250, true),
            product("3", "Ekmek", 1000, true),
            product("4", "Silinmiş", 300, false),
        ];
        let out = classify(&rows, &existing).unwrap();
        let kinds: Vec<_> = out.iter().map(|r| (r.row_number, r.kind)).collect();
        assert_eq!(
            kinds,
            vec![
                (2, RowKind::Add),
                (3, RowKind::Update),
                (4, RowKind::Unchanged),
                (5, RowKind::Update),
                (6, RowKind::Invalid),
                (7, RowKind::Invalid),
                (8, RowKind::Invalid),
                (9, RowKind::Invalid),
            ]
        );
        assert!(out[3].reactivate);
        assert_eq!(out[1].old_price_cents, Some(1250));
        assert!(out[6].reason.as_deref().unwrap().contains("satır 8, 9"));
    }

    #[test]
    fn missing_header_is_an_error() {
        let rows = vec![vec![text("a"), text("b")]];
        assert!(classify(&rows, &[]).is_err());
    }
}
