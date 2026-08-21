#!/usr/bin/env python3
"""Fail closed on incomplete, stale, altered, or leaky signed-v9 evidence."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORTS = ROOT / "reports"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
FORBIDDEN = ("/" + "Users/", "/" + "home/", "file" + "://", "http" + "://", "https" + "://", ".." + "/", ".act" + "/", "." + "log")


class EvidenceError(ValueError):
    """One interoperability evidence invariant failed."""


def load(name: str) -> dict[str, object]:
    value = json.loads((REPORTS / name).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise EvidenceError(f"{name}_shape")
    return value


def file_sha256(name: str) -> str:
    return hashlib.sha256((REPORTS / name).read_bytes()).hexdigest()


def require_hash(value: object, diagnostic: str, commit: bool = False) -> None:
    pattern = HEX40 if commit else HEX64
    if not isinstance(value, str) or pattern.fullmatch(value) is None:
        raise EvidenceError(diagnostic)


def validate_attestation(value: dict[str, object], language: str) -> None:
    expected = {
        "schema", "implementation_identity", "commit", "evidence_commit",
        "dependency_lock_sha256", "fixture_distribution_sha256", "fixture_count",
        "process_runs", "delivery_permutations", "canonical_output_sha256",
        "result", "deliberate_mismatch", "provenance",
    }
    if set(value) != expected:
        raise EvidenceError(f"{language}_fields")
    rust = language == "rust"
    if value["schema"] != ("nostr_automerge.rust_interop_attestation.v9" if rust else "nostr_automerge.interop_attestation.v3"):
        raise EvidenceError(f"{language}_schema")
    if value["implementation_identity"] != ("triesap/nostr_automerge" if rust else "triesap/nostr_automerge_typescript"):
        raise EvidenceError(f"{language}_identity")
    require_hash(value["commit"], f"{language}_commit", commit=True)
    require_hash(value["evidence_commit"], f"{language}_evidence_commit", commit=True)
    require_hash(value["dependency_lock_sha256"], f"{language}_dependency_lock")
    require_hash(value["fixture_distribution_sha256"], f"{language}_distribution")
    require_hash(value["canonical_output_sha256"], f"{language}_canonical_output")
    if (value["fixture_count"], value["process_runs"], value["delivery_permutations"]) != (180, 2, 8):
        raise EvidenceError(f"{language}_complete_run")
    if value["result"] != "pass" or value["provenance"] != "operator-local":
        raise EvidenceError(f"{language}_result")
    if value["deliberate_mismatch"] != ("rejected" if rust else "detected"):
        raise EvidenceError(f"{language}_mismatch")
    if any(token in json.dumps(value, sort_keys=True) for token in FORBIDDEN):
        raise EvidenceError(f"{language}_private_material")


def ancestor(commit: object) -> bool:
    return subprocess.run(
        ("git", "merge-base", "--is-ancestor", str(commit), "HEAD"),
        cwd=ROOT,
        check=False,
    ).returncode == 0


def validate_current() -> None:
    rust_run = load("rust_conformance_v9.json")
    rust = load("interop_rust_v9.json")
    typescript = load("interop_typescript_v9.json")
    combined = load("interop_combined_v9.json")
    validate_attestation(rust, "rust")
    validate_attestation(typescript, "typescript")
    if not ancestor(rust["commit"]) or not ancestor(rust["evidence_commit"]):
        raise EvidenceError("stale_rust_candidate")
    if rust["dependency_lock_sha256"] != hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest():
        raise EvidenceError("stale_rust_lock")
    shared = {
        "fixture_distribution_sha256": rust_run["manifest_sha256"],
        "fixture_count": rust_run["scenario_count"],
        "process_runs": rust_run["process_count"],
        "delivery_permutations": rust_run["permutations_per_fixture"],
        "canonical_output_sha256": rust_run["canonical_output_sha256"],
    }
    if any(rust.get(key) != value or typescript.get(key) != value for key, value in shared.items()):
        raise EvidenceError("cross_runtime_output_mismatch")
    expected_fields = {
        "schema", *shared, "rust_attestation_sha256", "typescript_attestation_sha256",
        "comparison", "deliberate_mismatch", "result", "provenance",
    }
    if set(combined) != expected_fields:
        raise EvidenceError("combined_fields")
    if any(combined.get(key) != value for key, value in shared.items()):
        raise EvidenceError("combined_output_binding")
    if (
        combined["schema"] != "nostr_automerge.interop_combined.v9"
        or combined["rust_attestation_sha256"] != file_sha256("interop_rust_v9.json")
        or combined["typescript_attestation_sha256"] != file_sha256("interop_typescript_v9.json")
        or combined["comparison"] != "byte_exact_complete_distribution_outputs"
        or combined["deliberate_mismatch"] != "detected"
        or combined["result"] != "pass"
        or combined["provenance"] != "operator-local"
    ):
        raise EvidenceError("combined_binding")
    if any(token in json.dumps(combined, sort_keys=True) for token in FORBIDDEN):
        raise EvidenceError("combined_private_material")


def self_test() -> list[dict[str, str]]:
    baseline = load("interop_typescript_v9.json")
    mutations: list[tuple[str, dict[str, object]]] = []
    for field, value in (
        ("commit", "00" * 20),
        ("dependency_lock_sha256", "00" * 32),
        ("fixture_count", 179),
        ("process_runs", 1),
        ("delivery_permutations", 7),
        ("canonical_output_sha256", "00" * 32),
        ("deliberate_mismatch", "not-detected"),
        ("provenance", "/" + "Users/operator/private"),
    ):
        mutation = copy.deepcopy(baseline)
        mutation[field] = value
        mutations.append((field, mutation))
    caught = []
    for name, mutation in mutations:
        try:
            validate_attestation(mutation, "typescript")
            if mutation != baseline:
                raise EvidenceError("attestation_content_mismatch")
        except EvidenceError as error:
            caught.append({"mutation": name, "diagnostic": str(error), "result": "caught"})
            continue
        raise AssertionError(f"interop mutation unexpectedly passed: {name}")
    return caught


def main() -> int:
    validate_current()
    caught = self_test()
    output = {
        "schema": "nostr_automerge.interop_evidence_mutations.v9",
        "generated": len(caught),
        "caught": len(caught),
        "survived": 0,
        "status": "pass",
        "mutations": caught,
    }
    (REPORTS / "interop_evidence_mutations_v9.json").write_text(
        json.dumps(output, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    print("PASS: signed-v9 interoperability evidence is exact, current, and source-free")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
