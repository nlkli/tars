//! Serialization and deserialization types for the OpenAI-compatible chat
//! completion API.
//!
//! # Module layout
//!
//! | Module | Contents |
//! |---|---|
//! | [`constants`] | String constants for roles, finish reasons, content-part types, etc. |
//! | [`message`] | Request message types: [`message::MessageParam`], [`message::ContentPart`], etc. |
//! | [`tool`] | Tool definitions, tool-choice configuration, and tool-call types. |
//! | [`completion`] | [`completion::ChatCompletion`] (request) and [`completion::ChatCompletionResponse`] (response). |
//! | [`streaming`] | [`streaming::ChatCompletionChunk`] and related SSE types. |
//! | [`logprob`] | Log-probability types. |
//! | [`model`] | Model-listing response types. |
//!
//! # Quick start
//!
//! ```rust,ignore
//! use oai::{
//!     completion::ChatCompletion,
//!     message::MessageParam,
//! };
//!
//! let request = ChatCompletion::builder()
//!     .model("gpt-4o")
//!     .message(MessageParam::user("Hello!"))
//!     .temperature(0.7)
//!     .build();
//!
//! let body = serde_json::to_string(&request)?;
//! ```

pub mod completion;
pub mod consts;
pub mod logprob;
pub mod message;
pub mod model;
pub mod streaming;
pub mod tool;

pub mod llamacpp;

// ---------------------------------------------------------------------------
// Flat re-exports – mirror the original single-file public surface so that
// downstream code does not need to update import paths.
// ---------------------------------------------------------------------------

// Constants
pub use consts::*;

// Message types
pub use message::{
    AudioResponseData, ChatCompletionContentPart, ChatCompletionMessageContent,
    ChatCompletionMessageParam, ContentPart, FileData, ImageUrl, InputAudioData, MessageContent,
    MessageParam,
};

// Tool types
pub use tool::{
    AllowedToolChoice, AllowedToolsMode, AllowedToolsType, ChatCompletionAllowedToolChoice,
    ChatCompletionMessageToolCall, ChatCompletionNamedCustomToolChoice,
    ChatCompletionNamedFunctionToolChoice, ChatCompletionTool, ChatCompletionToolChoice,
    CustomToolCall, CustomToolDefinition, CustomToolFormat, CustomToolType, FunctionDefinition,
    FunctionToolCall, FunctionToolType, GrammarDefinition, NamedCustomToolChoice,
    NamedFunctionToolChoice, Tool, ToolCall, ToolChoice, ToolChoiceMode, ToolName,
};

// Completion types
pub use completion::{
    ChatCompletion, ChatCompletionAnnotation, ChatCompletionAudio, ChatCompletionBuilder,
    ChatCompletionChoice, ChatCompletionMessage, ChatCompletionResponse, CompletionTokensDetails,
    CompletionUsage, PromptTokensDetails, UrlCitation,
};

// Streaming types
pub use streaming::{
    ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionChunkChoiceDelta,
    DeltaFunctionToolCall, DeltaToolCall,
};

// Log-probability types
pub use logprob::{ChatCompletionLogprobs, ChatCompletionTokenLogprob, TokenLogprob, TopLogprob};

// Model listing
pub use model::{ListModelsResponse, Model};
