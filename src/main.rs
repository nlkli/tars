use anyhow::Result;
mod chat;
mod llamacpp;
mod openai;
mod term;
mod tools;

use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // let mut terminal = term::Terminal::spawn("zsh", |_| {}, None)?;
    // terminal.execute("cd Projects/tars/src", None)?;
    // let mut e = terminal.execute("cat main.rs", None)?;
    //
    // println!("ex = {:#?}", e);
    // println!("plan = {:#?}", e.plain_output());
    //
    // return Ok(());

    let terminal = term::Terminal::spawn("zsh", |_| {}, None)?;
    let terminal_tool = tools::TerminalTool::new(terminal, 4096, None, None);
    let fs_tool = tools::FileSystemTool::new();
    let mut tm = tools::ToolManager::new();
    tm.add(terminal_tool);
    tm.add(fs_tool);

    let client = openai::OpenaiClient::builder()
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

    let mut builder = openai::models::ChatCompletion::builder()
        .model("gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M");

    builder = tm.register_all(builder);

    let chat_completion = builder
        .stream()
        .tool_choice(openai::models::ChatCompletionToolChoice::auto())
        .build();

    let c = chat::Chat::new(client, chat_completion, tm);
    let (tx, mut rx) = chat::spawn_chat(c);

    let prompt = r#"
Напиши самую простую демку используя язык c и библиотеку raylib. Демка должна быть интересная и с
движением. Проект демки должен находится на
рабочем столе ~/Desktop/...  Создай папку проекта. raylib установлен через brew. создай build.sh скрипт для
сборки. Используй верный путь для линковки raylib. 
# Компиляция и линковка для macOS (Apple Silicon/arm64).
# Добавляем пути к заголовочным файлам и библиотекам Homebrew
clang -I/opt/homebrew/include -L/opt/homebrew/lib main.c -o $OUTPUT_NAME -lraylib -framework OpenGL -framework Cocoa -framework CoreVideo -framework IOKit
Сделай скрипт сборки исполняемым и запусти его. Код и все комментарии должны быть на английском.
Все должно работать исправно!
        "#;

    let _ = tx.send(Vec::from([
        openai::models::ChatCompletionMessageParam::user(prompt),
    ]));

    let mut log_file = std::fs::File::create(".log")?;
    while let Some(e) = rx.recv().await {
        // println!("{:#?}", e);
        writeln!(log_file, "{:#?}", e)?;
    }

    Ok(())
}
