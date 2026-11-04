use crate::domain::{
    CartChangedDetails, CartLineOut, CheckoutInput, CheckoutResult, PaymentMethod, PriceChange,
    RemovedLine, Sale, SaleDetail, SaleItem, SaleListItem, SaleStatus, SalesListQuery, SalesPage,
    line_total, normalize_lines, normalize_void_reason, parse_day,
};
use crate::error::{AppError, AppResult};
use crate::repos::ProductRepo;
use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use rusqlite::{Connection, OptionalExtension, params};

pub const DEFAULT_PAGE_SIZE: u32 = 50;
pub const MAX_PAGE_SIZE: u32 = 200;

const SALE_COLUMNS: &str = "id, store_id, total_cents, status, payment_method, business_day,
     receipt_no, paid_at, created_at, voided_at, void_reason";

pub struct SaleRepo;

/// UTC timestamps are written with a fixed width so they sort correctly as text.
pub fn utc_stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

impl SaleRepo {
    /// Prices, names and barcodes come from the DB. If the cashier saw a different price,
    /// or a product is gone, nothing is written and `CartChanged` carries the fresh cart.
    pub fn checkout(
        conn: &Connection,
        store_id: &str,
        input: &CheckoutInput,
        now: DateTime<FixedOffset>,
    ) -> AppResult<CheckoutResult> {
        let lines = normalize_lines(&input.items).map_err(AppError::Message)?;

        let tx = conn.unchecked_transaction()?;

        let mut current = Vec::with_capacity(lines.len());
        let mut changed = Vec::new();
        let mut removed = Vec::new();
        for line in &lines {
            match ProductRepo::get_active_by_id_in_store(&tx, store_id, &line.product_id)? {
                None => removed.push(RemovedLine {
                    product_id: line.product_id.clone(),
                    name: line.name_hint.clone(),
                }),
                Some(product) => {
                    if let Some(expected) = line.expected_unit_price_cents {
                        if expected != product.price_cents {
                            changed.push(PriceChange {
                                product_id: product.id.clone(),
                                name: product.name.clone(),
                                old_unit_price_cents: expected,
                                new_unit_price_cents: product.price_cents,
                            });
                        }
                    }
                    current.push(CartLineOut {
                        product_id: product.id,
                        barcode: product.barcode,
                        name: product.name,
                        unit_price_cents: product.price_cents,
                        qty: line.qty,
                    });
                }
            }
        }
        if !changed.is_empty() || !removed.is_empty() {
            return Err(AppError::CartChanged(CartChangedDetails {
                changed,
                removed,
                current,
            }));
        }

        let mut total_cents: i64 = 0;
        let mut line_totals = Vec::with_capacity(current.len());
        for line in &current {
            let lt = line_total(line.unit_price_cents, line.qty).map_err(AppError::Message)?;
            total_cents = total_cents
                .checked_add(lt)
                .ok_or_else(|| AppError::Message("Tutar taşması".into()))?;
            line_totals.push(lt);
        }

        let sale_id = uuid::Uuid::new_v4().to_string();
        let paid_at = utc_stamp(now.with_timezone(&Utc));
        let business_day = now.date_naive().format("%Y-%m-%d").to_string();
        let receipt_no: i64 = tx.query_row(
            "SELECT COALESCE(MAX(receipt_no), 0) + 1 FROM sales
             WHERE store_id = ?1 AND business_day = ?2",
            params![store_id, business_day],
            |r| r.get(0),
        )?;

        tx.execute(
            "INSERT INTO sales (id, store_id, total_cents, status, payment_method,
                                business_day, receipt_no, paid_at, created_at)
             VALUES (?1, ?2, ?3, 'completed', ?4, ?5, ?6, ?7, ?7)",
            params![
                sale_id,
                store_id,
                total_cents,
                input.payment_method.as_str(),
                business_day,
                receipt_no,
                paid_at
            ],
        )?;

