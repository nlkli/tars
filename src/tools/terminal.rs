use std::collections::VecDeque;

use crate::{
    openai::models::{
        ChatCompletionBuilder, ChatCompletionMessageParam, ChatCompletionMessageToolCall,
        ChatCompletionTool, CustomToolCall, FunctionDefinition, FunctionToolCall,
    },
    term::{Execution, Terminal},
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Serialize)]
pub struct TerminalToolCallError {
    pub error: String,
}

impl TerminalToolCallError {
    fn as_content(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    // fn tool_does_not_exist(name: &str) -> Self {
    //     Self {
    //         error: format!("Tool '{}' does not exist.", name),
    //     }
    // }

    fn invalid_args(serde_error: &str) -> Self {
        Self {
            error: format!("Invalid arguments: {}", serde_error),
        }
    }

    fn command_index_out_of_range(i: usize, max: usize) -> Self {
        Self {
            error: format!("Command index '{i}' out of range: max({max})"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EexecuteTerminalCommandsArgs {
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReadCommandOutputArgs {
    pub index: usize,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
}

impl super::Tool for TerminalTool {
    fn name_space(&self) -> &[&str] {
        self.name_space
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
    name_space: &'static [&'static str],
    term: Terminal,
    history: VecDeque<Execution>,
    execution_tx: Option<UnboundedSender<Execution>>,
}

impl TerminalTool {
    pub fn new(t: Terminal, execution_tx: Option<UnboundedSender<Execution>>) -> Self {
        Self {
            // title: "terminal",
            name_space: &["execute_terminal_commands", "read_command_output"],
            term: t,
            history: VecDeque::new(),
            execution_tx,
        }
    }

    // pub fn title(&self) -> &str {
    //     self.title
    // }

    fn register(&self, mut builder: ChatCompletionBuilder) -> ChatCompletionBuilder {
        let execute_terminal_commands = FunctionDefinition {
            name: "execute_terminal_commands".into(),
            description: Some(
                "Execute one or more shell commands in a Unix terminal. Commands are executed sequentially in the order provided. The tool returns the index of the last scheduled command. To inspect command output, call read_command_output with bhe returned index.".into(),
            ),
            parameters: Some(
r#"{
  "type": "object",
  "properties": {
    "commands": {
      "type": "array<string>",
      "description": "List of shell commands to execute sequentially.",
    }
  },
  "required": ["commands"],
  "additionalProperties": false
}"#
                .into(),
            ),
            strict: Some(true),
        };

        let read_command_output = FunctionDefinition {
            name: "read_command_output".into(),
            description: Some(
                "Retrieve the textual output of a previously executed terminal command using its command index.".into(),
            ),
            parameters: Some(
r#"{
  "type": "object",
  "properties": {
    "index": {
      "type": "number",
      "description": "Index of a previously executed command."
    },
    "offset": {
      "type": "number",
      "description": "Character offset to start reading from. Defaults to 0."
    },
    "limit": {
      "type": "number",
      "description": "Maximum number of characters to return. Defaults to 4096."
    }
  },
  "required": ["index"],
  "additionalProperties": false
}"#
                .into(),
            ),
            strict: Some(true),
        };
        builder = builder.tool(ChatCompletionTool::function(execute_terminal_commands));
        builder = builder.tool(ChatCompletionTool::function(read_command_output));
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

    fn call_function(
        &mut self,
        id: &str,
        f: &FunctionToolCall,
    ) -> Option<ChatCompletionMessageParam> {
        let mut content = String::new();
        match f.name.as_str() {
            "execute_terminal_commands" => {
                match serde_json::from_str::<EexecuteTerminalCommandsArgs>(&f.arguments) {
                    Ok(args) => {
                        for command in args.commands.iter() {
                            match self.term.execute(command) {
                                Ok(ex) => {
                                    // let val = serde_json::json!({"last_command_index": i});
                                    // content = serde_json::to_string(&val).unwrap();
                                    if let Some(ref tx) = self.execution_tx {
                                        let _ = tx.send(ex.clone());
                                    }
                                    self.history.push_back(ex);
                                }
                                Err(e) => {
                                    content = TerminalToolCallError::invalid_args(&e.to_string())
                                        .as_content();
                                    break;
                                }
                            }
                        }
                        if content.is_empty() {
                            content = format!(
                                "last_command_index = {}\nuse read_command_output for read",
                                self.history.len() - 1
                            );
                        }
                    }
                    Err(err) => {
                        content =
                            TerminalToolCallError::invalid_args(&err.to_string()).as_content();
                    }
                }
            }
            "read_command_output" => {
                match serde_json::from_str::<ReadCommandOutputArgs>(&f.arguments) {
                    Ok(args) => match self.history.get_mut(args.index) {
                        Some(ex) => {
                            // let val = serde_json::json!({"command_output": &v.output});
                            // content = serde_json::to_string(&val).unwrap();
                            let output = ex.plain_output();
                            let chars = output.chars().collect::<Vec<_>>();
                            let offset = args.offset.unwrap_or(0);
                            let limit = args.limit.unwrap_or(4096);
                            let end = (offset + limit).min(chars.len());
                            let chunk = chars[offset..end].iter().collect::<String>();
                            let more = end < chars.len();
                            if !more && offset == 0 {
                                content = chunk;
                            } else {
                                let json_content = serde_json::json!({
                                    "o": chunk,
                                    "n": end,
                                    "m": more
                                });
                                content = serde_json::to_string(&json_content).unwrap_or_default();
                            }
                        }
                        None => {
                            content = TerminalToolCallError::command_index_out_of_range(
                                args.index,
                                self.history.len() - 1,
                            )
                            .as_content();
                        }
                    },
                    Err(e) => {
                        content = TerminalToolCallError::invalid_args(&e.to_string()).as_content();
                    }
                }
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
