use crate::error::{AppError, AppResult};
use chrono::{DateTime, FixedOffset, NaiveDate, Timelike, Utc};
use serde::Serialize;

pub const TOP_PRODUCTS: i64 = 10;
pub const MAX_RANGE_DAYS: i64 = 92;

/// Accepts `YYYY-MM-DD` only.
pub fn parse_day(value: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| AppError::Message(format!("Geçersiz tarih: {value}")))
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TopProduct {
    pub product_id: Option<String>,
    pub barcode: String,
    pub name: String,
    pub qty: i64,
    pub revenue_cents: i64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Default)]
pub struct HourBucket {
    pub hour: u32,
    pub count: i64,
    pub total_cents: i64,
}

/// `gross` counts every sale recorded that day; `net = gross - voided - returns`.
/// Cash/card totals and `sale_count` cover non-voided sales only.
#[derive(Debug, Clone, Serialize)]
pub struct DailyReport {
    pub business_day: String,
    pub sale_count: i64,
    pub gross_total_cents: i64,
    pub cash_total_cents: i64,
    pub card_total_cents: i64,
    pub voided_count: i64,
    pub voided_total_cents: i64,
    pub returns_total_cents: i64,
    pub net_total_cents: i64,
    pub average_basket_cents: i64,
    pub first_sale_at: Option<String>,
    pub last_sale_at: Option<String>,
    pub top_by_qty: Vec<TopProduct>,
    pub top_by_revenue: Vec<TopProduct>,
    pub hourly: Vec<HourBucket>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DayRow {
    pub business_day: String,
    pub sale_count: i64,
    pub net_total_cents: i64,
    pub cash_total_cents: i64,
    pub card_total_cents: i64,
    pub voided_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TodaySummary {
    pub business_day: String,
    pub sale_count: i64,
    pub net_total_cents: i64,
}

/// 24 buckets by local hour of `paid_at`.
pub fn hour_buckets(sales: &[(DateTime<Utc>, i64)], tz: FixedOffset) -> Vec<HourBucket> {
    let mut buckets: Vec<HourBucket> = (0..24)
        .map(|hour| HourBucket {
            hour,
            ..Default::default()
        })
        .collect();
    for (paid_at, total) in sales {
        let hour = paid_at.with_timezone(&tz).hour() as usize;
        buckets[hour].count += 1;
        buckets[hour].total_cents += total;
    }
    buckets
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn buckets_use_local_hours() {
        let tz = FixedOffset::east_opt(3 * 3600).unwrap();
        let sales = vec![
            (Utc.with_ymd_and_hms(2026, 11, 4, 20, 59, 0).unwrap(), 100), // 23:59 local
            (Utc.with_ymd_and_hms(2026, 11, 4, 6, 30, 0).unwrap(), 50),   // 09:30 local
            (Utc.with_ymd_and_hms(2026, 11, 4, 6, 45, 0).unwrap(), 25),
        ];
        let b = hour_buckets(&sales, tz);
        assert_eq!(b.len(), 24);
        assert_eq!(b[23].count, 1);
        assert_eq!(b[9].count, 2);
        assert_eq!(b[9].total_cents, 75);
        assert_eq!(b[20].count, 0);
    }

    #[test]
    fn parses_days() {
        assert!(parse_day("2026-11-04").is_ok());
        assert!(parse_day("04.11.2026").is_err());
        assert!(parse_day("2026-13-01").is_err());
    }
}
