use aipocket_core::Settings;
use anyhow::Result;
use reqwest::{Client, RequestBuilder, header};
use serde_json::Value;
use std::time::Duration;

use crate::RetryPolicy;
use crate::retry::KeyPool;

const USER_AGENT: &str = concat!("AIPocket/", env!("CARGO_PKG_VERSION"));

#[derive(Clone)]
pub struct GithubClient {
    http: Client,
    base_url: String,
    tokens: KeyPool,
    version: String,
    retry: RetryPolicy,
    request_timeout: Duration,
}

impl GithubClient {
    pub fn new(http: Client, settings: &Settings) -> Self {
        Self {
            http,
            base_url: settings.github_api_base_url.trim_end_matches('/').into(),
            tokens: KeyPool::new(
                settings
                    .github_token_list()
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            ),
            version: settings.github_api_version.clone(),
            retry: RetryPolicy::new(
                3,
                Duration::from_millis(100),
                Duration::from_secs_f64(settings.github_rate_limit_max_wait_seconds.max(0.0)),
            ),
            request_timeout: Duration::from_secs_f64(settings.github_request_timeout.max(0.001)),
        }
    }

    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry = policy;
        self
    }

    fn request_with_token(&self, path: &str, token: &str) -> RequestBuilder {
        self.http
            .get(format!("{}{}", self.base_url, path))
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header("X-GitHub-Api-Version", &self.version)
            .header(header::ACCEPT, "application/vnd.github+json")
            .timeout(self.request_timeout)
    }

    async fn get_json<F>(&self, path: &str, configure: F) -> Result<Value>
    where
        F: Fn(RequestBuilder) -> RequestBuilder,
    {
        self.tokens
            .execute_json("github", &self.retry, |token| {
                configure(self.request_with_token(path, token))
            })
            .await
    }

    pub async fn rate_limit(&self) -> Result<Value> {
        self.get_json("/rate_limit", |request| request).await
    }

    pub async fn search_code(&self, query: &str, page: usize, per_page: usize) -> Result<Value> {
        self.get_json("/search/code", |request| {
            request.query(&[
                ("q", query.to_string()),
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ])
        })
        .await
    }

    pub async fn search_commits(&self, query: &str, page: usize, per_page: usize) -> Result<Value> {
        self.get_json("/search/commits", |request| {
            request.query(&[
                ("q", query.to_string()),
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ])
        })
        .await
    }

    pub async fn commit(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        page: usize,
        per_page: usize,
    ) -> Result<Value> {
        let path = format!("/repos/{owner}/{repo}/commits/{sha}");
        self.get_json(&path, |request| {
            request.query(&[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ])
        })
        .await
    }

    pub async fn blob(&self, owner: &str, repo: &str, sha: &str) -> Result<Value> {
        let path = format!("/repos/{owner}/{repo}/git/blobs/{sha}");
        self.get_json(&path, |request| request).await
    }

    pub async fn file_history(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        page: usize,
        per_page: usize,
    ) -> Result<Value> {
        let api_path = format!("/repos/{owner}/{repo}/commits");
        self.get_json(&api_path, |request| {
            request.query(&[
                ("path", path.to_string()),
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ])
        })
        .await
    }
}
