use crate::{
    provider::{ChatCompletionOutput, ChatCompletionStream, ProviderClient},
    tools::ToolManager,
};
use anyhow::Result;
use llm_provider_models::{
    ChatCompletion, ChatCompletionChunkChoice, ChatCompletionMessage, ChatCompletionMessageParam,
    ChatCompletionMessageToolCall, ChatCompletionResponse, FINISH_REASON_TOOL_CALLS,
    FunctionToolCall,
};
// use std::error::Error;
// use std::fmt;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::StreamExt;

type ChatSender = UnboundedSender<Vec<ChatCompletionMessageParam>>;
type ChatEventSender = UnboundedSender<Result<ChatEvent>>;

// type ChatReceiver = UnboundedReceiver<Vec<ChatCompletionMessageParam>>;
// type ChatEventReceiver = UnboundedReceiver<Result<ChatEvent>>;

// #[derive(Debug)]
// enum ChatEventError {
//     CreateChatCompletion(String),
// }
//
// impl fmt::Display for ChatEventError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::CreateChatCompletion(s) => write!(f, "CreateChatCompletionError: {s}"),
//         }
//     }
// }
//
// impl Error for ChatEventError {}

#[derive(Debug)]
pub enum ChatEvent {
    Message(ChatCompletionMessage),
    ChunkChoice(Result<ChatCompletionChunkChoice>),
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

    fn take_message_param(&mut self) -> Option<ChatCompletionMessageParam> {
        if let Some(role) = self.role.take() {
            return Some(ChatCompletionMessageParam::new(
                role,
                self.content.take().unwrap_or_default(),
            ));
        }
        None
    }
}

pub struct Chat {
    client: ProviderClient,
    completion: ChatCompletion,
    tool_manager: ToolManager,
    buffer: StreamMessageBuffer,
}

impl Chat {
    pub fn new(
        client: ProviderClient,
        chat_completion: ChatCompletion,
        tool_manager: ToolManager,
    ) -> Self {
        Self {
            client: client,
            completion: chat_completion,
            tool_manager,
            buffer: StreamMessageBuffer::default(),
        }
    }

    pub fn spawn(mut self, event_tx: ChatEventSender) -> ChatSender {
        let (chat_tx, mut chat_rx) = mpsc::unbounded_channel::<Vec<ChatCompletionMessageParam>>();

        {
            let chat_tx = chat_tx.clone();

            tokio::spawn(async move {
                while let Some(messages) = chat_rx.recv().await {
                    if messages.is_empty() {
                        continue;
                    }

                    self.completion.messages.extend(messages);

                    match self.client.create_chat_completion(&self.completion).await {
                        Ok(output) => {
                            if match output {
                                ChatCompletionOutput::Stream(stream) => {
                                    self.stream_handle(stream, &event_tx, &chat_tx).await
                                }
                                ChatCompletionOutput::Response(response) => {
                                    self.response_handle(response, &event_tx, &chat_tx).await
                                }
                            } {
                                return;
                            }
                        }
                        Err(e) => {
                            if event_tx.send(Err(e.into())).is_err() {
                                return;
                            }
                        }
                    };
                }
            });
        }

        chat_tx
    }

    async fn stream_handle(
        &mut self,
        mut stream: ChatCompletionStream,
        event_tx: &ChatEventSender,
        chat_tx: &ChatSender,
    ) -> bool {
        while let Some(chunk_result) = stream.next().await {
            let mut chunk = match chunk_result {
                Ok(chunk) => chunk,
                Err(e) => return event_tx.send(Err(e.into())).is_err(),
            };

            // only first choice prosessing
            // for choice in chunk.choices {}

            let Some(choice) = chunk.choices.first_mut() else {
                continue;
            };

            if event_tx
                .send(Ok(ChatEvent::ChunkChoice(Ok(choice.clone()))))
                .is_err()
            {
                return true;
            }

            if let Some(role) = choice.delta.role.take() {
                if !role.is_empty() {
                    self.buffer.role.replace(role);
                }
            }

            if let Some(content) = choice.delta.content.take() {
                self.buffer.push_content(content);
            }

            if let Some(mut tool_call) = choice
                .delta
                .tool_calls
                .take()
                .and_then(|calls| calls.into_iter().next())
            {
                if let Some(id) = tool_call.id.take() {
                    self.buffer.tool_call = Some(ChatCompletionMessageToolCall::Function {
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
                    &mut self.buffer.tool_call
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

            if finish_reason == FINISH_REASON_TOOL_CALLS {
                if let Some(tool_call) = self.buffer.tool_call.take() {
                    let tool_message = self.tool_manager.call(&tool_call);

                    self.update_tools_context_system_message();

                    if let Some(message_param) = self.buffer.take_message_param() {
                        self.completion
                            .messages
                            .push(message_param.tool_calls(vec![tool_call]));
                    }

                    if chat_tx.send(vec![tool_message]).is_err() {
                        return true;
                    }
                }

                continue;
            }

            if let Some(message_param) = self.buffer.take_message_param() {
                self.completion.messages.push(message_param);
            }
        }

        false
    }

    async fn response_handle(
        &mut self,
        mut response: ChatCompletionResponse,
        event_tx: &ChatEventSender,
        chat_tx: &ChatSender,
    ) -> bool {
        let Some(message) = response.choices.first_mut().map(|c| &mut c.message) else {
            return false;
        };

        if event_tx
            .send(Ok(ChatEvent::Message(message.clone())))
            .is_err()
        {
            return true;
        }

        if let Some(tool_calls) = message.tool_calls.take() {
            let tool_messages = tool_calls
                .iter()
                .map(|tc| self.tool_manager.call(tc))
                .collect::<Vec<_>>();

            self.update_tools_context_system_message();

            self.completion.messages.push(
                ChatCompletionMessageParam::new(&message.role, message.content_or_default())
                    .tool_calls(tool_calls),
            );

            return chat_tx.send(tool_messages).is_err();
        }

        self.completion
            .messages
            .push(ChatCompletionMessageParam::new(
                &message.role,
                message.content_or_default(),
            ));

        false
    }

    fn update_tools_context_system_message(&mut self) {
        let mut tools_context_system_message_content = String::new();

        let _ = self
            .tool_manager
            .write_context(&mut tools_context_system_message_content);

        if let Some(tool_context_system_message) =
            self.completion.messages.iter_mut().rfind(|m| m.is_system())
        {
            if let Some(content) = tool_context_system_message.as_mut_text_content() {
                *content = tools_context_system_message_content;
            }
        }
    }
}
