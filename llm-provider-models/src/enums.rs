use serde::{Deserialize, Serialize};

/// Content part type identifiers for [`crate::message::ContentPart`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContentPart {
    #[default]
    Text,
    ImageUrl,
    Audio,
    File,
    Refusal,
}

impl ContentPart {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::ImageUrl => "image_url",
            Self::Audio => "input_audio",
            Self::File => "file",
            Self::Refusal => "refusal",
        }
    }
}

/// Audio format identifiers for [`crate::message::InputAudioData`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    #[default]
    Mp3,
    Wav,
}

impl AudioFormat {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
        }
    }
}

/// Image detail level options for [`crate::message::ImageUrl`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    #[default]
    Auto,
    Low,
    High,
}

impl ImageDetail {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Auto => "auto",
            Self::Low => "low",
            Self::High => "high",
        }
    }
}

/// Message role identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Developer,
    System,
    #[default]
    User,
    Assistant,
    Tool,
    Function,
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Developer => "developer",
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
            Self::Function => "function",
        }
    }
}

/// Reasoning effort level options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    #[default]
    Medium,
    High,
    #[serde(rename = "xhigh")]
    XHigh,
}

impl ReasoningEffort {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "xhigh",
        }
    }
}

/// Finish reason values returned in completion choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    #[default]
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    FunctionCall,
}

impl FinishReason {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Stop => "stop",
            Self::Length => "length",
            Self::ToolCalls => "tool_calls",
            Self::ContentFilter => "content_filter",
            Self::FunctionCall => "function_call",
        }
    }
}

/// Tool call type identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ToolCallType {
    #[default]
    Function,
    Custom,
}

impl ToolCallType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Function => "function",
            Self::Custom => "custom",
        }
    }
}

use std::convert::TryFrom;

impl TryFrom<&str> for ContentPart {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "text" => Ok(Self::Text),
            "image_url" => Ok(Self::ImageUrl),
            "input_audio" => Ok(Self::Audio),
            "file" => Ok(Self::File),
            "refusal" => Ok(Self::Refusal),
            _ => Err(format!("unknown content part type: {s:?}")),
        }
    }
}

impl TryFrom<&str> for AudioFormat {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "mp3" => Ok(Self::Mp3),
            "wav" => Ok(Self::Wav),
            _ => Err(format!("unknown audio format: {s:?}")),
        }
    }
}

impl TryFrom<&str> for ImageDetail {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "auto" => Ok(Self::Auto),
            "low" => Ok(Self::Low),
            "high" => Ok(Self::High),
            _ => Err(format!("unknown image detail: {s:?}")),
        }
    }
}

impl TryFrom<&str> for Role {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "developer" => Ok(Self::Developer),
            "system" => Ok(Self::System),
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "tool" => Ok(Self::Tool),
            "function" => Ok(Self::Function),
            _ => Err(format!("unknown role: {s:?}")),
        }
    }
}

impl TryFrom<&str> for ReasoningEffort {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "none" => Ok(Self::None),
            "minimal" => Ok(Self::Minimal),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "xhigh" => Ok(Self::XHigh),
            _ => Err(format!("unknown reasoning effort: {s:?}")),
        }
    }
}

impl TryFrom<&str> for FinishReason {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "stop" => Ok(Self::Stop),
            "length" => Ok(Self::Length),
            "tool_calls" => Ok(Self::ToolCalls),
            "content_filter" => Ok(Self::ContentFilter),
            "function_call" => Ok(Self::FunctionCall),
            _ => Err(format!("unknown finish reason: {s:?}")),
        }
    }
}

impl TryFrom<&str> for ToolCallType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "function" => Ok(Self::Function),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("unknown tool call type: {s:?}")),
        }
    }
}
