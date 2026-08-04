#!/usr/bin/env python3
"""Validate the complete companion specification import."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REQUIRED = (
    "ACCEPTANCE_CRITERIA.md",
    "API_CONTRACTS.md",
    "ARCHITECTURE.md",
    "AUTOMERGE_PROFILE.md",
    "CHECKPOINT_PROFILE.md",
    "CONFORMANCE.md",
    "CONTROL_AND_AUTHORIZATION.md",
    "DATA_MODEL.md",
    "FUTURE_FARM_WORKSPACES_CONTEXT.md",
    "NIP_DRAFT.md",
    "NIP_PR_DESCRIPTION.md",
    "NORMATIVE_REQUIREMENTS.md",
    "NOSTR_AUTOMERGE_V1_SPEC.md",
    "OUT_OF_SCOPE_AND_FUTURE_WORK.md",
    "PRODUCT_SPEC.md",
    "SECURITY.md",
    "VERSIONING_AND_COMPATIBILITY.md",
    "WIRE_FORMAT.md",
)
ADAPTED = {"ACCEPTANCE_CRITERIA.md"}


def sha256(path: Path) -> str:
    """Return the SHA-256 digest of *path*."""

    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    """Load a JSON object."""

    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected JSON object: {path.relative_to(ROOT)}")
    return value


def main() -> int:
    """Validate companion completeness, hashes, and requirement references."""

    package = load_json(ROOT / "docs/provenance/source_package_manifest.json")
    source_hashes = {
        item["path"]: item["sha256"]
        for item in package.get("files", [])
        if isinstance(item, dict) and "path" in item and "sha256" in item
    }
    adaptation = load_json(ROOT / "docs/import_adaptation.json")
    adapted_hashes = {
        Path(item["path"]).name: item["target_sha256"]
        for item in adaptation.get("imported_files", [])
        if isinstance(item, dict) and item.get("adapted") is True
    }

    for name in REQUIRED:
        path = ROOT / "spec" / name
        if not path.is_file() or path.stat().st_size == 0:
            raise AssertionError(f"missing or empty companion spec: {name}")
        source_digest = source_hashes.get(f"specs/{name}")
        if not isinstance(source_digest, str):
            raise AssertionError(f"source manifest missing companion spec: {name}")
        expected = adapted_hashes.get(name) if name in ADAPTED else source_digest
        if sha256(path) != expected:
            raise AssertionError(f"companion spec digest mismatch: {name}")

    requirements = load_json(ROOT / "spec/requirements.json")
    for requirement in requirements.get("requirements", []):
        if not isinstance(requirement, dict):
            raise AssertionError("requirement entry must be an object")
        source = requirement.get("source")
        if not isinstance(source, str):
            raise AssertionError(f"requirement source missing: {requirement.get('id')}")
        if not (ROOT / source).is_file():
            raise AssertionError(
                f"requirement source does not resolve: {requirement.get('id')} -> {source}"
            )

    print("PASS: companion specification set")
    print(f"- required_specs={len(REQUIRED)}")
    print(f"- adapted_specs={len(ADAPTED)}")
    print(f"- requirement_sources={len(requirements.get('requirements', []))}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
