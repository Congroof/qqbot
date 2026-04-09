pub mod payload;
pub mod resp;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// API 动作 trait，每个 API 请求类型实现此 trait。
/// `ACTION` 为协议规定的 action 字符串，`Response` 为对应的响应数据类型。
pub trait ApiAction: Serialize + Send + Sync {
    const ACTION: &'static str;
    type Response: DeserializeOwned + Send;
}

/// WebSocket API 请求包装，包含 action、params 和 echo。
#[derive(Debug, Serialize)]
pub(crate) struct ApiRequest {
    pub action: &'static str,
    pub params: serde_json::Value,
    pub echo: String,
}

/// WebSocket API 响应，包含 status、retcode、data 和 echo。
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T = serde_json::Value> {
    pub status: String,
    pub retcode: i32,
    pub data: Option<T>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub echo: Option<String>,
}
