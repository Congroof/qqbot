use serde::{Deserialize, Serialize};

use super::common::{
    ApiErrorBody, FinishReason, FunctionCall, LogitBias, ResponseFormat, RetryStrategy, Role,
    SecText, StreamOptions, Tool, ToolChoice, Usage,
};

// ---------------------------------------------------------------------------
// Request
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<LogitBias>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 推理力度 "low" | "medium" | "high"
    pub reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 服务等级 "auto" | "default"
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
    // Gateway extension fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_thinking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_llm_arguments: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_strategy: Option<RetryStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_text: Option<SecText>,
    #[serde(skip)]
    pub stream: bool,
}

/// Message content: either a plain text string or a multimodal array of content parts.
///
/// Serializes as a raw JSON string or array thanks to `#[serde(untagged)]`:
/// - `MessageContent::Text("hello")` → `"hello"`
/// - `MessageContent::Parts(vec![...])` → `[{"type":"text","text":"hello"}, ...]`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<serde_json::Value>),
}

impl MessageContent {
    /// If this is a `Text` variant, return the inner `&str`.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(s) => Some(s),
            MessageContent::Parts(_) => None,
        }
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        MessageContent::Text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        MessageContent::Text(s.to_owned())
    }
}

impl From<Vec<serde_json::Value>> for MessageContent {
    fn from(parts: Vec<serde_json::Value>) -> Self {
        MessageContent::Parts(parts)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Message {
            role: Role::System,
            content: Some(MessageContent::Text(content.into())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            refusal: None,
            reasoning_content: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Message {
            role: Role::User,
            content: Some(MessageContent::Text(content.into())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            refusal: None,
            reasoning_content: None,
        }
    }

    /// Build a multimodal user message from content parts.
    ///
    /// ```ignore
    /// use serde_json::json;
    /// let msg = Message::user_multimodal(vec![
    ///     json!({"type": "text", "text": "What's in this image?"}),
    ///     json!({"type": "image_url", "image_url": {"url": "https://example.com/img.png"}}),
    /// ]);
    /// ```
    pub fn user_multimodal(parts: Vec<serde_json::Value>) -> Self {
        Message {
            role: Role::User,
            content: Some(MessageContent::Parts(parts)),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            refusal: None,
            reasoning_content: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Message {
            role: Role::Assistant,
            content: Some(MessageContent::Text(content.into())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            refusal: None,
            reasoning_content: None,
        }
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Message {
            role: Role::Tool,
            content: Some(MessageContent::Text(content.into())),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            refusal: None,
            reasoning_content: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Request Builder
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct ChatRequestBuilder {
    model: Option<String>,
    messages: Vec<Message>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    max_tokens: Option<u32>,
    max_completion_tokens: Option<u32>,
    n: Option<u32>,
    stop: Option<Vec<String>>,
    frequency_penalty: Option<f64>,
    presence_penalty: Option<f64>,
    logprobs: Option<bool>,
    top_logprobs: Option<u32>,
    logit_bias: Option<LogitBias>,
    seed: Option<i64>,
    response_format: Option<ResponseFormat>,
    tools: Option<Vec<Tool>>,
    tool_choice: Option<ToolChoice>,
    parallel_tool_calls: Option<bool>,
    reasoning_effort: Option<String>,
    service_tier: Option<String>,
    stream_options: Option<StreamOptions>,
    enable_thinking: Option<bool>,
    extended_llm_arguments: Option<serde_json::Value>,
    retry_strategy: Option<RetryStrategy>,
    sec_text: Option<SecText>,
    stream: bool,
}

impl ChatRequest {
    pub fn builder() -> ChatRequestBuilder {
        ChatRequestBuilder::default()
    }
}

impl ChatRequestBuilder {
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    pub fn temperature(mut self, t: f64) -> Self {
        self.temperature = Some(t);
        self
    }

    pub fn top_p(mut self, p: f64) -> Self {
        self.top_p = Some(p);
        self
    }

    pub fn max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = Some(n);
        self
    }

    pub fn max_completion_tokens(mut self, n: u32) -> Self {
        self.max_completion_tokens = Some(n);
        self
    }

    pub fn n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    pub fn frequency_penalty(mut self, v: f64) -> Self {
        self.frequency_penalty = Some(v);
        self
    }

    pub fn presence_penalty(mut self, v: f64) -> Self {
        self.presence_penalty = Some(v);
        self
    }

    pub fn logprobs(mut self, enabled: bool) -> Self {
        self.logprobs = Some(enabled);
        self
    }

    pub fn top_logprobs(mut self, n: u32) -> Self {
        self.top_logprobs = Some(n);
        self
    }

    pub fn logit_bias(mut self, bias: LogitBias) -> Self {
        self.logit_bias = Some(bias);
        self
    }

    pub fn seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = Some(format);
        self
    }

    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    pub fn parallel_tool_calls(mut self, enabled: bool) -> Self {
        self.parallel_tool_calls = Some(enabled);
        self
    }

    pub fn reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(effort.into());
        self
    }

    pub fn stream_options(mut self, options: StreamOptions) -> Self {
        self.stream_options = Some(options);
        self
    }

    pub fn enable_thinking(mut self, enabled: bool) -> Self {
        self.enable_thinking = Some(enabled);
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

    pub fn build(self) -> ChatRequest {
        ChatRequest {
            model: self.model.unwrap_or_default(),
            messages: self.messages,
            temperature: self.temperature,
            top_p: self.top_p,
            max_tokens: self.max_tokens,
            max_completion_tokens: self.max_completion_tokens,
            n: self.n,
            stop: self.stop,
            frequency_penalty: self.frequency_penalty,
            presence_penalty: self.presence_penalty,
            logprobs: self.logprobs,
            top_logprobs: self.top_logprobs,
            logit_bias: self.logit_bias,
            seed: self.seed,
            response_format: self.response_format,
            tools: self.tools,
            tool_choice: self.tool_choice,
            parallel_tool_calls: self.parallel_tool_calls,
            reasoning_effort: self.reasoning_effort,
            service_tier: self.service_tier,
            stream_options: self.stream_options,
            enable_thinking: self.enable_thinking,
            extended_llm_arguments: self.extended_llm_arguments,
            retry_strategy: self.retry_strategy,
            sec_text: self.sec_text,
            stream: self.stream,
        }
    }
}

// ---------------------------------------------------------------------------
// Response (non-streaming)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(default)]
    pub extended_resp_fields: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<ApiErrorBody>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: Option<FinishReason>,
}

// ---------------------------------------------------------------------------
// Response (streaming)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ChatStreamChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(default)]
    pub extended_resp_fields: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<FinishReason>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Delta {
    pub role: Option<Role>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<DeltaToolCall>>,
    pub refusal: Option<String>,
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeltaToolCall {
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub call_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<DeltaFunction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeltaFunction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}
