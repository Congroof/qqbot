use thiserror::Error;

pub type Result<T> = std::result::Result<T, AiChatError>;

#[derive(Debug, Error)]
pub enum AiChatError {
    #[error("config error: {0}")]
    Config(String),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("api error (code={code}): {message}")]
    Api { code: String, message: String },

    #[error("stream error: {0}")]
    Stream(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
