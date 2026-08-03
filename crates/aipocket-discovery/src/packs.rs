use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct ProviderPack {
    pub id: &'static str,
    pub fofa_queries: &'static [&'static str],
    pub shodan_queries: &'static [&'static str],
    pub github_terms: &'static [&'static str],
}
pub const PACKS: &[ProviderPack] = &[
    ProviderPack {
        id: "openai",
        fofa_queries: &["body=\"sk-\""],
        shodan_queries: &["http.html:sk-"],
        github_terms: &["sk- filename:.env"],
    },
    ProviderPack {
        id: "anthropic",
        fofa_queries: &["body=\"sk-ant-\""],
        shodan_queries: &["http.html:sk-ant-"],
        github_terms: &["sk-ant- filename:.env"],
    },
    ProviderPack {
        id: "gemini",
        fofa_queries: &[
            "body=\"GEMINI_API_KEY\"",
            "body=\"GOOGLE_API_KEY\" && body=\"AIza\"",
            "body=\"generativelanguage.googleapis.com\"",
        ],
        shodan_queries: &[
            "http.html:\"GEMINI_API_KEY\"",
            "http.html:\"GOOGLE_API_KEY\" http.html:AIza",
            "http.html:generativelanguage.googleapis.com",
        ],
        github_terms: &[
            "GEMINI_API_KEY",
            "GOOGLE_API_KEY AIza",
            "generativelanguage.googleapis.com AIza",
        ],
    },
    ProviderPack {
        id: "xai",
        fofa_queries: &[
            "body=\"XAI_API_KEY\"",
            "body=\"api.x.ai\"",
            "body=\"grok-4.6\"",
            "body=\"grok-4.7\"",
        ],
        shodan_queries: &[
            "http.html:\"XAI_API_KEY\"",
            "http.html:api.x.ai",
            "http.html:\"grok-4.6\"",
            "http.html:\"grok-4.7\"",
        ],
        github_terms: &["XAI_API_KEY", "api.x.ai xai-", "grok-4.6", "grok-4.7"],
    },
    ProviderPack {
        id: "qoder",
        fofa_queries: &[
            "body=\"QODER_PAT\"",
            "body=\"QODER_PERSONAL_ACCESS_TOKEN\"",
            "body=\"api.qoder.com\"",
            "body=\"Cantus\" && body=\"Qoder\"",
        ],
        shodan_queries: &[
            "http.html:\"QODER_PAT\"",
            "http.html:\"QODER_PERSONAL_ACCESS_TOKEN\"",
            "http.html:api.qoder.com",
            "http.html:Cantus http.html:Qoder",
        ],
        github_terms: &[
            "QODER_PAT pt-",
            "QODER_PERSONAL_ACCESS_TOKEN pt-",
            "api.qoder.com",
            "Cantus Qoder",
        ],
    },
    ProviderPack {
        id: "kiro",
        fofa_queries: &["body=\"KIRO_API_KEY\"", "body=\"ksk_\" && body=\"kiro\""],
        shodan_queries: &[
            "http.html:\"KIRO_API_KEY\"",
            "http.html:ksk_ http.html:kiro",
        ],
        github_terms: &["KIRO_API_KEY", "ksk_ kiro"],
    },
    ProviderPack {
        id: "aws_bedrock",
        fofa_queries: &[
            "body=\"AWS_BEARER_TOKEN_BEDROCK\"",
            "body=\"bedrock-runtime\" && body=\"amazonaws.com\"",
        ],
        shodan_queries: &[
            "http.html:\"AWS_BEARER_TOKEN_BEDROCK\"",
            "http.html:bedrock-runtime http.html:amazonaws.com",
        ],
        github_terms: &[
            "AWS_BEARER_TOKEN_BEDROCK",
            "bedrock-runtime.amazonaws.com Authorization Bearer",
        ],
    },
    ProviderPack {
        id: "cursor",
        fofa_queries: &[
            "body=\"CURSOR_API_KEY\"",
            "body=\"crsr_\" && body=\"api.cursor.com\"",
        ],
        shodan_queries: &[
            "http.html:\"CURSOR_API_KEY\"",
            "http.html:crsr_ http.html:api.cursor.com",
        ],
        github_terms: &["CURSOR_API_KEY", "crsr_ api.cursor.com"],
    },
    ProviderPack {
        id: "windsurf",
        fofa_queries: &[
            "body=\"WINDSURF_SERVICE_KEY\"",
            "body=\"CODEIUM_SERVICE_KEY\"",
            "body=\"GetTeamCreditBalance\" && body=\"service_key\"",
        ],
        shodan_queries: &[
            "http.html:\"WINDSURF_SERVICE_KEY\"",
            "http.html:\"CODEIUM_SERVICE_KEY\"",
            "http.html:GetTeamCreditBalance http.html:service_key",
        ],
        github_terms: &[
            "WINDSURF_SERVICE_KEY",
            "CODEIUM_SERVICE_KEY",
            "GetTeamCreditBalance service_key",
        ],
    },
    ProviderPack {
        id: "azure_openai",
        fofa_queries: &["body=\"openai.azure.com\""],
        shodan_queries: &["http.html:openai.azure.com"],
        github_terms: &["AZURE_OPENAI_API_KEY"],
    },
    ProviderPack {
        id: "cohere",
        fofa_queries: &[],
        shodan_queries: &[],
        github_terms: &["COHERE_API_KEY"],
    },
    ProviderPack {
        id: "deepseek",
        fofa_queries: &[],
        shodan_queries: &[],
        github_terms: &["DEEPSEEK_API_KEY"],
    },
    ProviderPack {
        id: "fireworks",
        fofa_queries: &[],
        shodan_queries: &[],
        github_terms: &["FIREWORKS_API_KEY"],
    },
    ProviderPack {
        id: "glm",
        fofa_queries: &[],
        shodan_queries: &[],
        github_terms: &["ZHIPUAI_API_KEY"],
    },
    ProviderPack {
        id: "kimi",
        fofa_queries: &[],
        shodan_queries: &[],
        github_terms: &["MOONSHOT_API_KEY"],
    },
    ProviderPack {
        id: "longcat",
        fofa_queries: &[],
        shodan_queries: &[],
        github_terms: &["LONGCAT_API_KEY"],
    },
    ProviderPack {
        id: "minimax",
        fofa_queries: &[],
        shodan_queries: &[],
        github_terms: &["MINIMAX_API_KEY"],
    },
    ProviderPack {
        id: "qwen",
        fofa_queries: &[],
        shodan_queries: &[],
        github_terms: &["DASHSCOPE_API_KEY"],
    },
    ProviderPack {
        id: "replicate",
        fofa_queries: &[],
        shodan_queries: &[],
        github_terms: &["REPLICATE_API_TOKEN"],
    },
    ProviderPack {
        id: "together",
        fofa_queries: &[],
        shodan_queries: &[],
        github_terms: &["TOGETHER_API_KEY"],
    },
    ProviderPack {
        id: "newapi",
        fofa_queries: &[
            "body=\"New API\" && body=\"one-api\"",
            "body=\"sk-\" && body=\"oneapi\"",
        ],
        shodan_queries: &[
            "http.html:\"New API\" http.html:one-api",
            "http.html:oneapi",
        ],
        github_terms: &["ONE_API_TOKEN", "NEW_API_TOKEN one-api"],
    },
    ProviderPack {
        id: "openwebui",
        fofa_queries: &["body=\"Open WebUI\""],
        shodan_queries: &["http.html:\"Open WebUI\""],
        github_terms: &["OPEN_WEBUI_SECRET_KEY", "WEBUI_SECRET_KEY"],
    },
    ProviderPack {
        id: "dify",
        fofa_queries: &["body=\"Dify\" && body=\"console\""],
        shodan_queries: &["http.html:Dify"],
        github_terms: &["DIFY_API_KEY", "DIFY_SECRET_KEY"],
    },
    ProviderPack {
        id: "langflow",
        fofa_queries: &["body=\"Langflow\""],
        shodan_queries: &["http.html:Langflow"],
        github_terms: &["LANGFLOW_API_KEY"],
    },
    ProviderPack {
        id: "flowise",
        fofa_queries: &["body=\"Flowise\""],
        shodan_queries: &["http.html:Flowise"],
        github_terms: &["FLOWISE_API_KEY"],
    },
    ProviderPack {
        id: "litellm",
        fofa_queries: &["body=\"LiteLLM\""],
        shodan_queries: &["http.html:LiteLLM"],
        github_terms: &["LITELLM_MASTER_KEY", "LITELLM_API_KEY"],
    },
    ProviderPack {
        id: "mistral",
        fofa_queries: &["body=\"MISTRAL_API_KEY\""],
        shodan_queries: &["http.html:MISTRAL_API_KEY"],
        github_terms: &["MISTRAL_API_KEY"],
    },
    ProviderPack {
        id: "groq",
        fofa_queries: &["body=\"GROQ_API_KEY\""],
        shodan_queries: &["http.html:GROQ_API_KEY"],
        github_terms: &["GROQ_API_KEY"],
    },
    ProviderPack {
        id: "perplexity",
        fofa_queries: &["body=\"PERPLEXITY_API_KEY\""],
        shodan_queries: &["http.html:PERPLEXITY_API_KEY"],
        github_terms: &["PERPLEXITY_API_KEY"],
    },
    ProviderPack {
        id: "openrouter",
        fofa_queries: &["body=\"OPENROUTER_API_KEY\""],
        shodan_queries: &["http.html:OPENROUTER_API_KEY"],
        github_terms: &["OPENROUTER_API_KEY"],
    },
    ProviderPack {
        id: "volcengine",
        fofa_queries: &["body=\"ARK_API_KEY\" && body=\"volces.com\""],
        shodan_queries: &["http.html:ARK_API_KEY"],
        github_terms: &["ARK_API_KEY", "VOLC_ACCESSKEY"],
    },
    ProviderPack {
        id: "vllm",
        fofa_queries: &["title=\"vllm\""],
        shodan_queries: &["http.title:vllm"],
        github_terms: &["vllm --api-key", "VLLM_API_KEY"],
    },
];
pub fn registry() -> BTreeMap<&'static str, &'static ProviderPack> {
    PACKS.iter().map(|pack| (pack.id, pack)).collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pack_ids_are_unique_and_new_families_cover_every_source() {
        assert_eq!(registry().len(), PACKS.len());
        for id in [
            "gemini",
            "xai",
            "qoder",
            "kiro",
            "aws_bedrock",
            "cursor",
            "windsurf",
            "newapi",
            "openwebui",
            "dify",
            "langflow",
            "flowise",
            "litellm",
            "mistral",
            "groq",
            "perplexity",
            "openrouter",
            "volcengine",
            "vllm",
        ] {
            let pack = registry()[id];
            assert!(!pack.fofa_queries.is_empty(), "{id} FOFA queries");
            assert!(!pack.shodan_queries.is_empty(), "{id} Shodan queries");
            assert!(!pack.github_terms.is_empty(), "{id} GitHub queries");
        }
        assert!(registry()["xai"].github_terms.contains(&"grok-4.6"));
        assert!(registry()["xai"].github_terms.contains(&"grok-4.7"));
        assert!(registry()["qoder"].github_terms.contains(&"Cantus Qoder"));
        assert!(
            registry()["gemini"]
                .github_terms
                .iter()
                .any(|query| query.contains("GEMINI_API_KEY"))
        );
        assert!(
            registry()["kiro"]
                .github_terms
                .iter()
                .any(|query| query.contains("KIRO_API_KEY"))
        );
        assert!(
            registry()["aws_bedrock"]
                .github_terms
                .iter()
                .any(|query| query.contains("AWS_BEARER_TOKEN_BEDROCK"))
        );
        assert!(
            registry()["cursor"]
                .github_terms
                .iter()
                .any(|query| query.contains("CURSOR_API_KEY"))
        );
        assert!(
            registry()["windsurf"]
                .github_terms
                .iter()
                .any(|query| query.contains("WINDSURF_SERVICE_KEY"))
        );
    }
}
