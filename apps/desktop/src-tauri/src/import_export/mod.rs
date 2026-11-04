pub mod classify;
pub mod export;
pub mod normalize;
pub mod read;

use crate::domain::CreateProductInput;
use crate::error::{AppError, AppResult};
use crate::repos::product::UpsertOutcome;
use crate::repos::ProductRepo;
use classify::{ClassifiedRow, RowKind};
use rusqlite::Connection;
use serde::Serialize;
use std::path::Path;

const PREVIEW_LIMIT: usize = 200;

#[derive(Debug, Default, Serialize)]
pub struct ImportCounts {
    pub add: usize,
    pub update: usize,
    pub unchanged: usize,
    pub invalid: usize,
}

#[derive(Debug, Serialize)]
pub struct ImportPreview {
    pub file_name: String,
    pub counts: ImportCounts,
    pub add: Vec<ClassifiedRow>,
    pub update: Vec<ClassifiedRow>,
    pub unchanged: Vec<ClassifiedRow>,
    pub invalid: Vec<ClassifiedRow>,
    /// True when at least one category was cut to `PREVIEW_LIMIT` rows.
    pub truncated: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct ImportResult {
    pub added: usize,
    pub updated: usize,
    pub reactivated: usize,
    pub skipped: usize,
}

pub fn analyze(conn: &Connection, store_id: &str, path: &Path) -> AppResult<Vec<ClassifiedRow>> {
    let rows = read::read_rows(path)?;
    let existing = ProductRepo::list_all(conn, store_id)?;
    classify::classify(&rows, &existing).map_err(AppError::Message)
}

pub fn preview(path: &Path, rows: Vec<ClassifiedRow>) -> ImportPreview {
    let mut out = ImportPreview {
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        counts: ImportCounts::default(),
        add: Vec::new(),
        update: Vec::new(),
        unchanged: Vec::new(),
        invalid: Vec::new(),
        truncated: false,
    };
    for row in rows {
        let (count, bucket) = match row.kind {
            RowKind::Add => (&mut out.counts.add, &mut out.add),
            RowKind::Update => (&mut out.counts.update, &mut out.update),
            RowKind::Unchanged => (&mut out.counts.unchanged, &mut out.unchanged),
            RowKind::Invalid => (&mut out.counts.invalid, &mut out.invalid),
        };
        *count += 1;
        if bucket.len() < PREVIEW_LIMIT {
            bucket.push(row);
        } else {
            out.truncated = true;
        }
    }
    out
}

/// Apply valid rows in one transaction. Unchanged and invalid rows are skipped.
pub fn apply(conn: &Connection, store_id: &str, rows: &[ClassifiedRow]) -> AppResult<ImportResult> {
    let mut result = ImportResult::default();
    let tx = conn.unchecked_transaction()?;
    for row in rows {
        if !matches!(row.kind, RowKind::Add | RowKind::Update) {
            result.skipped += 1;
            continue;
        }
        let input = CreateProductInput {
            barcode: row.barcode.clone(),
            name: row.name.clone(),
            price_cents: row.price_cents.unwrap_or_default(),
        };
        let (_, outcome) = ProductRepo::upsert_by_barcode(&tx, store_id, &input, false)?;
        match outcome {
            UpsertOutcome::Created => result.added += 1,
            UpsertOutcome::Updated => result.updated += 1,
            UpsertOutcome::Reactivated => result.reactivated += 1,
        }
    }
    tx.commit()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_and_migrate;

    #[test]
    fn export_then_import_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let state = open_and_migrate(dir.path().join("live.db")).unwrap();
        let conn = state.conn.lock();
        let store = "store_default";

        let csv = dir.path().join("liste.csv");
        std::fs::write(
            &csv,
            "Barkod;Ürün adı;Fiyat\n0869001;Süt 1L;12,50\n0869002;Çay 500g;1.250\n",
        )
        .unwrap();

        let rows = analyze(&conn, store, &csv).unwrap();
        let result = apply(&conn, store, &rows).unwrap();
        assert_eq!(result.added, 2);

        let tea = ProductRepo::get_by_barcode(&conn, store, "0869002").unwrap().unwrap();
        assert_eq!(tea.price_cents, 125_000);

        let xlsx = dir.path().join("urunler.xlsx");
        let products = ProductRepo::list_active(&conn, store).unwrap();
        export::write_products(&xlsx, &products).unwrap();

        let again = analyze(&conn, store, &xlsx).unwrap();
        assert!(again.iter().all(|r| r.kind == RowKind::Unchanged), "{again:?}");
        assert_eq!(again[0].barcode, "0869001", "leading zero must survive xlsx");
    }

    #[test]
    fn reactivates_deleted_product() {
        let dir = tempfile::tempdir().unwrap();
        let state = open_and_migrate(dir.path().join("live.db")).unwrap();
        let conn = state.conn.lock();
        let store = "store_default";

        let (product, _) = ProductRepo::upsert_by_barcode(
            &conn,
            store,
            &CreateProductInput {
                barcode: "777".into(),
                name: "Eski".into(),
                price_cents: 100,
            },
            true,
        )
        .unwrap();
        ProductRepo::deactivate(&conn, store, &product.id).unwrap();

        let csv = dir.path().join("liste.csv");
        std::fs::write(&csv, "barkod,ad,fiyat\n777,Yeni,2.5\n").unwrap();
        let rows = analyze(&conn, store, &csv).unwrap();
        let result = apply(&conn, store, &rows).unwrap();
        assert_eq!(result.reactivated, 1);

        let back = ProductRepo::get_by_barcode(&conn, store, "777").unwrap().unwrap();
        assert_eq!((back.name.as_str(), back.price_cents), ("Yeni", 250));
    }
}
