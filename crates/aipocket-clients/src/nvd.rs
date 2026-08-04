use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://services.nvd.nist.gov";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// NVD CVE API 2.0 client (anonymous, no API key required).
///
/// Rate limit without a key: 5 requests per 30s. Callers should keep
/// each sync under 5 requests.
#[derive(Clone)]
pub struct NvdClient {
    http: Client,
    base_url: String,
}

impl NvdClient {
    pub fn new(http: Client) -> Self {
        Self {
            http,
            base_url: DEFAULT_BASE_URL.into(),
        }
    }

    /// Search CVEs by keyword, restricted to entries published since
    /// `since` (RFC 3339). `limit` caps resultsPerPage (max 2000).
    pub async fn search(&self, keyword: &str, since: &str, limit: u32) -> Result<Value> {
        let response = self
            .http
            .get(format!("{}/rest/json/cves/2.0", self.base_url))
            .query(&[
                ("keywordSearch", keyword.to_string()),
                ("pubStartDate", since.to_string()),
                ("resultsPerPage", limit.to_string()),
            ])
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await
            .with_context(|| format!("NVD request failed (keyword={keyword})"))?
            .error_for_status()?;
        Ok(response.json().await?)
    }
}
