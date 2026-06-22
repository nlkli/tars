use crate::term::{Execution, Terminal};
use anyhow::Result;
use llm_provider_models::{
    ChatCompletionMessageParam, ChatCompletionMessageToolCall, CustomToolCall, FunctionDefinition,
    FunctionToolCall,
};
use serde::Deserialize;
use std::{collections::VecDeque, time::Duration};
use sysinfo::System;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum SystemToolErorr<'a> {
    InvalidArguments(&'a str),
    CommandExecution(&'a str),
    FileWriteFailed(&'a str),
}

impl<'a> SystemToolErorr<'a> {
    fn as_json_string(&self) -> String {
        let error = match self {
            Self::InvalidArguments(s) => format!("InvalidArguments: {s}"),
            Self::CommandExecution(s) => format!("CommandExecution: {s}"),
            Self::FileWriteFailed(s) => format!("FileWriteFailed: {s}"),
        };
        serde_json::json!({
            "error": error
        })
        .to_string()
    }
}

#[derive(Debug, Deserialize)]
pub struct WriteFileContentArgs {
    // #[serde(default)]
    pub path: String,
    // #[serde(default)]
    // pub abc_path: String,
    #[serde(default)]
    pub content: String,
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

impl super::Tool for SystemTool {
    fn name(&self) -> &str {
        "Terminal"
    }

    fn functions(&self) -> Vec<FunctionDefinition> {
        vec![
            FunctionDefinition {
                name: "execute_terminal_command".into(),
                description: Some(
                    "Execute a shell command in a persistent terminal session. Only run commands that terminate on their own. Never start interactive programs or commands that wait for input.".into(),
                ),
                parameters: Some(r#"{
  "type": "object",
  "properties": {
    "command": {
      "type": "string",
      "description": "REQUIRED. Shell command to execute."
    }
  },
  "required": ["command"],
  "additionalProperties": false
}"#
                    .into(),
                ),
                strict: Some(true),
            },
            FunctionDefinition {
                name: "continue_terminal_output".into(),
                description: Some("Read additional output from the previous terminal command when more_output_available=true.".into()),
                parameters: Some(
                    r#"{"type":"object","properties":{},"additionalProperties":false}"#.into(),
                ),
                strict: Some(true),
            },
            FunctionDefinition {
                name: "write_file_content".into(),
                description: Some(
                    "Write content to a file. 'path' and 'content' is REQUIRED and must always be provided. Path must begin with '/' or '~/'. Never use relative paths. Do not call this tool if a valid path is not known. The file is created if it does not exist and fully overwritten if it exists."
                    .into(),
                ),
                parameters: Some(r#"{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "REQUIRED. Absolute file path. Must start with '/' or '~/'."
    },
    "content": {
      "type": "string",
      "description": "REQUIRED. Content to write into the file."
    }
  },
  "required": ["path", "content"],
  "additionalProperties": false
}"#
                    .into(),
                ),
                strict: Some(true),
            }
        ]
    }

    fn call(&mut self, tc: &ChatCompletionMessageToolCall) -> Option<ChatCompletionMessageParam> {
        match tc {
            ChatCompletionMessageToolCall::Function { id, function } => {
                self.call_function(id, function)
            }
            ChatCompletionMessageToolCall::Custom { id, custom } => self.call_custom(id, custom),
        }
    }

    fn write_context(&mut self, w: &mut dyn std::fmt::Write) -> Result<()> {
        writeln!(w, "## Terminal")?;
        writeln!(
            w,
            "- Current working directory: {}",
            self.term.pwd()?.to_string_lossy()
        )?;
        writeln!(w, "- Shell: {}", self.term.shell)?;
        writeln!(
            w,
            "- Host name: {}",
            System::host_name().unwrap_or("-".into())
        )?;
        writeln!(
            w,
            "- OS: {}",
            System::long_os_version().unwrap_or("-".into())
        )?;
        writeln!(w, "- Kernel: {}", System::kernel_long_version())?;
        writeln!(w, "- CPU Arch: {}", System::cpu_arch())?;
        Ok(())
    }
}

pub struct SystemTool {
    term: Terminal,
    max_output_chunk_size: usize,
    pending_output_chunks: VecDeque<String>,
    execution_tx: Option<UnboundedSender<Execution>>,
    execute_duration: Option<Duration>,
}

impl SystemTool {
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
                            content = SystemToolErorr::InvalidArguments(
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
                                    content = SystemToolErorr::CommandExecution(&e.to_string())
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
                        content = SystemToolErorr::InvalidArguments(&e.to_string()).as_json_string();
                    }
                }
            }
            "continue_terminal_output" => {
                content = self.next_output_chunk();
            }
            "write_file_content" => {
                content = match serde_json::from_str::<WriteFileContentArgs>(&f.arguments) {
                    Ok(args) => {
                        let mut path = std::path::PathBuf::from(&args.path);
                        if let Ok(p) = path.strip_prefix("~/") {
                            if let Some(hd) = std::env::home_dir() {
                                path = hd.join(p);
                            }
                        }
                        if let Ok(p) = path.strip_prefix("$HOME/") {
                            if let Some(hd) = std::env::home_dir() {
                                path = hd.join(p);
                            }
                        }
                        match std::fs::write(path, args.content) {
                            Ok(_) => String::new(),
                            Err(e) => {
                                SystemToolErorr::FileWriteFailed(&e.to_string()).as_json_string()
                            }
                        }
                    }
                    Err(e) => SystemToolErorr::InvalidArguments(&e.to_string()).as_json_string(),
                };
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
