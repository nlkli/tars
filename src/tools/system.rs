use crate::term::{Execution, Terminal};
use anyhow::Result;
use llm_provider_models::{
    ChatCompletionMessageParam, ChatCompletionMessageToolCall, CustomToolCall, FunctionDefinition,
    FunctionToolCall,
};
use serde::Deserialize;
use std::{collections::VecDeque, fmt, io, path::PathBuf, time::Duration};
use sysinfo::System;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub enum SystemToolError {
    InvalidArguments(String),
    CommandExecution(String),
    FileWriteFailed { path: PathBuf, source: io::Error },
    FileAppendFailed { path: PathBuf, source: io::Error },
}

impl fmt::Display for SystemToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArguments(msg) => write!(f, "Invalid tool arguments: {msg}"),
            Self::CommandExecution(msg) => write!(f, "Command execution failed: {msg}"),
            Self::FileWriteFailed { path, source } => {
                write!(f, "Failed to write '{}': {source}", path.display())
            }
            Self::FileAppendFailed { path, source } => {
                write!(f, "Failed to append to '{}': {source}", path.display())
            }
        }
    }
}

impl SystemToolError {
    fn to_json(&self) -> String {
        serde_json::json!({ "error": self.to_string() }).to_string()
    }
}

#[derive(Debug, Deserialize)]
pub struct WriteFileContentArgs {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub file_path: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub append: bool,
}

impl WriteFileContentArgs {
    fn extract_path(&self) -> Option<&str> {
        if !self.path.is_empty() {
            return Some(&self.path);
        }
        if !self.file_path.is_empty() {
            return Some(&self.file_path);
        }
        None
    }
}

#[derive(Debug, Deserialize)]
pub struct ExecuteTerminalCommandArgs {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub commands: Option<Vec<String>>,
    #[serde(default)]
    pub silent: bool,
}

impl ExecuteTerminalCommandArgs {
    fn into_commands(self) -> Option<Vec<String>> {
        self.command.map(|c| vec![c]).or(self.commands)
    }
}

fn tool_execute_terminal_command() -> FunctionDefinition {
    FunctionDefinition {
        name: "execute_terminal_command".into(),
        description: Some(
            "Execute a shell command in a persistent, stateful session. All state persists across \
calls: working directory, env vars, background processes, filesystem. \
Full shell syntax supported (pipes, redirects, subshells, &&, etc.). \
Prefer non-interactive flags to avoid blocking on prompts. \
If more_output_available=true, call continue_terminal_output to read the next chunk. \
Prefer write_file_content over heredocs for writing files."
                .into(),
        ),
        parameters: Some(
            r#"{
  "type": "object",
  "properties": {
    "command": {
      "type": "string",
      "description": "Shell command to execute. Use && or ; for multi-step sequences."
    }
  },
  "required": ["command"],
  "additionalProperties": false
}"#
            .into(),
        ),
        strict: Some(true),
    }
}

fn tool_continue_terminal_output() -> FunctionDefinition {
    FunctionDefinition {
        name: "continue_terminal_output".into(),
        description: Some(
            "Fetch the next output chunk from the last terminal command. \
Only call when more_output_available=true AND the remaining output is needed — \
skip if sufficient context was already captured or the full output is not required."
                .into(),
        ),
        parameters: Some(
            r#"{"type":"object","properties":{},"additionalProperties":false}"#.into(),
        ),
        strict: Some(true),
    }
}

