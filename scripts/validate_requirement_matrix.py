#!/usr/bin/env python3
"""Fail-closed validation for normative requirement evidence."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUST_ID = "triesap/nostr_automerge"
TYPESCRIPT_ID = "triesap/nostr_automerge_typescript"
STATUSES = {"mandatory-pass", "applicable-local", "explicitly-deferred", "out-of-core"}
PROOF_FIELDS = {"implementation_identity", "implementation", "test", "family", "runner_job"}


class MatrixError(Exception):
    """One closed coverage invariant failed."""


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def validate(report: dict[str, object], typescript_root: Path | None = None) -> None:
    registry_bytes = (ROOT / "spec/requirements.json").read_bytes()
    registry = json.loads(registry_bytes)
    requirements = registry["requirements"]
    if set(report) != {"schema", "requirements_sha256", "requirement_count", "rows"}:
        raise MatrixError("unknown_or_missing_report_field")
    if report["schema"] != "nostr_automerge.requirement_coverage.v2":
        raise MatrixError("unknown_schema")
    if report["requirements_sha256"] != sha256(registry_bytes):
        raise MatrixError("stale_registry")
    rows = report["rows"]
    if not isinstance(rows, list):
        raise MatrixError("rows_not_array")
    expected_ids = [item["id"] for item in requirements]
    ids = [row.get("id") for row in rows if isinstance(row, dict)]
    if len(rows) != len(requirements) or ids != expected_ids or len(set(ids)) != len(ids):
        raise MatrixError("missing_duplicate_unknown_or_reordered")
    if report["requirement_count"] != len(requirements):
        raise MatrixError("count_mismatch")

    for requirement, row in zip(requirements, rows, strict=True):
        authority = row.get("authority")
        expected_authority = {
            "source": requirement["source"],
            "section": requirement["section"],
            "text_sha256": sha256(requirement["text"].encode()),
        }
        if authority != expected_authority:
            raise MatrixError(f"stale_authority:{requirement['id']}")
        status = row.get("status")
        if status not in STATUSES:
            raise MatrixError(f"unknown_status:{requirement['id']}")
        allowed = {"id", "status", "authority", "proofs", "rationale"}
        if not set(row).issubset(allowed):
            raise MatrixError(f"unknown_row_field:{requirement['id']}")
        if status in {"explicitly-deferred", "out-of-core"}:
            if "proofs" in row or not isinstance(row.get("rationale"), str) or not row["rationale"]:
                raise MatrixError(f"invalid_noncode_evidence:{requirement['id']}")
            continue
        if "rationale" in row:
            raise MatrixError(f"prose_substituted_for_proof:{requirement['id']}")
        proofs = row.get("proofs")
        if not isinstance(proofs, dict) or not proofs:
            raise MatrixError(f"missing_proof:{requirement['id']}")
        required_languages = {"rust", "typescript"} if status == "mandatory-pass" else set(proofs)
        if set(proofs) != required_languages or not set(proofs).issubset({"rust", "typescript"}):
            raise MatrixError(f"missing_or_unknown_implementation:{requirement['id']}")
        for language, proof in proofs.items():
            if not isinstance(proof, dict) or set(proof) != PROOF_FIELDS:
                raise MatrixError(f"invalid_proof_shape:{requirement['id']}:{language}")
            expected_identity = RUST_ID if language == "rust" else TYPESCRIPT_ID
            if proof["implementation_identity"] != expected_identity:
                raise MatrixError(f"cross_implementation_substitution:{requirement['id']}:{language}")
            for field in PROOF_FIELDS - {"implementation_identity"}:
                if not isinstance(proof[field], str) or not proof[field]:
                    raise MatrixError(f"prose_only_or_empty:{requirement['id']}:{language}:{field}")
            proof_root = ROOT if language == "rust" else typescript_root
            if proof_root is not None:
                for field in ("implementation", "test"):
                    path = (proof_root / proof[field]).resolve()
                    if proof_root.resolve() not in path.parents or not path.is_file():
                        raise MatrixError(f"stale_path:{requirement['id']}:{language}:{field}")


def self_test() -> None:
    registry_bytes = (ROOT / "spec/requirements.json").read_bytes()
    requirements = json.loads(registry_bytes)["requirements"]
    rows = [
        {
            "id": item["id"],
            "status": "out-of-core",
            "authority": {
                "source": item["source"],
                "section": item["section"],
                "text_sha256": sha256(item["text"].encode()),
            },
            "rationale": "Not part of the deterministic core implementation.",
        }
        for item in requirements
    ]
    baseline = {
        "schema": "nostr_automerge.requirement_coverage.v2",
        "requirements_sha256": sha256(registry_bytes),
        "requirement_count": len(rows),
        "rows": rows,
    }
    validate(baseline)
    mutations = []
    missing = copy.deepcopy(baseline)
    missing["rows"].pop()
    mutations.append(missing)
    duplicate = copy.deepcopy(baseline)
    duplicate["rows"][-1] = duplicate["rows"][0]
    mutations.append(duplicate)
    stale = copy.deepcopy(baseline)
    stale["requirements_sha256"] = "00" * 32
    mutations.append(stale)
    prose = copy.deepcopy(baseline)
    prose["rows"][0]["status"] = "mandatory-pass"
    mutations.append(prose)
    cross = copy.deepcopy(baseline)
    cross["rows"][0] = {
        "id": requirements[0]["id"],
        "status": "applicable-local",
        "authority": baseline["rows"][0]["authority"],
        "proofs": {"rust": {
            "implementation_identity": TYPESCRIPT_ID,
            "implementation": "src/lib.rs",
            "test": "src/lib.rs",
            "family": "negative",
            "runner_job": "conformance",
        }},
    }
    mutations.append(cross)
    for mutation in mutations:
        try:
            validate(mutation)
        except MatrixError:
            continue
        raise AssertionError("negative requirement matrix unexpectedly passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report", type=Path, default=ROOT / "reports/requirements_coverage.json")
    parser.add_argument("--typescript-root", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print("PASS: missing, duplicate, stale, prose-only, and substituted evidence fail closed")
        return 0
    report = json.loads(args.report.read_text(encoding="utf-8"))
    validate(report, args.typescript_root)
    print("PASS: every normative requirement has closed direct evidence")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
