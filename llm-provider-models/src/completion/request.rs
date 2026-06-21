use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::message::MessageParam;
use crate::tool::{Tool, ToolChoice};

// ---------------------------------------------------------------------------
// Request
// ---------------------------------------------------------------------------

/// A chat completion request body.
///
/// Construct via [`ChatCompletion::builder()`] or [`ChatCompletion::new()`].
///
/// Reference: <https://platform.openai.com/docs/api-reference/chat/create>
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ChatCompletion {
    /// Model identifier (e.g. `"gpt-4o"`).
    pub model: String,

    /// Ordered conversation history, including the latest user turn.
    pub messages: Vec<MessageParam>,

    /// When `true`, the API streams back partial deltas as SSE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Hard cap on the number of tokens the model may generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// Arbitrary key-value metadata attached to the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Number of independent completions to generate (`1`–`128`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,

    /// Penalizes repetition of already-used tokens (`-2.0`–`2.0`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Sampling temperature (`0.0`–`2.0`). Higher values produce more varied
    /// output; lower values are more deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// When `true`, log probabilities are returned for each output token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Effort level for extended-thinking models. See
    /// [`crate::constants::reasoning_effort`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,

    /// Governs whether and how the model calls tools.
    pub tool_choice: Option<ToolChoice>,

    /// Tools available for the model to call.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
}

impl ChatCompletion {
    /// Create a minimal request.
    pub fn new(model: impl Into<String>, messages: Vec<MessageParam>) -> Self {
        Self {
            model: model.into(),
            messages,
            ..Default::default()
        }
    }

    /// Return a [`ChatCompletionBuilder`] for fluent construction.
    pub fn builder() -> ChatCompletionBuilder {
        ChatCompletionBuilder::default()
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Fluent builder for [`ChatCompletion`].
#[derive(Debug, Default, Clone)]
pub struct ChatCompletionBuilder {
    model: String,
    messages: Vec<MessageParam>,
    stream: Option<bool>,
    max_completion_tokens: Option<u32>,
    metadata: Option<HashMap<String, String>>,
    n: Option<u8>,
    frequency_penalty: Option<f32>,
    temperature: Option<f32>,
    logprobs: Option<bool>,
    reasoning_effort: Option<String>,
    tool_choice: Option<ToolChoice>,
    tools: Vec<Tool>,
}

impl ChatCompletionBuilder {
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn messages(mut self, messages: Vec<MessageParam>) -> Self {
        self.messages = messages;
        self
    }

    /// Append a single message to the conversation.
    pub fn message(mut self, message: MessageParam) -> Self {
        self.messages.push(message);
        self
    }

    /// Enable streaming (SSE) output.
    pub fn stream(mut self) -> Self {
        self.stream = Some(true);
        self
    }

    pub fn max_completion_tokens(mut self, n: u32) -> Self {
        self.max_completion_tokens = Some(n);
        self
    }

    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn n(mut self, n: u8) -> Self {
        self.n = Some(n);
        self
    }

    pub fn frequency_penalty(mut self, penalty: f32) -> Self {
        self.frequency_penalty = Some(penalty);
        self
    }

    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Request per-token log probabilities in the response.
    pub fn logprobs(mut self) -> Self {
        self.logprobs = Some(true);
        self
    }

    pub fn reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(effort.into());
        self
    }

    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Append a single tool to the tool list.
    pub fn tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    /// Consume the builder and produce a [`ChatCompletion`].
    pub fn build(self) -> ChatCompletion {
        ChatCompletion {
            model: self.model,
            messages: self.messages,
            stream: self.stream,
            max_completion_tokens: self.max_completion_tokens,
            metadata: self.metadata,
            n: self.n,
            frequency_penalty: self.frequency_penalty,
            temperature: self.temperature,
            logprobs: self.logprobs,
            reasoning_effort: self.reasoning_effort,
            tool_choice: self.tool_choice,
            tools: self.tools,
        }
    }
}
