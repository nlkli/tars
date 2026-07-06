use crate::{
    compleation::ChatCompletionEx,
    provider::{ChatCompletionOutput, ChatCompletionStream, ProviderClient},
    tui::Sgr,
};
use anyhow::Result;
use llm_provider_models::{
    ChatCompletionChunkChoice, ChatCompletionChunkChoiceDelta, ChatCompletionMessage,
    ChatCompletionMessageParam, ChatCompletionMessageToolCall, ChatCompletionResponse,
    FINISH_REASON_STOP, FINISH_REASON_TOOL_CALLS, FunctionToolCall,
};
use std::{io::Write, sync::Arc};
use tokio::sync::{
    Mutex,
    mpsc::{self, UnboundedSender},
};
use tokio_stream::StreamExt;

type MessageSender = UnboundedSender<Vec<ChatCompletionMessageParam>>;
type EventSender = UnboundedSender<Result<ChatEvent>>;

/// Events emitted by the chat loop to the caller.
#[derive(Debug)]
pub enum ChatEvent {
    /// A complete message from a non-streaming response.
    Message(ChatCompletionMessage),
    /// A single chunk choice from a streaming response.
    ChunkChoice(Result<ChatCompletionChunkChoice>),
}

/// Accumulates partial content and tool-call data across stream chunks.
#[derive(Default)]
struct MessageBuffer {
    role: Option<String>,
    content: Option<String>,
    tool_call: Option<ChatCompletionMessageToolCall>,
}

impl MessageBuffer {
    fn push_content(&mut self, content: String) {
        match self.content.as_mut() {
            Some(buf) => buf.push_str(&content),
            None => self.content = Some(content),
        }
    }

    /// Drains the buffered role and content into a message param, if a role is present.
    fn take_message_param(&mut self) -> Option<ChatCompletionMessageParam> {
        let role = self.role.take()?;
        Some(ChatCompletionMessageParam::new(
            role,
            self.content.take().unwrap_or_default(),
        ))
    }
}

/// Drives an agentic chat loop: sends completions, handles streaming, and dispatches tool calls.
pub struct Chat {
    client: ProviderClient,
    completion: ChatCompletionEx,
    buffer: MessageBuffer,
}

impl Chat {
    pub fn new(client: ProviderClient, completion: ChatCompletionEx) -> Self {
        Self {
            client,
            completion,
            buffer: MessageBuffer::default(),
        }
    }

    /// Spawns the chat loop as a background task.
    ///
    /// Send batches of messages via the returned [`MessageSender`]; the loop appends
    /// them to the conversation, calls the provider, and forwards [`ChatEvent`]s to
    /// `event_tx`. Tool call results are fed back into the loop automatically.
    pub fn spawn(chat: Arc<Mutex<Chat>>, event_tx: EventSender) -> MessageSender {
        let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<Vec<ChatCompletionMessageParam>>();

        let msg_tx_clone = msg_tx.clone();
        tokio::spawn(async move {
            while let Some(messages) = msg_rx.recv().await {
                // if messages.is_empty() {
                //     continue;
                // }

                let mut chat_lock = chat.lock().await;

                chat_lock.completion.messages.extend(messages);

                let result = chat_lock
                    .client
                    .create_chat_completion(&chat_lock.completion.chat_completion())
                    .await;

                let should_close = match result {
                    Ok(ChatCompletionOutput::Stream(stream)) => {
                        chat_lock
                            .handle_stream(stream, &event_tx, &msg_tx_clone)
                            .await
                    }
                    Ok(ChatCompletionOutput::Response(response)) => {
                        chat_lock
                            .handle_response(response, &event_tx, &msg_tx_clone)
                            .await
                    }
                    Err(e) => event_tx.send(Err(e)).is_err(),
                };

                if should_close {
                    return;
                }
            }
        });

        msg_tx
    }

    /// Processes a streaming response chunk by chunk.
    ///
    /// Returns `true` if the event or message channel has closed and the loop should exit.
    async fn handle_stream(
        &mut self,
        mut stream: ChatCompletionStream,
        event_tx: &EventSender,
        msg_tx: &MessageSender,
    ) -> bool {
        while let Some(chunk_result) = stream.next().await {
            let mut chunk = match chunk_result {
                Ok(chunk) => chunk,
                Err(e) => return event_tx.send(Err(e)).is_err(),
            };

            // Only the first choice is processed.
            let Some(choice) = chunk.choices.first_mut() else {
                continue;
            };

            if event_tx
                .send(Ok(ChatEvent::ChunkChoice(Ok(choice.clone()))))
                .is_err()
            {
                return true;
            }

            if let Some(role) = choice.delta.role.take().filter(|r| !r.is_empty()) {
                self.buffer.role = Some(role);
            }

            if let Some(content) = choice.delta.content.take() {
                self.buffer.push_content(content);
            }

            self.accumulate_tool_call_chunk(choice);

            let Some(finish_reason) = &choice.finish_reason else {
                continue;
            };

            if finish_reason == FINISH_REASON_TOOL_CALLS {
                if let Some(tool_call) = self.buffer.tool_call.take() {
                    // !TODO
                    let tool_message = self.completion.tool_call(&tool_call).unwrap();

                    if let Some(param) = self.buffer.take_message_param() {
                        self.completion
                            .messages
                            .push(param.tool_calls(vec![tool_call]));
                    }

                    if msg_tx.send(vec![tool_message]).is_err() {
                        return true;
                    }
                }
                continue;
            }

            if let Some(param) = self.buffer.take_message_param() {
                self.completion.messages.push(param);
            }
        }

        false
    }

