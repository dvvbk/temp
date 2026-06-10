//! profak-core - rdzeń logiki ProFak przeniesiony z C# (ProFak.DB) do Rusta.
//! Warstwa niezależna od UI, używana przez powłokę Tauri.

pub mod db;
pub mod faktura;
pub mod liczby;
pub mod model;
pub mod numerator;
pub mod repo;

pub use rusqlite;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BladProFak {
    #[error("Błąd bazy danych: {0}")]
    Baza(#[from] rusqlite::Error),
    #[error("{0}")]
    Logika(String),
}

pub type Wynik<T> = Result<T, BladProFak>;
