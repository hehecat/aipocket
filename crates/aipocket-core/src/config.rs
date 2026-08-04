use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::CoreError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub fofa_keys: String,
    pub fofa_base_url: String,
    pub fofa_page_size: u32,
    pub fofa_max_pages: u32,
    pub fofa_timeout: f64,
    pub fofa_query_concurrency: usize,
    pub fofa_page_delay: f64,
    pub shodan_keys: String,
    pub shodan_base_url: String,
    pub shodan_max_pages: u32,
    pub shodan_timeout: f64,
    pub shodan_page_delay: f64,
    pub query_exploration_ratio: f64,
    pub fofa_query_budget: usize,
    pub shodan_query_budget: usize,
    pub shodan_shard_host_budget: usize,
    pub shodan_credit_budget: usize,
    pub maskgraph_key: String,
    pub maskgraph_max_pages: u32,
    pub maskgraph_page_delay: f64,
    pub scan_fast: bool,
    pub scan_prober: bool,
    pub prober_concurrency: usize,
    pub prober_batch_size: usize,
    pub max_requests_per_target: usize,
    pub generic_max_requests_per_target: usize,
    pub max_probe_redirects: usize,
    pub min_probe_evidence_score: i32,
    pub intrusive_checks: bool,
    pub authorized_probe_scope: String,
    /// Master switch for all outbound provider and credential requests.
    pub outbound_network_enabled: bool,
    /// Explicit authorization for FOFA, Shodan, GitHub, and Tavily requests.
    pub provider_discovery_authorized: bool,
    /// Explicit authorization to send discovered credentials to provider endpoints.
    pub credential_requests_authorized: bool,
    pub probe_vuln_classes: String,
    pub probe_max_risk: u8,
    pub probe_ssrf_enabled: bool,
    pub probe_sqli_enabled: bool,
    pub probe_rce_enabled: bool,
    pub weak_password_dict_path: String,
    pub weak_password_usernames: String,
    pub weak_password_max_attempts: usize,
    pub validate_concurrency: usize,
    pub validate_timeout: f64,
    pub validate_batch_size: usize,
    pub balance_batch_concurrency: usize,
    pub scheduler_enabled: bool,
    pub scheduler_interval: u64,
    pub scan_lock_ttl: u64,
    pub tavily_base_url: String,
    pub tavily_key: String,
    pub gpt_base_url: String,
    pub gpt_key: String,
    pub gpt_model: String,
    pub gpt_fast: bool,
    pub gpt_reasoning_effort: String,
    pub gpt_recheck_concurrency: usize,
    pub gpt_recheck_batch_size: usize,
    pub gpt_recheck_cooldown: f64,
    pub gpt_debug: bool,
    pub gpt_recheck: bool,
    pub results_dir: String,
    pub web_password: String,
    pub web_jwt_secret: String,
    pub web_token_ttl: u64,
    pub web_cors_origins: String,
    pub web_static_dir: String,
    pub web_log_buffer_lines: usize,
    pub dedup_enabled: bool,
    pub dedup_redis_url: String,
    pub dedup_host_ttl: u64,
    pub dedup_cred_ttl: u64,
    pub dedup_rejected_ttl: u64,
    pub dedup_transient_ttl: u64,
    pub dedup_balance_ttl: u64,
    pub database_url: String,
    pub pg_pool_min: u32,
    pub pg_pool_max: u32,
    pub pg_dual_write: bool,
    pub planner_metrics_version: u8,
    pub github_hunter_enabled: bool,
    pub github_tokens: String,
    pub github_api_base_url: String,
    pub github_api_version: String,
    pub github_commit_query_budget: usize,
    pub github_code_query_budget: usize,
    pub github_search_page_size: usize,
    pub github_max_pages_per_shard: usize,
    pub github_lookback_hours: u64,
    pub github_full_lookback_hours: u64,
    pub github_backfill_from: String,
    pub github_overlap_minutes: u64,
    pub github_request_timeout: f64,
    pub github_artifact_concurrency: usize,
    pub github_max_commit_files: usize,
    pub github_max_blob_bytes: usize,
    pub github_blob_fallback_budget: usize,
    pub github_file_history_enabled: bool,
    pub github_file_history_commit_limit: usize,
    pub github_rate_limit_max_wait_seconds: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fofa_keys: String::new(),
            fofa_base_url: "https://fofoapi.com".into(),
            fofa_page_size: 100,
            fofa_max_pages: 10,
            fofa_timeout: 30.0,
            fofa_query_concurrency: 3,
            fofa_page_delay: 0.3,
            shodan_keys: String::new(),
            shodan_base_url: "https://api.shodan.io".into(),
            shodan_max_pages: 10,
            shodan_timeout: 30.0,
            shodan_page_delay: 1.0,
            query_exploration_ratio: 0.2,
            fofa_query_budget: 24,
            shodan_query_budget: 16,
            shodan_shard_host_budget: 1000,
            shodan_credit_budget: 8,
            maskgraph_key: String::new(),
            maskgraph_max_pages: 3,
            maskgraph_page_delay: 1.0,
            scan_fast: false,
            scan_prober: true,
            prober_concurrency: 50,
            prober_batch_size: 200,
            max_requests_per_target: 600,
            generic_max_requests_per_target: 12,
            max_probe_redirects: 2,
            min_probe_evidence_score: 50,
            intrusive_checks: false,
            authorized_probe_scope: String::new(),
            outbound_network_enabled: false,
            provider_discovery_authorized: false,
            credential_requests_authorized: false,
            probe_vuln_classes: "*".into(),
            probe_max_risk: 1,
            probe_ssrf_enabled: false,
            probe_sqli_enabled: false,
            probe_rce_enabled: false,
            weak_password_dict_path: String::new(),
            weak_password_usernames: "admin,root".into(),
            weak_password_max_attempts: 0,
            validate_concurrency: 20,
            validate_timeout: 15.0,
            validate_batch_size: 500,
            balance_batch_concurrency: 6,
            scheduler_enabled: false,
            scheduler_interval: 3600,
            scan_lock_ttl: 7200,
            tavily_base_url: String::new(),
            tavily_key: String::new(),
            gpt_base_url: String::new(),
            gpt_key: String::new(),
            gpt_model: "gpt-4o-mini".into(),
            gpt_fast: false,
            gpt_reasoning_effort: "high".into(),
            gpt_recheck_concurrency: 5,
            gpt_recheck_batch_size: 10,
            gpt_recheck_cooldown: 5.0,
            gpt_debug: false,
            gpt_recheck: false,
            results_dir: "results".into(),
            web_password: String::new(),
            web_jwt_secret: String::new(),
            web_token_ttl: 86400,
            web_cors_origins: "*".into(),
            web_static_dir: String::new(),
            web_log_buffer_lines: 2000,
            dedup_enabled: true,
            dedup_redis_url: "redis://localhost:6379/0".into(),
            dedup_host_ttl: 604800,
            dedup_cred_ttl: 259200,
            dedup_rejected_ttl: 2592000,
            dedup_transient_ttl: 21600,
            dedup_balance_ttl: 86400,
            database_url: String::new(),
            pg_pool_min: 2,
            pg_pool_max: 10,
            pg_dual_write: false,
            planner_metrics_version: 3,
            github_hunter_enabled: true,
            github_tokens: String::new(),
            github_api_base_url: "https://api.github.com".into(),
            github_api_version: "2022-11-28".into(),
            github_commit_query_budget: 12,
            github_code_query_budget: 8,
            github_search_page_size: 100,
            github_max_pages_per_shard: 5,
            github_lookback_hours: 24,
            github_full_lookback_hours: 720,
            github_backfill_from: String::new(),
            github_overlap_minutes: 15,
            github_request_timeout: 20.0,
            github_artifact_concurrency: 8,
            github_max_commit_files: 3000,
            github_max_blob_bytes: 1_048_576,
            github_blob_fallback_budget: 100,
            github_file_history_enabled: true,
            github_file_history_commit_limit: 100,
            github_rate_limit_max_wait_seconds: 90.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkCapability {
    ProviderDiscovery,
    CredentialRequest,
}

