use super::http::{ApiClientConfig, ApiError, AuthContext, HttpClient, Paginated, Pagination};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct FabrexClient {
    http: Arc<HttpClient>,
    auth: Option<AuthContext>,
}

impl FabrexClient {
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

    pub async fn list_fabrics(&self) -> Result<Vec<FabrexFabric>, ApiError> {
        let response = self
            .http
            .get_json::<Paginated<FabrexFabric>>("/fabrics", self.auth.as_ref())
            .await?;
        Ok(response.data.items)
    }

    pub async fn list_fabrics_paginated(&self) -> Result<Paginated<FabrexFabric>, ApiError> {
        let response = self
            .http
            .get_json::<Paginated<FabrexFabric>>("/fabrics", self.auth.as_ref())
            .await?;
        Ok(response.data)
    }

    pub async fn list_endpoints(
        &self,
        fabric_id: &str,
        pagination: Option<Pagination>,
    ) -> Result<Paginated<FabrexEndpoint>, ApiError> {
        let path = format!("/fabrics/{fabric_id}/endpoints");
        let response = self
            .http
            .get_paginated::<FabrexEndpoint>(
                &path,
                &pagination.unwrap_or_default(),
                self.auth.as_ref(),
            )
            .await?;
        Ok(response.data)
    }

    pub async fn fabric_usage(&self, fabric_id: &str) -> Result<FabrexUsage, ApiError> {
        let path = format!("/fabrics/{fabric_id}/usage");
        let response = self
            .http
            .get_json::<FabrexUsage>(&path, self.auth.as_ref())
            .await?;
        Ok(response.data)
    }

    pub async fn reassign_endpoint(
        &self,
        fabric_id: &str,
        endpoint_id: &str,
        target_supernode: &str,
    ) -> Result<FabrexReassignmentResult, ApiError> {
        let path = format!(
            "/fabrics/{fabric_id}/endpoints/{endpoint_id}/reassign"
        );
        let payload = json!({
            "targetSupernodeId": target_supernode
        });
        let response = self
            .http
            .post_json::<FabrexReassignmentResult, _>(&path, &payload, self.auth.as_ref())
            .await?;
        Ok(response.data)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabrexFabric {
    pub id: String,
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabrexEndpoint {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub fabric_id: Option<String>,
    #[serde(default)]
    pub attached_supernode_id: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabrexUsage {
    pub fabric_id: String,
    pub utilization_percent: f64,
    pub total_endpoints: u32,
    pub assigned_endpoints: u32,
    #[serde(default)]
    pub alerts: Vec<UsageAlert>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageAlert {
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabrexReassignmentResult {
    pub request_id: String,
    pub status: String,
    #[serde(default)]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use tokio;

    #[tokio::test]
    async fn lists_fabrics() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/fabrics");
            then.status(200).json_body(json!({
                "items": [
                    {
                        "id": "fab-1",
                        "name": "Production Fabric",
                        "status": "Healthy",
                        "description": "Primary deployment"
                    }
                ],
                "next": null
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = FabrexClient::new(config).unwrap();
        let fabrics = client.list_fabrics().await.unwrap();
        assert_eq!(fabrics.len(), 1);
        assert_eq!(fabrics[0].name, "Production Fabric");
    }

    #[tokio::test]
    async fn lists_endpoints_with_pagination() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET)
                .path("/fabrics/fab-1/endpoints")
                .query_param("limit", "50")
                .query_param("cursor", "next-cursor");
            then.status(200).json_body(json!({
                "items": [
                    {
                        "id": "ep-1",
                        "name": "Endpoint A",
                        "attachedSupernodeId": "sn-1",
                        "status": "assigned"
                    }
                ],
                "next": null
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = FabrexClient::new(config).unwrap();
        let pagination = Pagination {
            limit: Some(50),
            cursor: Some("next-cursor".into()),
        };
        let endpoints = client
            .list_endpoints("fab-1", Some(pagination))
            .await
            .unwrap();

        assert_eq!(endpoints.items.len(), 1);
        assert_eq!(endpoints.items[0].id, "ep-1");
    }

    #[tokio::test]
    async fn retrieves_usage() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET)
                .path("/fabrics/fab-1/usage");
            then.status(200).json_body(json!({
                "fabricId": "fab-1",
                "utilizationPercent": 76.3,
                "totalEndpoints": 128,
                "assignedEndpoints": 92,
                "alerts": [
                    { "severity": "info", "message": "All systems normal" }
                ]
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = FabrexClient::new(config).unwrap();
        let usage = client.fabric_usage("fab-1").await.unwrap();
        assert_eq!(usage.fabric_id, "fab-1");
        assert_eq!(usage.utilization_percent, 76.3);
    }

    #[tokio::test]
    async fn reassigns_endpoint() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(POST)
                .path("/fabrics/fab-1/endpoints/ep-9/reassign")
                .json_body(json!({ "targetSupernodeId": "sn-42" }));
            then.status(202).json_body(json!({
                "requestId": "req-100",
                "status": "accepted",
                "message": "Reassignment in progress"
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = FabrexClient::new(config).unwrap();
        let result = client
            .reassign_endpoint("fab-1", "ep-9", "sn-42")
            .await
            .unwrap();

        assert_eq!(result.request_id, "req-100");
        assert_eq!(result.status, "accepted");
    }
}

