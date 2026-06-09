use crate::{
    openai::{
        OpenaiClient,
        models::{ChatCompletion, ChatCompletionToolChoice},
    },
    tools::ToolManager,
};

pub struct Chat {
    pub client: OpenaiClient,
    pub chat_completion: ChatCompletion,
    pub tool_manager: ToolManager,
}

impl Chat {
    pub fn new(
        client: OpenaiClient,
        chat_completion: ChatCompletion,
        tool_manager: ToolManager,
    ) -> Self {
        Self {
            client,
            tool_manager,
            chat_completion,
        }
    }

    pub fn run_interactive_mode(&mut self) {

    }
}
