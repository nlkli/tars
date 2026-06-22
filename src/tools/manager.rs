use anyhow::Result;
use llm_provider_models::{
    ChatCompletionBuilder, ChatCompletionMessageParam, ChatCompletionMessageToolCall,
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt::Write};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCallError {
    pub error: String,
}

impl ToolCallError {
    fn as_content(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn tool_does_not_exist(name: &str) -> Self {
        Self {
            error: format!("Tool '{}' does not exist.", name),
        }
    }
}

#[derive(Default)]
pub struct ToolManager {
    tools: VecDeque<Box<dyn super::Tool + Send>>,
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            tools: VecDeque::new(),
        }
    }

    pub fn add(&mut self, tool: impl super::Tool + Send + 'static) {
        self.tools.push_back(Box::new(tool));
    }

    pub fn register_all(&self, mut builder: ChatCompletionBuilder) -> ChatCompletionBuilder {
        // for tool in self.tools.iter() {
        //     builder = tool.register(builder);
        // }
        builder
    }

    pub fn call(&mut self, tc: &ChatCompletionMessageToolCall) -> ChatCompletionMessageParam {
        for tool in self.tools.iter_mut() {
            if let Some(p) = tool.call(tc) {
                return p;
            }
        }
        ChatCompletionMessageParam::tool(
            ToolCallError::tool_does_not_exist(tc.name()).as_content(),
            tc.id(),
        )
    }

    pub fn write_context(&mut self, w: &mut dyn Write) -> Result<()> {
        writeln!(w, "# Context")?;
        for tool in self.tools.iter_mut() {
            tool.write_context(w)?;
        }
        Ok(())
    }
}
