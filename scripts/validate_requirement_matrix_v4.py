#!/usr/bin/env python3
"""Fail closed on final Rust and opaque TypeScript requirement evidence."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path

from validate_interop_attestation_v3 import validate_current as validate_interop


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
FORBIDDEN = ("/" + "Users/", "/" + "home/", "file" + "://", "http" + "://", "https" + "://", ".." + "/", ".act" + "/", "." + "log")


class EvidenceError(ValueError):
    """One final-evidence invariant failed."""


def load(relative: str) -> dict[str, object]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise EvidenceError(f"object_shape:{relative}")
    return value


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def implementation_commit() -> str:
    return subprocess.run(
        ("git", "log", "-1", "--format=%H", "--", "crates", "tools", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "fixtures"),
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout.strip()


def git_path_exists(commit: str, path: str) -> bool:
    return subprocess.run(("git", "cat-file", "-e", f"{commit}:{path}"), cwd=ROOT, capture_output=True).returncode == 0


def validate_result(relative: str, digest: str, implementation: str, command: str) -> set[str]:
    path = ROOT / relative
    if path.is_absolute() and not str(path).startswith(str(ROOT)):
        raise EvidenceError("unsafe_result_path")
    if not path.is_file() or sha256(path) != digest:
        raise EvidenceError("result_hash")
    value = json.loads(path.read_text())
    if value.get("schema") != "nostr_automerge.execution_result.v2" or value.get("status") != "pass":
        raise EvidenceError("result_status")
    if value.get("implementation_commit") != implementation or value.get("command") != command:
        raise EvidenceError("result_binding")
    if value.get("cargo_lock_sha256") != sha256(ROOT / "Cargo.lock"):
        raise EvidenceError("rust_lock_hash")
    ids = value.get("executed_ids")
    if not isinstance(ids, list) or not ids or len(ids) != len(set(ids)):
        raise EvidenceError("executed_membership")
    return set(ids)


def validate(report: dict[str, object]) -> None:
    required = {"schema", "requirements_sha256", "applicability_sha256", "fixture_distribution_sha256", "rust_evidence_manifest_sha256", "typescript_overlay_sha256", "requirement_count", "rows"}
    if set(report) != required or report.get("schema") != "nostr_automerge.requirement_coverage.v4":
        raise EvidenceError("report_shape")
    requirements_path = ROOT / "spec/requirements.json"
    applicability_path = ROOT / "spec/requirements_applicability.json"
    distribution_path = ROOT / "fixtures/distribution/manifest_v4.json"
    manifest_path = ROOT / "reports/test_evidence_manifest_v4.json"
    overlay_path = ROOT / "reports/requirements_typescript_overlay_v4.json"
    expected_hashes = (sha256(requirements_path), sha256(applicability_path), sha256(distribution_path), sha256(manifest_path), sha256(overlay_path))
    actual_hashes = tuple(report[name] for name in ("requirements_sha256", "applicability_sha256", "fixture_distribution_sha256", "rust_evidence_manifest_sha256", "typescript_overlay_sha256"))
    if actual_hashes != expected_hashes or any(not HEX64.fullmatch(str(value)) for value in actual_hashes):
        raise EvidenceError("stale_authority_or_artifact")
    requirements = json.loads(requirements_path.read_text())["requirements"]
    applicability = json.loads(applicability_path.read_text())["classifications"]
    rows = report.get("rows")
    if report.get("requirement_count") != 87 or not isinstance(rows, list) or [row.get("id") for row in rows] != [item["id"] for item in requirements]:
        raise EvidenceError("missing_duplicate_unknown_or_reordered")
    manifest = load("reports/test_evidence_manifest_v4.json")
    current_implementation = implementation_commit()
    if manifest.get("implementation_commit") != current_implementation or manifest.get("cargo_lock_sha256") != sha256(ROOT / "Cargo.lock") or manifest.get("fixture_distribution_sha256") != sha256(distribution_path):
        raise EvidenceError("stale_rust_manifest")
    inventories = {name: validate_result(job["result_artifact"], job["result_sha256"], current_implementation, job["command"]) for name, job in manifest["jobs"].items()}
    attestation_path = ROOT / "reports/interop_typescript_v3.json"
    attestation = load("reports/interop_typescript_v3.json")
    validate_interop()
    overlay_report = load("reports/requirements_typescript_overlay_v4.json")
    if overlay_report.get("attestation_sha256") != sha256(attestation_path) or overlay_report.get("requirement_count") != len(overlay_report.get("requirements", {})):
        raise EvidenceError("overlay_report_binding")
    distribution = json.loads(distribution_path.read_text())
    fixture_profiles = {entry["fixture_id"]: entry["profile"] for entry in distribution["fixtures"]}
    expected_overlay_ids = {item["id"] for item in requirements if applicability[item["id"]] == "rust-and-typescript"}
    if set(overlay_report.get("requirements", {})) != expected_overlay_ids:
        raise EvidenceError("overlay_requirement_membership")
    for requirement, row in zip(requirements, rows, strict=True):
        identifier = requirement["id"]
        authority = {"source": requirement["source"], "section": requirement["section"], "text_sha256": hashlib.sha256(requirement["text"].encode()).hexdigest()}
        classification = applicability[identifier]
        if row.get("authority") != authority or row.get("applicability") != classification:
            raise EvidenceError(f"authority:{identifier}")
        if classification in {"out-of-core", "explicitly-deferred"}:
            if set(row) != {"id", "applicability", "authority", "status", "rationale"} or row.get("status") != "not-applicable":
                raise EvidenceError(f"nonapplicable:{identifier}")
            continue
        expected_fields = {"id", "applicability", "authority", "status", "rust_proof"} | ({"typescript_overlay"} if classification == "rust-and-typescript" else set())
        if set(row) != expected_fields or row.get("status") != "pass":
            raise EvidenceError(f"row_shape:{identifier}")
        proof = row["rust_proof"]
        if proof.get("implementation_identity") != "triesap/nostr_automerge" or proof.get("implementation_commit") != current_implementation or not git_path_exists(current_implementation, proof.get("implementation_path", "")) or proof.get("result") != "pass":
            raise EvidenceError(f"rust_implementation:{identifier}")
        job = proof.get("runner_job")
        if job not in inventories or proof.get("evidence_id") not in inventories[job]:
            raise EvidenceError(f"rust_execution:{identifier}")
        manifest_job = manifest["jobs"][job]
        if proof.get("execution_command") != manifest_job["command"] or proof.get("result_artifact") != manifest_job["result_artifact"] or proof.get("result_sha256") != manifest_job["result_sha256"]:
            raise EvidenceError(f"rust_result_binding:{identifier}")
        if classification == "rust-and-typescript":
            overlay = row["typescript_overlay"]
            if overlay != overlay_report["requirements"].get(identifier):
                raise EvidenceError(f"overlay_substitution:{identifier}")
            if overlay.get("implementation_identity") != "triesap/nostr_automerge_typescript" or overlay.get("implementation_commit") != attestation.get("commit") or overlay.get("attestation_sha256") != sha256(attestation_path) or overlay.get("dependency_lock_sha256") != attestation.get("dependency_lock_sha256") or overlay.get("fixture_distribution_sha256") != sha256(distribution_path) or overlay.get("result") != "pass":
                raise EvidenceError(f"typescript_binding:{identifier}")
            profiles, fixture_ids = overlay.get("profiles"), overlay.get("fixture_ids")
            if not isinstance(profiles, list) or not profiles or not set(profiles) <= set(attestation["profiles"]) or not isinstance(fixture_ids, list):
                raise EvidenceError(f"typescript_profiles:{identifier}")
            if fixture_ids:
                if overlay.get("scope") != "direct_signed_fixtures" or any(fixture not in inventories["signed-conformance"] or fixture_profiles.get(fixture) not in profiles for fixture in fixture_ids):
                    raise EvidenceError(f"typescript_fixtures:{identifier}")
            elif overlay.get("scope") != "complete_signed_profiles" or set(profiles) != set(attestation["profiles"]):
                raise EvidenceError(f"typescript_complete_profiles:{identifier}")
    serialized = json.dumps((report, overlay_report, attestation), sort_keys=True)
    if any(token in serialized for token in FORBIDDEN):
        raise EvidenceError("private_material")


def self_test(baseline: dict[str, object]) -> None:
    covered = next(index for index, row in enumerate(baseline["rows"]) if row.get("rust_proof"))
    cross = next(index for index, row in enumerate(baseline["rows"]) if row.get("typescript_overlay"))
    mutations: list[tuple[str, dict[str, object]]] = []
    missing = copy.deepcopy(baseline); missing["rows"].pop(); mutations.append(("missing_row", missing))
    reordered = copy.deepcopy(baseline); reordered["rows"][0], reordered["rows"][1] = reordered["rows"][1], reordered["rows"][0]; mutations.append(("reordered_rows", reordered))
    authority = copy.deepcopy(baseline); authority["requirements_sha256"] = "00" * 32; mutations.append(("authority_hash", authority))
    commit = copy.deepcopy(baseline); commit["rows"][covered]["rust_proof"]["implementation_commit"] = "00" * 20; mutations.append(("rust_commit", commit))
    evidence = copy.deepcopy(baseline); evidence["rows"][covered]["rust_proof"]["evidence_id"] = "nonexistent"; mutations.append(("evidence_id", evidence))
    result = copy.deepcopy(baseline); result["rows"][covered]["rust_proof"]["result_sha256"] = "00" * 32; mutations.append(("result_hash", result))
    absent = copy.deepcopy(baseline); absent["rows"][cross].pop("typescript_overlay"); mutations.append(("missing_typescript", absent))
    ts_commit = copy.deepcopy(baseline); ts_commit["rows"][cross]["typescript_overlay"]["implementation_commit"] = "00" * 20; mutations.append(("typescript_commit", ts_commit))
    ts_profile = copy.deepcopy(baseline); ts_profile["rows"][cross]["typescript_overlay"]["profiles"] = []; mutations.append(("typescript_profile", ts_profile))
    caught = []
    for name, mutation in mutations:
        try:
            validate(mutation)
        except EvidenceError as error:
            caught.append({"mutation": name, "diagnostic": str(error), "result": "caught"})
            continue
        raise AssertionError(f"evidence mutation survived: {name}")
    output = {"schema": "nostr_automerge.requirement_evidence_mutations.v4", "generated": len(caught), "caught": len(caught), "survived": 0, "status": "pass", "mutations": caught}
    (ROOT / "reports/requirements_evidence_mutations_v4.json").write_text(json.dumps(output, sort_keys=True, separators=(",", ":")) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    schema = load("tools/validation/requirement_coverage_v4.schema.json")
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        raise EvidenceError("schema_version")
    report = load("reports/requirements_coverage_v4.json")
    validate(report)
    if args.self_test:
        self_test(report)
    print("PASS: all 87 final requirement rows are exact, current, and fail closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
