#!/usr/bin/env python3
"""Create and validate the source-free TypeScript execution contract v3."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "reports/private_typescript_execution_contract_v3.json"
MANIFEST = ROOT / "fixtures/distribution/manifest_v4.json"
PROFILES = ["checkpoint", "core", "malformed", "property"]
HEX64 = re.compile(r"^[0-9a-f]{64}$")
FORBIDDEN = ("/Users/", "file://", "github.com/triesap/nostr_automerge_typescript", "../")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build() -> dict[str, object]:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    return {
        "schema": "nostr_automerge.typescript_execution_contract.v3",
        "protocol_revision": manifest["protocol_revision"],
        "fixture_distribution": {
            "id": manifest["distribution_id"],
            "sha256": sha256(MANIFEST),
            "fixture_count": len(manifest["fixtures"]),
        },
        "authority": {
            "companion_spec_sha256": sha256(ROOT / "spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
            "requirements_sha256": sha256(ROOT / "spec/requirements.json"),
            "automerge_profile_sha256": sha256(ROOT / "spec/AUTOMERGE_PROFILE.md"),
        },
        "profiles": PROFILES,
        "profile_schema": "nostr_automerge.typescript_signed_profile.v4",
        "comparison": "byte_exact_canonical_reports_without_normalization",
        "required_inputs": [
            "implementation_commit",
            "dependency_lock_sha256",
            "toolchain",
        ],
        "permitted_output": [
            "implementation_identity",
            "implementation_commit",
            "dependency_lock_sha256",
            "toolchain",
            "fixture_distribution_sha256",
            "profile_report_sha256",
            "result",
            "deliberate_mismatch",
            "provenance",
        ],
        "prohibited_output": [
            "source",
            "repository_url",
            "absolute_path",
            "raw_log",
            "workflow_state",
            "credential",
        ],
    }


def validate(value: object) -> None:
    if not isinstance(value, dict) or set(value) != {
        "schema", "protocol_revision", "fixture_distribution", "authority", "profiles",
        "profile_schema", "comparison", "required_inputs", "permitted_output",
        "prohibited_output",
    }:
        raise AssertionError("contract_fields")
    if value["schema"] != "nostr_automerge.typescript_execution_contract.v3":
        raise AssertionError("contract_schema")
    if value["profiles"] != PROFILES:
        raise AssertionError("contract_profiles")
    distribution = value["fixture_distribution"]
    if not isinstance(distribution, dict) or distribution.get("sha256") != sha256(MANIFEST):
        raise AssertionError("contract_distribution")
    authority = value["authority"]
    if not isinstance(authority, dict) or not all(
        isinstance(item, str) and HEX64.fullmatch(item) for item in authority.values()
    ):
        raise AssertionError("contract_authority")
    serialized = json.dumps(value, sort_keys=True)
    if any(token in serialized for token in FORBIDDEN):
        raise AssertionError("contract_private_material")


def self_test(value: dict[str, object]) -> None:
    mutations = []
    stale = copy.deepcopy(value)
    stale["fixture_distribution"]["sha256"] = "00" * 32
    mutations.append(stale)
    missing = copy.deepcopy(value)
    missing["profiles"].pop()
    mutations.append(missing)
    leaked = copy.deepcopy(value)
    leaked["source_path"] = "/Users/example/private.ts"
    mutations.append(leaked)
    for mutation in mutations:
        try:
            validate(mutation)
        except AssertionError:
            continue
        raise AssertionError("contract mutation unexpectedly passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    value = build()
    validate(value)
    if args.self_test:
        self_test(value)
    OUTPUT.write_text(json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n")
    print("PASS: source-free TypeScript execution contract v3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