fn tool_write_file_content() -> FunctionDefinition {
    FunctionDefinition {
        name: "write_file_content".into(),
        description: Some(
            "Write content to a file. Fully replaces existing contents by default; \
set append=true to add to the end instead. \
Creates the file if it does not exist. Always provide the complete intended file content."
                .into(),
        ),
        parameters: Some(
            r#"{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "REQUIRED. Absolute or relative path to the target file. Parent directories must exist."
    },
    "content": {
      "type": "string",
      "description": "REQUIRED. Complete file content to write."
    },
    "append": {
      "type": "bool",
      "description": "If true, content is appended to the end of the file. Default: false."
    }
  },
  "required": ["path", "content"],
  "additionalProperties": false
}"#
            .into(),
        ),
        strict: Some(true),
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
            chunk
        } else {
            serde_json::json!({
                "output": chunk,
                "more_output_available": true
            })
            .to_string()
        }
    }

    fn enqueue_output(&mut self, output: String) {
        self.pending_output_chunks = output
            .chars()
            .collect::<Vec<_>>()
            .chunks(self.max_output_chunk_size)
            .map(String::from_iter)
            .collect();
    }

    fn resolve_path(&mut self, raw: &str) -> PathBuf {
        let path = PathBuf::from(raw);
        if let Ok(relative) = path.strip_prefix("~/") {
            if let Some(home) = std::env::home_dir() {
                return home.join(relative);
            }
        }
        self.term
            .pwd()
            .ok()
            .or_else(std::env::home_dir)
            .unwrap_or_default()
            .join(path)
    }

    fn handle_execute_terminal_command(&mut self, args_str: &str) -> String {
        let args = match serde_json::from_str::<ExecuteTerminalCommandArgs>(args_str) {
            Ok(a) => a,
            Err(e) => return SystemToolError::InvalidArguments(e.to_string()).to_json(),
        };
        let silent = args.silent;
        let commands = match args.into_commands() {
            Some(c) => c,
            None => {
                return SystemToolError::InvalidArguments(
                    "missing required field `command`".into(),
                )
                .to_json();
            }
        };

        let mut output = String::new();
        for command in &commands {
            match self.term.execute(command, self.execute_duration) {
                Ok(mut ex) => {
                    if let Some(ref tx) = self.execution_tx {
                        let _ = tx.send(ex.clone());
                    }
                    output.push_str(ex.plain_output());
                }
                Err(e) => return SystemToolError::CommandExecution(e.to_string()).to_json(),
            }
        }

        self.enqueue_output(output);
        if silent {
            String::new()
        } else {
            self.next_output_chunk()
        }
    }

    fn handle_write_file_content(&mut self, args_str: &str) -> String {
        let args = match serde_json::from_str::<WriteFileContentArgs>(args_str) {
            Ok(a) => a,
            Err(e) => return SystemToolError::InvalidArguments(e.to_string()).to_json(),
        };
        let path = match args.extract_path() {
            Some(p) => self.resolve_path(p),
            None => {
                return SystemToolError::InvalidArguments("missing required field `path`".into())
                    .to_json();
            }
        };

        let result = if args.append {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .and_then(|mut f| io::Write::write_all(&mut f, args.content.as_bytes()))
                .map_err(|e| SystemToolError::FileAppendFailed {
                    path: path.clone(),
                    source: e,
                })
        } else {
            std::fs::write(&path, &args.content).map_err(|e| SystemToolError::FileWriteFailed {
                path: path.clone(),
                source: e,
            })
        };

        match result {
            Ok(_) => String::new(),
            Err(e) => e.to_json(),
        }
    }
}

impl super::Tool for SystemTool {
    fn name(&self) -> &str {
        "System"
    }

    fn functions(&self) -> Vec<FunctionDefinition> {
        vec![
            tool_execute_terminal_command(),
            tool_continue_terminal_output(),
            tool_write_file_content(),
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

    fn write_context(&mut self, w: &mut dyn fmt::Write) -> Result<()> {
        writeln!(w, "## Terminal")?;
        writeln!(w, "- CWD: {}", self.term.pwd()?.to_string_lossy())?;
        writeln!(w, "- Shell: {}", self.term.shell)?;
        writeln!(w, "## System")?;
        writeln!(w, "- Host: {}", System::host_name().unwrap_or("-".into()))?;
        writeln!(
            w,
            "- OS: {}",
            System::long_os_version().unwrap_or("-".into())
        )?;
        writeln!(w, "- Kernel: {}", System::kernel_long_version())?;
        writeln!(w, "- Arch: {}", System::cpu_arch())?;
        Ok(())
    }
}

impl SystemTool {
    fn call_function(
        &mut self,
        id: &str,
        f: &FunctionToolCall,
    ) -> Option<ChatCompletionMessageParam> {
        let content = match f.name.as_str() {
            "execute_terminal_command" => self.handle_execute_terminal_command(&f.arguments),
            "continue_terminal_output" => self.next_output_chunk(),
            "write_file_content" => self.handle_write_file_content(&f.arguments),
            _ => return None,
        };
        Some(ChatCompletionMessageParam::tool(content, id))
    }

    fn call_custom(&self, _id: &str, _c: &CustomToolCall) -> Option<ChatCompletionMessageParam> {
        unreachable!()
    }
}
