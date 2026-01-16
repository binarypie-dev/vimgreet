use thiserror::Error;

#[derive(Error, Debug)]
pub enum VimgreetError {
    #[error("greetd socket not found (GREETD_SOCK not set)")]
    SocketNotFound,

    #[error("IPC error: {0}")]
    Ipc(#[from] std::io::Error),

    #[error("Codec error: {0}")]
    Codec(#[from] greetd_ipc::codec::Error),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Session start failed: {0}")]
    SessionFailed(String),

    #[error("Terminal error: {0}")]
    Terminal(String),
}

pub type Result<T> = std::result::Result<T, VimgreetError>;
