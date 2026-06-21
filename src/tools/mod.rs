mod manager;
use llm_provider_models::{
    ChatCompletionBuilder, ChatCompletionMessageParam, ChatCompletionMessageToolCall,
};
pub use manager::*;
mod terminal;
pub use terminal::TerminalTool;
mod fs;
pub use fs::FileSystemTool;

#[allow(dead_code)]
pub trait Tool {
    fn name(&self) -> &str;
    fn function_names(&self) -> &[&str];
    fn register(&self, builder: ChatCompletionBuilder) -> ChatCompletionBuilder;
    fn call(&mut self, tc: &ChatCompletionMessageToolCall) -> Option<ChatCompletionMessageParam>;
    fn write_context(&mut self, w: &mut dyn std::fmt::Write) -> anyhow::Result<()>;
}
