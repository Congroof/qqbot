use crate::error::{AiChatError, Result};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub api_key: String,
    pub gateway_uid: String,
    pub product_name: String,
    pub intention_code: String,
    pub client_request_id: Option<String>,
    pub async_mode: bool,
}

#[derive(Debug, Default)]
pub struct ClientConfigBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    gateway_uid: Option<String>,
    product_name: Option<String>,
    intention_code: Option<String>,
    client_request_id: Option<String>,
    async_mode: bool,
}

impl ClientConfigBuilder {
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn gateway_uid(mut self, uid: impl Into<String>) -> Self {
        self.gateway_uid = Some(uid.into());
        self
    }

    pub fn product_name(mut self, name: impl Into<String>) -> Self {
        self.product_name = Some(name.into());
        self
    }

    pub fn intention_code(mut self, code: impl Into<String>) -> Self {
        self.intention_code = Some(code.into());
        self
    }

    pub fn client_request_id(mut self, id: impl Into<String>) -> Self {
        self.client_request_id = Some(id.into());
        self
    }

    pub fn async_mode(mut self, enabled: bool) -> Self {
        self.async_mode = enabled;
        self
    }

    pub fn build(self) -> Result<ClientConfig> {
        let base_url = self
            .base_url
            .ok_or_else(|| AiChatError::Config("base_url is required".into()))?;
        let api_key = self
            .api_key
            .ok_or_else(|| AiChatError::Config("api_key is required".into()))?;
        let gateway_uid = self
            .gateway_uid
            .ok_or_else(|| AiChatError::Config("gateway_uid is required".into()))?;
        let product_name = self
            .product_name
            .ok_or_else(|| AiChatError::Config("product_name is required".into()))?;
        let intention_code = self
            .intention_code
            .ok_or_else(|| AiChatError::Config("intention_code is required".into()))?;

        let base_url = base_url.trim_end_matches('/').to_string();

        Ok(ClientConfig {
            base_url,
            api_key,
            gateway_uid,
            product_name,
            intention_code,
            client_request_id: self.client_request_id,
            async_mode: self.async_mode,
        })
    }
}
