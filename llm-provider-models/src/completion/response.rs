use serde::{Deserialize, Serialize};

use crate::logprob::ChatCompletionLogprobs;
use crate::tool::ToolCall;

// ---------------------------------------------------------------------------
// Top-level response
// ---------------------------------------------------------------------------

/// Response body for a non-streaming chat completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Unique identifier for this completion.
    pub id: String,

    /// One or more generated choices (typically one unless `n > 1`).
    pub choices: Vec<ChatCompletionChoice>,

    /// Unix timestamp of when the completion was created.
    pub created: u64,

    /// Model that produced this completion.
    pub model: String,

    /// Token usage statistics, when available.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,
}

// ---------------------------------------------------------------------------
// Choice
// ---------------------------------------------------------------------------

/// A single candidate completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    /// Why the model stopped generating. See [`crate::constants::finish_reason`].
    pub finish_reason: String,

    /// Zero-based index in the `choices` array.
    pub index: u32,

    /// Per-token log probabilities, present only when `logprobs` was `true`.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<ChatCompletionLogprobs>,

    /// The generated message.
    pub message: ChatCompletionMessage,
}

// ---------------------------------------------------------------------------
// Generated message
// ---------------------------------------------------------------------------

/// An assistant message returned inside a [`ChatCompletionChoice`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessage {
    /// Always `"assistant"`.
    pub role: String,

    /// Text content of the reply, or `None` when the response is a tool call
    /// or refusal.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Extended reasoning text produced by reasoning-capable models.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,

    /// Model-generated refusal text, when the request was declined.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,

    /// URL citations attached to the message, if any.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<ChatCompletionAnnotation>>,

    /// Audio output, present when the `audio` modality was requested.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<ChatCompletionAudio>,

    /// Tool calls requested by the model, when `finish_reason` is
    /// `"tool_calls"`.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl ChatCompletionMessage {
    /// Returns the text content, or an empty string when it is absent.
    pub fn content_or_default(&self) -> &str {
        self.content.as_deref().unwrap_or("")
    }

    /// Returns `true` when the model issued one or more tool calls.
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .is_some_and(|calls| !calls.is_empty())
    }
}

// ---------------------------------------------------------------------------
// Annotations
// ---------------------------------------------------------------------------

/// An inline annotation attached to a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatCompletionAnnotation {
    /// A citation that links a span of output text to a source URL.
    #[serde(rename = "url_citation")]
    UrlCitation { url_citation: UrlCitation },
}

/// Source URL and the character span it annotates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlCitation {
    /// Start character index (inclusive) in the message content.
    pub start_index: u32,

    /// End character index (exclusive) in the message content.
    pub end_index: u32,

    /// Page or document title of the cited source.
    pub title: String,

    /// URL of the cited source.
    pub url: String,
}

// ---------------------------------------------------------------------------
// Audio output
// ---------------------------------------------------------------------------

/// Audio output returned when the `audio` modality is enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionAudio {
    /// Opaque identifier for this audio response, used as a back-reference
    /// in subsequent turns via [`crate::message::AudioResponseData`].
    pub id: String,

    /// Base64-encoded audio bytes in the format requested.
    pub data: String,

    /// Unix timestamp after which this audio response can no longer be
    /// referenced.
    pub expires_at: i64,

    /// Transcript of the generated audio.
    pub transcript: String,
}

// ---------------------------------------------------------------------------
// Usage
// ---------------------------------------------------------------------------

/// Aggregate token counts for a completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionUsage {
    pub completion_tokens: u32,

    pub prompt_tokens: u32,

    pub total_tokens: u32,

    #[serde(default)]
    pub completion_tokens_details: Option<CompletionTokensDetails>,

    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

/// Breakdown of completion token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_prediction_tokens: Option<u32>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,

    /// Tokens consumed by the model's internal reasoning process.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_prediction_tokens: Option<u32>,
}

/// Breakdown of prompt token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,

    /// Tokens served from the prompt cache, reducing latency and cost.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
}
