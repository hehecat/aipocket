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
        match self
            .keys
            .execute_json("shodan", &self.retry, |key| {
                self.http
                    .get(format!("{}/shodan/host/search", self.base_url))
                    .query(&[("key", key), ("query", query), ("page", page.as_str())])
            })
            .await
        {
            Ok(value) => Ok(value),
            Err(error) => {
                // Free/dev Shodan plans reject filter queries (http.html:, port:, ...)
                // with HTTP 401 "Insufficient query credits" even though the key is
                // valid and credits remain. Retry once with filters stripped to a
                // bare OR-joined term search (verified to work on dev plans).
                if error.to_string().contains("credentials exhausted") {
                    if let Some(simple) = strip_filters_for_dev_plan(query) {
                        tracing::warn!(
                            source = "shodan",
                            original_query = query,
                            fallback_query = simple,
                            "filter query rejected by plan; retrying with stripped query"
                        );
                        return self
                            .keys
                            .execute_json("shodan", &self.retry, |key| {
                                self.http
                                    .get(format!("{}/shodan/host/search", self.base_url))
                                    .query(&[("key", key), ("query", &simple), ("page", page.as_str())])
                            })
                            .await;
                    }
                }
                Err(error)
            }
        }
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

/// Extract bare search terms from a Shodan filter query (e.g. `http.html:"litellm"`
/// `http.html:"sk-"` -> `litellm OR sk-`). Free/dev plans reject filter queries with
/// HTTP 401 "Insufficient query credits"; a plain term search still works there.
/// Returns None when the query carries no quoted terms to fall back on.
fn strip_filters_for_dev_plan(query: &str) -> Option<String> {
    let mut terms: Vec<String> = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();
    for ch in query.chars() {
        if ch == '"' {
            if in_quote {
                if !current.is_empty() {
                    terms.push(current.clone());
                }
                current.clear();
            }
            in_quote = !in_quote;
        } else if in_quote {
            current.push(ch);
        }
    }
    if terms.is_empty() && !query.contains('"') {
        // No quoted terms: extract the value of each `filter:value` token
        // (e.g. `http.html:sk-` -> `sk-`), keeping bare terms as-is.
        for token in query.split_whitespace() {
            if let Some((_, value)) = token.split_once(':') {
                let value = value.trim();
                if !value.is_empty() {
                    terms.push(value.to_string());
                }
            } else {
                terms.push(token.to_string());
            }
        }
    }
    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" OR "))
    }
}
