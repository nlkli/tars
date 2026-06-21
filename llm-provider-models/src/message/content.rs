use serde::{Deserialize, Serialize};

use crate::consts::role;
use crate::tool::call::ToolCall;

use super::part::{ContentPart, FileData};

// ---------------------------------------------------------------------------
// Message content wrapper
// ---------------------------------------------------------------------------

/// The body of a [`MessageParam`].
///
/// Most messages carry a plain [`String`]; multimodal messages carry a list of
/// [`ContentPart`]s. The `None` variant is used for assistant messages that
/// consist solely of tool calls.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text body.
    Text(String),

    /// One or more typed content parts (text, image, audio, file).
    Parts(Vec<ContentPart>),

    /// No content (e.g. an assistant message that only contains tool calls).
    #[default]
    None,
}

impl MessageContent {
    /// Returns `true` if the content is [`MessageContent::None`].
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns the inner text if this is a [`MessageContent::Text`] variant.
    pub fn as_text(&self) -> Option<&str> {
        if let Self::Text(t) = self {
            Some(t.as_str())
        } else {
            None
        }
    }

    /// Returns a mutable reference to the inner text if this is a
    /// [`MessageContent::Text`] variant.
    pub fn as_mut_text(&mut self) -> Option<&mut String> {
        if let Self::Text(t) = self {
            Some(t)
        } else {
            None
        }
    }

    /// Returns the parts slice if this is a [`MessageContent::Parts`] variant.
    pub fn as_parts(&self) -> Option<&[ContentPart]> {
        if let Self::Parts(p) = self {
            Some(p)
        } else {
            None
        }
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<Vec<ContentPart>> for MessageContent {
    fn from(parts: Vec<ContentPart>) -> Self {
        Self::Parts(parts)
    }
}

// ---------------------------------------------------------------------------
// Backward-compat type alias
// ---------------------------------------------------------------------------

/// Alias kept for backward compatibility. Prefer [`MessageContent`].
pub type ChatCompletionMessageContent = MessageContent;

// ---------------------------------------------------------------------------
// Audio back-reference
// ---------------------------------------------------------------------------

/// Reference to a prior audio response, used to include it in a follow-up
/// turn without re-uploading the audio bytes.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AudioResponseData {
    /// The `id` from the [`ChatCompletionAudio`](crate::completion::response::ChatCompletionAudio)
    /// object you want to reference.
    pub id: String,
}

// ---------------------------------------------------------------------------
// Message parameter
// ---------------------------------------------------------------------------

/// A single message in a chat conversation, sent as part of a request.
///
/// Use the role-specific constructors ([`developer`](Self::developer),
/// [`system`](Self::system), [`user`](Self::user), [`assistant`](Self::assistant),
/// [`tool`](Self::tool)) for the most convenient API.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MessageParam {
    /// Conversation role (`"user"`, `"assistant"`, `"system"`, `"developer"`,
    /// `"tool"`). See [`crate::constants::role`].
    pub role: String,

    /// Textual or multipart body of the message.
    pub content: MessageContent,

    /// Set when the assistant message carries a model refusal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,

    /// Optional name to disambiguate participants with the same role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Back-reference to a prior audio response (assistant messages only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioResponseData>,

    /// Tool calls requested by the assistant (assistant messages only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// The tool call this message is responding to (tool messages only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl MessageParam {
    // -----------------------------------------------------------------------
    // Core constructors
    // -----------------------------------------------------------------------

    /// Create a text message with the given role.
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: MessageContent::Text(content.into()),
            ..Default::default()
        }
    }

    /// Create a message with a multi-part body for the given role.
    pub fn new_with_parts(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: MessageContent::Parts(Vec::new()),
            ..Default::default()
        }
    }

    // -----------------------------------------------------------------------
    // Role-specific constructors
    // -----------------------------------------------------------------------

    /// Create a `developer` role message.
    pub fn developer(content: impl Into<String>) -> Self {
        Self::new(role::DEVELOPER, content)
    }

    /// Create a `system` role message.
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(role::SYSTEM, content)
    }

    /// Create a `user` role message.
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(role::USER, content)
    }

    /// Create an `assistant` role message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(role::ASSISTANT, content)
    }

    /// Create a `tool` role message that supplies the result of a tool call.
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        let mut p = Self::new(role::TOOL, content);
        p.tool_call_id = Some(tool_call_id.into());
        p
    }

    // -----------------------------------------------------------------------
    // Builder-style setters (consume & return self)
    // -----------------------------------------------------------------------

    /// Append a content part to a multi-part message.
    ///
    /// Has no effect if the content is not [`MessageContent::Parts`].
    pub fn push_part(mut self, part: ContentPart) -> Self {
        if let MessageContent::Parts(ref mut parts) = self.content {
            parts.push(part);
        }
        self
    }

    /// Add a text part to a multi-part message.
    pub fn with_text(self, text: impl Into<String>) -> Self {
        self.push_part(ContentPart::text(text))
    }

    /// Add an image URL part to a multi-part message.
    pub fn with_image(self, url: impl Into<String>, detail: Option<String>) -> Self {
        self.push_part(ContentPart::image_url(url, detail))
    }

    /// Add a file part to a multi-part message.
    pub fn with_file(self, file_data: FileData) -> Self {
        self.push_part(ContentPart::file(file_data))
    }

    /// Set the optional participant name.
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Attach an audio back-reference (assistant messages only).
    pub fn set_audio(mut self, audio: AudioResponseData) -> Self {
        self.audio = Some(audio);
        self
    }

    /// Attach tool calls (assistant messages only).
    pub fn tool_calls(mut self, calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(calls);
        self
    }

    // -----------------------------------------------------------------------
    // Role predicates
    // -----------------------------------------------------------------------

    pub fn is_developer(&self) -> bool {
        self.role == role::DEVELOPER
    }

    pub fn is_system(&self) -> bool {
        self.role == role::SYSTEM
    }

    pub fn is_user(&self) -> bool {
        self.role == role::USER
    }

    pub fn is_assistant(&self) -> bool {
        self.role == role::ASSISTANT
    }

    pub fn is_tool(&self) -> bool {
        self.role == role::TOOL
    }

    // -----------------------------------------------------------------------
    // Content accessors
    // -----------------------------------------------------------------------

    /// Mutable reference to the inner text, if content is [`MessageContent::Text`].
    pub fn as_mut_text_content(&mut self) -> Option<&mut String> {
        self.content.as_mut_text()
    }
}

// ---------------------------------------------------------------------------
// Backward-compat type alias
// ---------------------------------------------------------------------------

/// Alias kept for backward compatibility. Prefer [`MessageParam`].
pub type ChatCompletionMessageParam = MessageParam;
