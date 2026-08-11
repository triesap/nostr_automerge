#!/usr/bin/env python3
"""Validate the deliberately small public interoperability attestation."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


ROOT_KEYS = {
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
PROFILES = {"core", "checkpoint", "malformed", "property", "projection"}
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")


def validate(value: object) -> None:
    if not isinstance(value, dict) or set(value) != ROOT_KEYS:
        raise AssertionError("attestation root fields")
    if value["schema"] != "nostr_automerge.interop_attestation.v2":
        raise AssertionError("attestation schema")
    if value["implementation_identity"] != "triesap/nostr_automerge_typescript":
        raise AssertionError("implementation identity")
    if not isinstance(value["commit"], str) or not HEX40.fullmatch(value["commit"]):
        raise AssertionError("implementation commit")
    for field in ("dependency_lock_sha256", "fixture_distribution_sha256"):
        if not isinstance(value[field], str) or not HEX64.fullmatch(value[field]):
            raise AssertionError(field)
    toolchain = value["toolchain"]
    if not isinstance(toolchain, dict) or set(toolchain) != {"node", "pnpm", "typescript"}:
        raise AssertionError("toolchain fields")
    if not all(isinstance(item, str) and 0 < len(item) <= 128 for item in toolchain.values()):
        raise AssertionError("toolchain value")
    profiles = value["profiles"]
    if not isinstance(profiles, dict) or set(profiles) != PROFILES:
        raise AssertionError("profile fields")
    for profile in profiles.values():
        if not isinstance(profile, dict) or set(profile) != {"report_sha256", "result"}:
            raise AssertionError("profile shape")
        if not isinstance(profile["report_sha256"], str) or not HEX64.fullmatch(
            profile["report_sha256"]
        ):
            raise AssertionError("profile digest")
        if profile["result"] != "pass":
            raise AssertionError("profile result")
    if value["result"] != "pass" or value["deliberate_mismatch"] != "detected":
        raise AssertionError("attestation result")
    if value["provenance"] != "operator-local":
        raise AssertionError("attestation provenance")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("path", type=Path)
    arguments = parser.parse_args()
    validate(json.loads(arguments.path.read_text(encoding="utf-8")))
    print("PASS: opaque interoperability attestation v2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
