#!/usr/bin/env python3
"""Demonstrate the signed-v9 matrix validator's semantic-name weakness."""

from __future__ import annotations

import copy
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from validate_requirement_matrix_v9 import validate  # noqa: E402


REQUIREMENT = "NCRDT-NIP01-002"
UNRELATED_ASSERTION = "invalid_raw_corpus_has_exact_stable_diagnostics"
DIAGNOSTIC = (
    "FINDING_078 reproduced: a semantically unrelated named assertion "
    "passes requirement validation"
)


def main() -> int:
    report = json.loads((ROOT / "reports/requirements_coverage_v9.json").read_text())
    mutation = copy.deepcopy(report)
    row = next(item for item in mutation["rows"] if item["id"] == REQUIREMENT)
    if row["rust_proof"]["test_path"] != "crates/nostr_automerge/tests/nip01_conformance.rs":
        raise AssertionError("unexpected proof path")
    row["rust_proof"]["evidence_ids"] = [UNRELATED_ASSERTION]
    validate(mutation)
    print(DIAGNOSTIC)
    print(f"observed=accepted:{REQUIREMENT}:{UNRELATED_ASSERTION}")
    print("desired=rejected:semantic-category-mismatch")
    return 78


if __name__ == "__main__":
    raise SystemExit(main())
