use crate::{
    openai::models::{
        ChatCompletionMessageParam, ChatCompletionMessageToolCall, ChatCompletionRequestBuilder,
        ChatCompletionTool, CustomToolCall, FunctionDefinition, FunctionToolCall,
    },
    term::Terminal,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
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
pub struct GetCommandOutputArgs {
    pub command_index: usize,
}

pub struct ToolManager {
    terminal: Terminal,
}

impl ToolManager {
    pub fn new(terminal: Terminal) -> Self {
        Self { terminal }
    }

    pub fn tools_registration(
        &self,
        mut builder: ChatCompletionRequestBuilder,
    ) -> ChatCompletionRequestBuilder {
        let execute_terminal_commands = FunctionDefinition {
            name: "execute_terminal_commands".into(),
            description: Some(
                "Execute one or more shell commands in a Unix terminal. Commands are executed sequentially in the order provided. The tool returns the index of the last scheduled command. To inspect command output, call get_command_output with the returned index.".into(),
            ),
            parameters: Some(
r#"{
  "type": "object",
  "properties": {
    "commands": {
      "type": "array[string]",
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

        let get_command_output = FunctionDefinition {
            name: "get_command_output".into(),
            description: Some(
                "Retrieve the textual output of a previously executed terminal command using its command index.".into(),
            ),
            parameters: Some(
r#"{
  "type": "object",
  "properties": {
    "index": {
      "type": "int",
      "description": "Index of a previously executed command."
    }
  },
  "required": ["command_index"],
  "additionalProperties": false
}"#
                .into(),
            ),
            strict: Some(true),
        };
        builder = builder.tool(ChatCompletionTool::function(execute_terminal_commands));
        builder = builder.tool(ChatCompletionTool::function(get_command_output));
        builder
    }

    pub fn call(&mut self, t: &ChatCompletionMessageToolCall) -> ChatCompletionMessageParam {
        match t {
            ChatCompletionMessageToolCall::Function { id, function } => {
                self.call_function(id, function)
            }
            ChatCompletionMessageToolCall::Custom { id, custom } => self.call_custom(id, custom),
        }
    }

    pub fn call_function(&mut self, id: &str, f: &FunctionToolCall) -> ChatCompletionMessageParam {
        let content: String;
        match f.name.as_str() {
            "execute_terminal_commands" => {
                match serde_json::from_str::<EexecuteTerminalCommandsArgs>(&f.arguments) {
                    Ok(args) => match self.terminal.execute_many(&args.commands) {
                        Ok(i) => {
                            // let val = serde_json::json!({"last_command_index": i});
                            // content = serde_json::to_string(&val).unwrap();
                            content = i.to_string();
                        }
                        Err(err) => {
                            content = ToolCallError::invalid_args(&err.to_string()).as_content();
                        }
                    },
                    Err(err) => {
                        content = ToolCallError::invalid_args(&err.to_string()).as_content();
                    }
                }
            }
            "get_command_output" => {
                match serde_json::from_str::<GetCommandOutputArgs>(&f.arguments) {
                    Ok(args) => match self.terminal.executions.get(args.command_index) {
                        Some(v) => {
                            // let val = serde_json::json!({"command_output": &v.output});
                            // content = serde_json::to_string(&val).unwrap();
                            content = v.output.clone();
                        }
                        None => {
                            content = ToolCallError::command_index_out_of_range(
                                args.command_index,
                                self.terminal.executions.len() - 1,
                            )
                            .as_content();
                        }
                    },
                    Err(err) => {
                        content = ToolCallError::invalid_args(&err.to_string()).as_content();
                    }
                }
            }
            _ => {
                content = ToolCallError::tool_does_not_exist(&f.name).as_content();
            }
        }
        ChatCompletionMessageParam::tool(content, id)
    }

    pub fn call_custom(&self, id: &str, f: &CustomToolCall) -> ChatCompletionMessageParam {
        unreachable!()
    }
}
