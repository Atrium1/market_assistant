use crate::error::{AppError, AppResult};
use calamine::{open_workbook_auto, Data, Reader};
use std::path::Path;

pub const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Empty,
    Text(String),
    Number(f64),
}

impl Cell {
    pub fn is_empty(&self) -> bool {
        match self {
            Cell::Empty => true,
            Cell::Text(s) => s.trim().is_empty(),
            Cell::Number(_) => false,
        }
    }

    pub fn as_text(&self) -> String {
        match self {
            Cell::Empty => String::new(),
            Cell::Text(s) => s.trim().to_string(),
            Cell::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{n:.0}")
                } else {
                    n.to_string()
                }
            }
        }
    }
}

pub fn read_rows(path: &Path) -> AppResult<Vec<Vec<Cell>>> {
    let meta = std::fs::metadata(path)
        .map_err(|_| AppError::Message("Dosya bulunamadı".into()))?;
    if meta.len() > MAX_FILE_BYTES {
        return Err(AppError::Message(
            "Dosya çok büyük (en fazla 10 MB)".into(),
        ));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "csv" | "txt" => {
            let bytes = std::fs::read(path)?;
            Ok(parse_csv(&decode_text(&bytes)))
        }
        "xlsx" | "xlsm" | "xls" | "ods" => read_spreadsheet(path),
        _ => Err(AppError::Message(
            "Desteklenmeyen dosya türü. Excel (.xlsx) veya CSV seçin".into(),
        )),
    }
}

fn read_spreadsheet(path: &Path) -> AppResult<Vec<Vec<Cell>>> {
    let unreadable = || AppError::Message("Excel dosyası okunamadı".into());
    let mut workbook = open_workbook_auto(path).map_err(|_| unreadable())?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| AppError::Message("Excel dosyasında sayfa yok".into()))?
        .map_err(|_| unreadable())?;

    Ok(range
        .rows()
        .map(|row| {
            row.iter()
                .map(|cell| match cell {
                    Data::Empty => Cell::Empty,
                    Data::Int(i) => Cell::Number(*i as f64),
                    Data::Float(f) => Cell::Number(*f),
                    Data::String(s) => Cell::Text(s.clone()),
                    Data::Bool(b) => Cell::Text(b.to_string()),
                    Data::DateTimeIso(s) | Data::DurationIso(s) => Cell::Text(s.clone()),
                    Data::DateTime(d) => Cell::Text(d.to_string()),
                    Data::Error(_) => Cell::Text("#HATA".into()),
                })
                .collect()
        })
        .collect())
}

/// UTF-8 (with or without BOM), falling back to Windows-1254 — what Turkish Excel writes
/// for a plain "CSV" save.
pub fn decode_text(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (text, _, _) = encoding_rs::WINDOWS_1254.decode(bytes);
            text.into_owned()
        }
    }
}

fn detect_delimiter(text: &str) -> u8 {
    let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    [b';', b'\t', b',']
        .into_iter()
        .max_by_key(|d| first.bytes().filter(|b| b == d).count())
        .filter(|d| first.bytes().any(|b| b == *d))
        .unwrap_or(b',')
}

pub fn parse_csv(text: &str) -> Vec<Vec<Cell>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(detect_delimiter(text))
        .from_reader(text.as_bytes());

    reader
        .records()
        .filter_map(|r| r.ok())
        .map(|record| {
            record
                .iter()
                .map(|v| {
                    if v.trim().is_empty() {
                        Cell::Empty
                    } else {
                        Cell::Text(v.to_string())
                    }
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_utf8_bom_and_cp1254() {
        let bom = [&[0xEF, 0xBB, 0xBF][..], "Süt".as_bytes()].concat();
        assert_eq!(decode_text(&bom), "Süt");

        let (cp1254, _, _) = encoding_rs::WINDOWS_1254.encode("Şeker;Çay;İçecek");
        assert_eq!(decode_text(&cp1254), "Şeker;Çay;İçecek");
    }

    #[test]
    fn detects_semicolon_and_comma() {
        let rows = parse_csv("Barkod;Ürün adı;Fiyat\n869;Süt 1L;12,50\n");
        assert_eq!(rows[1][2], Cell::Text("12,50".into()));

        let rows = parse_csv("barcode,name,price\n869,\"Süt, tam yağlı\",12.50\n");
        assert_eq!(rows[1][1], Cell::Text("Süt, tam yağlı".into()));
    }
}
