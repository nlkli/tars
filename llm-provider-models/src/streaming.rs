//! Server-sent event (SSE) chunk types returned during streaming completions.

use serde::{Deserialize, Serialize};

use crate::completion::CompletionUsage;

// ---------------------------------------------------------------------------
// Delta tool call
// ---------------------------------------------------------------------------

/// Incremental update to a function call within a streaming delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaFunctionToolCall {
    /// Function name, present only in the first chunk for this call.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Partial JSON argument string; concatenate across chunks to reconstruct.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// Incremental update to a tool call within a streaming delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaToolCall {
    /// Position of this tool call in the tool-call array.
    pub index: usize,

    /// Call identifier; present only in the first chunk for this call.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Discriminator (`"function"`, etc.); present only in the first chunk.
    #[serde(default, rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<DeltaFunctionToolCall>,
}

// ---------------------------------------------------------------------------
// Choice delta
// ---------------------------------------------------------------------------

/// Partial content of a single message choice in a streaming chunk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatCompletionChunkChoiceDelta {
    /// Role of the message being streamed; present only in the first chunk.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Partial text content; concatenate across chunks to reconstruct.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Partial refusal text; concatenate across chunks to reconstruct.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,

    /// Partial reasoning content from extended-thinking models.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,

    /// Partial tool call updates.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<DeltaToolCall>>,
}

// ---------------------------------------------------------------------------
// Chunk choice
// ---------------------------------------------------------------------------

/// A single choice in a streaming chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunkChoice {
    /// Zero-based index in the `choices` array.
    pub index: usize,

    /// Incremental content update for this choice.
    pub delta: ChatCompletionChunkChoiceDelta,

    /// Non-`None` only in the final chunk for this choice; see
    /// [`crate::constants::finish_reason`].
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Chunk
// ---------------------------------------------------------------------------

/// A single SSE event from a streaming chat completion response.
///
/// Clients accumulate these to reconstruct the full
/// [`ChatCompletionResponse`](crate::completion::ChatCompletionResponse).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// Shared identifier across all chunks for this completion.
    pub id: String,

    /// Always `"chat.completion.chunk"`.
    pub object: String,

    /// Unix timestamp when the chunk was created.
    pub created: u64,

    /// Model that produced the chunk.
    pub model: String,

    pub choices: Vec<ChatCompletionChunkChoice>,

    /// Token usage for the whole completion; present only in the final chunk
    /// when `stream_options.include_usage` is enabled.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,
}
