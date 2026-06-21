//! Types that make up the `messages` array in a chat completion request.
pub mod content;
pub mod part;

pub use content::{
    AudioResponseData, ChatCompletionMessageContent, ChatCompletionMessageParam, MessageContent,
    MessageParam,
};
pub use part::{ChatCompletionContentPart, ContentPart, FileData, ImageUrl, InputAudioData};