        let mut items = Vec::with_capacity(current.len());
        for (line, lt) in current.iter().zip(line_totals) {
            let item_id = uuid::Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO sale_items (
                    id, sale_id, product_id, barcode, name_snapshot,
                    unit_price_cents, qty, line_total_cents
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    item_id,
                    sale_id,
                    line.product_id,
                    line.barcode,
                    line.name,
                    line.unit_price_cents,
                    line.qty,
                    lt
                ],
            )?;
            items.push(SaleItem {
                id: item_id,
                sale_id: sale_id.clone(),
                product_id: Some(line.product_id.clone()),
                barcode: line.barcode.clone(),
                name_snapshot: line.name.clone(),
                unit_price_cents: line.unit_price_cents,
                qty: line.qty,
                line_total_cents: lt,
            });
        }

        tx.commit()?;

        Ok(CheckoutResult {
            sale: Sale {
                id: sale_id,
                store_id: store_id.to_string(),
                total_cents,
                status: SaleStatus::Completed.as_str().into(),
                payment_method: input.payment_method,
                business_day,
                receipt_no,
                paid_at: paid_at.clone(),
                created_at: paid_at,
                voided_at: None,
                void_reason: None,
            },
            items,
        })
    }

    /// Newest first; the cursor is the `paid_at|id` of the last row of the previous page.
    pub fn list(conn: &Connection, store_id: &str, q: &SalesListQuery) -> AppResult<SalesPage> {
        let from = parse_day(&q.from)?.format("%Y-%m-%d").to_string();
        let to = parse_day(&q.to)?.format("%Y-%m-%d").to_string();
        let limit = q.limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
        let (cursor_paid_at, cursor_id) = match q.cursor.as_deref() {
            Some(c) => {
                let (p, i) = c
                    .split_once('|')
                    .ok_or_else(|| AppError::Message("Geçersiz sayfa bilgisi".into()))?;
                (Some(p.to_string()), Some(i.to_string()))
            }
            None => (None, None),
        };

        let mut stmt = conn.prepare(
            "SELECT s.id, s.business_day, s.receipt_no, s.paid_at, s.total_cents,
                    s.payment_method, s.status, s.void_reason,
                    (SELECT COALESCE(SUM(qty), 0) FROM sale_items WHERE sale_id = s.id)
             FROM sales s
             WHERE s.store_id = ?1
               AND s.business_day BETWEEN ?2 AND ?3
               AND (?4 IS NULL OR s.status = ?4)
               AND (?5 IS NULL OR s.payment_method = ?5)
               AND (?6 IS NULL OR (s.paid_at, s.id) < (?6, ?7))
             ORDER BY s.paid_at DESC, s.id DESC
             LIMIT ?8",
        )?;
        let rows = stmt.query_map(
            params![
                store_id,
                from,
                to,
                q.status.map(|s| s.as_str()),
                q.payment_method.map(|p| p.as_str()),
                cursor_paid_at,
                cursor_id,
                limit + 1
            ],
            |r| {
                Ok(SaleListItem {
                    id: r.get(0)?,
                    business_day: r.get(1)?,
                    receipt_no: r.get(2)?,
                    paid_at: r.get(3)?,
                    total_cents: r.get(4)?,
                    payment_method: PaymentMethod::from_db(&r.get::<_, String>(5)?),
                    status: r.get(6)?,
                    void_reason: r.get(7)?,
                    item_count: r.get(8)?,
                })
            },
        )?;
        let mut items = rows.collect::<Result<Vec<_>, _>>()?;

        let next_cursor = if items.len() > limit as usize {
            items.truncate(limit as usize);
            items.last().map(|s| format!("{}|{}", s.paid_at, s.id))
        } else {
            None
        };
        Ok(SalesPage { items, next_cursor })
    }

    pub fn get(conn: &Connection, store_id: &str, id: &str) -> AppResult<SaleDetail> {
        let sale = find_sale(conn, store_id, id)?.ok_or_else(sale_not_found)?;
        let mut stmt = conn.prepare(
            "SELECT id, sale_id, product_id, barcode, name_snapshot,
                    unit_price_cents, qty, line_total_cents
             FROM sale_items WHERE sale_id = ?1 ORDER BY rowid",
        )?;
        let items = stmt
            .query_map([id], |r| {
                Ok(SaleItem {
                    id: r.get(0)?,
                    sale_id: r.get(1)?,
                    product_id: r.get(2)?,
                    barcode: r.get(3)?,
                    name_snapshot: r.get(4)?,
                    unit_price_cents: r.get(5)?,
                    qty: r.get(6)?,
                    line_total_cents: r.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SaleDetail { sale, items })
    }

    pub fn void(
        conn: &Connection,
        store_id: &str,
        id: &str,
        reason: &str,
        now: DateTime<Utc>,
    ) -> AppResult<Sale> {
        let reason = normalize_void_reason(reason).map_err(AppError::Message)?;
        let tx = conn.unchecked_transaction()?;
        let sale = find_sale(&tx, store_id, id)?.ok_or_else(sale_not_found)?;
        if sale.status == SaleStatus::Voided.as_str() {
            return Err(already_voided());
        }
        let changed = tx.execute(
            "UPDATE sales SET status = 'voided', voided_at = ?1, void_reason = ?2
             WHERE id = ?3 AND store_id = ?4 AND status = 'completed'",
            params![utc_stamp(now), reason, id, store_id],
        )?;
        if changed == 0 {
            return Err(already_voided());
        }
        let sale = find_sale(&tx, store_id, id)?.ok_or_else(sale_not_found)?;
        tx.commit()?;
        Ok(sale)
    }
}

fn sale_not_found() -> AppError {
    AppError::Message("Satış bulunamadı".into())
}

fn already_voided() -> AppError {
    AppError::Message("Bu satış zaten iptal edilmiş".into())
}

fn find_sale(conn: &Connection, store_id: &str, id: &str) -> AppResult<Option<Sale>> {
    conn.query_row(
        &format!("SELECT {SALE_COLUMNS} FROM sales WHERE id = ?1 AND store_id = ?2"),
        params![id, store_id],
        map_sale,
    )
    .optional()
    .map_err(AppError::from)
}

fn map_sale(r: &rusqlite::Row<'_>) -> rusqlite::Result<Sale> {
    Ok(Sale {
        id: r.get(0)?,
        store_id: r.get(1)?,
        total_cents: r.get(2)?,
        status: r.get(3)?,
        payment_method: PaymentMethod::from_db(&r.get::<_, String>(4)?),
        business_day: r.get(5)?,
        receipt_no: r.get(6)?,
        paid_at: r.get(7)?,
        created_at: r.get(8)?,
        voided_at: r.get(9)?,
        void_reason: r.get(10)?,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::db::{run_migrations, seed_default_store, DEFAULT_STORE_ID};
    use crate::domain::{CheckoutItemInput, CreateProductInput};
    use chrono::TimeZone;

    pub fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_default_store(&conn).unwrap();
        conn
    }

    pub fn tr(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(3 * 3600)
            .unwrap()
            .with_ymd_and_hms(y, m, d, h, min, 0)
            .unwrap()
    }

    pub fn add_product(conn: &Connection, barcode: &str, price: i64) -> String {
        ProductRepo::create(
            conn,
            DEFAULT_STORE_ID,
            &CreateProductInput {
                barcode: barcode.into(),
                name: format!("Ürün {barcode}"),
                price_cents: price,
            },
        )
        .unwrap()
        .id
    }

    pub fn input(lines: &[(&str, i64, Option<i64>)], method: PaymentMethod) -> CheckoutInput {
        CheckoutInput {
            items: lines
                .iter()
                .map(|(id, qty, price)| CheckoutItemInput {
                    product_id: id.to_string(),
                    qty: *qty,
                    expected_unit_price_cents: *price,
                    barcode: None,
                    name: Some("Eski ad".into()),
                })
                .collect(),
            payment_method: method,
        }
    }

    pub fn sell(
        conn: &Connection,
        lines: &[(&str, i64, Option<i64>)],
        method: PaymentMethod,
        at: DateTime<FixedOffset>,
    ) -> CheckoutResult {
        SaleRepo::checkout(conn, DEFAULT_STORE_ID, &input(lines, method), at).unwrap()
    }

    fn sale_count(conn: &Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM sales", [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn uses_db_price_and_name() {
        let conn = test_conn();
        let a = add_product(&conn, "111", 1500);
        let b = add_product(&conn, "222", 250);
        let r = sell(
            &conn,
            &[(&a, 2, Some(1500)), (&b, 1, None), (&a, 1, None)],
            PaymentMethod::Card,
            tr(2026, 11, 4, 10, 0),
        );
        assert_eq!(r.sale.total_cents, 1500 * 3 + 250);
        assert_eq!(r.items.len(), 2);
        assert_eq!(r.items[0].name_snapshot, "Ürün 111");
        assert_eq!(r.sale.payment_method, PaymentMethod::Card);
        let stored: String = conn
            .query_row("SELECT payment_method FROM sales", [], |r| r.get(0))
            .unwrap();
        assert_eq!(stored, "card");
    }

    #[test]
    fn price_change_rejects_with_fresh_cart() {
        let conn = test_conn();
        let a = add_product(&conn, "111", 1750);
        let b = add_product(&conn, "222", 300);
        let err = SaleRepo::checkout(
            &conn,
            DEFAULT_STORE_ID,
            &input(&[(&a, 2, Some(1500)), (&b, 1, Some(300))], PaymentMethod::Cash),
            tr(2026, 11, 4, 10, 0),
        )
        .unwrap_err();
        let AppError::CartChanged(d) = err else {
            panic!("expected cart_changed");
        };
        assert_eq!(d.changed.len(), 1);
        assert_eq!(d.changed[0].old_unit_price_cents, 1500);
        assert_eq!(d.changed[0].new_unit_price_cents, 1750);
        assert!(d.removed.is_empty());
        assert_eq!(d.current.len(), 2);
        assert_eq!(d.current[0].unit_price_cents, 1750);
        assert_eq!(d.current[0].qty, 2);
        assert_eq!(sale_count(&conn), 0);
    }

    #[test]
    fn deactivated_unknown_and_foreign_products_are_removed() {
        let conn = test_conn();
        let gone = add_product(&conn, "111", 100);
        ProductRepo::deactivate(&conn, DEFAULT_STORE_ID, &gone).unwrap();
        conn.execute(
            "INSERT INTO stores (id, name, created_at) VALUES ('other', 'Diğer', 'x')",
            [],
        )
        .unwrap();
        let foreign = ProductRepo::create(
            &conn,
            "other",
            &CreateProductInput {
                barcode: "999".into(),
                name: "Yabancı".into(),
                price_cents: 100,
            },
        )
        .unwrap()
        .id;
        let ok = add_product(&conn, "222", 500);

        let err = SaleRepo::checkout(
            &conn,
            DEFAULT_STORE_ID,
            &input(
                &[(&gone, 1, Some(100)), ("nope", 1, None), (&foreign, 1, Some(100)), (&ok, 1, Some(500))],
                PaymentMethod::Cash,
            ),
            tr(2026, 11, 4, 10, 0),
        )
        .unwrap_err();
        let AppError::CartChanged(d) = err else {
            panic!("expected cart_changed");
        };
        assert_eq!(d.removed.len(), 3);
        assert_eq!(d.removed[0].name.as_deref(), Some("Eski ad"));
        assert_eq!(d.current.len(), 1);
        assert_eq!(sale_count(&conn), 0);
    }

    #[test]
    fn name_only_change_does_not_reject() {
        let conn = test_conn();
        let a = add_product(&conn, "111", 100);
        let r = sell(&conn, &[(&a, 1, Some(100))], PaymentMethod::Cash, tr(2026, 11, 4, 9, 0));
        assert_eq!(r.items[0].name_snapshot, "Ürün 111");
    }

    #[test]
    fn overflow_is_rejected() {
        let conn = test_conn();
        let a = add_product(&conn, "111", i64::MAX / 2);
        let b = add_product(&conn, "222", i64::MAX / 2);
        let err = SaleRepo::checkout(
            &conn,
            DEFAULT_STORE_ID,
            &input(&[(&a, 2, None), (&b, 2, None)], PaymentMethod::Cash),
            tr(2026, 11, 4, 9, 0),
        )
        .unwrap_err();
        assert!(err.to_string().contains("taşması"));
    }

    #[test]
    fn receipt_numbers_restart_each_local_day() {
        let conn = test_conn();
        let a = add_product(&conn, "111", 100);
        let n = |at| sell(&conn, &[(&a, 1, None)], PaymentMethod::Cash, at).sale;
        let s1 = n(tr(2026, 11, 4, 9, 0));
        let s2 = n(tr(2026, 11, 4, 12, 0));
        let s3 = n(tr(2026, 11, 4, 23, 59));
        let s4 = n(tr(2026, 11, 5, 0, 1));
        assert_eq!(
            [s1.receipt_no, s2.receipt_no, s3.receipt_no, s4.receipt_no],
            [1, 2, 3, 1]
        );
        assert_eq!(s3.business_day, "2026-11-04");
        assert_eq!(s4.business_day, "2026-11-05");
        // 00:01 local is still the previous day in UTC
        assert!(s4.paid_at.starts_with("2026-11-04T21:01"));
    }

    #[test]
    fn list_pages_and_filters() {
        let conn = test_conn();
        let a = add_product(&conn, "111", 100);
        for h in 8..13 {
            let method = if h % 2 == 0 { PaymentMethod::Cash } else { PaymentMethod::Card };
            sell(&conn, &[(&a, 2, None)], method, tr(2026, 11, 4, h, 0));
        }
        sell(&conn, &[(&a, 1, None)], PaymentMethod::Cash, tr(2026, 11, 3, 10, 0));

        let mut q = SalesListQuery {
            from: "2026-11-04".into(),
            to: "2026-11-04".into(),
            status: None,
            payment_method: None,
            limit: Some(2),
            cursor: None,
        };
        let p1 = SaleRepo::list(&conn, DEFAULT_STORE_ID, &q).unwrap();
        assert_eq!(p1.items.len(), 2);
        assert_eq!(p1.items[0].receipt_no, 5);
        assert_eq!(p1.items[0].item_count, 2);
        q.cursor = p1.next_cursor.clone();
        let p2 = SaleRepo::list(&conn, DEFAULT_STORE_ID, &q).unwrap();
        assert_eq!(p2.items[0].receipt_no, 3);
        q.cursor = p2.next_cursor.clone();
        let p3 = SaleRepo::list(&conn, DEFAULT_STORE_ID, &q).unwrap();
        assert_eq!(p3.items.len(), 1);
        assert!(p3.next_cursor.is_none());

        q.cursor = None;
        q.limit = None;
        q.payment_method = Some(PaymentMethod::Card);
        assert_eq!(SaleRepo::list(&conn, DEFAULT_STORE_ID, &q).unwrap().items.len(), 2);

        q.payment_method = None;
        q.from = "2026-11-03".into();
        assert_eq!(SaleRepo::list(&conn, DEFAULT_STORE_ID, &q).unwrap().items.len(), 6);
    }

    #[test]
    fn void_is_once_and_needs_reason() {
        let conn = test_conn();
        let a = add_product(&conn, "111", 100);
        let sale = sell(&conn, &[(&a, 1, None)], PaymentMethod::Cash, tr(2026, 11, 4, 9, 0)).sale;
        assert!(SaleRepo::void(&conn, DEFAULT_STORE_ID, &sale.id, "  ", Utc::now()).is_err());
        let voided =
            SaleRepo::void(&conn, DEFAULT_STORE_ID, &sale.id, "Yanlış okutuldu", Utc::now())
                .unwrap();
        assert_eq!(voided.status, "voided");
        assert_eq!(voided.void_reason.as_deref(), Some("Yanlış okutuldu"));
        let again = SaleRepo::void(&conn, DEFAULT_STORE_ID, &sale.id, "tekrar", Utc::now());
        assert!(again.unwrap_err().to_string().contains("zaten iptal"));
        assert!(SaleRepo::void(&conn, DEFAULT_STORE_ID, "yok", "x", Utc::now()).is_err());

        let detail = SaleRepo::get(&conn, DEFAULT_STORE_ID, &sale.id).unwrap();
        assert_eq!(detail.items.len(), 1);
        assert_eq!(detail.sale.status, "voided");
    }
}
