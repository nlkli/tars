//! Chat completion request and response types.

pub mod request;
pub mod response;

pub use request::{ChatCompletion, ChatCompletionBuilder};
pub use response::{
    ChatCompletionAnnotation, ChatCompletionAudio, ChatCompletionChoice, ChatCompletionMessage,
    ChatCompletionResponse, CompletionTokensDetails, CompletionUsage, PromptTokensDetails,
    UrlCitation,
};
