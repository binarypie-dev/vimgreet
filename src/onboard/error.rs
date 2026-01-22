use thiserror::Error;

#[derive(Error, Debug)]
pub enum OnboardError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command failed: {0}")]
    Command(String),

    #[error("User creation failed: {0}")]
    UserCreation(String),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, OnboardError>;
