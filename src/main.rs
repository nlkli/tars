use anyhow::Result;
mod chat;
mod llamacpp;
mod openai;
mod term;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    let terminal = term::Terminal::new("zsh")?;
    let terminal_tool = tools::TerminalTool::new(terminal, 4096, None);
    let mut tm = tools::ToolManager::new();
    tm.add(terminal_tool);

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
    //
    // return Ok(());

    // OmniCoder-Claude-uncensored-V2-Q4_K_M
    // gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M
    // Qwopus3.5-9B-Coder-MTP-Q4_K_M

    let mut builder =
        openai::models::ChatCompletion::builder().model("gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M");

    builder = tm.register_all(builder);

    let chat_completion = builder
        .tool_choice(openai::models::ChatCompletionToolChoice::auto())
        .build();

    let c = chat::Chat::new(client, chat_completion, tm);
    let (tx, mut rx) = chat::spawn_chat(c);

    let prompt = r#"
You are working directly on my computer and have full access to the local filesystem and terminal.

Your task is to create a complete, working C project using the Raylib library. Raylib is already installed on the system.

Requirements:

1. First, detect and verify the operating system, distribution, architecture, compiler, and installed Raylib version.
2. Create a new project folder named `raylib_demo` on my Desktop.
3. Inside the project folder, generate all required source files, build scripts, and any supporting files needed for compilation and execution.
4. Implement a simple but visually appealing Raylib demo, for example:

   * a moving player object,
   * keyboard controls,
   * animated objects,
   * collision detection,
   * FPS counter,
   * basic game loop.
5. Write clean, well-structured, and fully commented C code.
6. Create a build script that automatically compiles the project with all required compiler and linker flags for the detected platform.
7. The build script must work without manual modification.
8. Create a run script that:

   * builds the project if needed,
   * launches the executable.
9. Verify that the project compiles successfully.
10. Run the executable and confirm that it starts correctly without errors.
11. If compilation fails, automatically diagnose and fix the issue until the project builds successfully.
12. At the end, provide:

    * the full path to the project folder,
    * the exact build command used,
    * the exact run command used,
    * a brief description of the demo.

Important:

* Do not ask for confirmation.
* Do not stop after generating code.
* Actually create the files and project structure on disk.
* Ensure the final project is fully functional and ready to build and run immediately.
* Prefer portable solutions that work on the detected operating system.
* If multiple compilers are available, choose the most appropriate one automatically.
* Validate all paths before creating files.
* The final result should be a complete, ready-to-run Raylib project located on my Desktop.
        "#;

    let _ = tx.send(Vec::from([
        openai::models::ChatCompletionMessageParam::user(prompt),
    ]));

    while let Some(e) = rx.recv().await {
        println!("{:#?}", e);
    }

    Ok(())
}
