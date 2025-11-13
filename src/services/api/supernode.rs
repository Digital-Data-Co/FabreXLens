use super::http::{ApiClientConfig, ApiError, AuthContext, HttpClient, Paginated, Pagination};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct SupernodeClient {
    http: Arc<HttpClient>,
    auth: Option<AuthContext>,
}

impl SupernodeClient {
    pub fn new(config: ApiClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http: Arc::new(HttpClient::new(config)?),
            auth: None,
        })
    }

    pub fn with_auth(mut self, auth: AuthContext) -> Self {
        self.auth = Some(auth);
        self
    }

    pub async fn list_nodes(&self) -> Result<Vec<SupernodeNode>, ApiError> {
        let response = self
            .http
            .get_json::<Paginated<SupernodeNode>>("/nodes", self.auth.as_ref())
            .await?;
        Ok(response.data.items)
    }

    pub async fn list_nodes_paginated(
        &self,
        pagination: Option<Pagination>,
    ) -> Result<Paginated<SupernodeNode>, ApiError> {
        let response = self
            .http
            .get_paginated::<SupernodeNode>(
                "/nodes",
                &pagination.unwrap_or_default(),
                self.auth.as_ref(),
            )
            .await?;
        Ok(response.data)
    }

    pub async fn node_health(&self, node_id: &str) -> Result<SupernodeHealth, ApiError> {
        let path = format!("/nodes/{node_id}/health");
        let response = self
            .http
            .get_json::<SupernodeHealth>(&path, self.auth.as_ref())
            .await?;
        Ok(response.data)
    }

    pub async fn invoke_action(
        &self,
        node_id: &str,
        action: &str,
        payload: Option<serde_json::Value>,
    ) -> Result<SupernodeActionResponse, ApiError> {
        let path = format!("/nodes/{node_id}/actions/{action}");
        let body = payload.unwrap_or_else(|| json!({}));
        let response = self
            .http
            .post_json::<SupernodeActionResponse, _>(&path, &body, self.auth.as_ref())
            .await?;
        Ok(response.data)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupernodeNode {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupernodeHealth {
    pub node_id: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    #[serde(default)]
    pub issues: Vec<SupernodeIssue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupernodeIssue {
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupernodeActionResponse {
    pub request_id: String,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use tokio;

    #[tokio::test]
    async fn lists_nodes() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/nodes");
            then.status(200).json_body(json!({
                "items": [
                    {
                        "id": "node-1",
                        "name": "Fabric Director",
                        "role": "controller",
                        "status": "online"
                    }
                ],
                "next": null
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = SupernodeClient::new(config).unwrap();
        let nodes = client.list_nodes().await.unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].role, "controller");
    }

    #[tokio::test]
    async fn lists_nodes_paginated() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET)
                .path("/nodes")
                .query_param("limit", "10")
                .query_param("cursor", "cursor-99");
            then.status(200).json_body(json!({
                "items": [],
                "next": null
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = SupernodeClient::new(config).unwrap();
        let page = client
            .list_nodes_paginated(Some(Pagination {
                limit: Some(10),
                cursor: Some("cursor-99".into()),
            }))
            .await
            .unwrap();

        assert!(page.items.is_empty());
    }

    #[tokio::test]
    async fn fetches_health() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/nodes/node-1/health");
            then.status(200).json_body(json!({
                "nodeId": "node-1",
                "cpuPercent": 65.5,
                "memoryPercent": 72.4,
                "issues": [
                    { "severity": "warning", "description": "Firmware update recommended" }
                ]
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = SupernodeClient::new(config).unwrap();
        let health = client.node_health("node-1").await.unwrap();
        assert_eq!(health.node_id, "node-1");
        assert_eq!(health.issues.len(), 1);
    }

    #[tokio::test]
    async fn invokes_action() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(POST)
                .path("/nodes/node-1/actions/restart")
                .json_body(json!({ "graceful": true }));
            then.status(202).json_body(json!({
                "requestId": "req-500",
                "status": "accepted"
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = SupernodeClient::new(config).unwrap();
        let response = client
            .invoke_action(
                "node-1",
                "restart",
                Some(json!({ "graceful": true })),
            )
            .await
            .unwrap();

        assert_eq!(response.request_id, "req-500");
        assert_eq!(response.status, "accepted");
    }
}

