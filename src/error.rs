use thiserror::Error;

#[derive(Debug, Error)]
#[error("[silicon.nvim]: {0}")]
pub enum Error {
    Generic(String),
    Anyhow(#[from] anyhow::Error),
    NvimOxi(#[from] nvim_oxi::Error),
    NvimApi(#[from] nvim_oxi::api::Error),
    Syntect(#[from] syntect::Error),
    Font(#[from] silicon::error::FontError),
    Color(#[from] silicon::error::ParseColorError),
    Format(#[from] time::error::Format),
    Parse(#[from] time::error::InvalidFormatDescription),
    Lua(#[from] nvim_oxi::lua::Error)
}