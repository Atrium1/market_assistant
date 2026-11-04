use crate::domain::Product;
use crate::error::{AppError, AppResult};
use rust_xlsxwriter::{Format, Workbook, XlsxError};
use std::path::Path;

fn xlsx_err(e: XlsxError) -> AppError {
    AppError::Message(format!("Excel dosyası yazılamadı: {e}"))
}

pub fn write_products(path: &Path, products: &[Product]) -> AppResult<()> {
    let rows: Vec<(String, String, f64)> = products
        .iter()
        .map(|p| (p.barcode.clone(), p.name.clone(), p.price_cents as f64 / 100.0))
        .collect();
    write_sheet(path, &rows)
}

pub fn write_template(path: &Path) -> AppResult<()> {
    write_sheet(
        path,
        &[(
            "8690000000001".into(),
            "ÖRNEK - bu satırı silip kendi ürünlerinizi yazın".into(),
            25.5,
        )],
    )
}

fn write_sheet(path: &Path, rows: &[(String, String, f64)]) -> AppResult<()> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Ürünler").map_err(xlsx_err)?;

    let header = Format::new().set_bold();
    // Text format keeps leading zeros and stops Excel turning barcodes into 8,69E+12.
    let text = Format::new().set_num_format("@");
    let money = Format::new().set_num_format("#,##0.00");

    sheet.set_column_width(0, 18).map_err(xlsx_err)?;
    sheet.set_column_width(1, 42).map_err(xlsx_err)?;
    sheet.set_column_width(2, 12).map_err(xlsx_err)?;
    sheet.set_column_format(0, &text).map_err(xlsx_err)?;
    sheet.set_column_format(2, &money).map_err(xlsx_err)?;

    for (col, title) in ["Barkod", "Ürün adı", "Fiyat"].iter().enumerate() {
        sheet
            .write_string_with_format(0, col as u16, *title, &header)
            .map_err(xlsx_err)?;
    }
    for (i, (barcode, name, price)) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        sheet
            .write_string_with_format(r, 0, barcode, &text)
            .map_err(xlsx_err)?;
        sheet.write_string(r, 1, name).map_err(xlsx_err)?;
        sheet
            .write_number_with_format(r, 2, *price, &money)
            .map_err(xlsx_err)?;
    }

    workbook.save(path).map_err(xlsx_err)
}
