use crate::{
    openai::{
        ChatCompletionValue, OpenaiClient,
        models::{
            ChatCompletion, ChatCompletionChunkChoice, ChatCompletionMessage,
            ChatCompletionMessageParam, ChatCompletionMessageToolCall, FINISH_REASON_TOOL_CALLS,
            FunctionToolCall,
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

#[derive(Default)]
struct StreamMessageBuffer {
    role: Option<String>,
    content: Option<String>,
    tool_call: Option<ChatCompletionMessageToolCall>,
}

impl StreamMessageBuffer {
    fn push_content(&mut self, content: String) {
        match self.content.as_mut() {
            Some(buf) => buf.push_str(&content),
            None => self.content = Some(content),
        }
    }

    fn take_message(&mut self) -> ChatCompletionMessageParam {
        ChatCompletionMessageParam::new(
            self.role.take().unwrap_or_default(),
            self.content.take().unwrap_or_default(),
        )
    }
}

pub fn spawn_chat(
    chat: Chat,
) -> (
    UnboundedSender<Vec<ChatCompletionMessageParam>>,
    UnboundedReceiver<ChatEvent>,
) {
    let chat = Arc::new(Mutex::new(chat));

    let (chat_tx, chat_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let (completion_tx, completion_rx) = tokio::sync::mpsc::unbounded_channel();

    spawn_completion_processor(chat.clone(), chat_tx.clone(), event_tx, completion_rx);

    spawn_chat_worker(chat, chat_rx, completion_tx);

    (chat_tx, event_rx)
}

fn spawn_completion_processor(
    chat: Arc<Mutex<Chat>>,
    chat_tx: UnboundedSender<Vec<ChatCompletionMessageParam>>,
    event_tx: UnboundedSender<ChatEvent>,
    mut completion_rx: UnboundedReceiver<ChatCompletionValue>,
) {
    tokio::spawn(async move {
        let mut buffer = StreamMessageBuffer::default();

        while let Some(value) = completion_rx.recv().await {
            match value {
                ChatCompletionValue::Chunk(chunk_result) => {
                    let mut chunk = match chunk_result {
                        Ok(chunk) => chunk,
                        Err(err) => {
                            let _ = event_tx.send(ChatEvent::ChunkChoice(Err(err)));
                            continue;
                        }
                    };

                    let Some(choice) = chunk.choices.first_mut() else {
                        continue;
                    };

                    let _ = event_tx.send(ChatEvent::ChunkChoice(Ok(choice.clone())));

                    if let Some(role) = choice.delta.role.take() {
                        if !role.is_empty() {
                            buffer.role.replace(role);
                        }
                    }

                    if let Some(content) = choice.delta.content.take() {
                        buffer.push_content(content);
                    }

                    if let Some(mut tool_call) = choice
                        .delta
                        .tool_calls
                        .take()
                        .and_then(|calls| calls.into_iter().next())
                    {
                        if let Some(id) = tool_call.id.take() {
                            buffer.tool_call = Some(ChatCompletionMessageToolCall::Function {
                                id,
                                function: FunctionToolCall {
                                    name: tool_call
                                        .function
                                        .as_mut()
                                        .and_then(|f| f.name.take())
                                        .unwrap_or_default(),
                                    ..Default::default()
                                },
                            });
                        }

                        if let Some(ChatCompletionMessageToolCall::Function { function, .. }) =
                            &mut buffer.tool_call
                        {
                            function.arguments.push_str(
                                &tool_call
                                    .function
                                    .as_mut()
                                    .and_then(|f| f.arguments.take())
                                    .unwrap_or_default(),
                            );
                        }
                    }

                    let Some(finish_reason) = &choice.finish_reason else {
                        continue;
                    };

                    let mut chat = chat.lock().await;

                    if finish_reason == FINISH_REASON_TOOL_CALLS {
                        if let Some(tool_call) = buffer.tool_call.take() {
                            let tool_message = chat.tool_manager.call(&tool_call);

                            chat.completion
                                .messages
                                .push(buffer.take_message().tool_calls(vec![tool_call]));

                            let _ = chat_tx.send(vec![tool_message]);
                        }

                        continue;
                    }

                    chat.completion.messages.push(buffer.take_message());
                }

                ChatCompletionValue::Response(mut response) => {
                    let Some(message) = response.choices.first_mut().map(|c| &mut c.message) else {
                        continue;
                    };

                    let _ = event_tx.send(ChatEvent::Message(message.clone()));

                    let mut chat = chat.lock().await;

                    if let Some(tool_calls) = message.tool_calls.take() {
                        let tool_messages = tool_calls
                            .iter()
                            .map(|tc| chat.tool_manager.call(tc))
                            .collect::<Vec<_>>();

                        chat.completion.messages.push(
                            ChatCompletionMessageParam::new(
                                &message.role,
                                message.content_or_default(),
                            )
                            .tool_calls(tool_calls),
                        );

                        let _ = chat_tx.send(tool_messages);
                        continue;
                    }

                    chat.completion
                        .messages
                        .push(ChatCompletionMessageParam::new(
                            &message.role,
                            message.content_or_default(),
                        ));
                }
            }
        }
    });
}

fn spawn_chat_worker(
    chat: Arc<Mutex<Chat>>,
    mut chat_rx: UnboundedReceiver<Vec<ChatCompletionMessageParam>>,
    completion_tx: UnboundedSender<ChatCompletionValue>,
) {
    tokio::spawn(async move {
        while let Some(messages) = chat_rx.recv().await {
            if messages.is_empty() {
                continue;
            }

            let mut chat = chat.lock().await;

            chat.completion.messages.extend(messages);

            println!("{:#?}", chat.completion.messages);

            if let Err(err) = chat
                .client
                .create_chat_completion(&chat.completion, completion_tx.clone())
                .await
            {
                eprintln!("ERROR: create_chat_completion: {}", err);
            }
        }
    });
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
