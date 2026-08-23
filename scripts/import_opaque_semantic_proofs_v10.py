#!/usr/bin/env python3
"""Import only approved opaque semantic-proof identifiers from a private root."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "reports/opaque_semantic_proofs_v10.json"
PUBLIC_CANDIDATE = "920c768946a2d33449905a0b0891942fa8fb9afe"
PRIVATE_EVIDENCE_CANDIDATE = "b812328043cd514ad8909c0f926435005d27d1fd"
PRIVATE_SOURCE_CANDIDATE = "36db673b8e5b62df69a5ee321b2e13c040fc8237"
PRIVATE_ARTIFACT_SHA256 = "ec1f10d92ab050cd1ab8d8917e85f7b0f0762b7341e3c24f9ff4c3dc9bf66443"
PRIVATE_RESULT_IDENTITY = "b81d1be479c59b8b29be9b896a4bb6fa0af502b3faa9771ac4142049ee1433c2"


class ImportError(ValueError):
    """The private semantic-proof artifact is stale or malformed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ImportError(diagnostic)


def digest(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def load(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"object:{path.name}")
    return value


def opaque_fixture(identifier: str) -> str:
    return f"opaque_fixture_{digest(identifier.encode())}"


def validate_private(value: dict[str, Any], raw: bytes) -> None:
    require(digest(raw) == PRIVATE_ARTIFACT_SHA256, "private:file")
    require(value.get("schema") == "nostr_automerge.private_semantic_proof_ids.v10.v1", "private:schema")
    require(value.get("status") == "pass", "private:status")
    require(value.get("source_candidate") == PRIVATE_SOURCE_CANDIDATE, "private:source_candidate")
    require(value.get("public_authority_candidate") == PUBLIC_CANDIDATE, "private:public_candidate")
    require(value.get("requirement_count") == 113, "private:requirements")
    require(value.get("report_clause_count") == 21, "private:clauses")
    require(value.get("finding_count") == 21, "private:findings")
    require(value.get("closed_finding_count") == 20, "private:closed")
    require(value.get("held_finding_id") == "FINDING_080", "private:held")
    require(value.get("result_identity_sha256") == PRIVATE_RESULT_IDENTITY, "private:identity")


def build(value: dict[str, Any]) -> dict[str, Any]:
    requirements = []
    fixture_rows = 0
    test_rows = 0
    for row in value["requirements"]:
        fixtures = row["fixture_ids"]
        tests = row["tests"]
        require((bool(fixtures) + bool(tests)) == 1, f"private:proof_kind:{row['id']}")
        if fixtures:
            fixture_rows += 1
            proof_ids = [opaque_fixture(fixtures[0])]
            proof_kind = "opaque_fixture"
        else:
            test_rows += 1
            require(len(tests) == 1, f"private:test_count:{row['id']}")
            proof_ids = [tests[0]["id"]]
            proof_kind = "opaque_test"
        requirements.append({"id": row["id"], "proof_kind": proof_kind, "proof_ids": proof_ids})
    clauses = [
        {"id": row["id"], "proof_ids": [row["test"]["id"]]}
        for row in value["report_clauses"]
    ]
    findings = [
        {"id": row["id"], "status": "closed", "proof_ids": [row["test"]["id"]]}
        for row in value["findings"]
    ]
    findings.insert(
        7,
        {
            "id": "FINDING_080",
            "status": "held",
            "proof_ids": [f"opaque_hold_{digest(PRIVATE_RESULT_IDENTITY.encode())}"],
        },
    )
    report = {
        "schema": "nostr_automerge.opaque_semantic_proofs.v10.v1",
        "status": "pass",
        "checkpoint": "step_1279",
        "public_candidate": PUBLIC_CANDIDATE,
        "opaque_evidence_candidate": PRIVATE_EVIDENCE_CANDIDATE,
        "opaque_implementation_candidate": PRIVATE_SOURCE_CANDIDATE,
        "opaque_record_sha256": PRIVATE_ARTIFACT_SHA256,
        "opaque_record_identity_sha256": PRIVATE_RESULT_IDENTITY,
        "requirement_count": 113,
        "fixture_requirement_count": fixture_rows,
        "assertion_requirement_count": test_rows,
        "report_clause_count": len(clauses),
        "finding_count": len(findings),
        "closed_finding_count": 20,
        "held_finding_count": 1,
        "requirements": requirements,
        "report_clauses": clauses,
        "findings": findings,
    }
    require(fixture_rows == 49 and test_rows == 64, "private:requirement_partition")
    require(len(clauses) == 21 and len(findings) == 21, "private:subject_counts")
    return {**report, "result_identity_sha256": digest(canonical(report))}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--private-artifact", required=True, type=Path)
    parser.add_argument("--check", action="store_true")
    arguments = parser.parse_args()
    private_artifact = arguments.private_artifact.resolve()
    raw = private_artifact.read_bytes()
    value = json.loads(raw)
    require(isinstance(value, dict), "private:object")
    validate_private(value, raw)
    candidate = subprocess.run(
        ("git", "rev-parse", "HEAD"),
        cwd=private_artifact.parent,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    require(candidate == PRIVATE_EVIDENCE_CANDIDATE, "private:evidence_candidate")
    report = build(value)
    encoded = canonical(report) + b"\n"
    if arguments.check:
        require(OUTPUT.read_bytes() == encoded, "output:stale")
        print("PASS: opaque semantic proof import v10")
    else:
        OUTPUT.write_bytes(encoded)
        print(f"WROTE: {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ImportError as error:
        raise SystemExit(f"FAIL: {error}") from error
