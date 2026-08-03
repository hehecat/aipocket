# Changelog

## 2026-08-03 - T7 incremental hardening and acceptance

### Added

- Added fail-closed SSRF policy enforcement for outbound target probes, including literal-address, complete DNS-answer, and per-redirect validation.
- Added focused SSRF regression tests and login UX regression coverage.
- Added `检查报告T7.md` with reproducible gate results and explicit validation boundaries.

### Changed

- Corrected the pinned pnpm 10.25.0 integrity value so cold Corepack resolution and the documented frontend gates are reproducible.
- Localized login failures without exposing backend details; added alert focus/ARIA behavior, clear-on-edit handling, and a 44x44 password visibility control.

### Acceptance results

- Rust formatting, Clippy with warnings denied, workspace tests, and 6/6 focused SSRF tests passed.
- Frontend lint passed with 0 errors, all 42 tests passed, and the production build passed.
- The `aipocket-t6-deploy` stack recovered non-destructively with all four services healthy; API port 18006 and Web port 13086 both returned HTTP 200.
- Existing audit-ledger cascade deletion remains unchanged because current contracts define it within the run lifecycle and do not promise durable compliance retention.

## 2026-08-03 - T6 independent inspection

### Added

- Added `检查报告T6.md`, covering independent quality, deployment, security, reliability, and browser verification.
- Recorded reproducible command results and explicit tested/static/unverified boundaries.

### Inspection results

- P0: none found.
- P1: conditional authenticated SSRF when outbound scanning is enabled; standard frontend `pnpm` gates blocked by a Corepack tarball hash mismatch.
- P2: request-ledger cascade deletion semantics; one transient backend exit during parallel Compose restart; three login UX/accessibility findings.
- Rust formatting, clippy, and workspace tests passed: 130 passed, 0 failed, 10 ignored.
- Checked-out frontend tools passed lint (0 errors, 14 warnings), 38 tests, TypeScript compilation, and production build. These fallback results do not replace the blocked original `pnpm` commands.
- Local Compose images built. A host-specific Docker address-pool workaround was required; afterward all four services became healthy and recovered from a non-destructive restart.
- Desktop and mobile login-before-auth flows, route guards, empty submission, invalid-password handling, and responsive layout were exercised in Chrome headless CDP.

### Changed

- No product source, test, dependency, lockfile, Compose, or generated application file was changed by T6 inspection.
