use reqwest::Client as HttpClient;

#[derive(Default)]
pub struct ClientBuilder {
    base_url: String,
    api_key: Option<String>,
    http_client: Option<HttpClient>,
}

impl ClientBuilder {
    pub fn new<S: Into<String>>(base_url: S) -> Self {
        Self {
            base_url: base_url.into(),
            ..Default::default()
        }
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
}
