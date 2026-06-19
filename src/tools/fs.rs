use crate::openai::models::{
    ChatCompletionBuilder, ChatCompletionMessageParam, ChatCompletionMessageToolCall,
    ChatCompletionTool, FunctionDefinition, FunctionToolCall,
};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone)]
pub enum FileSystemToolError<'a> {
    InvalidArguments(&'a str),
    FileWriteFailed(&'a str),
}

impl<'a> FileSystemToolError<'a> {
    fn as_json_string(&self) -> String {
        let error = match self {
            Self::InvalidArguments(s) => format!("InvalidArguments: {s}"),
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

// impl WriteFileContentArgs {
//     fn extract_path() -> &str {
//
//     }
// }

impl super::Tool for FileSystemTool {
    fn name(&self) -> &str {
        "FileSystem"
    }

    fn function_names(&self) -> &[&str] {
        &["write_file_content"]
    }

    fn register(&self, mut builder: ChatCompletionBuilder) -> ChatCompletionBuilder {
        let write_file_content = FunctionDefinition {
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
        };
        builder = builder.tool(ChatCompletionTool::function(write_file_content));
        builder
    }

    fn call(&mut self, tc: &ChatCompletionMessageToolCall) -> Option<ChatCompletionMessageParam> {
        match tc {
            ChatCompletionMessageToolCall::Function { id, function } => {
                self.call_function(id, function)
            }
            ChatCompletionMessageToolCall::Custom { .. } => unreachable!(),
        }
    }
}

pub struct FileSystemTool;

impl FileSystemTool {
    pub fn new() -> Self {
        Self
    }

    fn call_function(
        &self,
        tool_call_id: &str,
        fc: &FunctionToolCall,
    ) -> Option<ChatCompletionMessageParam> {
        if fc.name != "write_file_content" {
            return None;
        }
        let content = match serde_json::from_str::<WriteFileContentArgs>(&fc.arguments) {
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
                match fs::write(path, args.content) {
                    Ok(_) => String::new(),
                    Err(e) => FileSystemToolError::FileWriteFailed(&e.to_string()).as_json_string(),
                }
            }
            Err(e) => FileSystemToolError::InvalidArguments(&e.to_string()).as_json_string(),
        };

        Some(ChatCompletionMessageParam::tool(content, tool_call_id))
    }
}
