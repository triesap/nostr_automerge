#!/usr/bin/env python3
"""Validate imported specification provenance and repository adaptations."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
ADAPTATION_PATH = ROOT / "docs/import_adaptation.json"
FORBIDDEN_PUBLIC_TEXT = (
    "/" + "Users/",
    "docs/" + "handoff/",
    "domains/" + "triesap/",
    "triesap/" + "dev",
)


def fail(message: str) -> None:
    """Raise a deterministic validation failure."""

    raise AssertionError(message)


def load_json(path: Path) -> dict[str, Any]:
    """Load a JSON object from *path*."""

    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        fail(f"expected JSON object: {path.relative_to(ROOT)}")
    return value


def sha256(path: Path) -> str:
    """Return the lowercase SHA-256 digest of *path*."""

    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate_adaptation() -> list[str]:
    """Validate the adaptation manifest and every imported target file."""

    adaptation = load_json(ADAPTATION_PATH)
    if adaptation.get("schema") != "nostr_automerge.import_adaptation.v1":
        fail("unsupported import adaptation schema")
    if adaptation.get("target_repository") != "triesap/nostr_automerge":
        fail("unexpected target repository")

    source_artifact = adaptation.get("source_artifact")
    if not isinstance(source_artifact, dict):
        fail("source_artifact must be an object")
    manifest_digest = source_artifact.get("package_manifest_sha256")
    if not isinstance(manifest_digest, str) or len(manifest_digest) != 64:
        fail("source package manifest digest is missing or malformed")

    manifest_path = ROOT / "docs/provenance/source_package_manifest.json"
    if sha256(manifest_path) != manifest_digest:
        fail("source package manifest digest mismatch")

    imported_files = adaptation.get("imported_files")
    if not isinstance(imported_files, list) or not imported_files:
        fail("imported_files must be a non-empty array")

    seen_paths: set[str] = set()
    adapted_count = 0
    for item in imported_files:
        if not isinstance(item, dict):
            fail("imported file entry must be an object")
        relative = item.get("path")
        if not isinstance(relative, str) or not relative:
            fail("imported file path is missing")
        if relative in seen_paths:
            fail(f"duplicate imported file path: {relative}")
        seen_paths.add(relative)

        path = ROOT / relative
        try:
            path.relative_to(ROOT)
        except ValueError:
            fail(f"imported file escapes repository: {relative}")
        if not path.is_file():
            fail(f"imported file is missing: {relative}")

        actual = sha256(path)
        if actual != item.get("target_sha256"):
            fail(f"target digest mismatch: {relative}")
        adapted = item.get("adapted")
        if not isinstance(adapted, bool):
            fail(f"adapted flag must be boolean: {relative}")
        if adapted:
            adapted_count += 1
        elif actual != item.get("source_sha256"):
            fail(f"unrecorded adaptation: {relative}")

    return [
        f"imported_files={len(imported_files)}",
        f"adapted_files={adapted_count}",
        "source_manifest=pass",
    ]


def validate_protocol_authority() -> list[str]:
    """Validate repository identity and frozen protocol values."""

    revision = load_json(ROOT / "spec/protocol_revision.json")
    if revision.get("repository") != "triesap/nostr_automerge":
        fail("protocol revision has stale repository identity")
    if revision.get("actor_domain") != "nostr-crdt/automerge/actor/v1":
        fail("normative actor domain changed")
    if revision.get("sealed") is not True:
        fail("protocol revision is not sealed")

    requirements = load_json(ROOT / "spec/requirements.json")
    entries = requirements.get("requirements")
    if not isinstance(entries, list) or not entries:
        fail("requirements registry is empty")
    identifiers = [entry.get("id") for entry in entries if isinstance(entry, dict)]
    if len(identifiers) != len(entries) or len(set(identifiers)) != len(entries):
        fail("requirement IDs are missing or duplicated")
    for entry in entries:
        source = entry.get("source")
        if not isinstance(source, str) or not source.startswith(("spec/", "implementation/")):
            fail(f"requirement source is not repository-relative: {entry.get('id')}")

    sequence = load_json(ROOT / "implementation/commit_sequence.json")
    steps = sequence.get("steps")
    if sequence.get("step_count") != 192 or not isinstance(steps, list):
        fail("implementation sequence must contain 192 steps")
    if len(steps) != 192:
        fail("implementation step array length mismatch")
    for index, step in enumerate(steps, 1):
        if not isinstance(step, dict):
            fail(f"implementation step {index} is not an object")
        if step.get("sequence") != index or step.get("step_id") != f"step_{index:03d}":
            fail(f"implementation sequence mismatch at step {index}")

    return [
        f"requirements={len(entries)}",
        "sealed_revision=pass",
        "commit_sequence=192_pass",
    ]


def validate_standalone_content() -> list[str]:
    """Reject private checkout paths from public repository content."""

    checked = 0
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or ".git" in path.parts:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        for forbidden in FORBIDDEN_PUBLIC_TEXT:
            if forbidden in text:
                fail(f"private path marker {forbidden!r} in {path.relative_to(ROOT)}")
        checked += 1
    return [f"standalone_text_files={checked}"]


def main() -> int:
    """Run every import validation and print deterministic results."""

    checks = [
        *validate_adaptation(),
        *validate_protocol_authority(),
        *validate_standalone_content(),
    ]
    print("PASS: specification import")
    for check in checks:
        print(f"- {check}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
