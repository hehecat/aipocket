use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.maskgraph.com/mg";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// MaskGraph search API client.
///
/// Auth: pass the account API key via the `key` query parameter (no headers).
/// Anonymous requests work only for plain domain searches; key-mode terms
/// (`sk-`, `AKIA`, ...) return status -3 "please login" without a key.
#[derive(Clone)]
pub struct MaskGraphClient {
    http: Client,
    base_url: String,
    key: Option<String>,
}

impl MaskGraphClient {
    pub fn new(http: Client, key: Option<String>) -> Self {
        Self {
            http,
            base_url: DEFAULT_BASE_URL.into(),
            key,
        }
    }

    /// Run one MaskGraph search. `query` may carry MaskGraph terms
    /// (plain words, `ip:`, `port:`, `title:`, ...). `page` is 1-based.
    pub async fn search(&self, query: &str, page: u32) -> Result<Value> {
        let mut params: Vec<(&str, String)> = vec![
            ("query", query.to_string()),
            ("device", "pc".to_string()),
            ("page", page.to_string()),
        ];
        if let Some(key) = &self.key {
            params.push(("key", key.clone()));
        }
        let text = self
            .http
            .get(format!("{}/search", self.base_url))
            .query(&params)
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await
            .with_context(|| format!("maskgraph search request failed (query={query})"))?
            .error_for_status()
            .with_context(|| format!("maskgraph search http error (query={query})"))?
            .text()
            .await
            .with_context(|| "maskgraph search response read failed")?;
        let value: Value = serde_json::from_str(&text).with_context(|| {
            format!(
                "maskgraph search invalid json (query={query}): {}",
                truncate(&text, 240)
            )
        })?;
        let status = value.get("status").and_then(Value::as_i64).unwrap_or(-1);
        if status != 0 {
            let message = value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            anyhow::bail!("maskgraph search failed status={status}: {message} (query={query})");
        }
        Ok(value)
    }
}

fn truncate(text: &str, limit: usize) -> String {
    let mut out: String = text.chars().take(limit).collect();
    if text.chars().count() > limit {
        out.push_str("...");
    }
    out
}

/// Convert a Shodan-style filter query (`http.html:"sk-"`, `http.html:sk-ant-`)
/// into MaskGraph plain terms (`sk-`, `sk-ant-`). Multiple terms are OR-joined
/// (verified: `XAI_API_KEY OR api.x.ai` returns results; bare `XAI_API_KEY`
/// alone matched nothing in MaskGraph's index).
pub fn plain_query(query: &str) -> String {
    let mut terms = Vec::new();
    for part in query.split_whitespace() {
        let term = match part.split_once(':') {
            Some((prefix, rest)) if is_filter_prefix(prefix) => rest,
            _ => part,
        };
        let term = term.trim_matches('"');
        if !term.is_empty() {
            terms.push(term.to_string());
        }
    }
    terms.join(" OR ")
}

/// Split a Shodan-style query into its plain single terms (filter prefixes and
/// quotes stripped, deduplicated, original order preserved).
///
/// MaskGraph's engine treats `OR` as literal text and whitespace-separated
/// terms as AND, so a multi-token query such as `anythingllm OR
/// OPENAI_API_KEY` matches nothing (verified against the live API: the single
/// term `anythingllm` returns hits while both OR- and AND-joined forms return
/// zero). Callers that search MaskGraph must therefore issue each term as its
/// own query.
pub fn plain_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    for part in query.split_whitespace() {
        let term = match part.split_once(':') {
            Some((prefix, rest)) if is_filter_prefix(prefix) => rest,
            _ => part,
        };
        let term = term.trim_matches('"');
        if !term.is_empty() && !terms.iter().any(|t| t == term) {
            terms.push(term.to_string());
        }
    }
    terms
}

fn is_filter_prefix(prefix: &str) -> bool {
    !prefix.is_empty()
        && prefix
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

#[cfg(test)]
mod tests {
    use super::{plain_query, plain_terms};

    #[test]
    fn strips_shodan_filters() {
        assert_eq!(plain_query(r#"http.html:"sk-""#), "sk-");
        assert_eq!(plain_query("http.html:sk-ant-"), "sk-ant-");
        assert_eq!(
            plain_query(r#"http.html:"GEMINI_API_KEY" http.html:"GOOGLE_API_KEY""#),
            "GEMINI_API_KEY OR GOOGLE_API_KEY"
        );
        assert_eq!(plain_query("sk-"), "sk-");
        assert_eq!(
            plain_query(r#"http.html:"api.x.ai" http.html:"xai-""#),
            "api.x.ai OR xai-"
        );
    }

    #[test]
    fn splits_into_plain_terms() {
        assert_eq!(
            plain_terms(r#"http.html:"anythingllm" http.html:"OPENAI_API_KEY""#),
            vec!["anythingllm", "OPENAI_API_KEY"]
        );
        assert_eq!(plain_terms(r#"http.html:"sk-""#), vec!["sk-"]);
        assert_eq!(plain_terms("sk-"), vec!["sk-"]);
        // duplicates are dropped, order preserved
        assert_eq!(
            plain_terms(r#"http.html:"sk-" http.html:"sk-" http.html:"sk-ant-""#),
            vec!["sk-", "sk-ant-"]
        );
        assert!(plain_terms("http.html:").is_empty());
    }
}
