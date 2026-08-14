#!/usr/bin/env python3
"""Validate the normative requirements registry and negative fixtures."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
ID_PATTERN = re.compile(r"^NCRDT-[A-Z0-9]+(?:-[A-Z0-9]+)*$")
REQUIRED_FIELDS = frozenset({"id", "section", "text", "source"})


class RegistryError(Exception):
    """A stable requirements-registry validation error."""

    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code


def load_json(path: Path) -> dict[str, Any]:
    """Load a JSON object."""

    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise RegistryError("registry_not_object")
    return value


def validate(registry: dict[str, Any], *, resolve_sources: bool) -> None:
    """Validate one requirements registry."""

    if set(registry) != {"schema", "project", "requirement_count", "requirements"}:
        raise RegistryError("invalid_registry_fields")
    if registry["schema"] not in {
        "nostr_automerge.requirements.v1",
        "nostr_automerge.requirements.v2",
        "nostr_automerge.requirements.v3",
        "nostr_automerge.requirements.v4",
    }:
        raise RegistryError("invalid_registry_schema")
    if resolve_sources and registry["schema"] != "nostr_automerge.requirements.v4":
        raise RegistryError("invalid_registry_schema")
    if registry["project"] != "nostr_automerge_v1_spec":
        raise RegistryError("invalid_registry_project")
    requirements = registry["requirements"]
    if not isinstance(requirements, list) or not requirements:
        raise RegistryError("requirements_not_array")
    if registry["requirement_count"] != len(requirements):
        raise RegistryError("requirement_count_mismatch")
    if resolve_sources and registry["requirement_count"] != 119:
        raise RegistryError("requirement_count_mismatch")

    seen: set[str] = set()
    for requirement in requirements:
        if not isinstance(requirement, dict):
            raise RegistryError("requirement_not_object")
        if set(requirement) != REQUIRED_FIELDS:
            raise RegistryError("missing_requirement_field")
        identifier = requirement["id"]
        if not isinstance(identifier, str) or ID_PATTERN.fullmatch(identifier) is None:
            raise RegistryError("invalid_requirement_id")
        if identifier in seen:
            raise RegistryError("duplicate_requirement_id")
        seen.add(identifier)
        for field in ("section", "text", "source"):
            if not isinstance(requirement[field], str) or not requirement[field]:
                raise RegistryError("invalid_requirement_field")
        source = requirement["source"]
        if not source.startswith(("spec/", "implementation/")) or ".." in Path(source).parts:
            raise RegistryError("invalid_requirement_source")
        if resolve_sources and not (ROOT / source).is_file():
            raise RegistryError("missing_requirement_source")


def main() -> int:
    """Validate the canonical registry and each negative fixture."""

    schema = load_json(ROOT / "tools/validation/requirements_schema.json")
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        raise AssertionError("requirements schema must use JSON Schema 2020-12")

    registry = load_json(ROOT / "spec/requirements.json")
    validate(registry, resolve_sources=True)

    fixtures = sorted((ROOT / "tools/validation/fixtures").glob("requirements_*.json"))
    for path in fixtures:
        fixture = load_json(path)
        expected = fixture.get("expected_error")
        candidate = fixture.get("registry")
        if not isinstance(expected, str) or not isinstance(candidate, dict):
            raise AssertionError(f"malformed negative fixture: {path.name}")
        try:
            validate(candidate, resolve_sources=False)
        except RegistryError as error:
            if error.code != expected:
                raise AssertionError(
                    f"{path.name}: expected {expected}, received {error.code}"
                ) from error
        else:
            raise AssertionError(f"negative fixture unexpectedly passed: {path.name}")

    print("PASS: normative requirements registry")
    print(f"- requirements={registry['requirement_count']}")
    print(f"- negative_fixtures={len(fixtures)}")
    print("- source_references=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
