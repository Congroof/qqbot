use std::sync::Arc;

use crate::client::Inner;
use crate::error::{AiChatError, Result};
use crate::types::common::ApiErrorResponse;
use crate::types::image::{ImageRequest, ImageResponse};

/// Handle for image generation operations.
pub struct ImageEndpoint {
    inner: Arc<Inner>,
}

impl ImageEndpoint {
    pub(crate) fn new(inner: Arc<Inner>) -> Self {
        Self { inner }
    }

    fn url(&self) -> String {
        format!("{}/api/v3/images/generations", self.inner.config.base_url)
    }

    /// Generate one or more images from a text prompt.
    pub async fn generate(&self, request: ImageRequest) -> Result<ImageResponse> {
        let response = self
            .inner
            .http
            .post(self.url())
            .header("X-Action-Id", Inner::generate_action_id())
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Self::parse_error(status.as_u16(), &text));
        }

        let image_response: ImageResponse = response.json().await?;
        Ok(image_response)
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
