use anyhow::{Result, bail};
use async_stream::stream;
use llm_provider_models::{
    ChatCompletion, ChatCompletionChunk, ChatCompletionResponse, ListModelsResponse, llamacpp,
};
use reqwest::{Client as HttpClient, RequestBuilder, Response};
use tokio_stream::{Stream, StreamExt};

pub type ChatCompletionStream =
    std::pin::Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>;

/// Response from `create_chat_completion`, either a complete response or a streaming one.
pub enum ChatCompletionOutput {
    Stream(ChatCompletionStream),
    Response(ChatCompletionResponse),
}

/// Builder for [`Client`].
#[derive(Debug, Clone, Default)]
pub struct ClientBuilder {
    base_url: String,
    api_key: Option<String>,
    http_client: Option<HttpClient>,
}

#[allow(dead_code)]
impl ClientBuilder {
    pub fn base_url(mut self, v: impl Into<String>) -> Self {
        self.base_url = v.into();
        self
    }

    pub fn api_key(mut self, v: impl Into<String>) -> Self {
        self.api_key = Some(v.into());
        self
    }

    pub fn http_client(mut self, v: HttpClient) -> Self {
        self.http_client = Some(v);
        self
    }

    /// Panics if `base_url` was not set.
    pub fn build(self) -> Client {
        assert!(
            !self.base_url.is_empty(),
            "ClientBuilder: base_url is required"
        );
        Client {
            base_url: self.base_url,
            api_key: self.api_key,
            http_client: self.http_client.unwrap_or_default(),
        }
    }
}

/// Async HTTP client for OpenAI-compatible LLM provider APIs.
#[derive(Debug, Clone, Default)]
pub struct Client {
    pub base_url: String,
    api_key: Option<String>,
    http_client: HttpClient,
}

#[allow(dead_code)]
impl Client {
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<String>,
        http_client: HttpClient,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            api_key,
            http_client,
        }
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    async fn call_api(&self, mut builder: RequestBuilder) -> Result<Response> {
        if let Some(ref api_key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {api_key}"));
        }
        let response = builder.send().await?;
        if !response.status().is_success() {
            bail!("provider API error: {}", response.text().await?);
        }
        Ok(response)
    }

    /// Lists available models via the OpenAI-compatible `/models` endpoint.
    pub async fn models(&self) -> Result<ListModelsResponse> {
        Ok(self
            .call_api(self.http_client.get(format!("{}/models", self.base_url)))
            .await?
            .json()
            .await?)
    }

    /// Lists available models via the llama.cpp-specific `/models` endpoint.
    pub async fn llamacpp_models(&self) -> Result<llamacpp::ListModelsResponse> {
        Ok(self
            .call_api(self.http_client.get(format!("{}/models", self.base_url)))
            .await?
            .json()
            .await?)
    }

    /// Sends a chat completion request.
    ///
    /// Returns a [`ChatCompletionOutput::Stream`] when `completion.stream` is `true`,
    /// or a [`ChatCompletionOutput::Response`] otherwise.
    pub async fn create_chat_completion(
        &self,
        completion: &ChatCompletion,
    ) -> Result<ChatCompletionOutput> {
        let response = self
            .call_api(
                self.http_client
                    .post(format!("{}/chat/completions", self.base_url))
                    .json(&completion),
            )
            .await?;

        if !completion.stream.unwrap_or_default() {
            return Ok(ChatCompletionOutput::Response(response.json().await?));
        }

        // Parse SSE (Server-Sent Events) stream line by line.
        let s = stream! {
            let mut bytes_stream = response.bytes_stream();
            let mut line_buf = Vec::<u8>::new();

            while let Some(chunk_result) = bytes_stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        for byte in chunk {
                            if byte == b'\n' {
                                if let Some(data) = String::from_utf8_lossy(&line_buf)
                                    .strip_prefix("data:")
                                    .map(str::trim)
                                    .and_then(|s| serde_json::from_str::<ChatCompletionChunk>(s).ok())
                                {
                                    yield Ok(data);
                                }
                                line_buf.clear();
                            } else {
                                line_buf.push(byte);
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(e.into());
                        break;
                    }
                }
            }
        };

        Ok(ChatCompletionOutput::Stream(Box::pin(s)))
    }
}

#[allow(dead_code)]
pub type ProviderClient = Client;

#[allow(dead_code)]
pub type ProviderClientBuilder = ClientBuilder;
