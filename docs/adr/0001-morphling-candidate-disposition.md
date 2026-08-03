# ADR 0001: Morphling candidate disposition

- Status: Accepted for T3
- Date: 2026-08-02
- Scope: discovery aggregation, artifact extraction, asset attribution, and scheduling

## Context

T3 does not add scanning sources. AIPocket already has Rust provider clients, a `DiscoverySource` boundary, target canonicalization, bounded GitHub artifact extraction, and a cancellation-aware Tokio interval scheduler. Candidate research therefore answers whether to call an external component, rewrite only the needed behavior behind an existing boundary, or drop it from this milestone.

Evidence was captured from each repository at the exact revision below using `git ls-remote` and raw files pinned to that revision. A repository's maturity or license is not by itself evidence that integration improves AIPocket. Any future adoption requires a separate threat model, fixture benchmark, operational design, and license review.

The repository and revision links below are the authoritative upstream GitHub locations. `morphling-evidence-manifest.json` also records each GitHub commit API URL so `verify_morphling_evidence.py --online` can confirm that the response identifies the exact 40-character commit, rather than merely accepting a reachable page.

## Decisions

| AIPocket component | Candidate at exact revision | Verified license evidence | Decision | Reason |
| --- | --- | --- | --- | --- |
| Discovery aggregation | [projectdiscovery/uncover](https://github.com/projectdiscovery/uncover/tree/3f7b74af20b24a7d5477d1dc77f6ba881219b0de) `3f7b74af20b24a7d5477d1dc77f6ba881219b0de` | [MIT `LICENSE.md`](https://raw.githubusercontent.com/projectdiscovery/uncover/3f7b74af20b24a7d5477d1dc77f6ba881219b0de/LICENSE.md), SHA-256 `49982116f64385bdf8582a379e591a19d6040dec22e50e29dc311596d357af23` | **Drop** | It is a Go provider aggregator and overlaps AIPocket's existing provider clients and `DiscoverySource` contract. Calling it adds a second credential/configuration, retry, normalization, and authorization plane. Rewriting it adds no required T3 source and violates the frozen no-new-source scope. Keep the current Rust boundary. |
| Artifact extraction | [trufflesecurity/trufflehog](https://github.com/trufflesecurity/trufflehog/tree/82df476e759c1448517fac6bfb3677685cdcd78a) `82df476e759c1448517fac6bfb3677685cdcd78a` | [AGPL-3.0 `LICENSE`](https://raw.githubusercontent.com/trufflesecurity/trufflehog/82df476e759c1448517fac6bfb3677685cdcd78a/LICENSE), SHA-256 `33d580ea3d93edcb787c2fa826bbf5afcec3058ce4c894a6beaad3eb4f389d2b` | **Rewrite** | Do not call a second scanner process or copy detector code. Preserve AIPocket's bounded patch/text inputs, synthetic-fixture tests, provider endpoint attribution, and zero-network benchmark. Independently implement only detector behaviors demonstrated necessary by new sanitized fixtures. This keeps budgets and attribution in one Rust path while treating the candidate as behavioral research, not source material. |
| Asset attribution | [owasp-amass/amass](https://github.com/owasp-amass/amass/tree/79299dce87b0085db0f2f4ef3e9c52cccb49f514) `79299dce87b0085db0f2f4ef3e9c52cccb49f514` | [Apache-2.0 `LICENSE`](https://raw.githubusercontent.com/owasp-amass/amass/79299dce87b0085db0f2f4ef3e9c52cccb49f514/LICENSE), SHA-256 `52c474ee3b3c9e3ca40263ec3abf32fb6b9bcadd918759026b687dd71176c676` | **Drop** | Its DNS/asset graph collection would broaden discovery, introduce new network calls and persistence, and cannot be evaluated by the frozen offline Morphling exam. Current attribution remains source/query metadata plus artifact path/object/line provenance. Reconsider only in a separately authorized source-expansion milestone. |
| Scheduling | [mvniekerk/tokio-cron-scheduler](https://github.com/mvniekerk/tokio-cron-scheduler/tree/ee987dd3917fbde8130bd05d6649897dde87523c) `ee987dd3917fbde8130bd05d6649897dde87523c` | Exact-revision [`Cargo.toml`](https://raw.githubusercontent.com/mvniekerk/tokio-cron-scheduler/ee987dd3917fbde8130bd05d6649897dde87523c/Cargo.toml) declares `MIT/Apache-2.0`; [`LICENSE-MIT`](https://raw.githubusercontent.com/mvniekerk/tokio-cron-scheduler/ee987dd3917fbde8130bd05d6649897dde87523c/LICENSE-MIT) SHA-256 `6078f6a8b89739d0eeb4bd31de11236623664a1db35315d494d043f7c3a370cc`; [`LICENSE-APACHE`](https://raw.githubusercontent.com/mvniekerk/tokio-cron-scheduler/ee987dd3917fbde8130bd05d6649897dde87523c/LICENSE-APACHE) SHA-256 `703e3deb15df5a610b5e2c7bc65f6296c86d13f62fdff3947aecdc176f1e8db8` | **Rewrite** | T3 needs a fixed interval, immediate cancellation, and the same authorization gate as manual execution. The existing `tokio::time::interval` loop provides that with no persistence or cron parser. Keep this small implementation; independently add missed-tick or calendar behavior only when requirements and tests demand it. Do not add the candidate dependency now. |

## Consequences

No external executable, service, source, credential flow, or runtime dependency is added in T3. The only research-directed implementation path is clean-room, fixture-driven extension of artifact extraction and, if requirements grow, the small scheduler. Candidate revisions and license evidence are pinned so this decision remains reproducible even if default branches move.

The offline Morphling benchmark remains the acceptance gate for canonicalization, extraction, attribution, duplicate-output suppression, request/retry budgets, and failure rate. It is not evidence about live provider accuracy, DNS graph quality, external rate limits, or production scheduler recovery.

## Same-exam evidence

The frozen synthetic fixture `benchmarks/morphling/fixtures/sanitized_exam.json` has SHA-256 `ac508d21de4897cfb7da31f2dce6ebbb504ad84ff123f628be9c46726f72bd0e`. The runner compiled baseline `de79f7d3115aead54bdb859d292fb4a8cf74382a` and current `de79f7d3115aead54bdb859d292fb4a8cf74382a+worktree.210b1f181683ce0c77f24dd1412ce213b12f58459d6c2094bb2a517be3fe57e8` against that same fixture, with three repetitions per label and a zero network budget. Pinned evidence hashes are: environment `741fe831405ad952ba178fb569308a8da86a8e4873f0bd055376da2629376cb2`, scored runs `4dfefa71195648a5894dc7e263dcef2f37c877cf28a31c6f129fbf9e4f6b4366`, and summary `686e811e0d38d9cbb1904aeb42d5b60ae6bc3051b0bbdc07992c9e739d5db58a`.

| Objective dimension | Baseline | Current | Disposition |
| --- | ---: | ---: | --- |
| Coverage / attribution accuracy | 1.0 / 1.0 | 1.0 / 1.0 | Meets baseline |
| False positives / false negatives / failures | 0 / 0 / 0 | 0 / 0 / 0 | Meets baseline |
| Duplicate outputs / requests / retries | 0 / 0 / 0 | 0 / 0 / 0 | Meets baseline and zero-cost gate |
| Median measured duration, three repetitions | 25,007,946 ns | 21,015,050 ns | 15.97% lower in this run |

The quality result is deterministic across the three repetitions for both labels. Duration is reported as an observation, not a generalized performance claim: three local repetitions on a tiny synthetic fixture do not establish production throughput. The defensible Morphling conclusion is that current meets the baseline on all gated quality, stability, and network-cost dimensions while preserving the component decisions above.
