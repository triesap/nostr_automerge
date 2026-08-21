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
TRANSITION_STAGES = (
    "transition_installed",
    "companion_authority_installed",
    "requirements_appended",
    "checkpoint_expectations_corrected",
    "distribution_locked",
    "checkpoint_control_fixtures_added",
    "carrier_independence_fixtures_added",
    "interruption_fixtures_added",
    "target_work_fixtures_added",
    "distribution_complete",
)


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


def validate(
    registry: dict[str, Any], *, resolve_sources: bool, expected_count: int | None = None
) -> None:
    """Validate one requirements registry."""

    if set(registry) != {"schema", "project", "requirement_count", "requirements"}:
        raise RegistryError("invalid_registry_fields")
    if registry["schema"] not in {
        "nostr_automerge.requirements.v1",
        "nostr_automerge.requirements.v2",
        "nostr_automerge.requirements.v3",
        "nostr_automerge.requirements.v4",
        "nostr_automerge.requirements.v5",
        "nostr_automerge.requirements.v6",
    }:
        raise RegistryError("invalid_registry_schema")
    if resolve_sources and registry["schema"] != "nostr_automerge.requirements.v6":
        raise RegistryError("invalid_registry_schema")
    if registry["project"] != "nostr_automerge_v1_spec":
        raise RegistryError("invalid_registry_project")
    requirements = registry["requirements"]
    if not isinstance(requirements, list) or not requirements:
        raise RegistryError("requirements_not_array")
    if registry["requirement_count"] != len(requirements):
        raise RegistryError("requirement_count_mismatch")
    if expected_count is not None and registry["requirement_count"] != expected_count:
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
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        raise AssertionError("requirements schema properties")
    if (
        properties.get("schema", {}).get("const")
        != "nostr_automerge.requirements.v6"
        or properties.get("requirement_count", {}).get("enum") != [139, 148]
        or properties.get("requirements", {}).get("minItems") != 139
        or properties.get("requirements", {}).get("maxItems") != 148
    ):
        raise AssertionError("requirements schema transition counts")
    branches = schema.get("oneOf")
    branch_counts = []
    if isinstance(branches, list):
        for branch in branches:
            branch_properties = branch.get("properties", {}) if isinstance(branch, dict) else {}
            count = branch_properties.get("requirement_count", {}).get("const")
            limits = branch_properties.get("requirements", {})
            branch_counts.append((count, limits.get("minItems"), limits.get("maxItems")))
    if branch_counts != [(139, 139, 139), (148, 148, 148)]:
        raise AssertionError("requirements schema exact branches")

    transition = load_json(ROOT / "spec/authority_transition_v10.json")
    stages = transition.get("stage_order")
    current_stage = transition.get("current_stage")
    if stages != list(TRANSITION_STAGES) or current_stage not in TRANSITION_STAGES:
        raise AssertionError("requirements transition stage")
    expected_count = 139 if TRANSITION_STAGES.index(current_stage) < TRANSITION_STAGES.index("requirements_appended") else 148

    registry = load_json(ROOT / "spec/requirements.json")
    validate(registry, resolve_sources=True, expected_count=expected_count)
    try:
        validate(
            registry,
            resolve_sources=True,
            expected_count=148 if expected_count == 139 else 139,
        )
    except RegistryError as error:
        if error.code != "requirement_count_mismatch":
            raise AssertionError("wrong-stage requirement count diagnostic") from error
    else:
        raise AssertionError("wrong-stage requirement count unexpectedly passed")

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
