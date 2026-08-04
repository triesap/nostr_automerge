#!/usr/bin/env python3
"""Validate the deterministic Automerge 0.10.0 qualification report."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def digest(relative: str) -> str:
    """Return the SHA-256 of one repository artifact."""

    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def main() -> None:
    """Require exact pins, gates, limits, targets, and artifact identities."""

    report = json.loads((ROOT / "reports/automerge_qualification.json").read_text())
    if set(report) != {
        "artifacts", "automerge", "gates", "limits", "report_schema", "status", "vectors"
    }:
        raise SystemExit("FAIL: Automerge qualification report fields")
    if report["report_schema"] != "nostr_automerge.automerge_qualification.v1":
        raise SystemExit("FAIL: Automerge qualification report schema")
    if report["status"] != "qualified" or not all(report["gates"].values()):
        raise SystemExit("FAIL: unresolved Automerge qualification gate")
    if report["automerge"] != {
        "crate_checksum": "09b78abcbba93428b9465b26cb2816a5b4654cce507f099a84a8c1b311cb3633",
        "source_revision": "a4f584c86358dd07f83f36708573e1c8d1bd8161",
        "version": "0.10.0",
    }:
        raise SystemExit("FAIL: Automerge qualification pin")
    if report["limits"] != {
        "change_bytes": 32768, "dependencies": 256, "operations": 16384
    }:
        raise SystemExit("FAIL: Automerge qualification limits")
    expected_artifacts = {
        "canonical_change_sha256": digest(
            "fixtures/v1_draft/automerge_changes/basic/change.hex"
        ),
        "panic_audit_sha256": digest("docs/qualification/automerge_reencode.md"),
        "semantic_matrix_sha256": digest(
            "fixtures/v1_draft/automerge_semantics/matrix.json"
        ),
    }
    if report["artifacts"] != expected_artifacts:
        raise SystemExit("FAIL: stale Automerge qualification artifact digest")
    targets = sorted((ROOT / "fuzz/fuzz_targets").glob("automerge_*.rs"))
    if [target.stem for target in targets] != [
        "automerge_decode", "automerge_framing", "automerge_reencode"
    ]:
        raise SystemExit("FAIL: Automerge fuzz targets")
    if report["vectors"] != {
        "canonical_changes": 1, "fuzz_targets": 3, "semantic_labels": 20
    }:
        raise SystemExit("FAIL: Automerge qualification vector counts")
    adapter = "\n".join(
        path.read_text() for path in sorted(
            (ROOT / "crates/nostr_automerge/src/automerge_adapter").glob("*.rs")
        )
    )
    if "catch_unwind" in adapter or ".bytes()" in adapter:
        raise SystemExit("FAIL: forbidden Automerge acceptance path")
    print("PASS: Automerge 0.10.0 qualification")
    print("- gates=8")
    print("- fuzz_targets=3")
    print("- semantic_labels=20")


if __name__ == "__main__":
    main()
