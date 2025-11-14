use super::http::{ApiClientConfig, ApiError, AuthContext, HttpClient, Paginated, Pagination};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct GryfClient {
    http: Arc<HttpClient>,
    auth: Option<AuthContext>,
}

impl GryfClient {
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

    pub async fn list_workloads(&self) -> Result<Vec<GryfWorkload>, ApiError> {
        let response = self
            .http
            .get_json::<Paginated<GryfWorkload>>("/workloads", self.auth.as_ref())
            .await?;
        Ok(response.data.items)
    }

    pub async fn list_workloads_paginated(
        &self,
        pagination: Option<Pagination>,
    ) -> Result<Paginated<GryfWorkload>, ApiError> {
        let response = self
            .http
            .get_paginated::<GryfWorkload>(
                "/workloads",
                &pagination.unwrap_or_default(),
                self.auth.as_ref(),
            )
            .await?;
        Ok(response.data)
    }

    pub async fn workload(&self, workload_id: &str) -> Result<GryfWorkloadDetail, ApiError> {
        let path = format!("/workloads/{workload_id}");
        let response = self
            .http
            .get_json::<GryfWorkloadDetail>(&path, self.auth.as_ref())
            .await?;
        Ok(response.data)
    }

    pub async fn reassign_workload(
        &self,
        workload_id: &str,
        target_fabric: &str,
        reason: Option<&str>,
    ) -> Result<GryfReassignmentResult, ApiError> {
        let path = format!("/workloads/{workload_id}/reassign");
        let payload = json!({
            "targetFabricId": target_fabric,
            "reason": reason
        });
        let response = self
            .http
            .post_json::<GryfReassignmentResult, _>(&path, &payload, self.auth.as_ref())
            .await?;
        Ok(response.data)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GryfWorkload {
    pub id: String,
    pub name: String,
    pub state: String,
    #[serde(default)]
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GryfWorkloadDetail {
    #[serde(flatten)]
    pub workload: GryfWorkload,
    #[serde(default)]
    pub tasks: Vec<GryfTask>,
    #[serde(default)]
    pub metrics: Vec<GryfMetric>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GryfTask {
    pub id: String,
    pub node: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GryfMetric {
    pub key: String,
    pub value: f64,
    #[serde(default)]
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GryfReassignmentResult {
    pub request_id: String,
    pub status: String,
    #[serde(default)]
    pub details: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use tokio;

    #[tokio::test]
    async fn lists_workloads() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/workloads");
            then.status(200).json_body(json!({
                "items": [
                    {
                        "id": "wrk-1",
                        "name": "Inference Pipeline",
                        "state": "Running",
                        "owner": "ops-team"
                    }
                ],
                "next": null
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = GryfClient::new(config).unwrap();
        let workloads = client.list_workloads().await.unwrap();
        assert_eq!(workloads.len(), 1);
        assert_eq!(workloads[0].name, "Inference Pipeline");
    }

    #[tokio::test]
    async fn lists_workloads_paginated() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET)
                .path("/workloads")
                .query_param("limit", "25")
                .query_param("cursor", "wrk-10");
            then.status(200).json_body(json!({
                "items": [],
                "next": null
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = GryfClient::new(config).unwrap();
        let page = client
            .list_workloads_paginated(Some(Pagination {
                limit: Some(25),
                cursor: Some("wrk-10".into()),
            }))
            .await
            .unwrap();

        assert!(page.items.is_empty());
    }

    #[tokio::test]
    async fn fetches_workload_detail() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/workloads/wrk-42");
            then.status(200).json_body(json!({
                "id": "wrk-42",
                "name": "Training Job",
                "state": "Pending",
                "tasks": [
                    { "id": "task-1", "node": "node-a", "status": "Scheduled" }
                ],
                "metrics": [
                    { "key": "gpu_utilization", "value": 42.0, "unit": "percent" }
                ]
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = GryfClient::new(config).unwrap();
        let detail = client.workload("wrk-42").await.unwrap();
        assert_eq!(detail.workload.id, "wrk-42");
        assert_eq!(detail.tasks.len(), 1);
    }

    #[tokio::test]
    async fn reassigns_workload() {
        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(POST)
                .path("/workloads/wrk-1/reassign")
                .json_body(json!({
                    "targetFabricId": "fab-2",
                    "reason": "balancing"
                }));
            then.status(202).json_body(json!({
                "requestId": "req-200",
                "status": "accepted",
                "details": "Rebalancing initiated"
            }));
        });

        let config = ApiClientConfig::try_from_url(&server.url("/")).unwrap();
        let client = GryfClient::new(config).unwrap();
        let result = client
            .reassign_workload("wrk-1", "fab-2", Some("balancing"))
            .await
            .unwrap();

        assert_eq!(result.request_id, "req-200");
        assert_eq!(result.status, "accepted");
    }
}
