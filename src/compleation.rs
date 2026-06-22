use llm_provider_models::{ChatCompletion, ChatCompletionBuilder, MessageParam, enums::ReasoningEffort};
use std::ops::{Deref, DerefMut};
use crate::{term::Terminal, tools::{SystemTool, ToolManager}};
use anyhow::Result;

#[derive(Default)]
pub struct ChatCompletionExBuilder {
    inner_builder: ChatCompletionBuilder,
    tool_manager: ToolManager,
}

impl ChatCompletionExBuilder {
    pub fn with_system_tool(mut self, terminal: Terminal, max_output_chunk_size: usize) -> Self {
        self.tool_manager.add(SystemTool::new(
                terminal,
                max_output_chunk_size,
                None,
                None,
        ));
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.inner_builder = self.inner_builder.model(model);
        self
    }

    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.inner_builder = self.inner_builder.message(MessageParam::system(system));
        self
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.inner_builder = self.inner_builder.message(MessageParam::user(prompt));
        self
    }

    /// Enable streaming (SSE) output.
    pub fn stream(mut self) -> Self {
        self.inner_builder = self.inner_builder.stream();
        self
    }

    pub fn max_tokens(mut self, n: u32) -> Self {
        self.inner_builder = self.inner_builder.max_completion_tokens(n);
        self
    }

    pub fn temperature(mut self, temp: f32) -> Self {
        self.inner_builder = self.inner_builder.temperature(temp);
        self
    }

    pub fn reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.inner_builder = self.inner_builder.reasoning_effort(effort.as_str());
        self
    }

    pub fn build(mut self) -> Result<ChatCompletionEx> {
        let mut system_context = String::new();
        self.tool_manager.write_context(&mut system_context);
        Ok(ChatCompletionEx {
            inner: self.inner_builder.build(),
            tool_manager: self.tool_manager,
            system_context: system_context,
        })
    }
}

pub struct ChatCompletionEx{
    inner: ChatCompletion,
    tool_manager: ToolManager,
    system_context: String,
};

impl Deref for ChatCompletionEx {
    type Target = ChatCompletion;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ChatCompletionEx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ChatCompletionEx {
    pub fn builder() -> ChatCompletionExBuilder {
        ChatCompletionExBuilder::default()
    }
}
