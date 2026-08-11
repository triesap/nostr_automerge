#!/usr/bin/env python3
"""Fail-closed validation for executed requirement evidence v3."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "tools/validation/requirement_coverage_v3.schema.json"
SHA256 = re.compile(r"^[0-9a-f]{64}$")
COMMIT = re.compile(r"^[0-9a-f]{40}$")
TS_PROFILES = {"core", "checkpoint", "malformed", "property", "projection"}


class EvidenceError(Exception):
    """One v3 evidence invariant failed."""


def validate_shape(report: dict[str, object]) -> None:
    required = {
        "schema", "requirements_sha256", "applicability_sha256",
        "fixture_distribution_sha256", "requirement_count", "rows",
    }
    if set(report) != required:
        raise EvidenceError("unknown_or_missing_report_field")
    if report["schema"] != "nostr_automerge.requirement_coverage.v3":
        raise EvidenceError("unknown_schema")
    for field in ("requirements_sha256", "applicability_sha256", "fixture_distribution_sha256"):
        if not isinstance(report[field], str) or not SHA256.fullmatch(report[field]):
            raise EvidenceError(f"invalid_digest:{field}")
    if report["requirement_count"] != 87 or not isinstance(report["rows"], list) or len(report["rows"]) != 87:
        raise EvidenceError("requirement_count")


def git_path_exists(commit: str, relative: str) -> bool:
    return subprocess.run(
        ("git", "cat-file", "-e", f"{commit}:{relative}"), cwd=ROOT,
        capture_output=True,
    ).returncode == 0


def validate_result_artifact(path: Path, expected_sha256: str) -> None:
    if not path.is_file():
        raise EvidenceError("missing_result_artifact")
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != expected_sha256:
        raise EvidenceError("result_artifact_hash")
    result = json.loads(data)
    if result.get("status") != "pass" or not COMMIT.fullmatch(result.get("source_commit", "")):
        raise EvidenceError("result_artifact_not_passing")
    if not result.get("executed_ids") or not result.get("command") or not result.get("output_sha256"):
        raise EvidenceError("result_artifact_incomplete")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_manifest_hashes(manifest: dict[str, object]) -> None:
    signed = manifest["jobs"]["signed-conformance"]
    if signed["fixture_distribution_sha256"] != sha256(ROOT / "fixtures/distribution/manifest_v3.json"):
        raise EvidenceError("fixture_distribution_hash")
    for job in manifest["jobs"].values():
        relative = Path(job["result_artifact"])
        if relative.is_absolute() or ".." in relative.parts:
            raise EvidenceError("unsafe_result_artifact_path")
        validate_result_artifact(ROOT / relative, job["result_sha256"])


def hash_self_test() -> None:
    manifest = json.loads((ROOT / "reports/test_evidence_manifest.json").read_text())
    validate_manifest_hashes(manifest)
    mutations = []
    stale_distribution = copy.deepcopy(manifest)
    stale_distribution["jobs"]["signed-conformance"]["fixture_distribution_sha256"] = "00" * 32
    mutations.append(stale_distribution)
    stale_result = copy.deepcopy(manifest)
    stale_result["jobs"]["rust-tests"]["result_sha256"] = "00" * 32
    mutations.append(stale_result)
    unsafe_path = copy.deepcopy(manifest)
    unsafe_path["jobs"]["rust-tests"]["result_artifact"] = "../private.json"
    mutations.append(unsafe_path)
    for mutation in mutations:
        try:
            validate_manifest_hashes(mutation)
        except EvidenceError:
            continue
        raise AssertionError("material artifact-hash mutation unexpectedly passed")


def fixture_execution_index() -> dict[str, tuple[str, str]]:
    distribution = json.loads((ROOT / "fixtures/distribution/manifest_v3.json").read_text())
    declared = {
        entry["fixture_id"]: (entry["profile"], sha256(ROOT / entry["expected_path"]))
        for entry in distribution["fixtures"]
    }
    executed: dict[str, tuple[str, str]] = {}
    for path in sorted(ROOT.glob("reports/rust_signed_*.json")):
        profile = json.loads(path.read_text())
        if profile.get("status") != "pass":
            raise EvidenceError(f"fixture_profile_not_passing:{path.name}")
        for result in profile["reports"]:
            fixture_id = result["fixture_id"]
            if fixture_id in executed:
                raise EvidenceError(f"duplicate_fixture_execution:{fixture_id}")
            executed[fixture_id] = (profile["profile"], result["report_sha256"])
    if declared != executed:
        raise EvidenceError("fixture_distribution_execution_mismatch")
    return executed


def fixture_self_test() -> None:
    executed = fixture_execution_index()
    if "nonexistent-fixture" in executed:
        raise AssertionError("nonexistent fixture unexpectedly executed")
    fixture_id, (profile, digest) = next(iter(executed.items()))
    if not profile or not SHA256.fullmatch(digest):
        raise AssertionError(f"invalid fixture execution entry: {fixture_id}")
    if digest == "00" * 32:
        raise AssertionError("mutated fixture digest unexpectedly matched")


def test_execution_index() -> set[str]:
    manifest = json.loads((ROOT / "reports/test_evidence_manifest.json").read_text())
    job = manifest["jobs"]["rust-tests"]
    if job.get("status") != "pass" or job.get("command") != "cargo test --workspace --tests --locked":
        raise EvidenceError("rust_test_job_not_passing_or_unfiltered")
    result = json.loads((ROOT / job["result_artifact"]).read_text())
    tests = job.get("test_ids", [])
    if tests != result.get("executed_ids") or len(tests) != len(set(tests)):
        raise EvidenceError("rust_test_execution_membership")
    return set(tests)


def test_self_test() -> None:
    tests = test_execution_index()
    for nonexistent in ("nonexistent::test", "filtered_out_test", "ignored_test"):
        if nonexistent in tests:
            raise AssertionError("nonexecuted Rust test unexpectedly resolved")


def validate_typescript_attestation(attestation: dict[str, object]) -> None:
    required = {
        "schema", "implementation_identity", "commit", "dependency_lock_sha256",
        "toolchain", "fixture_distribution_sha256", "profiles", "result",
        "deliberate_mismatch", "provenance",
    }
    if set(attestation) != required:
        raise EvidenceError("typescript_attestation_shape")
    if attestation["schema"] != "nostr_automerge.interop_attestation.v2":
        raise EvidenceError("typescript_attestation_schema")
    if attestation["implementation_identity"] != "triesap/nostr_automerge_typescript":
        raise EvidenceError("typescript_implementation_identity")
    if not COMMIT.fullmatch(attestation["commit"]):
        raise EvidenceError("typescript_commit")
    for field in ("dependency_lock_sha256", "fixture_distribution_sha256"):
        if not SHA256.fullmatch(attestation[field]):
            raise EvidenceError(f"typescript_{field}")
    if set(attestation["profiles"]) != TS_PROFILES:
        raise EvidenceError("typescript_profile_membership")
    if any(set(profile) != {"report_sha256", "result"} or not SHA256.fullmatch(profile["report_sha256"]) or profile["result"] != "pass" for profile in attestation["profiles"].values()):
        raise EvidenceError("typescript_profile_result")
    if attestation["result"] != "pass" or attestation["deliberate_mismatch"] != "detected" or attestation["provenance"] != "operator-local":
        raise EvidenceError("typescript_result")
    serialized = json.dumps(attestation, sort_keys=True)
    workstation_home = "/" + "Users/"
    if "://" in serialized or workstation_home in serialized or "../" in serialized:
        raise EvidenceError("typescript_private_material")


def typescript_self_test() -> None:
    baseline = {
        "schema": "nostr_automerge.interop_attestation.v2",
        "implementation_identity": "triesap/nostr_automerge_typescript",
        "commit": "11" * 20, "dependency_lock_sha256": "22" * 32,
        "toolchain": {"node": "26.5.1", "pnpm": "10.30.3", "typescript": "6.0.0"},
        "fixture_distribution_sha256": sha256(ROOT / "fixtures/distribution/manifest_v3.json"),
        "profiles": {name: {"report_sha256": "33" * 32, "result": "pass"} for name in TS_PROFILES},
        "result": "pass", "deliberate_mismatch": "detected", "provenance": "operator-local",
    }
    validate_typescript_attestation(baseline)
    for field, value in (
        ("implementation_identity", "other/private"),
        ("commit", "00"),
        ("fixture_distribution_sha256", "44"),
    ):
        mutation = copy.deepcopy(baseline)
        mutation[field] = value
        try:
            validate_typescript_attestation(mutation)
        except EvidenceError:
            continue
        raise AssertionError(f"invalid TypeScript {field} unexpectedly passed")
    missing_profile = copy.deepcopy(baseline)
    missing_profile["profiles"].pop("projection")
    try:
        validate_typescript_attestation(missing_profile)
    except EvidenceError:
        return
    raise AssertionError("missing TypeScript profile unexpectedly passed")


def validate(report: dict[str, object]) -> None:
    validate_shape(report)
    requirements_path = ROOT / "spec/requirements.json"
    applicability_path = ROOT / "spec/requirements_applicability.json"
    distribution_path = ROOT / "fixtures/distribution/manifest_v3.json"
    if report["requirements_sha256"] != sha256(requirements_path) or report["applicability_sha256"] != sha256(applicability_path) or report["fixture_distribution_sha256"] != sha256(distribution_path):
        raise EvidenceError("stale_authority_or_distribution")
    requirements = json.loads(requirements_path.read_text())["requirements"]
    applicability = json.loads(applicability_path.read_text())["classifications"]
    rows = report["rows"]
    if [row.get("id") for row in rows] != [item["id"] for item in requirements] or len({row["id"] for row in rows}) != 87:
        raise EvidenceError("missing_duplicate_unknown_or_reordered")
    tests = test_execution_index()
    fixtures = fixture_execution_index()
    for requirement, row in zip(requirements, rows, strict=True):
        identifier = requirement["id"]
        expected_authority = {
            "source": requirement["source"], "section": requirement["section"],
            "text_sha256": hashlib.sha256(requirement["text"].encode()).hexdigest(),
        }
        if row.get("authority") != expected_authority or row.get("applicability") != applicability[identifier]:
            raise EvidenceError(f"stale_row_authority:{identifier}")
        classification = applicability[identifier]
        if classification in {"out-of-core", "explicitly-deferred"}:
            if set(row) != {"id", "applicability", "authority", "status", "rationale"} or row["status"] != "not-applicable":
                raise EvidenceError(f"invalid_noncode_row:{identifier}")
            continue
        expected_status = "pass" if classification == "rust-only" else "held"
        expected_fields = {"id", "applicability", "authority", "status", "proofs"}
        if expected_status == "held":
            expected_fields.add("hold")
        if set(row) != expected_fields or row["status"] != expected_status:
            raise EvidenceError(f"invalid_covered_row:{identifier}")
        proofs = row["proofs"]
        if len(proofs) != 1 or proofs[0].get("language") != "rust":
            raise EvidenceError(f"typescript_proof_not_yet_attested:{identifier}")
        proof = proofs[0]
        if proof["implementation_identity"] != "triesap/nostr_automerge" or proof["result"] != "pass":
            raise EvidenceError(f"cross_implementation_or_failure:{identifier}")
        if not COMMIT.fullmatch(proof["implementation_commit"]) or not git_path_exists(proof["implementation_commit"], proof["implementation_path"]):
            raise EvidenceError(f"stale_implementation:{identifier}")
        validate_result_artifact(ROOT / proof["result_artifact"], proof["result_sha256"])
        result = json.loads((ROOT / proof["result_artifact"]).read_text())
        if result["source_commit"] != proof["implementation_commit"] or result["command"] != proof["execution_command"]:
            raise EvidenceError(f"stale_execution_binding:{identifier}")
        if proof["evidence_kind"] == "signed_fixture":
            if proof["evidence_id"] not in fixtures:
                raise EvidenceError(f"missing_fixture:{identifier}")
        elif proof["evidence_kind"] in {"cargo_test", "policy"}:
            if proof["evidence_id"] not in tests:
                raise EvidenceError(f"missing_test:{identifier}")
        else:
            raise EvidenceError(f"unknown_evidence_kind:{identifier}")


def matrix_self_test() -> None:
    baseline = json.loads((ROOT / "reports/requirements_coverage_v3.json").read_text())
    covered_index = next(index for index, row in enumerate(baseline["rows"]) if row.get("proofs"))
    noncode_index = next(index for index, row in enumerate(baseline["rows"]) if row["applicability"] == "out-of-core")
    fixture_index = next(index for index, row in enumerate(baseline["rows"]) if row.get("proofs", [{}])[0].get("evidence_kind") == "signed_fixture")
    test_index = next(index for index, row in enumerate(baseline["rows"]) if row.get("proofs", [{}])[0].get("evidence_kind") in {"cargo_test", "policy"})
    mutations: list[tuple[str, dict[str, object]]] = []
    missing = copy.deepcopy(baseline); missing["rows"].pop(); mutations.append(("removed_row", missing))
    duplicate = copy.deepcopy(baseline); duplicate["rows"][-1] = duplicate["rows"][0]; mutations.append(("duplicate_row", duplicate))
    stale = copy.deepcopy(baseline); stale["requirements_sha256"] = "00" * 32; mutations.append(("stale_authority", stale))
    unauthorized = copy.deepcopy(baseline); unauthorized["rows"][noncode_index] = copy.deepcopy(baseline["rows"][covered_index]); unauthorized["rows"][noncode_index]["id"] = baseline["rows"][noncode_index]["id"]; mutations.append(("unauthorized_out_of_core", unauthorized))
    commit = copy.deepcopy(baseline); commit["rows"][covered_index]["proofs"][0]["implementation_commit"] = "00" * 20; mutations.append(("changed_commit", commit))
    artifact = copy.deepcopy(baseline); artifact["rows"][covered_index]["proofs"][0]["result_artifact"] = "reports/results/missing.json"; mutations.append(("missing_artifact", artifact))
    wrong_hash = copy.deepcopy(baseline); wrong_hash["rows"][covered_index]["proofs"][0]["result_sha256"] = "00" * 32; mutations.append(("wrong_hash", wrong_hash))
    missing_test = copy.deepcopy(baseline); missing_test["rows"][test_index]["proofs"][0]["evidence_id"] = "nonexistent::test"; mutations.append(("missing_test", missing_test))
    missing_fixture = copy.deepcopy(baseline); missing_fixture["rows"][fixture_index]["proofs"][0]["evidence_id"] = "nonexistent-fixture"; mutations.append(("missing_fixture", missing_fixture))
    failing = copy.deepcopy(baseline); failing["rows"][covered_index]["proofs"][0]["result"] = "fail"; mutations.append(("failing_result", failing))
    cross = copy.deepcopy(baseline); cross["rows"][covered_index]["proofs"][0]["implementation_identity"] = "triesap/nostr_automerge_typescript"; mutations.append(("cross_implementation", cross))
    caught = []
    for name, mutation in mutations:
        try:
            validate(mutation)
        except EvidenceError as error:
            caught.append({"mutation": name, "diagnostic": str(error), "result": "caught"})
            continue
        raise AssertionError(f"material evidence mutation survived: {name}")
    commit = subprocess.run(("git", "rev-parse", "HEAD"), cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()
    report = {
        "schema": "nostr_automerge.requirement_evidence_mutations.v1",
        "source_commit": commit, "generated": len(mutations), "caught": len(caught),
        "survived": 0, "status": "pass", "mutations": caught,
    }
    (ROOT / "reports/requirements_evidence_mutations_v3.json").write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n")


def result_self_test() -> None:
    manifest = json.loads((ROOT / "reports/test_evidence_manifest.json").read_text())
    for job in manifest["jobs"].values():
        validate_result_artifact(ROOT / job["result_artifact"], job["result_sha256"])
        try:
            validate_result_artifact(ROOT / job["result_artifact"], "00" * 32)
        except EvidenceError as error:
            if str(error) == "result_artifact_hash":
                continue
        raise AssertionError("tampered result digest unexpectedly passed")


def schema_self_test() -> None:
    schema = json.loads(SCHEMA.read_text())
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        raise AssertionError("v3 schema does not declare JSON Schema 2020-12")
    required_proof = set(schema["$defs"]["proof"]["required"])
    expected = {
        "language", "implementation_identity", "implementation_commit",
        "implementation_path", "evidence_kind", "evidence_id",
        "execution_command", "runner_job", "result_artifact",
        "result_sha256", "result",
    }
    if required_proof != expected:
        raise AssertionError("v3 proof fields are incomplete")
    baseline = {
        "schema": "nostr_automerge.requirement_coverage.v3",
        "requirements_sha256": "00" * 32,
        "applicability_sha256": "11" * 32,
        "fixture_distribution_sha256": "22" * 32,
        "requirement_count": 87,
        "rows": [{} for _ in range(87)],
    }
    validate_shape(baseline)
    for field in ("applicability_sha256", "fixture_distribution_sha256"):
        mutated = copy.deepcopy(baseline)
        del mutated[field]
        try:
            validate_shape(mutated)
        except EvidenceError:
            continue
        raise AssertionError(f"missing {field} unexpectedly passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report", type=Path, default=ROOT / "reports/requirements_coverage_v3.json")
    parser.add_argument("--schema-self-test", action="store_true")
    parser.add_argument("--result-self-test", action="store_true")
    parser.add_argument("--hash-self-test", action="store_true")
    parser.add_argument("--fixture-self-test", action="store_true")
    parser.add_argument("--test-self-test", action="store_true")
    parser.add_argument("--typescript-self-test", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.schema_self_test:
        schema_self_test()
        print("PASS: requirement evidence schema v3 is complete and fail-closed")
        return 0
    if args.result_self_test:
        result_self_test()
        print("PASS: passing execution results are present and hash-bound")
        return 0
    if args.hash_self_test:
        hash_self_test()
        print("PASS: result and distribution hashes fail closed on drift")
        return 0
    if args.fixture_self_test:
        fixture_self_test()
        print("PASS: every distributed fixture occurs in one passing profile")
        return 0
    if args.test_self_test:
        test_self_test()
        print("PASS: every Rust test proof resolves to a passing unfiltered job")
        return 0
    if args.typescript_self_test:
        typescript_self_test()
        print("PASS: opaque TypeScript proof metadata fails closed without source disclosure")
        return 0
    if args.self_test:
        matrix_self_test()
        print("PASS: every material requirement evidence mutation was caught")
        return 0
    validate(json.loads(args.report.read_text()))
    print("PASS: all 87 requirements have closed executed evidence or approved holds")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
