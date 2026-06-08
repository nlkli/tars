use anyhow::{Result, bail};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

const DEVELOPER_ROLE: &str = "developer";
const SYSTEM_ROLE: &str = "system";
const USER_ROLE: &str = "user";
const ASSISTANT_ROLE: &str = "assistant";
const TOOL_ROLE: &str = "tool";
const FUNCTION_ROLE: &str = "function";

const REASONING_EFFORT_NONE: &str = "none";
const REASONING_EFFORT_MINIMAL: &str = "minimal";
const REASONING_EFFORT_LOW: &str = "low";
const REASONING_EFFORT_MEDIUM: &str = "medium";
const REASONING_EFFORT_HIGH: &str = "high";
const REASONING_EFFORT_XHIGH: &str = "xhigh";

const FINISH_REASON_STOP: &str = "stop";
const FINISH_REASON_LENGTH: &str = "length";
const FINISH_REASON_TOOL_CALLS: &str = "tool_calls";
const FINISH_REASON_CONTENT_FILTER: &str = "content_filter";
const FINISH_REASON_FUNCTION_CALL: &str = "function_call";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessageParam {
    pub role: String,
    pub content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ChatCompletionMessageParam {
    pub fn new<S1: Into<String>, S2: Into<String>>(role: S1, content: S2) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
            name: None,
        }
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

    pub fn tool<S: Into<String>>(content: S) -> Self {
        Self::new(TOOL_ROLE, content)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatCompletionTool {
    #[serde(rename = "function")]
    Function { function: FunctionDefinition },

    #[serde(rename = "custom")]
    Custom { custom: CustomToolDefinition },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolDefinition {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<CustomToolFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CustomToolFormat {
    #[serde(rename = "text")]
    Text,

    #[serde(rename = "grammar")]
    Grammar { grammar: GrammarDefinition },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarDefinition {
    pub definition: String,
    pub syntax: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            kind: AllowedToolsType::AllowedTools,
            mode: mode,
            tools: tools,
        })
    }

    pub fn function(name: impl Into<String>) -> Self {
        Self::Function(ChatCompletionNamedFunctionToolChoice {
            kind: FunctionToolType::Function,
            function: ToolName { name: name.into() },
        })
    }

    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(ChatCompletionNamedCustomToolChoice {
            kind: CustomToolType::Custom,
            custom: ToolName { name: name.into() },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    None,
    Auto,
    Required,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionAllowedToolChoice {
    #[serde(rename = "type")]
    pub kind: AllowedToolsType,

    pub mode: AllowedToolsMode,

    pub tools: Vec<ChatCompletionTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedToolsType {
    #[serde(rename = "allowed_tools")]
    AllowedTools,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AllowedToolsMode {
    Auto,
    Required,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolName {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionNamedFunctionToolChoice {
    #[serde(rename = "type")]
    pub kind: FunctionToolType,

    pub function: ToolName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionToolType {
    #[serde(rename = "function")]
    Function,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionNamedCustomToolChoice {
    #[serde(rename = "type")]
    pub kind: CustomToolType,

    pub custom: ToolName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomToolType {
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Debug, Default, Clone)]
pub struct ChatCompletionRequestBuilder {
    model: String,
    messages: Vec<ChatCompletionMessageParam>,
    stream: Option<bool>,
    max_completion_tokens: Option<u32>,
    metadata: Option<HashMap<String, String>>,
    n: Option<u8>,
    frequency_penalty: Option<f32>,
    temperature: Option<f32>,
    reasoning_effort: Option<String>,
    tool_choice: Option<ChatCompletionNamedCustomToolChoice>,
    tools: Vec<ChatCompletionTool>,
}

impl ChatCompletionRequestBuilder {
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

    pub fn tool(mut self, v: ChatCompletionTool) -> Self {
        self.tools.push(v);
        self
    }

    pub fn tools(mut self, v: Vec<ChatCompletionTool>) -> Self {
        self.tools = v;
        self
    }

    pub fn build(self) -> ChatCompletionRequest {
        assert!(
            !self.model.is_empty(),
            "ChatCompletionRequestBuilder: model name is required"
        );
        ChatCompletionRequest {
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
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

    tool_choice: Option<ChatCompletionNamedCustomToolChoice>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ChatCompletionTool>,
}

impl ChatCompletionRequest {
    pub fn new<S: Into<String>>(model: S, messages: Vec<ChatCompletionMessageParam>) -> Self {
        Self {
            model: model.into(),
            messages: messages,
            ..Default::default()
        }
    }

    pub fn builder() -> ChatCompletionRequestBuilder {
        ChatCompletionRequestBuilder::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub created: u64,
    pub model: String,
    pub usage: Option<CompletionUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    pub finish_reason: String,
    pub index: u32,
    pub logprobs: Option<ChatCompletionLogprobs>,
    pub message: ChatCompletionMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionLogprobs {
    pub content: Option<Vec<ChatCompletionTokenLogprob>>,
    pub refusal: Option<Vec<ChatCompletionTokenLogprob>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionTokenLogprob {
    pub token: String,
    pub bytes: Option<Vec<u8>>,
    pub logprob: f32,
    pub top_logprobs: Vec<TopLogprob>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLogprob {
    pub token: String,
    pub bytes: Option<Vec<u8>>,
    pub logprob: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessage {
    pub role: String,
    pub content: Option<String>,
    pub refusal: Option<String>,
    pub annotations: Option<Vec<ChatCompletionAnnotation>>,
    // pub audio: Option<ChatCompletionAudio>,
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
}

impl ChatCompletionMessage {
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .is_some_and(|calls| !calls.is_empty())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatCompletionAnnotation {
    #[serde(rename = "url_citation")]
    UrlCitation { url_citation: UrlCitation },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionUsage {
    pub completion_tokens: u32,
    pub prompt_tokens: u32,
    pub total_tokens: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_prediction_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_prediction_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct OpenaiClientBuilder {
    base_url: String,
    api_key: Option<String>,
    http_client: Option<HttpClient>,
}

impl OpenaiClientBuilder {
    pub fn base_url<S: Into<String>>(mut self, v: S) -> Self {
        self.base_url = v.into();
        self
    }

    pub fn api_key<S: Into<String>>(mut self, v: S) -> Self {
        self.api_key.replace(v.into());
        self
    }

    pub fn http_client(mut self, v: HttpClient) -> Self {
        self.http_client.replace(v);
        self
    }

    pub fn build(self) -> OpenaiClient {
        assert!(
            !self.base_url.is_empty(),
            "OpenaiClientBuilder: base_url is required"
        );
        OpenaiClient {
            base_url: self.base_url,
            api_key: self.api_key,
            http_client: self.http_client.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OpenaiClient {
    pub base_url: String,
    api_key: Option<String>,
    http_client: HttpClient,
}

impl OpenaiClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url: base_url,
            api_key: api_key,
            http_client: HttpClient::default(),
        }
    }

    pub fn builder() -> OpenaiClientBuilder {
        OpenaiClientBuilder::default()
    }

    pub async fn create_chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let mut builder = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url));

        if let Some(ref api_key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = builder.json(request).send().await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            bail!("OpenAI API error: {body}");
        }

        Ok(response.json::<ChatCompletionResponse>().await?)
    }
}
