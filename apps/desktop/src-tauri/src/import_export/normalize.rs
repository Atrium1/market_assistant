use super::read::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Barcode,
    Name,
    Price,
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnMap {
    pub barcode: usize,
    pub name: usize,
    pub price: usize,
}

/// Turkish-aware lowercase with diacritics folded: "ÜRÜN ADI" and "urun adi" compare equal.
pub fn fold_tr(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ç' | 'Ç' => 'c',
            'ğ' | 'Ğ' => 'g',
            'ı' | 'I' | 'İ' | 'i' => 'i',
            'ö' | 'Ö' => 'o',
            'ş' | 'Ş' => 's',
            'ü' | 'Ü' => 'u',
            other => other.to_lowercase().next().unwrap_or(other),
        })
        .collect()
}

pub fn match_header(raw: &str) -> Option<Column> {
    let key: String = fold_tr(raw).chars().filter(|c| c.is_alphanumeric()).collect();
    match key.as_str() {
        "barkod" | "barkodno" | "barcode" | "ean" => Some(Column::Barcode),
        "urunadi" | "urunismi" | "urun" | "ad" | "adi" | "isim" | "name" => Some(Column::Name),
        "fiyat" | "fiyati" | "satisfiyati" | "satisfiyat" | "price" => Some(Column::Price),
        _ => None,
    }
}

/// Find the header row among the first few rows; returns its index and column positions.
pub fn find_header(rows: &[Vec<Cell>]) -> Option<(usize, ColumnMap)> {
    for (idx, row) in rows.iter().enumerate().take(5) {
        let (mut barcode, mut name, mut price) = (None, None, None);
        for (col, cell) in row.iter().enumerate() {
            match match_header(&cell.as_text()) {
                Some(Column::Barcode) if barcode.is_none() => barcode = Some(col),
                Some(Column::Name) if name.is_none() => name = Some(col),
                Some(Column::Price) if price.is_none() => price = Some(col),
                _ => {}
            }
        }
        if let (Some(barcode), Some(name), Some(price)) = (barcode, name, price) {
            return Some((idx, ColumnMap { barcode, name, price }));
        }
    }
    None
}

pub fn parse_barcode(cell: &Cell) -> Result<String, String> {
    match cell {
        Cell::Empty => Err("Barkod boş".into()),
        Cell::Number(n) => {
            if *n < 0.0 || n.fract() != 0.0 {
                Err("Barkod geçersiz".into())
            } else {
                Ok(format!("{n:.0}"))
            }
        }
        Cell::Text(s) => {
            let s = s.trim();
            if s.is_empty() {
                return Err("Barkod boş".into());
            }
            let upper = s.to_uppercase();
            if upper.contains("E+") && s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                return Err(format!(
                    "Barkod bilimsel gösterimde ({s}); Excel'de sütunu metin yapın"
                ));
            }
            Ok(s.to_string())
        }
    }
}

pub fn barcode_warning(barcode: &str) -> Option<String> {
    if barcode.chars().all(|c| c.is_ascii_digit()) {
        None
    } else {
        Some("Barkod rakam dışı karakter içeriyor".into())
    }
}

pub fn parse_name(cell: &Cell) -> Result<String, String> {
    let name = cell.as_text();
    if name.is_empty() {
        Err("Ürün adı boş".into())
    } else {
        Ok(name)
    }
}

pub fn parse_price(cell: &Cell) -> Result<i64, String> {
    match cell {
        Cell::Empty => Err("Fiyat boş".into()),
        Cell::Number(n) => cents_from_f64(*n),
        Cell::Text(s) => parse_price_text(s),
    }
}

fn cents_from_f64(n: f64) -> Result<i64, String> {
    if !n.is_finite() {
        return Err("Fiyat geçersiz".into());
    }
    if n < 0.0 {
        return Err("Fiyat negatif olamaz".into());
    }
    Ok((n * 100.0).round() as i64)
}

/// Accepts "12,50", "12.50", "1.250", "1.234,50", "1,234.50", "₺12,50", "12,50 TL".
/// With both separators the last one is decimal; a lone "." followed by exactly three
/// digits is a Turkish thousands separator.
pub fn parse_price_text(raw: &str) -> Result<i64, String> {
    let cleaned: String = raw
        .replace('₺', "")
        .replace("TL", "")
        .replace("tl", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if cleaned.is_empty() {
        return Err("Fiyat boş".into());
    }
    if cleaned.starts_with('-') {
        return Err("Fiyat negatif olamaz".into());
    }

    let last_comma = cleaned.rfind(',');
    let last_dot = cleaned.rfind('.');
    let normalized = match (last_comma, last_dot) {
        (Some(c), Some(d)) if c > d => cleaned.replace('.', "").replace(',', "."),
        (Some(_), Some(_)) => cleaned.replace(',', ""),
        (Some(_), None) => {
            if cleaned.matches(',').count() > 1 {
                cleaned.replace(',', "")
            } else {
                cleaned.replace(',', ".")
            }
        }
        (None, Some(d)) => {
            let decimals = cleaned.len() - d - 1;
            if cleaned.matches('.').count() > 1 || decimals == 3 {
                cleaned.replace('.', "")
            } else {
                cleaned
            }
        }
        (None, None) => cleaned,
    };

    let value: f64 = normalized
        .parse()
        .map_err(|_| format!("Fiyat anlaşılamadı ({})", raw.trim()))?;
    cents_from_f64(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prices() {
        let cases = [
            ("12,50", 1250),
            ("12.50", 1250),
            ("12", 1200),
            ("1.250", 125_000),
            ("1.234,50", 123_450),
            ("1,234.50", 123_450),
            ("₺ 7,5", 750),
            ("19,99 TL", 1999),
            ("0,05", 5),
        ];
        for (input, expected) in cases {
            assert_eq!(parse_price_text(input), Ok(expected), "{input}");
        }
        assert!(parse_price_text("-3").is_err());
        assert!(parse_price_text("abc").is_err());
        assert_eq!(parse_price(&Cell::Number(12.499)), Ok(1250));
    }

    #[test]
    fn headers_are_turkish_tolerant() {
        assert_eq!(match_header("BARKOD"), Some(Column::Barcode));
        assert_eq!(match_header("Ürün Adı"), Some(Column::Name));
        assert_eq!(match_header("ÜRÜN ADI"), Some(Column::Name));
        assert_eq!(match_header("urun_adi"), Some(Column::Name));
        assert_eq!(match_header("Satış Fiyatı"), Some(Column::Price));
        assert_eq!(match_header("Stok"), None);
    }

    #[test]
    fn finds_header_below_title_row() {
        let rows = vec![
            vec![Cell::Text("Fiyat Listesi".into())],
            vec![
                Cell::Text("Fiyat".into()),
                Cell::Text("Barkod".into()),
                Cell::Text("Ürün adı".into()),
            ],
        ];
        let (idx, map) = find_header(&rows).unwrap();
        assert_eq!(idx, 1);
        assert_eq!((map.barcode, map.name, map.price), (1, 2, 0));
    }

    #[test]
    fn barcodes() {
        assert_eq!(
            parse_barcode(&Cell::Number(8690000000001.0)),
            Ok("8690000000001".into())
        );
        assert!(parse_barcode(&Cell::Text("8,69E+12".into()))
            .unwrap_err()
            .contains("bilimsel"));
        assert!(parse_barcode(&Cell::Empty).is_err());
        assert!(barcode_warning("ABC-1").is_some());
        assert!(barcode_warning("0123").is_none());
    }
}
