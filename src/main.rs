use anyhow::Result;
use llm_provider_models::{
    ChatCompletion, ChatCompletionChunkChoiceDelta, ChatCompletionMessageParam,
    ChatCompletionToolChoice, FINISH_REASON_STOP,
};
mod chat;
mod term;
mod tools;

mod provider;

use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // let mut terminal = term::Terminal::spawn("zsh".into(), |_| {}, None)?;

    // let mut e1 = terminal.execute(r#"curl -s -X GET "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd""#, Some(Duration::from_secs(7)))?;
    // let mut e2 = terminal.execute("ls", None)?;
    // let mut e3 = terminal.execute("ls", None)?;

    // println!("plan = {:#?}", e1);
    // println!("plan = {:#?}", e1.plain_output());
    // println!("plan = {:#?}", e2.plain_output());
    // println!("plan = {:#?}", e3.plain_output());
    //
    // terminal.execute("cd Projects/tars/src", None)?;
    //
    // println!("pwd = {:#?}", terminal.pwd()?);
    //
    // let mut e = terminal.execute("cat main.rs", None)?;
    //
    // println!("ex = {:#?}", e);
    // println!("plan = {:#?}", e.plain_output());
    //
    // println!("pwd = {:#?}", terminal.pwd()?);
    //
    // return Ok(());

    let terminal = term::Terminal::spawn("zsh".into(), |_| {}, None)?;
    let terminal_tool = tools::TerminalTool::new(terminal, 2048, None, None);
    let fs_tool = tools::FileSystemTool::new();
    let mut tm = tools::ToolManager::new();
    tm.add(terminal_tool);
    tm.add(fs_tool);

    let client = provider::Client::builder()
        .base_url("http://localhost:8080/v1")
        .build();

    // let llama_client = llamacpp::Client::builder()
    //     .base_url("http://localhost:8080/v1")
    //     .build();
    //
    // let models = llama_client.models().await?;
    //
    // println!("{:?}", models);

    // OmniCoder-Claude-uncensored-V2-Q4_K_M
    // gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M
    // Qwopus3.5-9B-Coder-MTP-Q4_K_M

    let mut builder =
        ChatCompletion::builder().model("gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M");

    builder = tm.register_all(builder);

    let chat_completion = builder
        .stream()
        .tool_choice(ChatCompletionToolChoice::auto())
        .build();

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let chat_tx = chat::Chat::new(client, chat_completion, tm).spawn(event_tx);

    //     let prompt = r#"
    // Напиши самую простую демку используя язык c и библиотеку raylib. Демка должна быть интересная и с
    // движением. Проект демки должен находится на
    // рабочем столе ~/Desktop/...  Создай папку проекта. raylib установлен через brew. создай build.sh скрипт для
    // сборки. Не забудь перейти в директорию проекта для сборки. Используй верный путь для линковки raylib.
    // # Компиляция и линковка для macOS (Apple Silicon/arm64).
    // # Добавляем пути к заголовочным файлам и библиотекам Homebrew
    // clang -I/opt/homebrew/include -L/opt/homebrew/lib main.c -o $OUTPUT_NAME -lraylib -framework OpenGL -framework Cocoa -framework CoreVideo -framework IOKit
    // Сделай скрипт сборки исполняемым и запусти его. Код и все комментарии должны быть на английском.
    // Все должно работать исправно! Запусти программу после готовности.
    //         "#;

    let stdin = std::io::stdin();

    let mut stdout = std::io::stdout();

    let _ = write!(stdout, ": ");
    let _ = stdout.flush();

    let mut prompt = String::new();
    stdin.read_line(&mut prompt)?;
    let _ = chat_tx.send(Vec::from([ChatCompletionMessageParam::user(prompt)]));

    let mut prev_delta = ChatCompletionChunkChoiceDelta::default();

    while let Some(event_result) = event_rx.recv().await {
        let Ok(event) = event_result else {
            continue;
        };
        match event {
            chat::ChatEvent::Message(_message) => todo!(),
            chat::ChatEvent::ChunkChoice(chunk_choice_result) => {
                // Skip invalid chunks.
                let Ok(chunk_choice) = chunk_choice_result else {
                    continue;
                };

                let mut delta = chunk_choice.delta;

                if delta.reasoning_content.is_some() && prev_delta.reasoning_content.is_none() {
                    let _ = stdout.write_all(b"\x1b[34m");
                }
                if delta.reasoning_content.is_none() && prev_delta.reasoning_content.is_some() {
                    let _ = stdout.write_all(b"\x1b[0m");
                    let _ = stdout.write_all(b"\n\n");
                }

                if delta.tool_calls.is_some() && prev_delta.tool_calls.is_none() {
                    let _ = stdout.write_all(b"\x1b[35m");
                }
                if delta.tool_calls.is_none() && prev_delta.tool_calls.is_some() {
                    let _ = stdout.write_all(b"\x1b[0m");
                    let _ = stdout.write_all(b"\n");
                }

                // Print role if present.
                if let Some(role) = &delta.role {
                    let _ = writeln!(stdout, "\n\x1b[32m{role}:\x1b[0m");
                }

                // Print assistant content.
                if let Some(content) = &delta.content {
                    let _ = stdout.write_all(content.as_bytes());
                }

                // Print reasoning content.
                if let Some(reasoning) = &delta.reasoning_content {
                    let _ = stdout.write_all(reasoning.as_bytes());
                }

                // Print tool name when a new tool call starts.
                if let Some(tool_call) = delta.tool_calls.take().and_then(|v| v.into_iter().next())
                {
                    if tool_call.id.is_some() {
                        if let Some(function) = &tool_call.function {
                            if let Some(name) = &function.name {
                                let _ = writeln!(stdout, "{name}");
                            }
                        }
                    }

                    // Stream tool arguments.
                    if let Some(function) = &tool_call.function {
                        if let Some(args) = &function.arguments {
                            let _ = stdout.write_all(args.as_bytes());
                        }
                    }
                }

                // Print separator when generation is finished.
                if let Some(ref finish_reason) = chunk_choice.finish_reason {
                    let _ = stdout.write_all(b"\n-----\n");

                    if finish_reason == FINISH_REASON_STOP {
                        let _ = write!(stdout, ": ");
                        let _ = stdout.flush();
                        let mut prompt = String::new();
                        stdin.read_line(&mut prompt)?;
                        let _ = chat_tx.send(Vec::from([ChatCompletionMessageParam::user(prompt)]));
                        let _ = stdout.write_all(b"\n");
                    }
                }

                let _ = stdout.flush();
                prev_delta = delta;
            }
        }
    }

    Ok(())
}
