#!/usr/bin/env python3
"""Run the local two-repository differential interoperability lane."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path


def run(command: list[str], cwd: Path, *, capture: bool = False, env: dict[str, str] | None = None) -> str:
    result = subprocess.run(
        command,
        cwd=cwd,
        check=True,
        text=True,
        capture_output=capture,
        env={**os.environ, **(env or {})},
    )
    return result.stdout if capture else ""


def git(root: Path, *arguments: str) -> str:
    return run(["git", *arguments], root, capture=True).strip()


def load_json(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected object: {path.name}")
    return value


def version_starts_with(command: list[str], cwd: Path, expected: str) -> None:
    actual = run(command, cwd, capture=True).strip()
    if expected not in actual:
        raise AssertionError(f"toolchain mismatch: expected {expected!r} in {actual!r}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rust-repo", required=True, type=Path)
    parser.add_argument("--typescript-repo", required=True, type=Path)
    parser.add_argument("--fixtures", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    rust = args.rust_repo.resolve(strict=True)
    typescript = args.typescript_repo.resolve(strict=True)
    fixtures = args.fixtures.resolve(strict=True)
    rust_root = Path(git(rust, "rev-parse", "--show-toplevel")).resolve()
    typescript_root = Path(git(typescript, "rev-parse", "--show-toplevel")).resolve()
    if rust_root != rust or typescript_root != typescript or rust_root == typescript_root:
        raise AssertionError("interop roles must be distinct repository roots")
    if rust.name != "nostr_automerge" or typescript.name != "nostr_automerge_typescript":
        raise AssertionError("unexpected local repository identities")

    rust_runner = load_json(rust / "local_runner_manifest.json")
    typescript_runner = load_json(typescript / "local_runner_manifest.json")
    rust_tools = rust_runner["toolchain"]
    typescript_tools = typescript_runner["toolchain"]
    assert isinstance(rust_tools, dict) and isinstance(typescript_tools, dict)
    version_starts_with(["rustc", "--version"], rust, str(rust_tools["rust"]))
    version_starts_with(["node", "--version"], typescript, str(typescript_tools["node"]))
    version_starts_with(["pnpm", "--version"], typescript, str(typescript_tools["pnpm"]))

    manifest_path = fixtures / "distribution" / "manifest.json"
    manifest_bytes = manifest_path.read_bytes()
    manifest_sha = hashlib.sha256(manifest_bytes).hexdigest()
    manifest = json.loads(manifest_bytes)
    lock = load_json(typescript / "fixtures" / "distribution.lock.json")
    if lock["manifest_sha256"] != manifest_sha or lock["distribution_id"] != manifest["distribution_id"]:
        raise AssertionError("fixture distribution does not match the TypeScript lock")
    run(["git", "merge-base", "--is-ancestor", str(lock["rust_commit"]), "HEAD"], rust)

    run(["cargo", "build", "--workspace", "--locked"], rust)
    run(["cargo", "test", "--workspace", "--all-targets", "--locked"], rust)
    run(["pnpm", "build"], typescript)
    run(["pnpm", "test"], typescript, env={"NOSTR_AUTOMERGE_FIXTURES": str(fixtures)})

    rust_command = [
        "cargo", "run", "--quiet", "-p", "nostr_automerge_conformance",
        "--locked", "--", "run_corpus", str(fixtures),
    ]
    typescript_command = ["node", "dist/src/cli.js", "run_corpus", str(fixtures)]
    rust_first = run(rust_command, rust, capture=True).encode()
    rust_second = run(rust_command, rust, capture=True).encode()
    typescript_first = run(typescript_command, typescript, capture=True).encode()
    typescript_second = run(typescript_command, typescript, capture=True).encode()
    if rust_first != rust_second or typescript_first != typescript_second:
        raise AssertionError("implementation corpus output is nondeterministic")
    if rust_first != typescript_first:
        raise AssertionError("specification-classified canonical report mismatch")
    if rust_first == typescript_first + b" ":
        raise AssertionError("deliberate mismatch was not detected")

    corpus = json.loads(rust_first)
    profiles = {str(name): "pass" for name in manifest["profiles"]}
    summary = {
        "canonical_report_bytes": "identical",
        "ci_policy": "local_act_pass",
        "corpus_sha256": hashlib.sha256(rust_first).hexdigest(),
        "deliberate_mismatch": "detected",
        "distribution_id": manifest["distribution_id"],
        "evaluated_rust_commit": git(rust, "rev-parse", "HEAD"),
        "evaluated_typescript_commit": git(typescript, "rev-parse", "HEAD"),
        "fixture_count": corpus["total"],
        "manifest_sha256": manifest_sha,
        "mismatch_classifications": ["specification", "fixture", "rust", "typescript", "upstream_automerge"],
        "mismatches": [],
        "profiles": profiles,
        "runner_versions": {
            "act": rust_tools["act"],
            "node": typescript_tools["node"],
            "pnpm": typescript_tools["pnpm"],
            "rust": rust_tools["rust"],
        },
        "schema": "nostr_automerge.local_interop.v1",
        "status": "local_differential_pass",
    }
    output = args.output if args.output.is_absolute() else rust / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(summary, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
