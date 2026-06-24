use crate::{compleation::ChatCompletionEx, fuzzy, provider::ProviderClient, term::Terminal};
use anyhow::Result;
use llm_provider_models::enums::ReasoningEffort;
use std::path::PathBuf;

/// CLI arguments parsed from environment variables and command-line input.
#[derive(Clone, Debug, Default)]
pub struct Args {
    /// API base URL (overrides `TARS_BASE_URL`)
    pub base_url: Option<String>,

    /// API key (overrides `TARS_API_KEY`)
    pub api_key: Option<String>,

    /// Model name or fuzzy pattern (overrides `TARS_MODEL`)
    pub model: Option<String>,

    /// User prompt segments joined into a single message
    pub prompt: Vec<String>,

    /// Sampling temperature (0.0–2.0)
    pub temperature: Option<String>,

    /// Maximum tokens to generate
    pub max_tokens: Option<String>,

    /// Reasoning effort level for supported models
    pub reasoning_effort: Option<ReasoningEffort>,

    /// System prompt segments
    pub system: Vec<String>,

    /// Working directory for the spawned terminal
    pub working_dir: Option<PathBuf>,

    /// Keep the session alive after the first response
    pub interactive: bool,
}

const VERSION: &str = "tars 0.1.0 [https://github.com/nlkli/tars]";
const HELP: &str = "\
Usage: tars [OPTIONS] [PROMPT...] [DIR]

Arguments:
  [PROMPT...]     One or more prompt strings (concatenated)
  [DIR]           Working directory for the terminal session

Options:
  -m, --model <MODEL>             Model name or fuzzy pattern  [env: TARS_MODEL]
  -p, --prompt <TEXT|PATH>        Append a prompt segment (repeatable)
  -s, --system <TEXT|PATH>        Append a system prompt segment (repeatable)
  -t, --temp, --temperature <F>   Sampling temperature (e.g. 0.7)
  -x, --max-tokens <N>            Maximum tokens to generate
  --re, --reasoning-effort <LEVEL>
                                  Reasoning effort: low | medium | high
  --wd, --working-dir <DIR>       Override the working directory
      --base-url, --bu <URL>      API base URL  [env: TARS_BASE_URL]
      --api-key, --ak <KEY>       API key       [env: TARS_API_KEY]
  -i, --interactive               Stay interactive after the first response
  -h, --help                      Print this help message
  -V, --version                   Print version information

Environment variables:
  TARS_BASE_URL   Default API base URL
  TARS_API_KEY    Default API key
  TARS_MODEL      Default model name
  SHELL           Shell used for the terminal session (default: bash)

Examples:
  tars 'Explain Rust lifetimes'
  tars -m gpt-4o -t 0.5 'Write a haiku about Ferris'
  tars --interactive ./my_project
";

