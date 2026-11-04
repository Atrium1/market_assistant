use crate::auth::require_owner;
use crate::db::AppState;
use crate::domain::{
    CreateProductInput, Product, UpdateProductInput, validate_create,
};
use crate::error::{AppError, AppResult};
use crate::repos::ProductRepo;
use tauri::State;

#[tauri::command]
pub fn products_list(state: State<'_, AppState>) -> AppResult<Vec<Product>> {
    let conn = state.conn.lock();
    ProductRepo::list_active(&conn, &state.store_id)
}

#[tauri::command]
pub fn products_get_by_barcode(
    state: State<'_, AppState>,
    barcode: String,
) -> AppResult<Product> {
    let conn = state.conn.lock();
    ProductRepo::get_by_barcode(&conn, &state.store_id, &barcode)?
        .ok_or(AppError::NotFound)
}

#[tauri::command]
pub fn products_create(
    state: State<'_, AppState>,
    input: CreateProductInput,
) -> AppResult<Product> {
    require_owner(&state)?;
    validate_create(&input).map_err(AppError::Message)?;
    let conn = state.conn.lock();
    let (product, _) = ProductRepo::upsert_by_barcode(&conn, &state.store_id, &input, true)?;
    Ok(product)
}

#[tauri::command]
pub fn products_update(
    state: State<'_, AppState>,
    input: UpdateProductInput,
) -> AppResult<Product> {
    require_owner(&state)?;
    let conn = state.conn.lock();
    ProductRepo::update(&conn, &state.store_id, &input)
}

#[tauri::command]
pub fn products_deactivate(state: State<'_, AppState>, id: String) -> AppResult<()> {
    require_owner(&state)?;
    let conn = state.conn.lock();
    ProductRepo::deactivate(&conn, &state.store_id, &id)
}
