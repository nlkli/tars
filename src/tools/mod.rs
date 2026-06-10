mod manager;
use crate::openai::models::{
    ChatCompletionBuilder, ChatCompletionMessageParam, ChatCompletionMessageToolCall,
};
pub use manager::*;
mod terminal;
pub use terminal::TerminalTool;

pub trait Tool {
    fn name_space(&self) -> &[&str];
    fn register(&self, builder: ChatCompletionBuilder) -> ChatCompletionBuilder;
    fn call(&mut self, tc: &ChatCompletionMessageToolCall) -> Option<ChatCompletionMessageParam>;
}
