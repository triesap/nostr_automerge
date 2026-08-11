#!/usr/bin/env python3
"""Reject private TypeScript execution material from the public Rust tree."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
from validate_interop_attestation_v2 import validate


ROOT = Path(__file__).resolve().parents[1]
ATTESTATION = ROOT / "reports" / "interop_typescript_v2.json"


def tracked() -> list[str]:
    return subprocess.run(
        ["git", "ls-files"], cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.splitlines()


def main() -> int:
    files = tracked()
    forbidden = [
        path
        for path in files
        if path.startswith((".act/", ".github/workflows/"))
        or path.endswith((".log", ".ts", ".mjs"))
        or "typescript_signed_" in path
    ]
    if forbidden:
        raise AssertionError(f"private TypeScript material: {forbidden[0]}")
    if ATTESTATION.exists():
        value = json.loads(ATTESTATION.read_text(encoding="utf-8"))
        validate(value)
        text = ATTESTATION.read_text(encoding="utf-8")
        for token in ("/" + "Users/", "../", "file://", "http://", "https://", ".act/", ".log"):
            if token in text:
                raise AssertionError(f"private attestation token: {token}")
    print("PASS: no private TypeScript material leaked")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
