#!/usr/bin/env python3
"""Issue a path-free TypeScript execution contract to operator storage."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path


def command(root: Path, *arguments: str) -> str:
    return subprocess.run(
        arguments, cwd=root, check=True, capture_output=True, text=True
    ).stdout.strip()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--typescript-root", required=True, type=Path)
    arguments = parser.parse_args()
    root = arguments.typescript_root.resolve(strict=True)
    if Path(command(root, "git", "rev-parse", "--show-toplevel")) != root:
        raise AssertionError("TypeScript root is not an independent Git repository")
    package = json.loads((root / "package.json").read_text(encoding="utf-8"))
    output_root = Path(os.environ.get("NOSTR_AUTOMERGE_OUTPUT_ROOT", ".local/evidence"))
    output_root.mkdir(parents=True, exist_ok=True)
    contract = {
        "commands": ["pnpm check", "pnpm signed:profiles"],
        "dependency_lock_sha256": hashlib.sha256(
            (root / "pnpm-lock.yaml").read_bytes()
        ).hexdigest(),
        "implementation_commit": command(root, "git", "rev-parse", "HEAD"),
        "implementation_identity": "triesap/nostr_automerge_typescript",
        "outputs": [
            "typescript_signed_core.json",
            "typescript_signed_checkpoint.json",
            "typescript_signed_malformed.json",
            "typescript_signed_property.json",
        ],
        "profiles": ["core", "checkpoint", "malformed", "property", "projection"],
        "provenance": "operator-local",
        "schema": "nostr_automerge.typescript_execution_contract.v2",
        "toolchain": {
            "node": package["engines"]["node"],
            "pnpm": package["engines"]["pnpm"],
            "typescript": package["devDependencies"]["typescript"],
        },
    }
    (output_root / "typescript_execution_contract_v2.json").write_text(
        json.dumps(contract, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    print("PASS: operator-only TypeScript execution contract")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
