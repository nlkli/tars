use anyhow::Result;
// use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
mod llama_server;
mod openai;

#[tokio::main]
async fn main() -> Result<()> {
    // let pty_system = NativePtySystem::default();
    //
    // let pair = pty_system
    //     .openpty(PtySize {
    //         rows: 24,
    //         cols: 80,
    //         pixel_width: 0,
    //         pixel_height: 0,
    //     })
    //     .unwrap();
    //
    // let cmd = CommandBuilder::new("zsh");
    // let mut child = pair.slave.spawn_command(cmd)?;
    //
    // drop(pair.slave);
    //
    // let mut reader = pair.master.try_clone_reader()?;
    // let mut master_writer = pair.master.take_writer()?;
    //
    // let mut buffer = [0u8; 1024];
    // loop {
    //     let mut input = String::new();
    //     std::io::stdin().read_line(&mut input).unwrap();
    //
    //     if input.trim() == "exit" {
    //         break;
    //     }
    //
    //     if master_writer.write_all(input.as_bytes()).is_err() {
    //         eprintln!("Error writing to PTY");
    //         break;
    //     }
    //
    //     match reader.read(&mut buffer) {
    //         Ok(0) => break,
    //         Ok(n) => {
    //             let output = String::from_utf8_lossy(&buffer[..n]);
    //             println!("{}", output);
    //         }
    //         Err(e) => {
    //             eprintln!("Error reading from PTY: {}", e);
    //             break;
    //         }
    //     }
    // }
    //
    // println!("Waiting for Bash to exit...");
    // let status = child.wait().unwrap();
    // println!("Bash exited with status: {:?}", status);

    let client = openai::OpenaiClient::builder()
        .base_url("http://localhost:8080/v1")
        .build();

    // OmniCoder-Claude-uncensored-V2-Q4_K_M
    // Qwopus3.5-9B-Coder-MTP-Q4_K_M
    // gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M

    let req = openai::ChatCompletionRequest::builder()
        .model("gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M")
        .message(openai::ChatCompletionMessageParam::user("hello"))
        .build();

    let res = client.create_chat_completion(&req).await?;

    println!("{:#?}", res.choices);

    Ok(())
}
