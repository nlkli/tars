use serde::{Deserialize, Serialize};

use crate::consts::content_part;

// ---------------------------------------------------------------------------
// Leaf types
// ---------------------------------------------------------------------------

/// A URL or base64-encoded image, with an optional detail level hint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageUrl {
    /// URL or base64-encoded image data (`data:<mime>;base64,<data>`).
    pub url: String,

    /// Resolution hint for vision models (`"auto"`, `"low"`, `"high"`).
    /// See [`crate::constants::image_detail`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Raw audio payload for an input content part.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputAudioData {
    /// Base64-encoded audio bytes.
    pub data: String,

    /// Container format of the audio (`"wav"` or `"mp3"`).
    /// See [`crate::constants::audio_format`].
    pub format: String,
}

/// File payload that can be supplied either as inline base64 data or as a
/// previously-uploaded file ID.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FileData {
    /// Base64-encoded file contents. Mutually exclusive with [`file_id`](Self::file_id).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,

    /// ID of a file that was previously uploaded via the Files API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,

    /// Original filename, used to hint at MIME type when `file_data` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

impl FileData {
    /// Create a `FileData` from inline base64 content.
    pub fn from_data(data: impl Into<String>, filename: Option<impl Into<String>>) -> Self {
        Self {
            file_data: Some(data.into()),
            filename: filename.map(Into::into),
            ..Default::default()
        }
    }

    /// Create a `FileData` referencing a previously-uploaded file by ID.
    pub fn from_id(id: impl Into<String>) -> Self {
        Self {
            file_id: Some(id.into()),
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Content part
// ---------------------------------------------------------------------------

/// A single typed segment within a multi-part message.
///
/// Use the static constructors ([`text`](Self::text), [`image_url`](Self::image_url),
/// [`input_audio`](Self::input_audio), [`file`](Self::file)) rather than
/// constructing this struct manually so the `type_` field is always consistent.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ContentPart {
    /// Discriminator field; always matches the populated `Option` field below.
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrl>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio: Option<InputAudioData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileData>,
}

impl ContentPart {
    /// Plain-text content part.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            type_: content_part::TEXT.into(),
            text: Some(content.into()),
            ..Default::default()
        }
    }

    /// Image content part from a URL or `data:` URI.
    ///
    /// `detail` controls resolution processing; see [`crate::constants::image_detail`].
    pub fn image_url(url: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            type_: content_part::IMAGE_URL.into(),
            image_url: Some(ImageUrl {
                url: url.into(),
                detail,
            }),
            ..Default::default()
        }
    }

    /// Audio content part from base64-encoded bytes.
    ///
    /// `format` must be `"wav"` or `"mp3"`; see [`crate::constants::audio_format`].
    pub fn input_audio(data: impl Into<String>, format: impl Into<String>) -> Self {
        Self {
            type_: content_part::AUDIO.into(),
            input_audio: Some(InputAudioData {
                data: data.into(),
                format: format.into(),
            }),
            ..Default::default()
        }
    }

    /// File content part.
    pub fn file(file_data: FileData) -> Self {
        Self {
            type_: content_part::FILE.into(),
            file: Some(file_data),
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Backward-compat type aliases
// ---------------------------------------------------------------------------

/// Alias kept for backward compatibility. Prefer [`ContentPart`].
pub type ChatCompletionContentPart = ContentPart;
