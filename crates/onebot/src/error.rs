use thiserror::Error;

#[derive(Debug, Error)]
pub enum OneBotError {
    #[error("auth failed: {0}")]
    Auth(String),

    #[error("websocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("connection closed")]
    ConnectionClosed,

    #[error("api call failed: retcode={retcode}, message={message}")]
    ApiError { retcode: i32, message: String },
}
