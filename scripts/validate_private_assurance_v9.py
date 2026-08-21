#!/usr/bin/env python3
"""Validate the final approved opaque private assurance import."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
GATES = {"conformance", "format", "package", "policy", "requirements", "resource", "supply_chain", "tests", "typecheck"}
HOLDS = ["source-mutating campaigns", "sustained fuzzing", "independent external review", "publication"]
FORBIDDEN = ("/" + "Users/", "/" + "home/", "file" + "://", "http" + "://", "https" + "://", ".." + "/", ".act" + "/", "." + "log", "domains/" + "labs")


def main() -> int:
    path = ROOT / "reports/private_assurance_v9.json"
    report = json.loads(path.read_text())
    attestation_path = ROOT / "reports/interop_typescript_v9.json"
    attestation = json.loads(attestation_path.read_text())
    expected_fields = {
        "schema", "implementation_identity", "implementation_candidate", "evidence_candidate",
        "attestation_candidate", "attestation_sha256", "readiness_sha256", "fixture_count",
        "permutations_per_fixture", "ordinary_gates", "private_source", "source_only",
        "scoped_worktree_clean", "result", "holds",
    }
    if set(report) != expected_fields or report["schema"] != "nostr_automerge.private_assurance.v9":
        raise AssertionError("private_assurance_fields")
    if report["implementation_identity"] != "triesap/nostr_automerge_typescript":
        raise AssertionError("private_identity")
    for field in ("implementation_candidate", "evidence_candidate", "attestation_candidate"):
        if not HEX40.fullmatch(str(report[field])):
            raise AssertionError(field)
    for field in ("attestation_sha256", "readiness_sha256"):
        if not HEX64.fullmatch(str(report[field])):
            raise AssertionError(field)
    if (
        report["implementation_candidate"] != attestation["commit"]
        or report["evidence_candidate"] != attestation["evidence_commit"]
        or report["attestation_sha256"] != hashlib.sha256(attestation_path.read_bytes()).hexdigest()
        or report["fixture_count"] != 180
        or report["permutations_per_fixture"] != 8
    ):
        raise AssertionError("private_attestation_binding")
    gates = report["ordinary_gates"]
    if set(gates) != GATES or set(gates.values()) != {"pass"}:
        raise AssertionError("private_ordinary_gates")
    if any(report[field] is not True for field in ("private_source", "source_only", "scoped_worktree_clean")):
        raise AssertionError("private_boundary")
    if report["result"] != "pass_with_explicit_holds" or report["holds"] != HOLDS:
        raise AssertionError("private_holds")
    if any(token in json.dumps(report, sort_keys=True) for token in FORBIDDEN):
        raise AssertionError("private_material_leak")
    print("PASS: final opaque private assurance is exact clean source-only and held")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
