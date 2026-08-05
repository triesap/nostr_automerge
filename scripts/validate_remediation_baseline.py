#!/usr/bin/env python3
"""Validate the durable draft-v1 remediation baseline."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GIT_ID = re.compile(r"[0-9a-f]{40}\Z")
SHA_256 = re.compile(r"[0-9a-f]{64}\Z")


def main() -> int:
    report = json.loads((ROOT / "reports/remediation_baseline.json").read_text())
    required = {
        "branch",
        "cargo_lock_sha256",
        "finding_count",
        "head",
        "next_step",
        "recorded",
        "reviewed_head",
        "schema",
        "status",
        "toolchains",
        "typescript_branch",
        "typescript_head",
    }
    if set(report) != required:
        raise AssertionError("remediation baseline fields are incomplete or unknown")
    for name in ("head", "reviewed_head", "typescript_head"):
        if not isinstance(report[name], str) or GIT_ID.fullmatch(report[name]) is None:
            raise AssertionError(f"invalid remediation baseline identifier: {name}")
    if SHA_256.fullmatch(report["cargo_lock_sha256"]) is None:
        raise AssertionError("invalid Cargo.lock digest")
    if report["schema"] != "nostr_automerge.remediation_baseline.v1":
        raise AssertionError("invalid remediation baseline schema")
    if report["finding_count"] != 13 or report["next_step"] != "step_194":
        raise AssertionError("invalid remediation baseline progression")
    digest = hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest()
    if report["cargo_lock_sha256"] != digest:
        raise AssertionError("stale Cargo.lock digest in remediation baseline")
    narrative = (ROOT / "docs/execution/remediation/baseline.md").read_text()
    for value in (report["head"], report["reviewed_head"], report["typescript_head"]):
        if value not in narrative:
            raise AssertionError("baseline narrative is not bound to report identities")
    print("PASS: remediation baseline is complete and bound to the current lock")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
