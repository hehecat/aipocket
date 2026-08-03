#!/usr/bin/env python3
import argparse
import hashlib
import json
import re
import urllib.request
from pathlib import Path


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fetch(url):
    request = urllib.request.Request(
        url,
        headers={"Accept": "application/vnd.github+json", "User-Agent": "aipocket-evidence-verifier"},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--online", action="store_true", help="verify exact commits and license hashes online")
    args = parser.parse_args()
    here = Path(__file__).resolve().parent
    root = here.parent.parent
    manifest = json.loads((here / "morphling-evidence-manifest.json").read_text())
    adr = (here / "0001-morphling-candidate-disposition.md").read_text()
    errors = []
    candidates = manifest.get("candidates", [])
    if manifest.get("schema_version") != 2 or len(candidates) != 4:
        errors.append("manifest must contain schema_version 2 and four candidates")
    seen_components = set()
    for candidate in candidates:
        component = candidate.get("component", "")
        revision = candidate.get("revision", "")
        if component in seen_components:
            errors.append(f"duplicate component: {component}")
        seen_components.add(component)
        if not re.fullmatch(r"[0-9a-f]{40}", revision):
            errors.append(f"invalid revision: {component}")
        expected_repo_url = f"https://github.com/{candidate.get('repository', '')}"
        if candidate.get("repository_url") != expected_repo_url:
            errors.append(f"invalid authoritative repository URL: {component}")
        if candidate.get("revision_url") != f"{expected_repo_url}/tree/{revision}":
            errors.append(f"invalid immutable revision URL: {component}")
        if revision not in adr or candidate.get("revision_url", "") not in adr:
            errors.append(f"ADR omits immutable revision evidence: {component}")
        if args.online:
            try:
                commit = json.loads(fetch(candidate.get("commit_api_url", "")))
                if commit.get("sha") != revision:
                    errors.append(f"commit API mismatch: {component}: {commit.get('sha')}")
            except Exception as exc:
                errors.append(f"commit fetch failed: {component}: {exc}")
        licenses = candidate.get("licenses", [])
        if not licenses:
            errors.append(f"missing license evidence: {component}")
        for license_item in licenses:
            url = license_item.get("url", "")
            expected = license_item.get("sha256", "")
            if revision not in url or url not in adr:
                errors.append(f"ADR omits immutable license URL: {component}/{license_item.get('path')}")
            if not re.fullmatch(r"[0-9a-f]{64}", expected) or expected not in adr:
                errors.append(f"ADR omits valid license hash: {component}/{license_item.get('path')}")
            if args.online:
                try:
                    actual = hashlib.sha256(fetch(url)).hexdigest()
                    if actual != expected:
                        errors.append(f"license hash mismatch: {component}/{license_item.get('path')}: {actual}")
                except Exception as exc:
                    errors.append(f"license fetch failed: {component}/{license_item.get('path')}: {exc}")

    benchmark = manifest.get("benchmark", {})
    evidence_files = [benchmark.get("fixture", {})] + benchmark.get("artifacts", [])
    for evidence in evidence_files:
        path = root / evidence.get("path", "")
        expected = evidence.get("sha256", "")
        if not path.is_file():
            errors.append(f"benchmark evidence missing: {evidence.get('path')}")
        elif sha256(path) != expected:
            errors.append(f"benchmark evidence hash mismatch: {evidence.get('path')}: {sha256(path)}")
        if expected not in adr:
            errors.append(f"ADR omits benchmark evidence hash: {evidence.get('path')}")

    environment_path = root / "benchmarks/morphling/results/environment.json"
    summary_path = root / "benchmarks/morphling/results/summary.json"
    if environment_path.is_file() and summary_path.is_file():
        environment = json.loads(environment_path.read_text())
        summary = json.loads(summary_path.read_text())
        fixture_hash = benchmark.get("fixture", {}).get("sha256")
        if environment.get("fixture_sha256") != fixture_hash:
            errors.append("environment does not identify the pinned common fixture")
        for label in ("baseline", "current"):
            expected_revision = benchmark.get(f"{label}_revision")
            if environment.get(f"{label}_revision") != expected_revision:
                errors.append(f"environment revision mismatch: {label}")
            if summary.get("repeat_count", {}).get(label, 0) < benchmark.get("minimum_repeats", 3):
                errors.append(f"insufficient benchmark repetitions: {label}")
            metrics = summary.get("labels", {}).get(label, {})
            if metrics.get("revision") != expected_revision:
                errors.append(f"summary revision mismatch: {label}")
            for gate, threshold in benchmark.get("quality_gates", {}).items():
                metric, direction = gate.rsplit("_", 1)
                actual = metrics.get(metric)
                if actual is None or (direction == "min" and actual < threshold) or (direction == "max" and actual > threshold):
                    errors.append(f"quality gate failed: {label}/{metric}: {actual} vs {direction} {threshold}")

    if errors:
        raise SystemExit("\n".join(errors))
    mode = "online commits+hashes" if args.online else "offline structure+benchmark"
    license_count = sum(len(candidate["licenses"]) for candidate in candidates)
    print(f"verified {len(candidates)} candidates, {license_count} license files, and {len(evidence_files)} benchmark files ({mode})")


if __name__ == "__main__":
    main()
