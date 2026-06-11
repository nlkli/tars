use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{
    Mutex, RwLock,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::{
    openai::{
        ChatCompletionValue, OpenaiClient, OpenaiClientBuilder,
        models::{
            ChatCompletion, ChatCompletionBuilder, ChatCompletionMessage,
            ChatCompletionMessageParam, ChatCompletionToolChoice,
        },
    },
    tools::ToolManager,
};

#[derive(Debug)]
pub enum ChatEvent {
    Message(Result<String>),
    Test(ChatCompletionMessage),
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
        tokio::spawn(async move {
            while let Some(cc_value) = conmpletion_rx.recv().await {
                match cc_value {
                    ChatCompletionValue::Chunk(_chunk) => {}
                    ChatCompletionValue::Response(mut response) => {
                        let mut chat_lock = chat.lock().await;

                        let first_message = response.choices.first_mut().map(|c| &mut c.message);

                        if first_message.is_none() {
                            continue;
                        }

                        let message = unsafe { first_message.unwrap_unchecked() };

                        // println!("{:#?}", message);
                        if let Some(tcs) = message.tool_calls.take() {
                            let message_params = tcs
                                .iter()
                                .map(|tc| chat_lock.tool_manager.call(tc))
                                .collect::<Vec<_>>();

                            let tool_calls_message = ChatCompletionMessageParam::new(
                                &message.role,
                                message.content_or_default(),
                            )
                            .tool_calls(tcs);

                            chat_lock.completion.messages.push(tool_calls_message);

                            let _ = chat_tx.send(message_params);

                            drop(chat_lock);

                            continue;
                        }

                        let _ = event_tx.send(ChatEvent::Test(message.clone()));
                        chat_lock
                            .completion
                            .messages
                            .push(ChatCompletionMessageParam::new(
                                &message.role,
                                message.content_or_default(),
                            ));

                        drop(chat_lock);
                    }
                }
            }
        });
    }
    tokio::spawn(async move {
        while let Some(mps) = chat_rx.recv().await {
            if mps.is_empty() {
                continue;
            }
            let mut chat_lock = chat.lock().await;
            chat_lock.completion.messages.extend_from_slice(&mps);
            println!("{:#?}", chat_lock.completion.messages);
            if let Err(e) = chat_lock
                .client
                .create_chat_completion(&chat_lock.completion, completion_tx.clone())
                .await
            {
                eprintln!("ERROR: create_chat_completion: {}", e);
                let _ = event_tx.send(ChatEvent::Message(Err(e.into())));
            }
            drop(chat_lock);
        }
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
