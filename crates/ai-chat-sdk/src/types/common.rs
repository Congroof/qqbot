use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Usage (with V3 token details)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_prediction_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_prediction_tokens: Option<u64>,
}

// ---------------------------------------------------------------------------
// Roles & finish reasons
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    #[serde(other)]
    Other,
}

// ---------------------------------------------------------------------------
// Function / Tool types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Mode(String),
    Specific {
        #[serde(rename = "type")]
        tool_type: String,
        function: ToolChoiceFunction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    pub name: String,
}

impl ToolChoice {
    pub fn auto() -> Self {
        ToolChoice::Mode("auto".into())
    }

    pub fn none() -> Self {
        ToolChoice::Mode("none".into())
    }

    pub fn function(name: impl Into<String>) -> Self {
        ToolChoice::Specific {
            tool_type: "function".into(),
            function: ToolChoiceFunction { name: name.into() },
        }
    }
}

impl Tool {
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Tool {
            tool_type: "function".into(),
            function: FunctionDefinition {
                name: name.into(),
                description: Some(description.into()),
                parameters: Some(parameters),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Gateway extension types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStrategy {
    pub retry_count: u32,
    /// 超时时间(单位：秒)
    pub timeout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecText {
    /// 安全场景标识
    pub scene: String,
    /// 安全检查来源
    pub from: String,
    /// 答案过滤标志（0=不过滤，1=过滤）
    pub answer_flag: u32,
    /// 附加审核文本
    pub extra_text: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    /// json_object | text
    pub format_type: String,
}

impl ResponseFormat {
    pub fn text() -> Self {
        Self {
            format_type: "text".into(),
        }
    }

    pub fn json_object() -> Self {
        Self {
            format_type: "json_object".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    pub include_usage: bool,
}

// ---------------------------------------------------------------------------
// V3 error body (returned inside `{"error": {...}}`)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorBody {
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorBody,
}

// ---------------------------------------------------------------------------
// Re-export HashMap for ChatRequest's logit_bias field
// ---------------------------------------------------------------------------

pub type LogitBias = HashMap<String, f64>;
