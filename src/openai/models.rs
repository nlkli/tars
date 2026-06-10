use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, default};

pub const CONTENT_PART_TEXT_TYPE: &str = "text";
pub const CONTENT_PART_IMAGE_TYPE: &str = "image_url";
pub const CONTENT_PART_AUDIO_TYPE: &str = "input_audio";
pub const CONTENT_PART_FILE_TYPE: &str = "file";
pub const CONTENT_PART_REFUSAL_TYPE: &str = "refusal";

pub const INPUT_AUDIO_DATA_FORMAT_MP3: &str = "mp3";
pub const INPUT_AUDIO_DATA_FORMAT_WAV: &str = "wav";

pub const IMAGE_URL_DETAIL_AUTO: &str = "auto";
pub const IMAGE_URL_DETAIL_LOW: &str = "low";
pub const IMAGE_URL_DETAIL_HIGH: &str = "high";

pub const DEVELOPER_ROLE: &str = "developer";
pub const SYSTEM_ROLE: &str = "system";
pub const USER_ROLE: &str = "user";
pub const ASSISTANT_ROLE: &str = "assistant";
pub const TOOL_ROLE: &str = "tool";
pub const FUNCTION_ROLE: &str = "function";

pub const REASONING_EFFORT_NONE: &str = "none";
pub const REASONING_EFFORT_MINIMAL: &str = "minimal";
pub const REASONING_EFFORT_LOW: &str = "low";
pub const REASONING_EFFORT_MEDIUM: &str = "medium";
pub const REASONING_EFFORT_HIGH: &str = "high";
pub const REASONING_EFFORT_XHIGH: &str = "xhigh";

pub const FINISH_REASON_STOP: &str = "stop";
pub const FINISH_REASON_LENGTH: &str = "length";
pub const FINISH_REASON_TOOL_CALLS: &str = "tool_calls";
pub const FINISH_REASON_CONTENT_FILTER: &str = "content_filter";
pub const FINISH_REASON_FUNCTION_CALL: &str = "function_call";

/// Content part for a message - supports text, image, audio, and file inputs.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ChatCompletionContentPart {
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

impl ChatCompletionContentPart {
    pub fn text<S: Into<String>>(content: S) -> Self {
        Self {
            type_: CONTENT_PART_TEXT_TYPE.into(),
            text: Some(content.into()),
            ..Default::default()
        }
    }

    /// URL or base64 data of the image.
    pub fn image_url<S: Into<String>>(url: S, detail: Option<String>) -> Self {
        Self {
            type_: CONTENT_PART_IMAGE_TYPE.into(),
            image_url: Some(ImageUrl {
                url: url.into(),
                detail: detail,
            }),
            ..Default::default()
        }
    }

    /// Base64-encoded audio data.
    /// Audio format (wav/mp3).
    pub fn input_audio<S: Into<String>>(data: S, format: S) -> Self {
        Self {
            type_: CONTENT_PART_AUDIO_TYPE.into(),
            input_audio: Some(InputAudioData {
                data: data.into(),
                format: format.into(),
            }),
            ..Default::default()
        }
    }

    /// Base64-encoded file data (optional, used when passing file as string).
    pub fn file<S: Into<String>>(data: Option<S>, id: Option<S>, name: Option<S>) -> Self {
        let mut fd = FileData::default();
        if let Some(data) = data {
            fd.file_data.replace(data.into());
        }
        if let Some(id) = id {
            fd.file_id.replace(id.into());
        }
        if let Some(name) = name {
            fd.filename.replace(name.into());
        }
        Self {
            type_: CONTENT_PART_FILE_TYPE.into(),
            file: Some(fd),
            ..Default::default()
        }
    }
}

