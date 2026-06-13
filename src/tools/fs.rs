use crate::openai::models::{ChatCompletionBuilder, ChatCompletionMessageToolCall};


#[derive(Debug, Clone, Serialize)]
#[serde(tag = "error")]
pub enum TerminalToolError {
    InvalidArguments { message: String },
    CommandExecution { message: String },
}

impl FileSystemError {
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

impl super::Tool for FileSystemTool {
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

pub struct FileSystemTool {
    // title: &'static str,
    term: Terminal,
    max_output_chunk_size: usize,
    pending_output_chunks: VecDeque<String>,
    execution_tx: Option<UnboundedSender<Execution>>,
}

impl FileSystemTool {
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
}
