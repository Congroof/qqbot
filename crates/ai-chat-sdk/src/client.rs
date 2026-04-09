use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

use crate::api::chat::ChatEndpoint;
use crate::api::image::ImageEndpoint;
use crate::config::{ClientConfig, ClientConfigBuilder};
use crate::error::Result;

/// Shared internal state for all endpoint handlers.
pub(crate) struct Inner {
    pub http: reqwest::Client,
    pub config: ClientConfig,
}

impl Inner {
    /// Build the default headers that must be sent with every request.
    pub fn default_headers(config: &ClientConfig) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        let auth_value = if config.api_key.starts_with("Bearer ") {
            config.api_key.clone()
        } else {
            format!("Bearer {}", config.api_key)
        };
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).map_err(|e| {
                crate::error::AiChatError::Config(format!("invalid api_key header: {e}"))
            })?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        headers.insert(
            "AI-Gateway-Uid",
            HeaderValue::from_str(&config.gateway_uid).map_err(|e| {
                crate::error::AiChatError::Config(format!("invalid gateway_uid: {e}"))
            })?,
        );
        headers.insert(
            "AI-Gateway-Product-Name",
            HeaderValue::from_str(&config.product_name).map_err(|e| {
                crate::error::AiChatError::Config(format!("invalid product_name: {e}"))
            })?,
        );
        headers.insert(
            "AI-Gateway-Intention-Code",
            HeaderValue::from_str(&config.intention_code).map_err(|e| {
                crate::error::AiChatError::Config(format!("invalid intention_code: {e}"))
            })?,
        );

        if config.async_mode {
            headers.insert("AI-Gateway-Async", HeaderValue::from_static("1"));
        }

        Ok(headers)
    }

    /// Generate a random hex action-id (32 hex chars = 16 random bytes).
    pub fn generate_action_id() -> String {
        uuid::Uuid::new_v4().simple().to_string()
    }
}

/// The main entry point for the AI Gateway SDK.
///
/// Construct via [`AiClient::builder()`], then use `.chat()` or `.image()`
/// to access the corresponding API endpoint.
#[derive(Clone)]
pub struct AiClient {
    pub(crate) inner: Arc<Inner>,
}

impl AiClient {
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::default()
    }

    pub fn new(config: ClientConfig) -> Result<Self> {
        let default_headers = Inner::default_headers(&config)?;
        let http = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()?;

        Ok(AiClient {
            inner: Arc::new(Inner { http, config }),
        })
    }

    pub fn chat(&self) -> ChatEndpoint {
        ChatEndpoint::new(self.inner.clone())
    }

    pub fn image(&self) -> ImageEndpoint {
        ImageEndpoint::new(self.inner.clone())
    }
}

impl ClientConfigBuilder {
    /// Shortcut: build the config *and* construct an [`AiClient`] in one step.
    pub fn build_client(self) -> Result<AiClient> {
        let config = self.build()?;
        AiClient::new(config)
    }
}
