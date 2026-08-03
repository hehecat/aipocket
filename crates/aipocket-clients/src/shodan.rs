use aipocket_core::Settings;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

use crate::RetryPolicy;
use crate::retry::KeyPool;
#[derive(Clone)]
pub struct ShodanClient {
    http: Client,
    base_url: String,
    keys: KeyPool,
    retry: RetryPolicy,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShodanInfo {
    pub plan: String,
    pub query_credits: i64,
    #[serde(default)]
    pub scan_credits: i64,
}
impl ShodanClient {
    pub fn new(http: Client, settings: &Settings) -> Self {
        let max_wait = Duration::from_secs_f64(settings.shodan_timeout.max(0.0));
        Self {
            http,
            base_url: settings.shodan_base_url.trim_end_matches('/').into(),
            keys: KeyPool::new(
                settings
                    .shodan_key_list()
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            ),
            retry: RetryPolicy::new(3, Duration::from_millis(100), max_wait),
        }
    }

    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry = policy;
        self
    }

    pub async fn info_all(&self) -> Vec<(String, Result<ShodanInfo, String>)> {
        let mut out = Vec::new();
        for key in self.keys.values() {
            let result = self
                .keys
                .execute_json_with_key("shodan", &self.retry, key, |key| {
                    self.http
                        .get(format!("{}/api-info", self.base_url))
                        .query(&[("key", key)])
                })
                .await
                .and_then(|value| {
                    serde_json::from_value(value).context("Shodan info response has invalid fields")
                })
                .map_err(|error| error.to_string());
            out.push((key.clone(), result));
        }
        out
    }

    pub async fn search(&self, query: &str, page: u32) -> Result<Value> {
        let page = page.to_string();
        self.keys
            .execute_json("shodan", &self.retry, |key| {
                self.http
                    .get(format!("{}/shodan/host/search", self.base_url))
                    .query(&[("key", key), ("query", query), ("page", page.as_str())])
            })
            .await
    }

    pub async fn count(&self, query: &str) -> Result<i64> {
        let value = self
            .keys
            .execute_json("shodan", &self.retry, |key| {
                self.http
                    .get(format!("{}/shodan/host/count", self.base_url))
                    .query(&[("key", key), ("query", query)])
            })
            .await?;
        value
            .get("total")
            .and_then(Value::as_i64)
            .context("Shodan count response missing total")
    }
}
