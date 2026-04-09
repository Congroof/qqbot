use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::Message;

use crate::api::{ApiAction, ApiRequest, ApiResponse};
use crate::error::OneBotError;
use crate::event::Event;

/// 正向 WebSocket 连接配置
#[derive(Debug, Clone)]
pub struct WsConfig {
    pub url: String,
    pub access_token: Option<String>,
}

impl WsConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            access_token: None,
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.access_token = Some(token.into());
        self
    }
}

struct PendingCall {
    tx: oneshot::Sender<serde_json::Value>,
}

struct OutgoingCmd {
    json: String,
    echo: String,
    resp_tx: oneshot::Sender<serde_json::Value>,
}

/// OneBot 11 正向 WebSocket 客户端。
///
/// 连接到 `/` 端点，同时提供事件接收和 API 调用能力。
pub struct WsClient {
    api: ApiCaller,
    event_rx: mpsc::UnboundedReceiver<Event>,
}

impl WsClient {
    /// 连接到 OneBot 正向 WebSocket 服务端。
    ///
    /// 返回 `WsClient`，通过 `api()` 获取可克隆的 API 调用句柄，
    /// 通过 `next_event()` 接收事件推送。
    pub async fn connect(config: WsConfig) -> Result<Self, OneBotError> {
        let mut request = config.url.into_client_request()?;

        if let Some(ref token) = config.access_token {
            let value = format!("Bearer {}", token);
            let header_value = value.parse().map_err(|_| {
                OneBotError::Auth("invalid access token characters".into())
            })?;
            request.headers_mut().insert(AUTHORIZATION, header_value);
        }

        let (ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
        tracing::info!("websocket connected");

        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        let pending: Arc<Mutex<HashMap<String, PendingCall>>> =
            Arc::new(Mutex::new(HashMap::new()));

        tokio::spawn(event_loop(ws_stream, event_tx, cmd_rx, pending));

        let api = ApiCaller { cmd_tx };

        Ok(Self { api, event_rx })
    }

    /// 获取可克隆的 API 调用句柄。
    pub fn api(&self) -> &ApiCaller {
        &self.api
    }

    /// 接收下一个事件，连接关闭时返回 `None`。
    pub async fn next_event(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

/// 可克隆的 API 调用句柄，内部通过 channel 与事件循环通信。
#[derive(Clone)]
pub struct ApiCaller {
    cmd_tx: mpsc::UnboundedSender<OutgoingCmd>,
}

impl ApiCaller {
    /// 调用任意实现了 `ApiAction` trait 的 API。
    ///
    /// 自动生成 echo ID 并等待匹配的响应。
    pub async fn call<A: ApiAction>(&self, action: A) -> Result<A::Response, OneBotError> {
        let echo = uuid::Uuid::new_v4().to_string();
        let params = serde_json::to_value(&action)?;

        let request = ApiRequest {
            action: A::ACTION,
            params,
            echo: echo.clone(),
        };
        let json = serde_json::to_string(&request)?;

        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = OutgoingCmd {
            json,
            echo,
            resp_tx,
        };

        self.cmd_tx
            .send(cmd)
            .map_err(|_| OneBotError::ConnectionClosed)?;

        let raw = resp_rx
            .await
            .map_err(|_| OneBotError::ConnectionClosed)?;

        let resp: ApiResponse<A::Response> = serde_json::from_value(raw)?;

        if resp.retcode != 0 {
            return Err(OneBotError::ApiError {
                retcode: resp.retcode,
                message: resp.message.unwrap_or_default(),
            });
        }

        resp.data.ok_or_else(|| OneBotError::Protocol(
            "api response data is null".into(),
        ))
    }

    /// 发送 API 请求并返回原始 `ApiResponse<serde_json::Value>`，不检查 retcode。
    pub async fn call_raw<A: ApiAction>(
        &self,
        action: A,
    ) -> Result<ApiResponse, OneBotError> {
        let echo = uuid::Uuid::new_v4().to_string();
        let params = serde_json::to_value(&action)?;

        let request = ApiRequest {
            action: A::ACTION,
            params,
            echo: echo.clone(),
        };
        let json = serde_json::to_string(&request)?;

        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = OutgoingCmd {
            json,
            echo,
            resp_tx,
        };

        self.cmd_tx
            .send(cmd)
            .map_err(|_| OneBotError::ConnectionClosed)?;

        let raw = resp_rx
            .await
            .map_err(|_| OneBotError::ConnectionClosed)?;

        let resp: ApiResponse = serde_json::from_value(raw)?;
        Ok(resp)
    }
}

/// 内部事件循环：管理 WebSocket 读写、事件分发、API 响应匹配。
async fn event_loop(
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    event_tx: mpsc::UnboundedSender<Event>,
    mut cmd_rx: mpsc::UnboundedReceiver<OutgoingCmd>,
    pending: Arc<Mutex<HashMap<String, PendingCall>>>,
) {
    let (mut write, mut read) = ws_stream.split();

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_incoming(&text, &event_tx, &pending).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("websocket connection closed");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if let Err(e) = write.send(Message::Pong(data)).await {
                            tracing::error!(error = %e, "failed to send pong");
                            break;
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        tracing::error!(error = %e, "websocket read error");
                        break;
                    }
                }
            }

            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(cmd) => {
                        {
                            let mut map = pending.lock().await;
                            map.insert(cmd.echo, PendingCall { tx: cmd.resp_tx });
                        }
                        if let Err(e) = write.send(Message::Text(cmd.json.into())).await {
                            tracing::error!(error = %e, "failed to send api request");
                            break;
                        }
                    }
                    None => {
                        tracing::info!("all api callers dropped, closing connection");
                        break;
                    }
                }
            }
        }
    }

    let _ = write.close().await;
}

async fn handle_incoming(
    text: &str,
    event_tx: &mpsc::UnboundedSender<Event>,
    pending: &Arc<Mutex<HashMap<String, PendingCall>>>,
) {
    let value: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "failed to parse incoming json");
            return;
        }
    };

    if let Some(echo) = value.get("echo").and_then(|v| v.as_str()) {
        let mut map = pending.lock().await;
        if let Some(call) = map.remove(echo) {
            let _ = call.tx.send(value);
        } else {
            tracing::warn!(echo = echo, "received response with unknown echo");
        }
        return;
    }

    match serde_json::from_value::<Event>(value.clone()) {
        Ok(event) => {
            if event_tx.send(event).is_err() {
                tracing::warn!("event receiver dropped");
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, raw = %value, "failed to deserialize event");
        }
    }
}
