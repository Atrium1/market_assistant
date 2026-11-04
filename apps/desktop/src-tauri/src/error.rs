use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("Ürün bulunamadı")]
    NotFound,
    #[error("Bu özellik henüz hazır değil")]
    NotImplemented,
    #[error("Veritabanı hatası: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Dosya hatası: {0}")]
    Io(#[from] std::io::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
