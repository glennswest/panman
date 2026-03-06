use thiserror::Error;

#[derive(Debug, Error)]
pub enum PanmanError {
    #[error("config error: {0}")]
    Config(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("websocket error: {0}")]
    WebSocket(String),

    #[error("display error: {0}")]
    Display(String),

    #[error("touch error: {0}")]
    Touch(String),

    #[error("ota error: {0}")]
    Ota(String),

    #[error("wifi error: {0}")]
    Wifi(String),

    #[error("hal error: {0}")]
    Hal(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("toml parse error: {0}")]
    Toml(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, PanmanError>;
