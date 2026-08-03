use anyhow::{Context, Result};
use reqwest::{RequestBuilder, StatusCode};
use serde_json::Value;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub base_backoff: Duration,
    pub max_wait: Duration,
}

impl RetryPolicy {
    pub fn new(max_attempts: usize, base_backoff: Duration, max_wait: Duration) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            base_backoff,
            max_wait,
        }
    }
}

#[derive(Clone)]
pub(crate) struct KeyPool {
    keys: Arc<[String]>,
    next: Arc<AtomicUsize>,
}

impl KeyPool {
    pub(crate) fn new(keys: Vec<String>) -> Self {
        Self {
            keys: keys.into(),
            next: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(crate) fn values(&self) -> &[String] {
        &self.keys
    }

    pub(crate) async fn execute_json_with_key<F>(
        &self,
        source: &str,
        policy: &RetryPolicy,
        key: &str,
        build: F,
    ) -> Result<Value>
    where
        F: Fn(&str) -> RequestBuilder,
    {
        KeyPool::new(vec![key.to_owned()])
            .execute_json(source, policy, build)
            .await
    }

    pub(crate) async fn execute_json<F>(
        &self,
        source: &str,
        policy: &RetryPolicy,
        build: F,
    ) -> Result<Value>
    where
        F: Fn(&str) -> RequestBuilder,
    {
        let key_count = self.keys.len();
        anyhow::ensure!(key_count > 0, "{source} credentials not configured");

        let start = self.next.fetch_add(1, Ordering::Relaxed) % key_count;
        let attempt_budget = policy.max_attempts.max(key_count);
        let mut auth_failures = vec![false; key_count];

        for attempt in 1..=attempt_budget {
            let scheduled_key = (start + attempt - 1) % key_count;
            let key_index = (0..key_count)
                .map(|offset| (scheduled_key + offset) % key_count)
                .find(|index| !auth_failures[*index])
                .expect("credential exhaustion is handled when an auth failure is recorded");
            tracing::debug!(
                source,
                attempt,
                attempt_budget,
                key_index,
                "provider request attempt"
            );

            let response = match build(&self.keys[key_index]).send().await {
                Ok(response) => response,
                Err(error) => {
                    let timeout = error.is_timeout();
                    let connect = error.is_connect();
                    let retryable = timeout || connect;
                    if !retryable || attempt == attempt_budget {
                        anyhow::bail!(
                            "{source} transport failure attempt={attempt}/{attempt_budget} retryable={retryable} timeout={timeout} connect={connect} request={}",
                            error.is_request()
                        );
                    }
                    wait_before_retry(source, attempt, attempt_budget, "transport", None, policy)
                        .await?;
                    continue;
                }
            };

            let status = response.status();
            let retry_after = retry_after(&response);
            if status.is_success() {
                let bytes = response
                    .bytes()
                    .await
                    .context("reading provider response")?;
                return serde_json::from_slice(&bytes).with_context(|| {
                    format!(
                        "{source} invalid JSON response status={status} bytes={}",
                        bytes.len()
                    )
                });
            }

            if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
                auth_failures[key_index] = true;
                if auth_failures.iter().all(|failed| *failed) {
                    anyhow::bail!(
                        "{source} credentials exhausted after {attempt} attempt(s); last_status={status}"
                    );
                }
                tracing::warn!(
                    source,
                    attempt,
                    attempt_budget,
                    status = status.as_u16(),
                    wait_reason = "credential_rotation",
                    "provider request retry"
                );
                continue;
            }

            let retryable = status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error();
            if !retryable || attempt == attempt_budget {
                anyhow::bail!(
                    "{source} response status={status} attempt={attempt}/{attempt_budget} retryable={retryable}"
                );
            }

            let reason = if status == StatusCode::TOO_MANY_REQUESTS {
                "rate_limit"
            } else {
                "server_error"
            };
            wait_before_retry(source, attempt, attempt_budget, reason, retry_after, policy).await?;
        }

        unreachable!("attempt budget is always at least one")
    }
}

async fn wait_before_retry(
    source: &str,
    attempt: usize,
    attempt_budget: usize,
    reason: &str,
    retry_after: Option<Duration>,
    policy: &RetryPolicy,
) -> Result<()> {
    let exponential = policy
        .base_backoff
        .checked_mul(1_u32 << attempt.saturating_sub(1).min(16))
        .unwrap_or(policy.max_wait);
    let wait = retry_after.unwrap_or(exponential);
    if wait > policy.max_wait {
        anyhow::bail!(
            "{source} retry wait exceeds budget attempt={attempt}/{attempt_budget} reason={reason} requested_ms={} max_ms={}",
            wait.as_millis(),
            policy.max_wait.as_millis()
        );
    }
    tracing::warn!(
        source,
        attempt,
        attempt_budget,
        wait_reason = reason,
        wait_ms = wait.as_millis() as u64,
        "provider request retry"
    );
    if !wait.is_zero() {
        tokio::time::sleep(wait).await;
    }
    Ok(())
}

fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(Duration::from_secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        extract::{Query, State},
        http::Response,
        routing::get,
    };
    use parking_lot::Mutex;
    use std::{collections::HashMap, sync::atomic::AtomicUsize};

