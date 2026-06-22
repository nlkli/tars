mod manager;
pub use manager::*;

mod system;
pub use system::SystemTool;

mod external;

use llm_provider_models::{
    ChatCompletionMessageParam, ChatCompletionMessageToolCall, FunctionDefinition,
};

pub trait Tool {
    fn name(&self) -> &str;
    fn functions(&self) -> Vec<FunctionDefinition>;
    fn call(&mut self, tc: &ChatCompletionMessageToolCall) -> Option<ChatCompletionMessageParam>;
    fn write_context(&mut self, w: &mut dyn std::fmt::Write) -> anyhow::Result<()>;
}
