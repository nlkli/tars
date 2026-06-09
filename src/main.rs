use anyhow::Result;
mod chat;
mod llamacpp;
mod openai;
mod term;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    let terminal = term::Terminal::new("zsh")?;
    let mut tm = tools::ToolManager::new(terminal);

    // terminal.execute_many(&[
    //     "pwd".into(),
    //     "pwd".into(),
    //     "ls".into(),
    //     "mkdir ~/Desktop/temp".into(),
    // ])?;
    //
    // for e in terminal.executions.iter() {
    //     print!("{}", e.raw_input);
    //     print!("{}", e.raw_output);
    // }
    //
    // return Ok(());

    let client = openai::OpenaiClient::builder()
        .base_url("http://localhost:8080/v1")
        .build();

    // let llama_client = llamacpp::Client::builder()
    //     .base_url("http://localhost:8080/v1")
    //     .build();

    // let models = llama_client.models().await?;

    let mut builder = openai::models::ChatCompletion::builder()
        .model("gemma-4-E4B-it-ultra-uncensored-heretic-Q4_K_M");

    builder = tm.tools_registration(builder);

    let req = builder
        .tool_choice(openai::models::ChatCompletionToolChoice::auto())
        .message(openai::models::ChatCompletionMessageParam::user(
            "создай папку temp на рабочем столе",
        ))
        .build();

    let res = client.create_chat_completion(&req).await?;

    println!("{:#?}", res);

    Ok(())
}