    #[derive(Clone)]
    struct Reply {
        status: StatusCode,
        body: &'static str,
        retry_after: Option<&'static str>,
        delay: Duration,
    }

    impl Reply {
        fn new(status: StatusCode, body: &'static str) -> Self {
            Self {
                status,
                body,
                retry_after: None,
                delay: Duration::ZERO,
            }
        }
    }

    struct MockState {
        replies: Vec<Reply>,
        attempts: AtomicUsize,
        keys: Mutex<Vec<String>>,
    }

    async fn sequence(
        State(state): State<Arc<MockState>>,
        Query(query): Query<HashMap<String, String>>,
    ) -> Response<Body> {
        let index = state.attempts.fetch_add(1, Ordering::SeqCst);
        state
            .keys
            .lock()
            .push(query.get("key").cloned().unwrap_or_default());
        let reply = state
            .replies
            .get(index)
            .unwrap_or_else(|| state.replies.last().expect("mock reply configured"));
        if !reply.delay.is_zero() {
            tokio::time::sleep(reply.delay).await;
        }
        let mut response = Response::builder().status(reply.status);
        if let Some(value) = reply.retry_after {
            response = response.header(reqwest::header::RETRY_AFTER, value);
        }
        response.body(Body::from(reply.body)).unwrap()
    }

    async fn mock_server(
        replies: Vec<Reply>,
    ) -> (String, Arc<MockState>, tokio::task::JoinHandle<()>) {
        let state = Arc::new(MockState {
            replies,
            attempts: AtomicUsize::new(0),
            keys: Mutex::new(Vec::new()),
        });
        let app = Router::new()
            .route("/", get(sequence))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (format!("http://{address}"), state, task)
    }

    async fn execute(
        replies: Vec<Reply>,
        keys: &[&str],
        policy: RetryPolicy,
        timeout: Duration,
    ) -> (Result<Value>, Arc<MockState>) {
        let (url, state, task) = mock_server(replies).await;
        let client = reqwest::Client::builder().timeout(timeout).build().unwrap();
        let pool = KeyPool::new(keys.iter().map(|key| (*key).to_owned()).collect());
        let result = pool
            .execute_json("test-provider", &policy, |key| {
                client.get(&url).query(&[("key", key)])
            })
            .await;
        task.abort();
        (result, state)
    }

    fn immediate(attempts: usize) -> RetryPolicy {
        RetryPolicy::new(attempts, Duration::ZERO, Duration::ZERO)
    }

