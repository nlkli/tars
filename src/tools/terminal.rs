use crate::{
    openai::models::{
        ChatCompletionBuilder, ChatCompletionMessageParam, ChatCompletionMessageToolCall,
        ChatCompletionTool, CustomToolCall, FunctionDefinition, FunctionToolCall,
    },
    term::{Execution, Terminal},
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "error")]
pub enum TerminalToolError {
    InvalidArguments { message: String },
    CommandExecution { message: String },
}

impl TerminalToolError {
    fn invalid_arguments(message: impl Into<String>) -> Self {
        Self::InvalidArguments {
            message: message.into(),
        }
    }

    fn command_execution(message: impl Into<String>) -> Self {
        Self::CommandExecution {
            message: message.into(),
        }
    }

    fn as_content(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecuteTerminalCommandsArgs {
    #[serde(default)]
    pub commands: Option<Vec<String>>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub silent: bool,
}

impl ExecuteTerminalCommandsArgs {
    fn take_commands(&mut self) -> Option<Vec<String>> {
        self.command
            .take()
            .map(|c| Vec::from([c]))
            .or(self.commands.take())
    }
}

impl super::Tool for TerminalTool {
    fn name_space(&self) -> &[&str] {
        &["execute_terminal_commands", "continue_output"]
    }

    fn register(&self, builder: ChatCompletionBuilder) -> ChatCompletionBuilder {
        self.register(builder)
    }

    fn call(&mut self, tc: &ChatCompletionMessageToolCall) -> Option<ChatCompletionMessageParam> {
        self.call(tc)
    }
}

pub struct TerminalTool {
    // title: &'static str,
    term: Terminal,
    max_output_chunk_size: usize,
    pending_output_chunks: VecDeque<String>,
    execution_tx: Option<UnboundedSender<Execution>>,
}

impl TerminalTool {
    pub fn new(
        t: Terminal,
        max_output_chunk_size: usize,
        execution_tx: Option<UnboundedSender<Execution>>,
    ) -> Self {
        Self {
            term: t,
            max_output_chunk_size,
            pending_output_chunks: VecDeque::new(),
            execution_tx,
        }
    }

    fn register(&self, mut builder: ChatCompletionBuilder) -> ChatCompletionBuilder {
        let execute_terminal_commands = FunctionDefinition {
            name: "execute_terminal_commands".into(),
            description: Some(
                "Execute one or more shell commands in the terminal. Set silent=true when output is not needed.".into(),
            ),
            parameters: Some(
                r#"{
  "type": "object",
  "properties": {
    "commands": {
      "type": "array",
      "items": {
        "type": "string"
      },
      "description": "Commands executed sequentially."
    },
    "silent": {
      "type": "boolean",
      "description": "When true, command output is suppressed."
    }
  },
  "required": ["commands"],
  "additionalProperties": false
}"#
                .into(),
            ),
            strict: Some(true),
        };

        let continue_output = FunctionDefinition {
            name: "continue_output".into(),
            description: Some("Read the next chunk of output from the previous command when more_output_available=true.".into()),
            parameters: Some(
                r#"{"type":"object","properties":{},"additionalProperties":false}"#.into(),
            ),
            strict: Some(true),
        };
        builder = builder.tool(ChatCompletionTool::function(execute_terminal_commands));
        builder = builder.tool(ChatCompletionTool::function(continue_output));
        builder
    }

    pub fn call(
        &mut self,
        tc: &ChatCompletionMessageToolCall,
    ) -> Option<ChatCompletionMessageParam> {
        match tc {
            ChatCompletionMessageToolCall::Function { id, function } => {
                self.call_function(id, function)
            }
            ChatCompletionMessageToolCall::Custom { id, custom } => self.call_custom(id, custom),
        }
    }

    fn next_output_chunk(&mut self) -> String {
        let Some(chunk) = self.pending_output_chunks.pop_front() else {
            return String::new();
        };

        if self.pending_output_chunks.is_empty() {
            return chunk;
        }

        serde_json::json!({
            "output": chunk,
            "more_output_available": true
        })
        .to_string()
    }

    fn call_function(
        &mut self,
        id: &str,
        f: &FunctionToolCall,
    ) -> Option<ChatCompletionMessageParam> {
        let content: String;
        match f.name.as_str() {
            "execute_terminal_commands" => {
                match serde_json::from_str::<ExecuteTerminalCommandsArgs>(&f.arguments) {
                    Ok(mut args) => {
                        let mut output = String::new();
                        let Some(commands) = args.take_commands() else {
                            content =
                                TerminalToolError::invalid_arguments("missing field `commands`")
                                    .as_content();
                            return Some(ChatCompletionMessageParam::tool(content, id));
                        };
                        for command in commands.iter() {
                            match self.term.execute(command) {
                                Ok(mut ex) => {
                                    if let Some(ref tx) = self.execution_tx {
                                        let _ = tx.send(ex.clone());
                                    }
                                    output.push_str(ex.plain_output());
                                }
                                Err(e) => {
                                    content = TerminalToolError::command_execution(&e.to_string())
                                        .as_content();
                                    return Some(ChatCompletionMessageParam::tool(content, id));
                                }
                            }
                        }
                        self.pending_output_chunks = output
                            .chars()
                            .collect::<Vec<_>>()
                            .chunks(self.max_output_chunk_size)
                            .map(|c| String::from_iter(c))
                            .collect::<VecDeque<_>>();
                        if args.silent {
                            return Some(ChatCompletionMessageParam::tool("", id));
                        }
                        content = self.next_output_chunk();
                    }
                    Err(err) => {
                        content =
                            TerminalToolError::invalid_arguments(&err.to_string()).as_content();
                    }
                }
            }
            "continue_output" => {
                content = self.next_output_chunk();
            }
            _ => {
                return None;
            }
        }
        Some(ChatCompletionMessageParam::tool(content, id))
    }

    fn call_custom(&self, _id: &str, _c: &CustomToolCall) -> Option<ChatCompletionMessageParam> {
        unreachable!()
    }
}