impl Settings {
    pub fn authorize_network(&self, capability: NetworkCapability) -> Result<(), CoreError> {
        if !self.outbound_network_enabled {
            return Err(CoreError::AuthorizationDenied(
                "outbound network is disabled",
            ));
        }
        match capability {
            NetworkCapability::ProviderDiscovery if !self.provider_discovery_authorized => Err(
                CoreError::AuthorizationDenied("provider discovery is not authorized"),
            ),
            NetworkCapability::CredentialRequest if !self.credential_requests_authorized => Err(
                CoreError::AuthorizationDenied("credential requests are not authorized"),
            ),
            _ => Ok(()),
        }
    }

    pub fn load() -> Result<Self, CoreError> {
        Self::load_from(Path::new(".env"))
    }

    pub fn load_from(path: &Path) -> Result<Self, CoreError> {
        let mut values = BTreeMap::new();
        if path.is_file() {
            let content =
                std::fs::read_to_string(path).map_err(|e| CoreError::Config(e.to_string()))?;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let Some((key, raw_value)) = trimmed.split_once('=') else {
                    continue;
                };
                let value = raw_value.trim();
                let value = if value.len() >= 2
                    && ((value.starts_with('"') && value.ends_with('"'))
                        || (value.starts_with('\'') && value.ends_with('\'')))
                {
                    &value[1..value.len() - 1]
                } else {
                    value
                };
                values.insert(key.trim().to_ascii_uppercase(), value.to_owned());
            }
        }
        for (key, value) in env::vars() {
            values.insert(key.to_ascii_uppercase(), value);
        }
        let mut raw = serde_json::to_value(Self::default()).expect("settings serialize");
        let object = raw.as_object_mut().expect("settings object");
        for (field, current) in object.iter_mut() {
            let key = field.to_ascii_uppercase();
            if let Some(value) = values.get(&key) {
                *current = parse_like(value, current)
                    .map_err(|message| CoreError::Config(format!("{key}: {message}")))?;
            }
        }
        let mut settings: Self =
            serde_json::from_value(raw).map_err(|e| CoreError::Config(e.to_string()))?;
        settings.normalize()?;
        Ok(settings)
    }

    fn normalize(&mut self) -> Result<(), CoreError> {
        self.fofa_keys = normalize_list(&self.fofa_keys);
        self.shodan_keys = normalize_list(&self.shodan_keys);
        self.github_tokens = normalize_list(&self.github_tokens);
        if self.validate_batch_size == 0 {
            return Err(CoreError::Config("VALIDATE_BATCH_SIZE must be >= 1".into()));
        }
        if self.balance_batch_concurrency == 0 {
            return Err(CoreError::Config(
                "BALANCE_BATCH_CONCURRENCY must be >= 1".into(),
            ));
        }
        if self.pg_pool_min > self.pg_pool_max {
            return Err(CoreError::Config(
                "PG_POOL_MIN must be <= PG_POOL_MAX".into(),
            ));
        }
        if self.probe_max_risk > 3 {
            return Err(CoreError::Config("PROBE_MAX_RISK must be 0..=3".into()));
        }
        let valid_classes = [
            "unauth_read",
            "weak_password",
            "idor",
            "ssrf",
            "sqli",
            "rce",
        ];
        let class_config = self.probe_vuln_classes.trim();
        if !class_config.is_empty()
            && !matches!(class_config.to_ascii_lowercase().as_str(), "*" | "all")
        {
            let unknown = class_config
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty() && !valid_classes.contains(value))
                .collect::<Vec<_>>();
            if !unknown.is_empty() {
                return Err(CoreError::Config(format!(
                    "PROBE_VULN_CLASSES contains unknown classes: {}",
                    unknown.join(",")
                )));
            }
        }
        Ok(())
    }

    pub fn validate_server(&self) -> Result<(), CoreError> {
        if self.web_password.is_empty() {
            return Err(CoreError::Config("WEB_PASSWORD must be configured".into()));
        }
        if self.web_jwt_secret.is_empty() {
            return Err(CoreError::Config(
                "WEB_JWT_SECRET must be configured".into(),
            ));
        }
        Ok(())
    }

    pub fn results_path(&self) -> PathBuf {
        PathBuf::from(&self.results_dir)
    }
    pub fn pg_enabled(&self) -> bool {
        !self.database_url.trim().is_empty()
    }
    pub fn write_jsonl(&self) -> bool {
        !self.pg_enabled() || self.pg_dual_write
    }
    pub fn fofa_key_list(&self) -> Vec<&str> {
        split_list(&self.fofa_keys)
    }
    pub fn shodan_key_list(&self) -> Vec<&str> {
        split_list(&self.shodan_keys)
    }
    pub fn github_token_list(&self) -> Vec<&str> {
        split_list(&self.github_tokens)
    }
    pub fn web_cors_origin_list(&self) -> Vec<&str> {
        if self.web_cors_origins.trim().is_empty() || self.web_cors_origins.trim() == "*" {
            vec!["*"]
        } else {
            split_list(&self.web_cors_origins)
        }
    }
}