impl Args {
    /// Parse CLI arguments and environment variables into [`Args`].
    ///
    /// Environment variables (`TARS_BASE_URL`, `TARS_API_KEY`, `TARS_MODEL`) are
    /// read first; explicit flags override them.
    pub fn parse() -> Self {
        let mut args = Self::default();

        // Seed from environment variables; flags may override below.
        args.base_url = std::env::var("TARS_BASE_URL").ok();
        args.api_key = std::env::var("TARS_API_KEY").ok();
        args.model = std::env::var("TARS_MODEL").ok();

        let mut last: Option<char> = None;

        for token in std::env::args().skip(1) {
            if let Some(key) = token.strip_prefix("--") {
                // Long flag
                match key {
                    "base-url" | "bu" => {
                        last = Some('u');
                    }
                    "api-key" | "ak" => {
                        last = Some('k');
                    }
                    "model" => {
                        last = Some('m');
                    }
                    "prompt" => {
                        last = Some('p');
                    }
                    "temperature" | "temp" => {
                        last = Some('t');
                    }
                    "max-tokens" => {
                        last = Some('x');
                    }
                    "reasoning-effort" | "re" => {
                        last = Some('r');
                    }
                    "system" => {
                        last = Some('s');
                    }
                    "working-dir" | "wd" => {
                        last = Some('d');
                    }
                    "interactive" => {
                        args.interactive = true;
                    }
                    "help" => {
                        print!("{}", HELP);
                        std::process::exit(0);
                    }
                    "version" => {
                        println!("{}", VERSION);
                        std::process::exit(0);
                    }
                    _ => {}
                }
            } else if let Some(flags) = token.strip_prefix('-') {
                // Short flags (may be combined, e.g. `-im`)
                for c in flags.chars() {
                    match c {
                        'm' => {
                            last = Some('m');
                        }
                        'p' => {
                            last = Some('p');
                        }
                        't' => {
                            last = Some('t');
                        }
                        'x' => {
                            last = Some('x');
                        }
                        's' => {
                            last = Some('s');
                        }
                        'i' => {
                            args.interactive = true;
                        }
                        'h' => {
                            print!("{}", HELP);
                            std::process::exit(0);
                        }
                        'V' => {
                            println!("{}", VERSION);
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
            } else {
                // Value token — either consumed by a pending flag or treated as a
                // positional argument (prompt text or working directory).
                if let Some(c) = last.take() {
                    match c {
                        'u' => {
                            args.base_url = Some(token);
                        }
                        'k' => {
                            args.api_key = Some(token);
                        }
                        'm' => {
                            args.model = Some(token);
                        }
                        'p' => {
                            args.prompt.push(token);
                        }
                        's' => {
                            args.system.push(token);
                        }
                        't' => {
                            args.temperature = token.parse::<f32>().ok().map(|v| v.to_string());
                        }
                        'x' => {
                            args.max_tokens = token.parse::<u32>().ok().map(|v| v.to_string());
                        }
                        'r' => {
                            if let Ok(effort) = ReasoningEffort::try_from(token.as_str()) {
                                args.reasoning_effort = Some(effort);
                            }
                        }
                        'd' => {
                            let path = PathBuf::from(token);
                            if path.is_dir() {
                                args.working_dir = Some(path);
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Bare positional: directory or prompt text.
                    let path = PathBuf::from(&token);
                    if let Ok(path) = path.canonicalize() {
                        args.working_dir = Some(path);
                    } else {
                        args.prompt.push(token);
                    }
                }
            }
        }

        args
    }

    /// Build a [`ProviderClient`] from the current configuration.
    ///
    /// Returns an error if `base_url` is absent or empty.
    pub fn provider_client(&self) -> Result<ProviderClient> {
        let base_url = self
            .base_url
            .as_deref()
            .filter(|u| !u.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("--base-url <URL> is required (or set TARS_BASE_URL)")
            })?;

        let mut builder = ProviderClient::builder().base_url(base_url);
        if let Some(ref key) = self.api_key {
            builder = builder.api_key(key);
        }
        Ok(builder.build())
    }

    /// Build a fully configured [`ChatCompletionEx`], including a live terminal session.
    ///
    /// This method fetches the available model list from the provider and uses fuzzy
    /// matching to resolve the requested model name.
    pub async fn chat_completion_ex(&self) -> Result<ChatCompletionEx> {
        let client = self.provider_client()?;

        // Resolve model name via fuzzy search against the provider's model list.
        let models_response = client.models().await?;
        let model_ids: Vec<&str> = models_response.data.iter().map(|m| m.id.as_str()).collect();

        let mut builder = ChatCompletionEx::builder();

        if let Some(ref pattern) = self.model {
            if let Some(matched) = fuzzy::search(pattern, &model_ids) {
                builder = builder.model(matched);
            }
        }

        if let Some(temp) = self.temperature.as_ref().and_then(|t| t.parse().ok()) {
            builder = builder.temperature(temp);
        }
        if let Some(max) = self.max_tokens.as_ref().and_then(|t| t.parse().ok()) {
            builder = builder.max_tokens(max);
        }
        if let Some(effort) = self.reasoning_effort {
            builder = builder.reasoning_effort(effort);
        }
        for s in &self.system {
            let path = PathBuf::from(s);
            if path.is_file() {
                let content = tokio::fs::read_to_string(path).await?;
                builder = builder.system(content);
            } else {
                builder = builder.system(s);
            }
        }
        for p in &self.prompt {
            let path = PathBuf::from(p);
            if path.is_file() {
                let content = tokio::fs::read_to_string(path).await?;
                builder = builder.prompt(content);
            } else {
                builder = builder.prompt(p);
            }
        }

        // Spawn a shell session and optionally change to the requested directory.
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".into());
        let mut terminal = Terminal::spawn(shell, |_| {}, None)?;
        if let Some(wd) = self.working_dir.as_deref().and_then(|p| p.to_str()) {
            let _ = terminal.execute(&format!("cd {wd}"), None)?;
        }

        builder = builder
            .stream()
            .with_system_tool(terminal, 2048)
            .tool_choice(llm_provider_models::ToolChoice::auto());

        builder.build()
    }
}
