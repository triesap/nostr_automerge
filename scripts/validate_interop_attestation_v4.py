#!/usr/bin/env python3
"""Fail closed on stale, altered, or leaky interoperability evidence v4."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORTS = ROOT / "reports"
PROFILES = {"core", "checkpoint", "malformed", "property", "projection"}
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
FORBIDDEN = ("/" + "Users/", "/" + "home/", "file" + "://", "http" + "://", "https" + "://", ".." + "/", ".act" + "/", "." + "log")


class EvidenceError(ValueError):
    """One interoperability evidence invariant failed."""


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_sha256(value: object) -> str:
    data = (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()
    return hashlib.sha256(data).hexdigest()


def load(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise EvidenceError("object_shape")
    return value


def validate_profiles(value: object) -> None:
    if not isinstance(value, dict) or set(value) != PROFILES:
        raise EvidenceError("profile_membership")
    for profile in value.values():
        if not isinstance(profile, dict) or set(profile) != {"report_sha256", "result"}:
            raise EvidenceError("profile_shape")
        if not HEX64.fullmatch(str(profile["report_sha256"])) or profile["result"] != "pass":
            raise EvidenceError("profile_result")


def validate_attestation(value: dict[str, object], language: str) -> None:
    common = {
        "schema", "implementation_identity", "commit", "dependency_lock_sha256", "toolchain",
        "fixture_distribution_sha256", "profiles", "result", "provenance",
    }
    expected = common | ({"deliberate_mismatch"} if language == "typescript" else set())
    if set(value) != expected:
        raise EvidenceError(f"{language}_fields")
    schema = (
        "nostr_automerge.interop_attestation.v4"
        if language == "typescript"
        else "nostr_automerge.rust_interop_attestation.v4"
    )
    identity = (
        "triesap/nostr_automerge_typescript"
        if language == "typescript"
        else "triesap/nostr_automerge"
    )
    if value["schema"] != schema or value["implementation_identity"] != identity:
        raise EvidenceError(f"{language}_identity")
    if not HEX40.fullmatch(str(value["commit"])):
        raise EvidenceError(f"{language}_commit")
    for field in ("dependency_lock_sha256", "fixture_distribution_sha256"):
        if not HEX64.fullmatch(str(value[field])):
            raise EvidenceError(f"{language}_{field}")
    if not isinstance(value["toolchain"], dict) or not value["toolchain"]:
        raise EvidenceError(f"{language}_toolchain")
    validate_profiles(value["profiles"])
    if value["result"] != "pass" or value["provenance"] != "operator-local":
        raise EvidenceError(f"{language}_result")
    if language == "typescript" and value["deliberate_mismatch"] != "detected":
        raise EvidenceError("typescript_mismatch")
    if any(token in json.dumps(value, sort_keys=True) for token in FORBIDDEN):
        raise EvidenceError(f"{language}_private_material")


def rust_source_commit() -> str:
    return subprocess.run(
        (
            "git", "log", "-1", "--format=%H", "--", "crates",
            "tools/nostr_automerge_conformance", "Cargo.toml", "Cargo.lock",
            "rust-toolchain.toml", "fixtures",
        ),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def validate_current() -> None:
    rust_path = REPORTS / "interop_rust_v4.json"
    typescript_path = REPORTS / "interop_typescript_v4.json"
    combined_path = REPORTS / "interop_combined_v4.json"
    rust = load(rust_path)
    typescript = load(typescript_path)
    combined = load(combined_path)
    validate_attestation(rust, "rust")
    validate_attestation(typescript, "typescript")
    manifest_hash = sha256(ROOT / "fixtures/distribution/manifest_v5.json")
    if rust["commit"] != rust_source_commit() or rust["dependency_lock_sha256"] != sha256(ROOT / "Cargo.lock"):
        raise EvidenceError("stale_rust_binding")
    if rust["fixture_distribution_sha256"] != manifest_hash or typescript["fixture_distribution_sha256"] != manifest_hash:
        raise EvidenceError("stale_distribution")
    if rust["profiles"] != typescript["profiles"]:
        raise EvidenceError("profile_mismatch")
    if set(combined) != {
        "schema", "fixture_distribution_sha256", "rust_attestation_sha256",
        "typescript_attestation_sha256", "profiles", "comparison", "deliberate_mismatch",
        "result", "provenance",
    }:
        raise EvidenceError("combined_fields")
    if combined["schema"] != "nostr_automerge.interop_combined.v4":
        raise EvidenceError("combined_schema")
    if (
        combined["fixture_distribution_sha256"] != manifest_hash
        or combined["rust_attestation_sha256"] != sha256(rust_path)
        or combined["typescript_attestation_sha256"] != sha256(typescript_path)
        or combined["profiles"] != rust["profiles"]
        or combined["comparison"] != "byte_exact_canonical_reports_without_normalization"
        or combined["deliberate_mismatch"] != "detected"
        or combined["result"] != "pass"
        or combined["provenance"] != "operator-local"
    ):
        raise EvidenceError("combined_binding")
    if any(token in json.dumps(combined, sort_keys=True) for token in FORBIDDEN):
        raise EvidenceError("combined_private_material")


def self_test() -> list[dict[str, str]]:
    baseline_path = REPORTS / "interop_typescript_v4.json"
    baseline = load(baseline_path)
    baseline_hash = sha256(baseline_path)
    mutations: list[tuple[str, dict[str, object]]] = []
    for field, value in (
        ("commit", "00" * 20),
        ("dependency_lock_sha256", "00" * 32),
        ("fixture_distribution_sha256", "00" * 32),
        ("provenance", "untrusted"),
    ):
        mutation = copy.deepcopy(baseline)
        mutation[field] = value
        mutations.append((field, mutation))
    missing_profile = copy.deepcopy(baseline)
    missing_profile["profiles"].pop("property")
    mutations.append(("profile_membership", missing_profile))
    leaked = copy.deepcopy(baseline)
    leaked["toolchain"]["path"] = "/" + "Users/operator/private"
    mutations.append(("private_path", leaked))
    caught = []
    for name, mutation in mutations:
        try:
            validate_attestation(mutation, "typescript")
            if canonical_sha256(mutation) != baseline_hash:
                raise EvidenceError("typescript_attestation_hash")
        except EvidenceError as error:
            caught.append({"mutation": name, "diagnostic": str(error), "result": "caught"})
            continue
        raise AssertionError(f"interop mutation unexpectedly passed: {name}")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    validate_current()
    if args.self_test:
        caught = self_test()
        output = {
            "schema": "nostr_automerge.interop_evidence_mutations.v4",
            "generated": len(caught),
            "caught": len(caught),
            "survived": 0,
            "status": "pass",
            "mutations": caught,
        }
        (REPORTS / "interop_evidence_mutations_v4.json").write_text(
            json.dumps(output, sort_keys=True, separators=(",", ":")) + "\n",
            encoding="utf-8",
        )
    print("PASS: final interoperability evidence v4 is exact, current, and source-free")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
