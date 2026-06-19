use crate::{
    openai::models::{
        ChatCompletionBuilder, ChatCompletionMessageParam, ChatCompletionMessageToolCall,
        ChatCompletionTool, CustomToolCall, FunctionDefinition, FunctionToolCall,
    },
    term::{Execution, Terminal},
};
use serde::Deserialize;
use std::{collections::VecDeque, time::Duration};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum TerminalToolError<'a> {
    InvalidArguments(&'a str),
    CommandExecution(&'a str),
}

impl<'a> TerminalToolError<'a> {
    fn as_json_string(&self) -> String {
        let error = match self {
            Self::InvalidArguments(s) => format!("InvalidArguments: {s}"),
            Self::CommandExecution(s) => format!("CommandExecution: {s}"),
        };
        serde_json::json!({
            "error": error
        })
        .to_string()
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
    fn name(&self) -> &str {
        "Terminal"
    }

    fn function_names(&self) -> &[&str] {
        &["execute_terminal_command", "continue_terminal_output"]
    }

    fn register(&self, mut builder: ChatCompletionBuilder) -> ChatCompletionBuilder {
        let execute_terminal_command = FunctionDefinition {
            name: "execute_terminal_command".into(),
            description: Some(
                "Execute a shell command in a persistent terminal session. POSIX-compatible. The terminal state persists between calls. Only run commands that terminate on their own. Never start interactive programs or commands that wait for input. Set silent=true when output is not needed.".into(),
            ),
            parameters: Some(r#"{
  "type": "object",
  "properties": {
    "command": {
      "type": "string",
      "description": "REQUIRED. Shell command to execute."
    },
    "silent": {
      "type": "boolean",
      "description": "Suppress command output when the result is not needed."
    }
  },
  "required": ["command"],
  "additionalProperties": false
}"#
                .into(),
            ),
            strict: Some(true),
        };
        let continue_output = FunctionDefinition {
            name: "continue_terminal_output".into(),
            description: Some("Read additional output from the previous terminal command when more_output_available=true.".into()),
            parameters: Some(
                r#"{"type":"object","properties":{},"additionalProperties":false}"#.into(),
            ),
            strict: Some(true),
        };
        builder = builder.tool(ChatCompletionTool::function(execute_terminal_command));
        builder = builder.tool(ChatCompletionTool::function(continue_output));
        builder
    }

    fn call(&mut self, tc: &ChatCompletionMessageToolCall) -> Option<ChatCompletionMessageParam> {
        match tc {
            ChatCompletionMessageToolCall::Function { id, function } => {
                self.call_function(id, function)
            }
            ChatCompletionMessageToolCall::Custom { id, custom } => self.call_custom(id, custom),
        }
    }
}

pub struct TerminalTool {
    term: Terminal,
    max_output_chunk_size: usize,
    pending_output_chunks: VecDeque<String>,
    execution_tx: Option<UnboundedSender<Execution>>,
    execute_duration: Option<Duration>,
}

impl TerminalTool {
    pub fn new(
        term: Terminal,
        max_output_chunk_size: usize,
        execution_tx: Option<UnboundedSender<Execution>>,
        execute_duration: Option<Duration>,
    ) -> Self {
        Self {
            term,
            max_output_chunk_size,
            pending_output_chunks: VecDeque::new(),
            execution_tx,
            execute_duration,
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
            "execute_terminal_command" => {
                match serde_json::from_str::<ExecuteTerminalCommandsArgs>(&f.arguments) {
                    Ok(mut args) => {
                        let mut output = String::new();
                        let Some(commands) = args.take_commands() else {
                            content = TerminalToolError::InvalidArguments(
                                "missing field
                                    `commands`"
                                    .into(),
                            )
                            .as_json_string();
                            return Some(ChatCompletionMessageParam::tool(content, id));
                        };
                        for command in commands.iter() {
                            match self.term.execute(command, self.execute_duration) {
                                Ok(mut ex) => {
                                    if let Some(ref tx) = self.execution_tx {
                                        let _ = tx.send(ex.clone());
                                    }
                                    output.push_str(ex.plain_output());
                                }
                                Err(e) => {
                                    content = TerminalToolError::CommandExecution(&e.to_string())
                                        .as_json_string();
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
                    Err(e) => {
                        content =
                            TerminalToolError::InvalidArguments(&e.to_string()).as_json_string();
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
