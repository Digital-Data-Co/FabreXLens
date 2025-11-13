use super::http::{ApiClientConfig, ApiError, HttpClient};
use crate::services::auth::RedfishSession;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct RedfishClient {
    http: Arc<HttpClient>,
}

impl RedfishClient {
    pub fn new(config: ApiClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http: Arc::new(HttpClient::new(config)?),
        })
    }

    pub async fn create_session(
        &self,
        username: &str,
        password: &str,
    ) -> Result<RedfishSession, ApiError> {
        let payload = json!({
            "UserName": username,
            "Password": password
        });

        let response = self
            .http
            .post_json::<RedfishSessionPayload, _>(
                "/redfish/v1/Sessions",
                &payload,
                None,
            )
            .await?;

        let token = response
            .headers
            .get("X-Auth-Token")
            .and_then(|value| value.to_str().ok())
            .ok_or(ApiError::MissingAuthToken)?;

        Ok(RedfishSession {
            session_id: response.data.id,
            auth_token: token.to_string(),
            expires_at: None,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RedfishSessionPayload {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(default)]
    pub user_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use tokio;

    #[tokio::test]
    async fn creates_session() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(POST)
                .path("/redfish/v1/Sessions")
                .json_body(json!({
                    "UserName": "admin",
                    "Password": "secret"
                }));
            then.status(201)
                .header("X-Auth-Token", "token123")
                .json_body(json!({
                    "Id": "session-1",
                    "UserName": "admin"
                }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = RedfishClient::new(config).unwrap();
        let session = client.create_session("admin", "secret").await.unwrap();
        assert_eq!(session.session_id, "session-1");
        assert_eq!(session.auth_token, "token123");
    }
}

