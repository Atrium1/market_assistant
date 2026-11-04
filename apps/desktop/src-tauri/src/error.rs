use crate::domain::CartChangedDetails;
use serde::Serialize;
use serde::ser::SerializeStruct;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("Ürün bulunamadı")]
    NotFound,
    #[error("Bu özellik henüz hazır değil")]
    NotImplemented,
    #[error("Bu işlem için sahip girişi gerekli.")]
    OwnerRequired,
    #[error("Oturum süresi doldu. Lütfen tekrar giriş yapın.")]
    SessionExpired,
    #[error("Fiyatı değişen ürünler var, sepet güncellendi — toplamı tekrar kontrol edin")]
    CartChanged(CartChangedDetails),
    #[error("Veritabanı hatası: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Dosya hatası: {0}")]
    Io(#[from] std::io::Error),
}

impl AppError {
    /// Stable machine-readable code; the frontend and a future HTTP API branch on this.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Message(_) => "message",
            AppError::NotFound => "not_found",
            AppError::NotImplemented => "not_implemented",
            AppError::OwnerRequired => "owner_required",
            AppError::SessionExpired => "session_expired",
            AppError::CartChanged(_) => "cart_changed",
            AppError::Db(_) => "db",
            AppError::Io(_) => "io",
        }
    }
}

/// Serialized as `{ code, message, details? }`.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let details = match self {
            AppError::CartChanged(d) => Some(d),
            _ => None,
        };
        let mut s = serializer.serialize_struct("AppError", 3)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.to_string())?;
        s.serialize_field("details", &details)?;
        s.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_code_message_details() {
        let plain = serde_json::to_value(AppError::Message("Sepet boş".into())).unwrap();
        assert_eq!(
            plain,
            serde_json::json!({ "code": "message", "message": "Sepet boş", "details": null })
        );

        let changed = serde_json::to_value(AppError::CartChanged(CartChangedDetails {
            changed: vec![],
            removed: vec![],
            current: vec![],
        }))
        .unwrap();
        assert_eq!(changed["code"], "cart_changed");
        assert!(changed["details"]["current"].is_array());
    }
}