    /// Accumulates a single streaming tool-call delta into the buffer.
    fn accumulate_tool_call_chunk(
        &mut self,
        choice: &mut llm_provider_models::ChatCompletionChunkChoice,
    ) {
        let Some(mut tool_call) = choice
            .delta
            .tool_calls
            .take()
            .and_then(|calls| calls.into_iter().next())
        else {
            return;
        };

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
            let args = tool_call
                .function
                .as_mut()
                .and_then(|f| f.arguments.take())
                .unwrap_or_default();
            function.arguments.push_str(&args);
        }
    }

    /// Processes a complete (non-streaming) response.
    ///
    /// Returns `true` if a channel has closed and the loop should exit.
    async fn handle_response(
        &mut self,
        mut response: ChatCompletionResponse,
        event_tx: &EventSender,
        msg_tx: &MessageSender,
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
            // !TODO
            let tool_messages = tool_calls
                .iter()
                .map(|tc| self.completion.tool_call(tc).unwrap())
                .collect::<Vec<_>>();

            self.completion.messages.push(
                ChatCompletionMessageParam::new(&message.role, message.content_or_default())
                    .tool_calls(tool_calls),
            );

            return msg_tx.send(tool_messages).is_err();
        }

        self.completion
            .messages
            .push(ChatCompletionMessageParam::new(
                &message.role,
                message.content_or_default(),
            ));

        false
    }
}

async fn input(chat: Arc<Mutex<Chat>>) -> Result<String> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    write!(stdout, "> ")?;
    stdout.flush()?;

    let mut prompt = String::new();

    loop {
        let mut line = String::new();
        stdin.read_line(&mut line)?;
        match line.as_str().trim_end() {
            "/clear" => {
                chat.lock().await.completion.clear();
                write!(stdout, "> ")?;
                stdout.flush()?;
            }
            "/undo" => {
                let _ = chat.lock().await.completion.undo_user_message();
                write!(stdout, "> ")?;
                stdout.flush()?;
            }
            _ => {
                prompt.push_str(line.trim_end());
                if line.trim_end() == "." {
                    write!(stdout, "\x1B[1A\x1B[2K\r")?;
                    stdout.flush()?;
                    break;
                }
                prompt.push('\n');
            }
        }
    }

    writeln!(stdout)?;

    Ok(prompt)
}

// :TODO
pub async fn run_interactive_chat(
    client: ProviderClient,
    completion: ChatCompletionEx,
) -> Result<()> {
    let chat = Arc::new(Mutex::new(Chat::new(client, completion)));

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let chat_tx = Chat::spawn(chat.clone(), event_tx);

    let chat_lock = chat.lock().await;

    if let Some(last_message) = chat_lock.completion.messages.last() {
        if !last_message.is_user() {
            let prompt = input(chat.clone()).await?;
            chat_tx.send(vec![ChatCompletionMessageParam::user(prompt)])?;
        } else {
            chat_tx.send(vec![])?;
        }
    } else {
        let prompt = input(chat.clone()).await?;
        chat_tx.send(vec![ChatCompletionMessageParam::user(prompt)])?;
    }

    drop(chat_lock);

    let mut stdout = std::io::stdout();
    let mut prev_delta = ChatCompletionChunkChoiceDelta::default();

    while let Some(event_result) = event_rx.recv().await {
        let event = match event_result {
            Ok(event) => event,
            Err(e) => {
                writeln!(
                    stdout,
                    "\n{}{}{}\n\n",
                    Sgr::Red.esc(),
                    e.to_string(),
                    Sgr::Reset.esc(),
                )?;
                stdout.flush()?;
                continue;
            }
        };
        match event {
            ChatEvent::Message(message) => {
                write!(
                    stdout,
                    "{}{}{}\n",
                    Sgr::Green.esc(),
                    message.role,
                    Sgr::Reset.esc(),
                )?;
                if let Some(ref content) = message.content {
                    write!(stdout, "{content}\n\n")?;
                }
                if let Some(ref reasoning_content) = message.reasoning_content {
                    write!(
                        stdout,
                        "{}{reasoning_content}{}\n\n",
                        Sgr::Magenta.esc(),
                        Sgr::Reset.esc()
                    )?;
                }
            }
            ChatEvent::ChunkChoice(chunk_choice_result) => {
                let chunk_choice = match chunk_choice_result {
                    Ok(chunk_choice) => chunk_choice,
                    Err(e) => {
                        writeln!(
                            stdout,
                            "\n{}{}{}\n\n",
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
                if let Some(tool_call) =
                    delta.tool_calls.as_ref().and_then(|v| v.into_iter().next())
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
                        write!(stdout, "\n\n")?;
                        let prompt = input(chat.clone()).await?;
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
