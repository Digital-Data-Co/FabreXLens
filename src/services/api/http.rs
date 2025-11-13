use reqwest::{header::HeaderMap, Client, Method, RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    pub base_url: Url,
    pub timeout: Duration,
    pub user_agent: String,
}

impl ApiClientConfig {
    pub fn try_from_url(url: &str) -> Result<Self, ApiError> {
        let base_url = Url::parse(url)?;
        Ok(Self::new(base_url))
    }

    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            timeout: Duration::from_secs(15),
            user_agent: format!("FabreXLens/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = agent.into();
        self
    }
}

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    config: ApiClientConfig,
}

impl HttpClient {
    pub fn new(config: ApiClientConfig) -> Result<Self, ApiError> {
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(config.user_agent.clone())
            .build()
            .map_err(ApiError::Request)?;

        Ok(Self { client, config })
    }

    fn url(&self, path: &str) -> Result<Url, ApiError> {
        self.config.base_url.join(path).map_err(ApiError::from)
    }

    fn apply_auth(&self, builder: RequestBuilder, auth: Option<&AuthContext>) -> RequestBuilder {
        if let Some(auth_ctx) = auth {
            auth_ctx.apply(builder)
        } else {
            builder
        }
    }

    pub async fn get_json<T>(&self, path: &str, auth: Option<&AuthContext>) -> Result<ApiResponse<T>, ApiError>
    where
        T: DeserializeOwned,
    {
        self.request_json(Method::GET, path, Option::<&()>::None, auth)
            .await
    }

    pub async fn get_paginated<T>(
        &self,
        path: &str,
        pagination: &Pagination,
        auth: Option<&AuthContext>,
    ) -> Result<ApiResponse<Paginated<T>>, ApiError>
    where
        T: DeserializeOwned,
    {
        let mut url = self.url(path)?;
        pagination.apply(&mut url);
        let builder = self.apply_auth(self.client.get(url), auth);
        let response = builder.send().await.map_err(ApiError::Request)?;
        Self::hydrate_response(response).await
    }

    pub async fn delete(&self, path: &str, auth: Option<&AuthContext>) -> Result<(), ApiError> {
        let url = self.url(path)?;
        let builder = self.apply_auth(self.client.request(Method::DELETE, url), auth);
        let response = builder.send().await.map_err(ApiError::Request)?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read body>".into());
            Err(ApiError::HttpStatus { status, body })
        }
    }

    pub async fn post_json<T, B>(
        &self,
        path: &str,
        body: &B,
        auth: Option<&AuthContext>,
    ) -> Result<ApiResponse<T>, ApiError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        self.request_json(Method::POST, path, Some(body), auth).await
    }

    pub async fn request_json<T, B>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        auth: Option<&AuthContext>,
    ) -> Result<ApiResponse<T>, ApiError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = self.url(path)?;
        let mut builder = self.client.request(method, url);
        if let Some(payload) = body {
            builder = builder.json(payload);
        }
        builder = self.apply_auth(builder, auth);

        let response = builder.send().await.map_err(ApiError::Request)?;
        Self::hydrate_response(response).await
    }

    async fn hydrate_response<T>(response: reqwest::Response) -> Result<ApiResponse<T>, ApiError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .text()
            .await
            .map_err(ApiError::Request)?;

        if !status.is_success() {
            return Err(ApiError::HttpStatus { status, body });
        }

        let data = serde_json::from_str(&body)
            .map_err(|source| ApiError::Deserialize { source, body })?;

        Ok(ApiResponse {
            data,
            status,
            headers,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct AuthContext {
    pub bearer_token: Option<String>,
    pub basic: Option<(String, String)>,
}

impl AuthContext {
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            bearer_token: Some(token.into()),
            ..Default::default()
        }
    }

    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            basic: Some((username.into(), password.into())),
            ..Default::default()
        }
    }

    pub fn apply(&self, mut builder: RequestBuilder) -> RequestBuilder {
        if let Some(token) = &self.bearer_token {
            builder = builder.bearer_auth(token);
        }
        if let Some((username, password)) = &self.basic {
            builder = builder.basic_auth(username, Some(password));
        }
        builder
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    #[serde(default)]
    pub next: Option<String>,
}

impl<T> Paginated<T> {
    pub fn has_more(&self) -> bool {
        self.next.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Pagination {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

impl Pagination {
    pub fn apply(&self, url: &mut Url) {
        if self.limit.is_none() && self.cursor.is_none() {
            return;
        }

        {
            let mut pairs = url.query_pairs_mut();
            if let Some(limit) = self.limit {
                pairs.append_pair("limit", &limit.to_string());
            }
            if let Some(cursor) = &self.cursor {
                pairs.append_pair("cursor", cursor);
            }
        }
    }
}

#[derive(Debug)]
pub struct ApiResponse<T> {
    pub data: T,
    pub status: StatusCode,
    pub headers: HeaderMap,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("HTTP {status}: {body}")]
    HttpStatus { status: StatusCode, body: String },
    #[error("failed to deserialize response: {source}")]
    Deserialize {
        source: serde_json::Error,
        body: String,
    },
    #[error("missing expected authentication token in response headers")]
    MissingAuthToken,
}

