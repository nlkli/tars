use anyhow::Result;
use llm_provider_models::{
    ChatCompletionChunkChoiceDelta, ChatCompletionMessageParam, FINISH_REASON_STOP,
};
mod app;
mod args;
mod chat;
mod compleation;
mod fuzzy;
mod provider;
mod term;
mod tools;

mod tui;

use std::io::{Read, Write};

use crate::tui::Sgr;

// OmniCoder-Claude-uncensored-V2-Q4_K_M
// gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M
// Qwopus3.5-9B-Coder-MTP-Q4_K_M

fn input() -> Result<String> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    write!(stdout, "\n\n: ")?;
    stdout.flush()?;

    let mut prompt = String::new();

    loop {
        let mut line = String::new();
        stdin.read_line(&mut line)?;
        prompt.push_str(line.trim_end());
        if line.trim_end() == "." {
            break;
        }
        prompt.push('\n');
    }

    writeln!(stdout)?;

    Ok(prompt)
}

#[tokio::main]
async fn main() -> Result<()> {
    let t = term::Terminal::spawn("zsh".into(), |_| {}, None)?;

    let chat_completion_ex = compleation::ChatCompletionEx::builder()
        .with_system_tool(t, 2048)
        .model("gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M")
        .tool_choice(llm_provider_models::ToolChoice::auto())
        .stream()
        .build()?;

    let client = provider::Client::builder()
        .base_url("http://localhost:8080/v1")
        .build();

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let chat_tx = chat::Chat::new(client, chat_completion_ex).spawn(event_tx);

    let mut stdout = std::io::stdout();

    let prompt = input()?;
    let _ = chat_tx.send(Vec::from([ChatCompletionMessageParam::user(prompt)]));

    let mut prev_delta = ChatCompletionChunkChoiceDelta::default();

    while let Some(event_result) = event_rx.recv().await {
        let Ok(event) = event_result else {
            continue;
        };
        match event {
            chat::ChatEvent::Message(_message) => todo!(),
            chat::ChatEvent::ChunkChoice(chunk_choice_result) => {
                let chunk_choice = match chunk_choice_result {
                    Ok(chunk_choice) => chunk_choice,
                    Err(e) => {
                        writeln!(
                            stdout,
                            "{}{}{}\n",
                            Sgr::Red.esc(),
                            e.to_string(),
                            Sgr::Reset.esc(),
                        )?;
                        stdout.flush()?;
                        continue;
                    }
                };

                let delta = chunk_choice.delta;

                if delta.reasoning_content.is_some() && prev_delta.reasoning_content.is_none() {
                    write!(stdout, "{}", Sgr::Blue.esc())?;
                }
                if delta.reasoning_content.is_none() && prev_delta.reasoning_content.is_some() {
                    write!(stdout, "{}\n\n", Sgr::Reset.esc())?;
                }

                if delta.tool_calls.is_some() && prev_delta.tool_calls.is_none() {
                    write!(stdout, "{}", Sgr::Magenta.esc())?;
                }
                if delta.tool_calls.is_none() && prev_delta.tool_calls.is_some() {
                    write!(stdout, "{}\n\n", Sgr::Reset.esc())?;
                }

                // Print role if present.
                if let Some(role) = &delta.role {
                    writeln!(stdout, "{}{role}{}", Sgr::Green.esc(), Sgr::Reset.esc())?;
                }

                // Print assistant content.
                if let Some(content) = &delta.content {
                    write!(stdout, "{content}")?;
                }

                // Print reasoning content.
                if let Some(reasoning_content) = &delta.reasoning_content {
                    write!(stdout, "{reasoning_content}")?;
                }

                // Print tool name when a new tool call starts.
                if let Some(tool_call) = delta.tool_calls.as_ref().and_then(|v| v.into_iter().next())
                {
                    if tool_call.id.is_some() {
                        if let Some(name) =
                            tool_call.function.as_ref().and_then(|f| f.name.as_ref())
                        {
                            writeln!(stdout, "{name}")?;
                        }
                    }

                    // Stream tool arguments.
                    if let Some(args) = tool_call
                        .function
                        .as_ref()
                        .and_then(|f| f.arguments.as_ref())
                    {
                        write!(stdout, "{args}")?;
                    }
                }

                // Print separator when generation is finished.
                if let Some(ref finish_reason) = chunk_choice.finish_reason {
                    if finish_reason == FINISH_REASON_STOP {
                        let prompt = input()?;
                        if chat_tx
                            .send(Vec::from([ChatCompletionMessageParam::user(prompt)]))
                            .is_err()
                        {
                            break;
                        }
                    }
                }

                stdout.flush()?;
                prev_delta = delta;
            }
        }
    }

    Ok(())
}
