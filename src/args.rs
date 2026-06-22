use std::path::PathBuf;

use llm_provider_models::enums::ReasoningEffort;

#[derive(Clone, Debug, Default)]
pub struct Args {
    pub model: Option<String>,
    pub prompt: Vec<String>,
    pub temperature: Option<String>,
    pub max_tokens: Option<String>,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub system: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub interactive: bool,
}

const VERSION: &str = "tars 0.1.0 [https://github.com/nlkli/tars]";
const HELP: &str = r#"
"#;

impl Args {
    pub fn parse() -> Self {
        let mut args = Self::default();

        let input = std::env::args();

        let mut last = None;
        for i in input.skip(1) {
            if i.starts_with("--") {
                let key = i.trim_start_matches("--");

                match key {
                    "model" => {
                        last.replace('m');
                    }
                    "prompt" => {
                        last.replace('p');
                    }
                    "temperature" => {
                        last.replace('t');
                    }
                    "temp" => {
                        last.replace('t');
                    }
                    "max-tokens" => {
                        last.replace('x');
                    }
                    "reasoning-effort" => {
                        last.replace('r');
                    }
                    "re" => {
                        last.replace('r');
                    }
                    "system" => {
                        last.replace('s');
                    }
                    "working-dir" => {
                        last.replace('d');
                    }
                    "wd" => {
                        last.replace('d');
                    }
                    "interactive" => args.interactive = true,
                    "help" => {
                        println!("{}", HELP);
                        std::process::exit(0);
                    }
                    "version" => {
                        println!("{}", VERSION);
                        std::process::exit(0);
                    }
                    _ => (),
                }
            } else if i.starts_with("-") {
                let chars = i.trim_start_matches("-").chars();

                for c in chars {
                    match c {
                        'm' => {
                            last.replace('m');
                        }
                        'p' => {
                            last.replace('p');
                        }
                        't' => {
                            last.replace('t');
                        }
                        'x' => {
                            last.replace('x');
                        }
                        'r' => {
                            last.replace('r');
                        }
                        's' => {
                            last.replace('s');
                        }
                        'i' => args.interactive = true,
                        'h' => {
                            println!("{}", HELP);
                            std::process::exit(0);
                        }
                        'V' => {
                            println!("{}", VERSION);
                            std::process::exit(0);
                        }
                        _ => (),
                    }
                }
            } else {
                if let Some(c) = last {
                    match c {
                        'm' => {
                            args.model.replace(i);
                        }
                        'p' => {
                            args.prompt.push(i);
                        }
                        't' => args.temperature = i.parse::<f32>().ok().map(|v| v.to_string()),
                        'x' => args.max_tokens = i.parse::<u32>().ok().map(|v| v.to_string()),
                        'r' => {
                            if let Ok(reasoning_effort) = ReasoningEffort::try_from(i.as_str()) {
                                args.reasoning_effort.replace(reasoning_effort);
                            }
                        }
                        's' => {
                            args.system.push(i);
                        }
                        'd' => {
                            let path = PathBuf::from(i);
                            if path.is_dir() {
                                args.working_dir.replace(path);
                            }
                        }
                        _ => (),
                    }
                    last = None;
                } else {
                    let path = PathBuf::from(&i);
                    if path.is_dir() {
                        args.working_dir.replace(path);
                    } else {
                        args.prompt.push(i);
                    }
                }
            }
        }
        args
    }
}
