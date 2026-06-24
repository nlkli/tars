use anyhow::Result;
use llm_provider_models::{ChatCompletionChunkChoiceDelta, FINISH_REASON_STOP};
use std::sync::Arc;
use tokio::sync::Mutex;
mod chat;
mod cli;
mod compleation;
mod fuzzy;
mod provider;
mod term;
mod tools;
mod tui;

use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    let a = cli::Args::parse();

    let chat_completion_ex = a.chat_completion_ex().await?;
    let client = a.provider_client()?;

    if a.interactive {
        return chat::run_interactive_chat(client, chat_completion_ex).await;
    }

    if !a.prompt.is_empty() {
        let c = chat::Chat::new(client, chat_completion_ex);
        let c = Arc::new(Mutex::new(c));
        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
        let chat_tx = chat::Chat::spawn(c, event_tx);
        chat_tx.send(vec![])?;

        let mut stdout = std::io::stdout();
        let mut prev_delta = ChatCompletionChunkChoiceDelta::default();

        while let Some(event_result) = event_rx.recv().await {
            let event = event_result?;
            match event {
                chat::ChatEvent::Message(_message) => todo!(),
                chat::ChatEvent::ChunkChoice(chunk_choice_result) => {
                    let chunk_choice = chunk_choice_result?;
                    let delta = chunk_choice.delta;

                    if delta.content.is_none() && prev_delta.content.is_some() {
                        writeln!(stdout)?;
                    }

                    if let Some(ref content) = delta.content {
                        write!(stdout, "{content}")?;
                    }

                    if let Some(ref finish_reason) = chunk_choice.finish_reason {
                        if finish_reason == FINISH_REASON_STOP {
                            break;
                        }
                    }

                    prev_delta = delta;
                }
            }
        }
    }

    Ok(())
}
