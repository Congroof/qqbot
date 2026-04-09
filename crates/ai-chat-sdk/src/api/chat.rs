use std::sync::Arc;

use crate::client::Inner;
use crate::error::{AiChatError, Result};
use crate::stream::ChatStream;
use crate::types::chat::{ChatRequest, ChatResponse};
use crate::types::common::ApiErrorResponse;

/// Handle for chat completion operations.
pub struct ChatEndpoint {
    inner: Arc<Inner>,
}

impl ChatEndpoint {
    pub(crate) fn new(inner: Arc<Inner>) -> Self {
        Self { inner }
    }

    fn url(&self) -> String {
        format!("{}/api/v3/chat/completions", self.inner.config.base_url)
    }

    /// Create a chat completion (non-streaming).
    pub async fn create(&self, request: ChatRequest) -> Result<ChatResponse> {
        let body = self.build_body(&request, false)?;

        let response = self
            .inner
            .http
            .post(self.url())
            .header("X-Action-Id", Inner::generate_action_id())
            .body(body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Self::parse_error(status.as_u16(), &text));
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response)
    }

    /// Create a streaming chat completion. Returns a [`ChatStream`] that
    /// yields `ChatStreamChunk` items as they arrive via SSE.
    pub async fn create_stream(&self, request: ChatRequest) -> Result<ChatStream> {
        let body = self.build_body(&request, true)?;

        let response = self
            .inner
            .http
            .post(self.url())
            .header("X-Action-Id", Inner::generate_action_id())
            .body(body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Self::parse_error(status.as_u16(), &text));
        }

        Ok(ChatStream::new(response))
    }

    /// Serialize the request body, injecting the `stream` field.
    fn build_body(&self, request: &ChatRequest, stream: bool) -> Result<String> {
        let mut value = serde_json::to_value(request)?;
        if let Some(obj) = value.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(stream));
        }
        Ok(serde_json::to_string(&value)?)
    }

    fn parse_error(status: u16, body: &str) -> AiChatError {
        if let Ok(err_resp) = serde_json::from_str::<ApiErrorResponse>(body) {
            AiChatError::Api {
                code: err_resp.error.code,
                message: err_resp.error.message,
            }
        } else {
            AiChatError::Api {
                code: status.to_string(),
                message: body.to_string(),
            }
        }
    }
}
