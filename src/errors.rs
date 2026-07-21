use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("UTF-8 validation error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("XDG directory error: {0}")]
    Xdg(#[from] xdg::BaseDirectoriesError),

    #[error("Glsl/Gtk error: {0}")]
    Glib(#[from] glib::Error),


    #[error("Operation failed: {0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
