use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

use aipocket_core::Settings;

use crate::RetryPolicy;
use crate::retry::KeyPool;

#[derive(Clone)]
pub struct FofaClient {
    http: Client,
    base_url: String,
    keys: KeyPool,
    retry: RetryPolicy,
}
impl FofaClient {
    pub fn new(http: Client, settings: &Settings) -> Self {
        let max_wait = Duration::from_secs_f64(settings.fofa_timeout.max(0.0));
        Self {
            http,
            base_url: settings.fofa_base_url.trim_end_matches('/').into(),
            keys: KeyPool::new(
                settings
                    .fofa_key_list()
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

    pub async fn search(&self, query: &str, page: u32, size: u32) -> Result<Value> {
        let qbase64 = STANDARD.encode(query);
        let page = page.to_string();
        let size = size.to_string();
        self.keys
            .execute_json("fofa", &self.retry, |key| {
                self.http
                    .get(format!("{}/api/v1/search/all", self.base_url))
                    .query(&[
                        ("key", key),
                        ("qbase64", qbase64.as_str()),
                        ("page", page.as_str()),
                        ("size", size.as_str()),
                        // Keep field list aligned with Python DEFAULT_FIELDS / discovery FOFA_FIELDS.
                        (
                            "fields",
                            "host,ip,port,protocol,title,header,banner,server,product,link,domain,cert",
                        ),
                    ])
            })
            .await
    }
    pub async fn check(&self) -> Result<Value> {
        self.search("title=\"123\"", 1, 1).await
    }
}
