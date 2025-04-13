use std::string::FromUtf8Error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),
    #[error(transparent)]
    Scraper(Box<dyn core::error::Error>),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
