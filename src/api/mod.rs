use anyhow::Context;
use reqwest::{Client, RequestBuilder, Response};
use serde::Serialize;
use uuid::Uuid;

pub mod branch;
pub mod epic;
pub mod story;
pub mod iteration;
pub mod user;

pub const API_BASE_URL: &str = "https://api.app.shortcut.com/api/v3";

#[derive(Clone)]
pub struct ApiClient {
    api_token: String,
    pub user_id: Uuid,
    http_client: Client,
}

pub fn get_full_path(endpoint: &str) -> String {
    // endpoint should not start with / as we append it when formatting
    assert!(!endpoint.starts_with("/"));
    format!("{}/{}", API_BASE_URL, endpoint)
}

impl ApiClient {
    pub async fn post_with_body<Body>(
        &self,
        endpoint: &str,
        body: &Body,
    ) -> anyhow::Result<Response>
    where
        Body: Serialize,
    {
        let full_path = get_full_path(endpoint);
        self.post_request(&full_path)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to GET {} with body", &full_path))
    }

    pub async fn get_with_body<Body>(&self, endpoint: &str, body: &Body) -> anyhow::Result<Response>
    where
        Body: Serialize,
    {
        let full_path = get_full_path(endpoint);
        self.get_request(&full_path)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to GET {} with body", &full_path))
    }

    pub async fn get(&self, endpoint: &str) -> anyhow::Result<Response> {
        let full_path = get_full_path(endpoint);
        self.get_request(&full_path)
            .send()
            .await
            .with_context(|| format!("Failed to GET {}", &full_path))
    }

    fn get_request(&self, path: &str) -> RequestBuilder {
        self.http_client
            .get(path)
            .header("Shortcut-Token", &self.api_token)
            .header("Content-Type", "application/json")
    }

    fn post_request(&self, path: &str) -> RequestBuilder {
        self.http_client
            .post(path)
            .header("Shortcut-Token", &self.api_token)
            .header("Content-Type", "application/json")
    }

    pub fn new(api_token: String, user_id: Uuid) -> Self {
        Self {
            api_token,
            user_id,
            http_client: Client::new(),
        }
    }
}
