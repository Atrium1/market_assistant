use crate::db::AppState;
use crate::domain::{CheckoutInput, CheckoutResult};
use crate::error::AppResult;
use crate::repos::SaleRepo;
use tauri::State;

#[tauri::command]
pub fn sales_checkout(
    state: State<'_, AppState>,
    input: CheckoutInput,
) -> AppResult<CheckoutResult> {
    let conn = state.conn.lock();
    SaleRepo::checkout(&conn, &state.store_id, &input)
}
