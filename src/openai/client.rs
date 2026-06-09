use crate::openai::models::{ChatCompletionRequest, ChatCompletionResponse, ListModelsResponse};
use anyhow::{Result, bail};
use reqwest::Client as HttpClient;

#[derive(Debug, Clone, Default)]
pub struct OpenaiClientBuilder {
    base_url: String,
    api_key: Option<String>,
    http_client: Option<HttpClient>,
}

impl OpenaiClientBuilder {
    pub fn base_url<S: Into<String>>(mut self, v: S) -> Self {
        self.base_url = v.into();
        self
    }

    pub fn api_key<S: Into<String>>(mut self, v: S) -> Self {
        self.api_key.replace(v.into());
        self
    }

    pub fn http_client(mut self, v: HttpClient) -> Self {
        self.http_client.replace(v);
        self
    }

    pub fn build(self) -> OpenaiClient {
        assert!(
            !self.base_url.is_empty(),
            "OpenaiClientBuilder: base_url is required"
        );
        OpenaiClient {
            base_url: self.base_url,
            api_key: self.api_key,
            http_client: self.http_client.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OpenaiClient {
    pub base_url: String,
    api_key: Option<String>,
    http_client: HttpClient,
}

impl OpenaiClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url: base_url,
            api_key: api_key,
            http_client: HttpClient::default(),
        }
    }

    pub fn builder() -> OpenaiClientBuilder {
        OpenaiClientBuilder::default()
    }

    pub async fn models(&self) -> Result<ListModelsResponse> {
        let mut builder = self.http_client.get(format!("{}/models", self.base_url));

        if let Some(ref api_key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = builder.send().await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            bail!("OpenAI API error: {body}");
        }

        Ok(response.json::<ListModelsResponse>().await?)
    }

    pub async fn create_chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let mut builder = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url));

        if let Some(ref api_key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = builder.json(request).send().await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            bail!("OpenAI API error: {body}");
        }

        Ok(response.json::<ChatCompletionResponse>().await?)
    }
}