    #[tokio::test]
    async fn rotates_after_unauthorized_and_returns_success_json() {
        let (result, state) = execute(
            vec![
                Reply::new(StatusCode::UNAUTHORIZED, "credential rejected"),
                Reply::new(StatusCode::OK, r#"{"ok":true}"#),
            ],
            &["first-secret", "second-secret"],
            immediate(2),
            Duration::from_secs(1),
        )
        .await;

        assert_eq!(result.unwrap()["ok"], true);
        assert_eq!(
            state.keys.lock().as_slice(),
            ["first-secret", "second-secret"]
        );
    }

    #[tokio::test]
    async fn never_reuses_a_rejected_credential() {
        let (result, state) = execute(
            vec![
                Reply::new(StatusCode::UNAUTHORIZED, "credential rejected"),
                Reply::new(StatusCode::INTERNAL_SERVER_ERROR, "temporary failure"),
                Reply::new(StatusCode::INTERNAL_SERVER_ERROR, "temporary failure"),
            ],
            &["first-secret", "second-secret"],
            immediate(3),
            Duration::from_secs(1),
        )
        .await;

        assert!(result.is_err());
        assert_eq!(
            state.keys.lock().as_slice(),
            ["first-secret", "second-secret", "second-secret"]
        );
    }

    #[tokio::test]
    async fn reports_key_exhaustion_without_credentials() {
        let (result, state) = execute(
            vec![
                Reply::new(StatusCode::UNAUTHORIZED, "first-secret"),
                Reply::new(StatusCode::FORBIDDEN, "second-secret"),
            ],
            &["first-secret", "second-secret"],
            immediate(2),
            Duration::from_secs(1),
        )
        .await;

        let error = format!("{:#}", result.unwrap_err());
        assert!(error.contains("credentials exhausted"));
        assert!(!error.contains("first-secret"));
        assert!(!error.contains("second-secret"));
        assert_eq!(state.attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn retries_rate_limit_and_server_error() {
        let (result, state) = execute(
            vec![
                Reply::new(StatusCode::TOO_MANY_REQUESTS, "slow down"),
                Reply::new(StatusCode::BAD_GATEWAY, "upstream failed"),
                Reply::new(StatusCode::OK, r#"{"ok":true}"#),
            ],
            &["key"],
            immediate(3),
            Duration::from_secs(1),
        )
        .await;

        assert_eq!(result.unwrap()["ok"], true);
        assert_eq!(state.attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn rejects_retry_after_beyond_wait_budget() {
        let mut reply = Reply::new(StatusCode::TOO_MANY_REQUESTS, "secret response");
        reply.retry_after = Some("2");
        let (result, state) = execute(
            vec![reply],
            &["secret-key"],
            RetryPolicy::new(2, Duration::ZERO, Duration::from_millis(10)),
            Duration::from_secs(1),
        )
        .await;

        let error = format!("{:#}", result.unwrap_err());
        assert!(error.contains("retry wait exceeds budget"));
        assert!(error.contains("requested_ms=2000"));
        assert!(!error.contains("secret-key"));
        assert_eq!(state.attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retries_timeout_then_succeeds() {
        let mut delayed = Reply::new(StatusCode::OK, r#"{"late":true}"#);
        delayed.delay = Duration::from_millis(100);
        let (result, state) = execute(
            vec![delayed, Reply::new(StatusCode::OK, r#"{"ok":true}"#)],
            &["key"],
            immediate(2),
            Duration::from_millis(20),
        )
        .await;

        assert_eq!(result.unwrap()["ok"], true);
        assert_eq!(state.attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn bad_json_error_does_not_include_body_or_key() {
        let (result, state) = execute(
            vec![Reply::new(StatusCode::OK, "secret-body-not-json")],
            &["secret-key"],
            immediate(1),
            Duration::from_secs(1),
        )
        .await;

        let error = format!("{:#}", result.unwrap_err());
        assert!(error.contains("invalid JSON response"));
        assert!(!error.contains("secret-body-not-json"));
        assert!(!error.contains("secret-key"));
        assert_eq!(state.attempts.load(Ordering::SeqCst), 1);
    }
}
