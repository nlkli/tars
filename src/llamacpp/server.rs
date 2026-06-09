use crate::llamacpp::models::ListModelsResponse;
use anyhow::{Result, bail};
use reqwest::Client as HttpClient;
use serde_json::json;

#[derive(Default)]
pub struct ClientBuilder {
    base_url: String,
    api_key: Option<String>,
    http_client: Option<HttpClient>,
}

impl ClientBuilder {
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

pub struct Client {
    pub base_url: String,
    api_key: Option<String>,
    http_client: HttpClient,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: base_url,
            api_key: None,
            http_client: HttpClient::default(),
        }
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
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

    pub async fn model_unload(&self, id: &str) -> Result<()> {
        let mut builder = self
            .http_client
            .get(format!("{}/models/unload", self.base_url));

        if let Some(ref api_key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = builder.json(&json!({"model": id})).send().await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            bail!("OpenAI API error: {body}");
        }

        Ok(())
    }

    pub async fn model_load(&self, id: &str) -> Result<()> {
        let mut builder = self
            .http_client
            .get(format!("{}/models/load", self.base_url));

        if let Some(ref api_key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = builder.json(&json!({"model": id})).send().await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            anyhow::bail!("OpenAI API error: {body}");
        }

        Ok(())
    }
}
