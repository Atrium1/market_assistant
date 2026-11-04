use crate::domain::{
    DailyReport, DayRow, MAX_RANGE_DAYS, TOP_PRODUCTS, TodaySummary, TopProduct, hour_buckets,
};
use crate::error::{AppError, AppResult};
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use rusqlite::{Connection, params};

pub struct ReportRepo;

struct DayTotals {
    sale_count: i64,
    gross: i64,
    cash: i64,
    card: i64,
    voided_count: i64,
    voided_total: i64,
    first: Option<String>,
    last: Option<String>,
}

fn day_key(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

fn day_totals(conn: &Connection, store_id: &str, day: &str) -> AppResult<DayTotals> {
    conn.query_row(
        "SELECT
            COALESCE(SUM(status = 'completed'), 0),
            COALESCE(SUM(total_cents), 0),
            COALESCE(SUM(CASE WHEN status = 'completed' AND payment_method = 'cash'
                              THEN total_cents ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN status = 'completed' AND payment_method = 'card'
                              THEN total_cents ELSE 0 END), 0),
            COALESCE(SUM(status = 'voided'), 0),
            COALESCE(SUM(CASE WHEN status = 'voided' THEN total_cents ELSE 0 END), 0),
            MIN(CASE WHEN status = 'completed' THEN paid_at END),
            MAX(CASE WHEN status = 'completed' THEN paid_at END)
         FROM sales
         WHERE store_id = ?1 AND business_day = ?2",
        params![store_id, day],
        |r| {
            Ok(DayTotals {
                sale_count: r.get(0)?,
                gross: r.get(1)?,
                cash: r.get(2)?,
                card: r.get(3)?,
                voided_count: r.get(4)?,
                voided_total: r.get(5)?,
                first: r.get(6)?,
                last: r.get(7)?,
            })
        },
    )
    .map_err(AppError::from)
}

fn top_products(
    conn: &Connection,
    store_id: &str,
    day: &str,
    order_by: &str,
) -> AppResult<Vec<TopProduct>> {
    let sql = format!(
        "SELECT si.product_id, si.barcode, MAX(si.name_snapshot),
                SUM(si.qty) AS qty, SUM(si.line_total_cents) AS revenue
         FROM sale_items si
         JOIN sales s ON s.id = si.sale_id
         WHERE s.store_id = ?1 AND s.business_day = ?2 AND s.status = 'completed'
         GROUP BY COALESCE(si.product_id, si.barcode)
         ORDER BY {order_by}
         LIMIT ?3"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![store_id, day, TOP_PRODUCTS], |r| {
        Ok(TopProduct {
            product_id: r.get(0)?,
            barcode: r.get(1)?,
            name: r.get(2)?,
            qty: r.get(3)?,
            revenue_cents: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

impl ReportRepo {
    pub fn daily(
        conn: &Connection,
        store_id: &str,
        day: NaiveDate,
        tz: FixedOffset,
    ) -> AppResult<DailyReport> {
        let key = day_key(day);
        let t = day_totals(conn, store_id, &key)?;

        let mut stmt = conn.prepare(
            "SELECT paid_at, total_cents FROM sales
             WHERE store_id = ?1 AND business_day = ?2 AND status = 'completed'",
        )?;
        let mut paid = Vec::new();
        for row in stmt.query_map(params![store_id, key], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })? {
            let (at, total) = row?;
            if let Ok(at) = DateTime::parse_from_rfc3339(&at) {
                paid.push((at.with_timezone(&Utc), total));
            }
        }

        let returns_total_cents = 0;
        let net = t.gross - t.voided_total - returns_total_cents;
        Ok(DailyReport {
            business_day: key.clone(),
            sale_count: t.sale_count,
            gross_total_cents: t.gross,
            cash_total_cents: t.cash,
            card_total_cents: t.card,
            voided_count: t.voided_count,
            voided_total_cents: t.voided_total,
            returns_total_cents,
            net_total_cents: net,
            average_basket_cents: if t.sale_count > 0 { net / t.sale_count } else { 0 },
            first_sale_at: t.first,
            last_sale_at: t.last,
            top_by_qty: top_products(conn, store_id, &key, "qty DESC, revenue DESC")?,
            top_by_revenue: top_products(conn, store_id, &key, "revenue DESC, qty DESC")?,
            hourly: hour_buckets(&paid, tz),
        })
    }

    /// One row per day in `from..=to`, including days without sales.
    pub fn range(
        conn: &Connection,
        store_id: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> AppResult<Vec<DayRow>> {
        if to < from {
            return Err(AppError::Message("Bitiş tarihi başlangıçtan önce olamaz".into()));
        }
        if (to - from).num_days() >= MAX_RANGE_DAYS {
            return Err(AppError::Message(format!(
                "En fazla {MAX_RANGE_DAYS} günlük rapor alınabilir"
            )));
        }
        let mut stmt = conn.prepare(
            "SELECT business_day,
                    COALESCE(SUM(status = 'completed'), 0),
                    COALESCE(SUM(CASE WHEN status = 'completed' THEN total_cents ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN status = 'completed' AND payment_method = 'cash'
                                      THEN total_cents ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN status = 'completed' AND payment_method = 'card'
                                      THEN total_cents ELSE 0 END), 0),
                    COALESCE(SUM(status = 'voided'), 0)
             FROM sales
             WHERE store_id = ?1 AND business_day BETWEEN ?2 AND ?3
             GROUP BY business_day",
        )?;
        let found = stmt
            .query_map(params![store_id, day_key(from), day_key(to)], |r| {
                Ok(DayRow {
                    business_day: r.get(0)?,
                    sale_count: r.get(1)?,
                    net_total_cents: r.get(2)?,
                    cash_total_cents: r.get(3)?,
                    card_total_cents: r.get(4)?,
                    voided_count: r.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(from
            .iter_days()
            .take_while(|d| *d <= to)
            .map(|d| {
                let key = day_key(d);
                found
                    .iter()
                    .find(|r| r.business_day == key)
                    .cloned()
                    .unwrap_or(DayRow {
                        business_day: key,
                        sale_count: 0,
                        net_total_cents: 0,
                        cash_total_cents: 0,
                        card_total_cents: 0,
                        voided_count: 0,
                    })
            })
            .collect())
    }

    pub fn today_summary(
        conn: &Connection,
        store_id: &str,
        today: NaiveDate,
    ) -> AppResult<TodaySummary> {
        let key = day_key(today);
        let t = day_totals(conn, store_id, &key)?;
        Ok(TodaySummary {
            business_day: key,
            sale_count: t.sale_count,
            net_total_cents: t.gross - t.voided_total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DEFAULT_STORE_ID;
    use crate::domain::PaymentMethod;
    use crate::repos::SaleRepo;
    use crate::repos::sale::tests::{add_product, sell, test_conn, tr};

    fn tz() -> FixedOffset {
        FixedOffset::east_opt(3 * 3600).unwrap()
    }

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn daily_totals_with_cash_card_and_void() {
        let conn = test_conn();
        let su = add_product(&conn, "111", 1000);
        let ekmek = add_product(&conn, "222", 500);
        sell(&conn, &[(&su, 2, None)], PaymentMethod::Cash, tr(2026, 11, 4, 9, 15));
        sell(&conn, &[(&ekmek, 4, None)], PaymentMethod::Card, tr(2026, 11, 4, 9, 45));
        let wrong =
            sell(&conn, &[(&su, 1, None)], PaymentMethod::Cash, tr(2026, 11, 4, 14, 0)).sale;
        SaleRepo::void(&conn, DEFAULT_STORE_ID, &wrong.id, "Yanlış", Utc::now()).unwrap();
        sell(&conn, &[(&su, 1, None)], PaymentMethod::Cash, tr(2026, 11, 4, 23, 59));
        sell(&conn, &[(&su, 5, None)], PaymentMethod::Cash, tr(2026, 11, 5, 0, 1));

        let r = ReportRepo::daily(&conn, DEFAULT_STORE_ID, day("2026-11-04"), tz()).unwrap();
        assert_eq!(r.sale_count, 3);
        assert_eq!(r.gross_total_cents, 2000 + 2000 + 1000 + 1000);
        assert_eq!(r.voided_count, 1);
        assert_eq!(r.voided_total_cents, 1000);
        assert_eq!(r.net_total_cents, 5000);
        assert_eq!(r.cash_total_cents, 3000);
        assert_eq!(r.card_total_cents, 2000);
        assert_eq!(r.average_basket_cents, 5000 / 3);
        assert_eq!(r.hourly[9].count, 2);
        assert_eq!(r.hourly[23].count, 1);
        assert_eq!(r.hourly[14].count, 0);
        assert_eq!(r.top_by_qty[0].barcode, "222");
        assert_eq!(r.top_by_qty[0].qty, 4);
        assert_eq!(r.top_by_revenue[0].barcode, "111");
        assert_eq!(r.top_by_revenue[0].revenue_cents, 3000);

        let today = ReportRepo::today_summary(&conn, DEFAULT_STORE_ID, day("2026-11-05")).unwrap();
        assert_eq!(today.sale_count, 1);
        assert_eq!(today.net_total_cents, 5000);
    }

    #[test]
    fn empty_day_is_zero() {
        let conn = test_conn();
        let r = ReportRepo::daily(&conn, DEFAULT_STORE_ID, day("2026-11-04"), tz()).unwrap();
        assert_eq!(r.sale_count, 0);
        assert_eq!(r.net_total_cents, 0);
        assert_eq!(r.average_basket_cents, 0);
        assert!(r.first_sale_at.is_none());
        assert!(r.top_by_qty.is_empty());
        assert_eq!(r.hourly.len(), 24);
    }

    #[test]
    fn range_fills_missing_days() {
        let conn = test_conn();
        let su = add_product(&conn, "111", 1000);
        sell(&conn, &[(&su, 1, None)], PaymentMethod::Card, tr(2026, 11, 2, 10, 0));
        sell(&conn, &[(&su, 2, None)], PaymentMethod::Cash, tr(2026, 11, 4, 10, 0));
        let rows =
            ReportRepo::range(&conn, DEFAULT_STORE_ID, day("2026-11-01"), day("2026-11-04"))
                .unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].sale_count, 0);
        assert_eq!(rows[1].card_total_cents, 1000);
        assert_eq!(rows[3].cash_total_cents, 2000);
        assert!(
            ReportRepo::range(&conn, DEFAULT_STORE_ID, day("2026-11-04"), day("2026-11-01"))
                .is_err()
        );
        assert!(
            ReportRepo::range(&conn, DEFAULT_STORE_ID, day("2026-01-01"), day("2026-12-31"))
                .is_err()
        );
    }
}
