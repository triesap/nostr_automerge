#!/usr/bin/env python3
"""Reject private TypeScript implementation leakage from the Rust repository."""

from __future__ import annotations

import argparse
import copy
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX_40 = re.compile(r"^[0-9a-f]{40}$")
HEX_64 = re.compile(r"^[0-9a-f]{64}$")
SOURCE_SUFFIXES = {".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".map"}
PACKAGE_NAMES = {"package.json", "pnpm-lock.yaml", "package-lock.json", "yarn.lock"}
EXEMPT_CONTENT = {
    "scripts/validate_typescript_private_boundary_v3.py",
    "docs/policy/private_typescript_boundary_v3.md",
}
FORBIDDEN_CONTENT = {
    "private_repository_url": re.compile(
        r"(?:https?://|ssh://|git@)[^\s)\]}>]*nostr_automerge_typescript",
        flags=re.IGNORECASE,
    ),
    "private_absolute_path": re.compile(
        rf"(?:/{'Users'}/|/{'home'}/|[A-Za-z]:\\\\)[^\s\"']*nostr_automerge",
        flags=re.IGNORECASE,
    ),
    "credential": re.compile(
        r"(?:BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY|ghp_[A-Za-z0-9]{20,}|"
        r"github_pat_[A-Za-z0-9_]{20,})"
    ),
}
ATTESTATION_KEYS = {
    "schema",
    "implementation_identity",
    "commit",
    "dependency_lock_sha256",
    "toolchain",
    "fixture_distribution_sha256",
    "profiles",
    "result",
    "deliberate_mismatch",
    "provenance",
}


def repository_paths() -> list[str]:
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    )
    return sorted(item.decode() for item in result.stdout.split(b"\0") if item)


def validate_attestation(value: object) -> None:
    if not isinstance(value, dict) or set(value) != ATTESTATION_KEYS:
        raise AssertionError("opaque attestation fields")
    if value["schema"] not in {
        "nostr_automerge.interop_attestation.v2",
        "nostr_automerge.interop_attestation.v3",
    }:
        raise AssertionError("opaque attestation schema")
    if value["implementation_identity"] != "triesap/nostr_automerge_typescript":
        raise AssertionError("opaque implementation identity")
    if not isinstance(value["commit"], str) or not HEX_40.fullmatch(value["commit"]):
        raise AssertionError("opaque implementation commit")
    for field in ("dependency_lock_sha256", "fixture_distribution_sha256"):
        if not isinstance(value[field], str) or not HEX_64.fullmatch(value[field]):
            raise AssertionError(f"opaque {field}")
    if value["result"] != "pass" or value["deliberate_mismatch"] != "detected":
        raise AssertionError("opaque attestation result")
    if value["provenance"] != "operator-local":
        raise AssertionError("opaque attestation provenance")
    toolchain = value["toolchain"]
    if not isinstance(toolchain, dict) or not toolchain:
        raise AssertionError("opaque toolchain")
    if not all(isinstance(item, str) and 0 < len(item) <= 128 for item in toolchain.values()):
        raise AssertionError("opaque toolchain values")
    profiles = value["profiles"]
    if not isinstance(profiles, dict) or not profiles:
        raise AssertionError("opaque profiles")
    for profile in profiles.values():
        if not isinstance(profile, dict) or set(profile) != {"report_sha256", "result"}:
            raise AssertionError("opaque profile fields")
        if not HEX_64.fullmatch(str(profile["report_sha256"])) or profile["result"] != "pass":
            raise AssertionError("opaque profile result")


def validate_path(relative: str) -> None:
    path = Path(relative)
    parts = path.parts
    if path.suffix.lower() in SOURCE_SUFFIXES:
        raise AssertionError(f"TypeScript or JavaScript source/package content: {relative}")
    if path.name in PACKAGE_NAMES:
        raise AssertionError(f"TypeScript package content: {relative}")
    if path.suffix.lower() == ".log":
        raise AssertionError(f"raw log content: {relative}")
    if ".act" in parts or parts[:2] == (".github", "workflows"):
        raise AssertionError(f"tracked workflow state: {relative}")


def validate_content(relative: str, text: str) -> None:
    if relative in EXEMPT_CONTENT:
        return
    for label, pattern in FORBIDDEN_CONTENT.items():
        if pattern.search(text):
            raise AssertionError(f"{label}: {relative}")


def validate_repository() -> None:
    for relative in repository_paths():
        validate_path(relative)
        absolute = ROOT / relative
        try:
            text = absolute.read_text(encoding="utf-8")
        except (UnicodeDecodeError, IsADirectoryError):
            continue
        validate_content(relative, text)
    validate_attestation(json.loads(attestation_path().read_text()))


def attestation_path() -> Path:
    current = ROOT / "reports/interop_typescript_v3.json"
    return current if current.is_file() else ROOT / "reports/interop_typescript_v2.json"


def expect_rejected(action: object, reason: str) -> None:
    try:
        action()  # type: ignore[operator]
    except AssertionError:
        return
    raise AssertionError(f"private-boundary mutation accepted: {reason}")


def self_test() -> None:
    valid = json.loads(attestation_path().read_text())
    validate_attestation(valid)
    leaked = copy.deepcopy(valid)
    leaked["source"] = "export const secret = true"
    expect_rejected(lambda: validate_attestation(leaked), "source field")
    expect_rejected(lambda: validate_path("private/engine.ts"), "TypeScript source")
    expect_rejected(lambda: validate_path(".act/workflows/private.yml"), "workflow state")
    expect_rejected(
        lambda: validate_content("report.md", "https://example.invalid/nostr_automerge_typescript"),
        "private URL",
    )
    expect_rejected(
        lambda: validate_content(
            "report.md", "/" + "Users/operator/dev/nostr_automerge"
        ),
        "private path",
    )
    expect_rejected(
        lambda: validate_content("report.md", "-----BEGIN OPENSSH PRIVATE KEY-----"),
        "credential",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    arguments = parser.parse_args()
    try:
        validate_repository()
        if arguments.self_test:
            self_test()
    except (AssertionError, KeyError, TypeError, ValueError) as error:
        print(f"FAIL: {error}")
        return 1
    print("PASS: private TypeScript boundary v3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
