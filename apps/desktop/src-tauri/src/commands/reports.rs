use crate::auth::require_owner;
use crate::db::AppState;
use crate::domain::{DailyReport, DayRow, TodaySummary, parse_day};
use crate::error::AppResult;
use crate::repos::ReportRepo;
use tauri::State;

#[tauri::command]
pub async fn reports_daily(state: State<'_, AppState>, date: String) -> AppResult<DailyReport> {
    require_owner(&state)?;
    let day = parse_day(&date)?;
    let tz = *chrono::Local::now().offset();
    let conn = state.conn.lock();
    ReportRepo::daily(&conn, &state.store_id, day, tz)
}

#[tauri::command]
pub async fn reports_range(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> AppResult<Vec<DayRow>> {
    require_owner(&state)?;
    let (from, to) = (parse_day(&from)?, parse_day(&to)?);
    let conn = state.conn.lock();
    ReportRepo::range(&conn, &state.store_id, from, to)
}

/// Public: the Kasa header shows today's net total so employees don't need to ask.
#[tauri::command]
pub async fn reports_today_summary(state: State<'_, AppState>) -> AppResult<TodaySummary> {
    let today = chrono::Local::now().date_naive();
    let conn = state.conn.lock();
    ReportRepo::today_summary(&conn, &state.store_id, today)
}