/// Image URL or base64-encoded data.
#[derive(Debug, Clone, Serialize)]
pub struct ImageUrl {
    /// URL or base64 data of the image.
    pub url: String,
    /// Detail level for vision models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Base64-encoded audio data.
#[derive(Debug, Clone, Serialize)]
pub struct InputAudioData {
    /// Base64-encoded audio data.
    pub data: String,
    /// Audio format (wav/mp3).
    pub format: String,
}

/// File data for input.
#[derive(Debug, Clone, Default, Serialize)]
pub struct FileData {
    /// Base64-encoded file data (optional, used when passing file as string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    /// File ID (optional, used when passing file as ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// Filename (optional, used when passing file as string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(untagged)]
pub enum ChatCompletionMessageContent {
    Text(String),
    Parts(Vec<ChatCompletionContentPart>),
    #[default]
    None,
}

/// Data about a previous audio response from the model.
#[derive(Debug, Clone, Serialize)]
pub struct AudioResponseData {
    /// Unique identifier for a previous audio response.
    pub id: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ChatCompletionMessageParam {
    pub role: String,
    pub content: ChatCompletionMessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioResponseData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatCompletionMessageParam {
    pub fn new<S1: Into<String>, S2: Into<String>>(role: S1, content: S2) -> Self {
        Self {
            role: role.into(),
            content: ChatCompletionMessageContent::Text(content.into()),
            ..Default::default()
        }
    }

    pub fn new_with_parts<S: Into<String>>(role: S) -> Self {
        Self {
            role: role.into(),
            content: ChatCompletionMessageContent::Parts(Vec::new()),
            ..Default::default()
        }
    }

    pub fn push_part(mut self, p: ChatCompletionContentPart) -> Self {
        if let ChatCompletionMessageContent::Parts(ref mut parts) = self.content {
            parts.push(p);
        }
        self
    }

    pub fn set_name<S: Into<String>>(mut self, v: S) -> Self {
        self.name.replace(v.into());
        self
    }

    pub fn set_audio(mut self, v: AudioResponseData) -> Self {
        self.audio.replace(v);
        self
    }

    pub fn tool_calls(mut self, tcs: Vec<ChatCompletionMessageToolCall>) -> Self {
        self.tool_calls.replace(tcs);
        self
    }

    pub fn developer<S: Into<String>>(content: S) -> Self {
        Self::new(DEVELOPER_ROLE, content)
    }

    pub fn system<S: Into<String>>(content: S) -> Self {
        Self::new(SYSTEM_ROLE, content)
    }

    pub fn user<S: Into<String>>(content: S) -> Self {
        Self::new(USER_ROLE, content)
    }

    pub fn assistant<S: Into<String>>(content: S) -> Self {
        Self::new(ASSISTANT_ROLE, content)
    }

    pub fn tool<S1: Into<String>, S2: Into<String>>(content: S1, tool_call_id: S2) -> Self {
        let mut p = Self::new(TOOL_ROLE, content);
        p.tool_call_id.replace(tool_call_id.into());
        p
    }

    pub fn is_developer(&self) -> bool {
        self.role == DEVELOPER_ROLE
    }

    pub fn is_system(&self) -> bool {
        self.role == SYSTEM_ROLE
    }

    pub fn is_user(&self) -> bool {
        self.role == USER_ROLE
    }

    pub fn is_assistant(&self) -> bool {
        self.role == ASSISTANT_ROLE
    }

    pub fn is_tool(&self) -> bool {
        self.role == TOOL_ROLE
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ChatCompletionTool {
    #[serde(rename = "function")]
    Function { function: FunctionDefinition },
    #[serde(rename = "custom")]
    Custom { custom: CustomToolDefinition },
}

impl ChatCompletionTool {
    pub fn function(fd: FunctionDefinition) -> Self {
        Self::Function { function: fd }
    }

    pub fn custom(ctd: CustomToolDefinition) -> Self {
        Self::Custom { custom: ctd }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomToolDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<CustomToolFormat>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum CustomToolFormat {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "grammar")]
    Grammar { grammar: GrammarDefinition },
}

#[derive(Debug, Clone, Serialize)]
pub struct GrammarDefinition {
    pub definition: String,
    pub syntax: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ChatCompletionToolChoice {
    Mode(ToolChoiceMode),

    AllowedTools(ChatCompletionAllowedToolChoice),

    Function(ChatCompletionNamedFunctionToolChoice),

    Custom(ChatCompletionNamedCustomToolChoice),
}

impl ChatCompletionToolChoice {
    pub fn none() -> Self {
        Self::Mode(ToolChoiceMode::None)
    }

    pub fn auto() -> Self {
        Self::Mode(ToolChoiceMode::Auto)
    }

    pub fn required() -> Self {
        Self::Mode(ToolChoiceMode::Required)
    }

    pub fn allowed_tools(mode: AllowedToolsMode, tools: Vec<ChatCompletionTool>) -> Self {
        Self::AllowedTools(ChatCompletionAllowedToolChoice {
            type_: AllowedToolsType::AllowedTools,
            mode: mode,
            tools: tools,
        })
    }

    pub fn function(name: impl Into<String>) -> Self {
        Self::Function(ChatCompletionNamedFunctionToolChoice {
            type_: FunctionToolType::Function,
            function: ToolName { name: name.into() },
        })
    }

    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(ChatCompletionNamedCustomToolChoice {
            type_: CustomToolType::Custom,
            custom: ToolName { name: name.into() },
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    None,
    Auto,
    Required,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionAllowedToolChoice {
    #[serde(rename = "type")]
    pub type_: AllowedToolsType,
    pub mode: AllowedToolsMode,
    pub tools: Vec<ChatCompletionTool>,
}

#[derive(Debug, Clone, Serialize)]
pub enum AllowedToolsType {
    #[serde(rename = "allowed_tools")]
    AllowedTools,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AllowedToolsMode {
    Auto,
    Required,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolName {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionNamedFunctionToolChoice {
    #[serde(rename = "type")]
    pub type_: FunctionToolType,

    pub function: ToolName,
}

#[derive(Debug, Clone, Serialize)]
pub enum FunctionToolType {
    #[serde(rename = "function")]
    Function,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionNamedCustomToolChoice {
    #[serde(rename = "type")]
    pub type_: CustomToolType,

    pub custom: ToolName,
}

#[derive(Debug, Clone, Serialize)]
pub enum CustomToolType {
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Debug, Default, Clone)]
pub struct ChatCompletionBuilder {
    model: String,
    messages: Vec<ChatCompletionMessageParam>,
    stream: Option<bool>,
    max_completion_tokens: Option<u32>,
    metadata: Option<HashMap<String, String>>,
    n: Option<u8>,
    frequency_penalty: Option<f32>,
    temperature: Option<f32>,
    reasoning_effort: Option<String>,
    tool_choice: Option<ChatCompletionToolChoice>,
    tools: Vec<ChatCompletionTool>,
}

impl ChatCompletionBuilder {
    pub fn model<S: Into<String>>(mut self, v: S) -> Self {
        self.model = v.into();
        self
    }

    pub fn messages(mut self, v: Vec<ChatCompletionMessageParam>) -> Self {
        self.messages = v;
        self
    }

    pub fn message(mut self, v: ChatCompletionMessageParam) -> Self {
        self.messages.push(v);
        self
    }

    pub fn stream(mut self) -> Self {
        self.stream.replace(true);
        self
    }

    pub fn max_completion_tokens(mut self, v: u32) -> Self {
        self.max_completion_tokens = Some(v);
        self
    }

    pub fn metadata(mut self, v: HashMap<String, String>) -> Self {
        self.metadata = Some(v);
        self
    }

    pub fn n(mut self, v: u8) -> Self {
        self.n = Some(v);
        self
    }

    pub fn frequency_penalty(mut self, v: f32) -> Self {
        self.frequency_penalty = Some(v);
        self
    }

    pub fn temperature(mut self, v: f32) -> Self {
        self.temperature = Some(v);
        self
    }

    pub fn reasoning_effort<S: Into<String>>(mut self, v: S) -> Self {
        self.reasoning_effort = Some(v.into());
        self
    }

    pub fn tool_choice(mut self, v: ChatCompletionToolChoice) -> Self {
        self.tool_choice.replace(v);
        self
    }

    pub fn tool(mut self, v: ChatCompletionTool) -> Self {
        self.tools.push(v);
        self
    }

    pub fn tools(mut self, v: Vec<ChatCompletionTool>) -> Self {
        self.tools = v;
        self
    }

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
            reasoning_effort: self.reasoning_effort,
            tool_choice: self.tool_choice,
            tools: self.tools,
        }
    }
}

/// https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create
#[derive(Debug, Clone, Default, Serialize)]
pub struct ChatCompletion {
    pub model: String,
    pub messages: Vec<ChatCompletionMessageParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// 1..=128
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,
    /// -2..=2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// 0..=2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    pub tool_choice: Option<ChatCompletionToolChoice>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ChatCompletionTool>,
}

impl ChatCompletion {
    pub fn new<S: Into<String>>(model: S, messages: Vec<ChatCompletionMessageParam>) -> Self {
        Self {
            model: model.into(),
            messages: messages,
            ..Default::default()
        }
    }

    pub fn builder() -> ChatCompletionBuilder {
        ChatCompletionBuilder::default()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub created: u64,
    pub model: String,
    #[serde(default)]
    pub usage: Option<CompletionUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChoice {
    pub finish_reason: String,
    pub index: u32,
    #[serde(default)]
    pub logprobs: Option<ChatCompletionLogprobs>,
    pub message: ChatCompletionMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionLogprobs {
    #[serde(default)]
    pub content: Option<Vec<ChatCompletionTokenLogprob>>,
    #[serde(default)]
    pub refusal: Option<Vec<ChatCompletionTokenLogprob>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionTokenLogprob {
    pub token: String,
    #[serde(default)]
    pub bytes: Option<Vec<u8>>,
    pub logprob: f32,
    pub top_logprobs: Vec<TopLogprob>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopLogprob {
    pub token: String,
    #[serde(default)]
    pub bytes: Option<Vec<u8>>,
    pub logprob: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionMessage {
    pub role: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub refusal: Option<String>,
    #[serde(default)]
    pub annotations: Option<Vec<ChatCompletionAnnotation>>,
    #[serde(default)]
    pub audio: Option<ChatCompletionAudio>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
}

impl ChatCompletionMessage {
    // pub fn as_param(&self) -> ChatCompletionMessageParam {
    //     let mut param =
    //         ChatCompletionMessageParam::new(&self.role, self.content.clone().unwrap_or_default());
    //     if let Some(ref audio) = self.audio {
    //         param.audio.replace(AudioResponseData {
    //             id: audio.id.clone(),
    //         });
    //     }
    //     param
    // }

    pub fn content_or_default(&self) -> &str {
        self.content.as_ref().map(|c| c.as_str()).unwrap_or("")
    }

    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .is_some_and(|calls| !calls.is_empty())
    }
}

/// Audio response object when the audio output modality is requested.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionAudio {
    /// Unique identifier for this audio response.
    pub id: String,
    /// Base64 encoded audio bytes generated by the model, in the format specified in the request.
    pub data: String,
    /// The Unix timestamp (in seconds) for when this audio response will no longer be accessible.
    pub expires_at: i64,
    /// Transcript of the audio generated by the model.
    pub transcript: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ChatCompletionAnnotation {
    #[serde(rename = "url_citation")]
    UrlCitation { url_citation: UrlCitation },
}

#[derive(Debug, Clone, Deserialize)]
pub struct UrlCitation {
    pub start_index: u32,
    pub end_index: u32,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatCompletionMessageToolCall {
    #[serde(rename = "function")]
    Function {
        id: String,
        function: FunctionToolCall,
    },
    #[serde(rename = "custom")]
    Custom { id: String, custom: CustomToolCall },
}

impl ChatCompletionMessageToolCall {
    pub fn id(&self) -> &str {
        match self {
            Self::Function { id, .. } => id,
            Self::Custom { id, .. } => id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Function { function, .. } => &function.name,
            Self::Custom { custom, .. } => &custom.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionToolCall {
    pub name: String,
    /// JSON string generated by the model.
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolCall {
    pub name: String,
    /// Raw tool input generated by the model.
    pub input: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionUsage {
    pub completion_tokens: u32,
    pub prompt_tokens: u32,
    pub total_tokens: u32,
    #[serde(default)]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionTokensDetails {
    #[serde(default)]
    pub accepted_prediction_tokens: Option<u32>,
    #[serde(default)]
    pub audio_tokens: Option<u32>,
    #[serde(default)]
    pub reasoning_tokens: Option<u32>,
    #[serde(default)]
    pub rejected_prediction_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub audio_tokens: Option<u32>,

    #[serde(default)]
    pub cached_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListModelsResponse {
    #[serde(default)]
    pub data: Vec<Model>,
    pub object: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    pub id: String,
    pub created: u64,
    pub owned_by: String,
    pub object: String,
}

/// Delta content in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChunkChoiceDelta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub refusal: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<DeltaToolCalls>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeltaToolCalls {
    pub index: usize,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub function: Option<DeltaFunctionToolCall>,
    #[serde(default)]
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeltaFunctionToolCall {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arguments: Option<String>,
}

/// A single choice in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChunkChoice {
    pub index: usize,
    pub delta: ChatCompletionChunkChoiceDelta,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// Streaming chat completion chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    // #[serde(default)]
    // pub system_fingerprint: Option<String>,
    pub choices: Vec<ChatCompletionChunkChoice>,
    #[serde(default)]
    pub usage: Option<CompletionUsage>,
    // #[serde(default)]
    // pub timings: Option<Timings>,
}
