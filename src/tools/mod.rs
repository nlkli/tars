mod manager;
use crate::openai::models::{
    ChatCompletionBuilder, ChatCompletionMessageParam, ChatCompletionMessageToolCall,
};
pub use manager::*;
mod terminal;
pub use terminal::TerminalTool;
mod fs;
pub use fs::FileSystemTool;

pub trait Tool {
    fn name(&self) -> &str;
    fn function_names(&self) -> &[&str];
    fn register(&self, builder: ChatCompletionBuilder) -> ChatCompletionBuilder;
    fn call(&mut self, tc: &ChatCompletionMessageToolCall) -> Option<ChatCompletionMessageParam>;
}