pub fn redact_json_secrets(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                let key = key.to_ascii_lowercase();
                if key.contains("key")
                    || key.contains("token")
                    || key.contains("password")
                    || key.contains("secret")
                    || key == "authorization"
                {
                    *value = serde_json::Value::String("[REDACTED]".into());
                } else {
                    redact_json_secrets(value);
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                redact_json_secrets(value);
            }
        }
        _ => {}
    }
}

fn normalize_list(value: &str) -> String {
    split_list(value).join(",")
}
fn split_list(value: &str) -> Vec<&str> {
    value
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .collect()
}
fn parse_like(value: &str, example: &serde_json::Value) -> Result<serde_json::Value, String> {
    match example {
        serde_json::Value::Bool(_) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Ok(true.into()),
            "0" | "false" | "no" | "off" => Ok(false.into()),
            _ => Err("expected boolean".into()),
        },
        serde_json::Value::Number(number) if number.is_i64() || number.is_u64() => value
            .trim()
            .parse::<u64>()
            .map(Into::into)
            .map_err(|_| "expected non-negative integer".into()),
        serde_json::Value::Number(_) => value
            .trim()
            .parse::<f64>()
            .map(|v| serde_json::json!(v))
            .map_err(|_| "expected number".into()),
        _ => Ok(value.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::Mutex;
    use std::fs;
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn loads_python_compatible_env_and_ignores_unknown() {
        let _guard = ENV_LOCK.lock();
        let dir = std::env::temp_dir().join(format!("aipocket-settings-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("config.env");
        fs::write(
            &file,
            "fofa_keys= a, b ,,c\nVALIDATE_BATCH_SIZE=4\nBALANCE_BATCH_CONCURRENCY=3\nUNKNOWN=value\n",
        )
        .unwrap();
        let settings = Settings::load_from(&file).unwrap();
        assert!(settings.validate_batch_size >= 1);
        assert_eq!(settings.balance_batch_concurrency, 3);
        assert_eq!(normalize_list(" a, b ,,c "), "a,b,c");
    }

    #[test]
    fn rejects_zero_validation_batch() {
        let mut settings = Settings {
            validate_batch_size: 0,
            ..Settings::default()
        };
        assert!(settings.normalize().is_err());
    }

    #[test]
    fn rejects_zero_batch_balance_concurrency() {
        let mut settings = Settings {
            balance_batch_concurrency: 0,
            ..Settings::default()
        };
        assert!(settings.normalize().is_err());
    }

    #[test]
    fn network_capabilities_are_default_deny_and_require_explicit_grants() {
        let mut settings = Settings::default();
        assert!(matches!(
            settings.authorize_network(NetworkCapability::ProviderDiscovery),
            Err(CoreError::AuthorizationDenied(
                "outbound network is disabled"
            ))
        ));
        assert!(matches!(
            settings.authorize_network(NetworkCapability::CredentialRequest),
            Err(CoreError::AuthorizationDenied(
                "outbound network is disabled"
            ))
        ));

        settings.outbound_network_enabled = true;
        assert!(matches!(
            settings.authorize_network(NetworkCapability::ProviderDiscovery),
            Err(CoreError::AuthorizationDenied(
                "provider discovery is not authorized"
            ))
        ));
        assert!(matches!(
            settings.authorize_network(NetworkCapability::CredentialRequest),
            Err(CoreError::AuthorizationDenied(
                "credential requests are not authorized"
            ))
        ));

        settings.provider_discovery_authorized = true;
        settings.credential_requests_authorized = true;
        assert!(
            settings
                .authorize_network(NetworkCapability::ProviderDiscovery)
                .is_ok()
        );
        assert!(
            settings
                .authorize_network(NetworkCapability::CredentialRequest)
                .is_ok()
        );
    }

    #[test]
    fn recursively_redacts_secret_fields() {
        let mut value = serde_json::json!({
            "api_key": "top-secret",
            "nested": {
                "Authorization": "Bearer private",
                "items": [
                    {"accessToken": "token-value", "name": "kept"},
                    {"password": "password-value", "count": 2}
                ]
            },
            "public": "visible"
        });

        redact_json_secrets(&mut value);

        assert_eq!(value["api_key"], "[REDACTED]");
        assert_eq!(value["nested"]["Authorization"], "[REDACTED]");
        assert_eq!(value["nested"]["items"][0]["accessToken"], "[REDACTED]");
        assert_eq!(value["nested"]["items"][0]["name"], "kept");
        assert_eq!(value["nested"]["items"][1]["password"], "[REDACTED]");
        assert_eq!(value["nested"]["items"][1]["count"], 2);
        assert_eq!(value["public"], "visible");
        assert!(!value.to_string().contains("top-secret"));
        assert!(!value.to_string().contains("Bearer private"));
        assert!(!value.to_string().contains("token-value"));
        assert!(!value.to_string().contains("password-value"));
    }

    #[test]
    fn validates_server_risk_classes_and_numeric_env_values() {
        let _guard = ENV_LOCK.lock();
        let dir =
            std::env::temp_dir().join(format!("aipocket-settings-edges-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("config.env");
        fs::write(
            &file,
            "# comment\nBROKEN_LINE\nPROBE_VULN_CLASSES=unauth_read,rce\n",
        )
        .unwrap();
        assert!(Settings::load_from(&file).is_ok());
        assert!(parse_like("2", &serde_json::json!(1)).unwrap().is_number());
        assert!(
            parse_like("2.5", &serde_json::json!(1.0))
                .unwrap()
                .is_number()
        );
        assert_eq!(
            parse_like("no", &serde_json::Value::Bool(true)).unwrap(),
            false
        );
        let mut server = Settings::default();
        assert!(server.validate_server().is_err());
        server.web_password = "password".into();
        assert!(server.validate_server().is_err());
        server.web_jwt_secret = "secret".into();
        assert!(server.validate_server().is_ok());
        assert_eq!(server.web_cors_origin_list(), vec!["*"]);
        server.web_cors_origins = "https://a.example, https://b.example".into();
        assert_eq!(
            server.web_cors_origin_list(),
            vec!["https://a.example", "https://b.example"]
        );
        assert!(server.results_path().ends_with("results"));
        assert!(!server.pg_enabled());
        assert!(server.write_jsonl());

        server.probe_vuln_classes = "unknown".into();
        assert!(server.normalize().is_err());
        server.probe_vuln_classes = "*".into();
        server.probe_max_risk = 4;
        assert!(server.normalize().is_err());
        server.probe_max_risk = 1;
        server.pg_pool_min = 3;
        server.pg_pool_max = 2;
        assert!(server.normalize().is_err());
        assert!(parse_like("maybe", &serde_json::Value::Bool(false)).is_err());
        assert!(parse_like("-1", &serde_json::json!(1)).is_err());
        assert!(parse_like("nan?", &serde_json::json!(1.0)).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
