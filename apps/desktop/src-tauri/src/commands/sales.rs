use crate::auth::require_owner;
use crate::db::AppState;
use crate::domain::{CheckoutInput, CheckoutResult, Sale, SaleDetail, SalesListQuery, SalesPage};
use crate::error::AppResult;
use crate::repos::SaleRepo;
use tauri::State;

#[tauri::command]
pub fn sales_checkout(
    state: State<'_, AppState>,
    input: CheckoutInput,
) -> AppResult<CheckoutResult> {
    let conn = state.conn.lock();
    SaleRepo::checkout(
        &conn,
        &state.store_id,
        &input,
        chrono::Local::now().fixed_offset(),
    )
}

#[tauri::command]
pub async fn sales_list(state: State<'_, AppState>, query: SalesListQuery) -> AppResult<SalesPage> {
    require_owner(&state)?;
    let conn = state.conn.lock();
    SaleRepo::list(&conn, &state.store_id, &query)
}

#[tauri::command]
pub async fn sales_get(state: State<'_, AppState>, id: String) -> AppResult<SaleDetail> {
    require_owner(&state)?;
    let conn = state.conn.lock();
    SaleRepo::get(&conn, &state.store_id, &id)
}

#[tauri::command]
pub async fn sales_void(
    state: State<'_, AppState>,
    id: String,
    reason: String,
) -> AppResult<Sale> {
    require_owner(&state)?;
    let conn = state.conn.lock();
    SaleRepo::void(&conn, &state.store_id, &id, &reason, chrono::Utc::now())
}
