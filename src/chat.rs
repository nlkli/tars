use crate::{
    openai::{
        ChatCompletionValue, OpenaiClient,
        models::{
            ChatCompletion, ChatCompletionChunkChoice, ChatCompletionMessage,
            ChatCompletionMessageParam,
        },
    },
    tools::ToolManager,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

#[derive(Debug)]
pub enum ChatEvent {
    Message(ChatCompletionMessage),
    ChunkChoice(Result<ChatCompletionChunkChoice>),
}

pub struct Chat {
    client: OpenaiClient,
    completion: ChatCompletion,
    tool_manager: ToolManager,
}

pub fn spawn_chat(
    chat: Chat,
) -> (
    UnboundedSender<Vec<ChatCompletionMessageParam>>,
    UnboundedReceiver<ChatEvent>,
) {
    let chat = Arc::new(Mutex::new(chat));

    let (chat_tx, mut chat_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let (completion_tx, mut conmpletion_rx) = tokio::sync::mpsc::unbounded_channel();

    {
        let chat = chat.clone();
        let chat_tx = chat_tx.clone();
        let event_tx = event_tx.clone();

        // let tool_cal

        tokio::spawn(async move {
            while let Some(cc_value) = conmpletion_rx.recv().await {
                match cc_value {
                    ChatCompletionValue::Chunk(chunk_result) => match chunk_result {
                        Ok(mut chunk) => {
                            let first_choice = chunk.choices.first_mut();
                            if first_choice.is_none() {
                                continue;
                            }
                            let first_choice = unsafe { first_choice.unwrap_unchecked() };
                            let _ = event_tx
                                .send(ChatEvent::ChunkChoice(Result::Ok(first_choice.clone())));

                            // if let Some(tool_calls) = first_choice.delta.tool_calls.take() {
                            //
                            // }
                        }
                        Err(e) => {
                            let _ = event_tx.send(ChatEvent::ChunkChoice(Result::Err(e)));
                        }
                    },
                    ChatCompletionValue::Response(mut response) => {
                        let first_message = response.choices.first_mut().map(|c| &mut c.message);
                        if first_message.is_none() {
                            continue;
                        }
                        let first_message = unsafe { first_message.unwrap_unchecked() };
                        let _ = event_tx.send(ChatEvent::Message(first_message.clone()));

                        let mut chat_lock = chat.lock().await;
                        if let Some(tool_calls) = first_message.tool_calls.take() {
                            let tool_messages = tool_calls
                                .iter()
                                .map(|tc| chat_lock.tool_manager.call(tc))
                                .collect::<Vec<_>>();
                            chat_lock.completion.messages.push(
                                ChatCompletionMessageParam::new(
                                    &first_message.role,
                                    first_message.content_or_default(),
                                )
                                .tool_calls(tool_calls),
                            );
                            let _ = chat_tx.send(tool_messages);
                            drop(chat_lock);
                            continue;
                        }

                        chat_lock
                            .completion
                            .messages
                            .push(ChatCompletionMessageParam::new(
                                &first_message.role,
                                first_message.content_or_default(),
                            ));

                        drop(chat_lock);
                    }
                }
            }
        });
    }
    tokio::spawn(async move {
        while let Some(messages) = chat_rx.recv().await {
            if messages.is_empty() {
                continue;
            }
            let mut chat_lock = chat.lock().await;
            chat_lock.completion.messages.extend_from_slice(&messages);
            if let Err(e) = chat_lock
                .client
                .create_chat_completion(&chat_lock.completion, completion_tx.clone())
                .await
            {
                eprintln!("ERROR: create_chat_completion: {}", e);
            }

            drop(chat_lock);
        }
        println!("done");
    });
    (chat_tx, event_rx)
}

impl Chat {
    pub fn new(
        client: OpenaiClient,
        chat_completion: ChatCompletion,
        tool_manager: ToolManager,
    ) -> Self {
        Self {
            client: client,
            completion: chat_completion,
            tool_manager,
        }
    }
}
