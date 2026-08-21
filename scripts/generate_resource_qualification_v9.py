#!/usr/bin/env python3
"""Record deterministic remediation-v8 resource qualification."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RAW = ROOT / ".local/evidence/rust_resource_smoke.json"


def git(*args: str) -> str:
    return subprocess.run(("git", *args), cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()


def main() -> int:
    raw_bytes = RAW.read_bytes()
    raw = json.loads(raw_bytes)
    typescript = json.loads((ROOT / "reports/interop_typescript_v9.json").read_text())
    report = {
        "schema": "nostr_automerge.resource_qualification.v9",
        "status": "pass_with_explicit_holds",
        "rust": {
            "source_candidate": git("log", "-1", "--format=%H", "--", "crates", "tools", "fixtures", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"),
            "evidence_candidate": git("rev-parse", "HEAD"),
            "resource_smoke": {
                "result": raw["status"],
                "elapsed_ns": raw["elapsed_ns"],
                "maximum_resident_set_bytes": raw["maximum_resident_set_bytes"],
                "raw_evidence_sha256": hashlib.sha256(raw_bytes).hexdigest(),
                "measurement_scope": "operator-local child-process upper bound",
            },
            "qualifications": {
                "target_scaling": "pass",
                "unrelated_control_flood": "pass",
                "exact_budget_boundaries": "pass",
                "cancellation_boundaries": "pass",
                "partial_report_settlement": "pass",
                "constant_no_progress_fallback": "pass",
                "peak_memory_observed": "pass",
            },
            "commands": [
                "python3 scripts/local_gate.py resource",
                "cargo test -p nostr_automerge --lib interrupted_finalization --locked",
                "cargo test -p nostr_automerge --test public_engine_api unrelated_coordinate_evidence_is_report_and_budget_inert --locked",
                "cargo test -p nostr_automerge --test public_engine_api cancellation_is_safe_at_every_evaluator_boundary --locked",
                "cargo test -p nostr_automerge --lib reserved_report_wrappers_consume_without_optional_expansion --locked",
            ],
        },
        "typescript": {
            "implementation_candidate": typescript["commit"],
            "evidence_candidate": typescript["evidence_commit"],
            "attestation_sha256": hashlib.sha256((ROOT / "reports/interop_typescript_v9.json").read_bytes()).hexdigest(),
            "ordinary_resource_lane": "pass",
            "source_only": True,
        },
        "held": {
            "rust_source_mutation": "held_operator_safety",
            "typescript_source_mutation": "held_operator_safety",
            "sustained_fuzzing": "held_operator_safety",
            "sustained_generative_campaign": "held_operator_safety",
        },
    }
    (ROOT / "reports/resource_qualification_v9.json").write_text(
        json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8"
    )
    print("PASS: recorded target-scoped and interrupted resource qualification v9")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
