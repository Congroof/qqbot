use serde::{Deserialize, Serialize};

use super::common::{ApiErrorBody, RetryStrategy, SecText, Usage};

// ---------------------------------------------------------------------------
// Request
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ImageRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    // Gateway extension fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_llm_arguments: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_strategy: Option<RetryStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_text: Option<SecText>,
}

// ---------------------------------------------------------------------------
// Request Builder
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct ImageRequestBuilder {
    model: Option<String>,
    prompt: Option<String>,
    n: Option<u32>,
    size: Option<String>,
    quality: Option<String>,
    style: Option<String>,
    response_format: Option<String>,
    user: Option<String>,
    aspect_ratio: Option<String>,
    negative_prompt: Option<String>,
    seed: Option<i64>,
    extended_llm_arguments: Option<serde_json::Value>,
    retry_strategy: Option<RetryStrategy>,
    sec_text: Option<SecText>,
}

impl ImageRequest {
    pub fn builder() -> ImageRequestBuilder {
        ImageRequestBuilder::default()
    }
}

impl ImageRequestBuilder {
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    pub fn size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn quality(mut self, quality: impl Into<String>) -> Self {
        self.quality = Some(quality.into());
        self
    }

    pub fn style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    pub fn response_format(mut self, format: impl Into<String>) -> Self {
        self.response_format = Some(format.into());
        self
    }

    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn aspect_ratio(mut self, ratio: impl Into<String>) -> Self {
        self.aspect_ratio = Some(ratio.into());
        self
    }

    pub fn negative_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.negative_prompt = Some(prompt.into());
        self
    }

    pub fn seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn extended_llm_arguments(mut self, args: serde_json::Value) -> Self {
        self.extended_llm_arguments = Some(args);
        self
    }

    pub fn retry_strategy(mut self, strategy: RetryStrategy) -> Self {
        self.retry_strategy = Some(strategy);
        self
    }

    pub fn sec_text(mut self, sec: SecText) -> Self {
        self.sec_text = Some(sec);
        self
    }

    pub fn build(self) -> ImageRequest {
        ImageRequest {
            model: self.model.unwrap_or_default(),
            prompt: self.prompt.unwrap_or_default(),
            n: self.n,
            size: self.size,
            quality: self.quality,
            style: self.style,
            response_format: self.response_format,
            user: self.user,
            aspect_ratio: self.aspect_ratio,
            negative_prompt: self.negative_prompt,
            seed: self.seed,
            extended_llm_arguments: self.extended_llm_arguments,
            retry_strategy: self.retry_strategy,
            sec_text: self.sec_text,
        }
    }
}

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ImageResponse {
    pub created: u64,
    pub data: Vec<ImageData>,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub extended_resp_fields: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<ApiErrorBody>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_prompt: Option<String>,
}
