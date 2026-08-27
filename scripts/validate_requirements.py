#!/usr/bin/env python3
"""Validate the normative requirements registry and negative fixtures."""

from __future__ import annotations

import copy
import json
import hashlib
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
PRIOR_REQUIREMENTS_PREFIX_SHA256 = "6db8480b875779df949b2051210d0e10a4641516599273771a2735b01c87bd1a"
PRIOR_APPLICABILITY_PREFIX_SHA256 = "adbe322df17baf6aa03c05bbd5a28e4522ff2207e93fc098866566b5d2f4c4ad"
APPENDED_REQUIREMENTS = (
    {
        "id": "NCRDT-RESOURCE-015",
        "section": "Metered persistent-state operations",
        "text": "Every runtime lookup, membership test, extension, or materialization over persistent branch state MUST charge and check cancellation before each visited persistent node and each inserted target item, or use a separately metered flattened representation.",
        "source": "spec/REPORT_CONTRACT.md",
    },
    {
        "id": "NCRDT-RESOURCE-016",
        "section": "No target-sized work after a stop",
        "text": "After a work-budget charge fails or cancellation is observed, evaluation MUST perform no further target-sized traversal, allocation, copy, comparison, serialization, or invariant construction and MUST return the constant-size no-progress result.",
        "source": "spec/REPORT_CONTRACT.md",
    },
    {
        "id": "NCRDT-VERSION-003",
        "section": "Unsupported change-shaped evidence has Event-only identity",
        "text": "An unsupported change-shaped event for which canonical Change bytes and a computed ChangeHash were not verified receives only an Event unsupported_revision outcome and MUST NOT create or influence a semantic ChangeHash disposition in draft v1.",
        "source": "spec/NIP_DRAFT.md",
    },
    {
        "id": "NCRDT-OWNERSHIP-001",
        "section": "Bounded destruction of persistent histories",
        "text": "Persistent control-ancestry and branch-state ownership used by the reference implementation MUST be destructible with bounded stack usage at qualified history depth; recursive teardown proportional to retained history is not permitted.",
        "source": "spec/ARCHITECTURE.md",
    },
)
APPENDED_APPLICABILITY = (
    ("NCRDT-RESOURCE-015", "rust-and-typescript"),
    ("NCRDT-RESOURCE-016", "rust-and-typescript"),
    ("NCRDT-VERSION-003", "rust-and-typescript"),
    ("NCRDT-OWNERSHIP-001", "rust-only"),
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


def projection_sha256(value: object) -> str:
    """Hash one closed JSON projection without filesystem formatting noise."""

    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def validate_v11_append(
    requirements: list[object], classifications: dict[str, object]
) -> None:
    """Require the immutable 148-row prefix and four exact v11 additions."""

    if projection_sha256(requirements[:148]) != PRIOR_REQUIREMENTS_PREFIX_SHA256:
        raise AssertionError("requirements append-only prefix")
    if tuple(requirements[148:]) != APPENDED_REQUIREMENTS:
        raise AssertionError("requirements exact v11 additions")
    items = list(classifications.items())
    if projection_sha256(items[:148]) != PRIOR_APPLICABILITY_PREFIX_SHA256:
        raise AssertionError("applicability append-only prefix")
    if tuple(items[148:]) != APPENDED_APPLICABILITY:
        raise AssertionError("applicability exact v11 additions")


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
        or properties.get("requirement_count", {}).get("enum") != [139, 148, 152]
        or properties.get("requirements", {}).get("minItems") != 139
        or properties.get("requirements", {}).get("maxItems") != 152
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
    if branch_counts != [(139, 139, 139), (148, 148, 148), (152, 152, 152)]:
        raise AssertionError("requirements schema exact branches")

    transition = load_json(ROOT / "spec/authority_transition_v10.json")
    stages = transition.get("stage_order")
    current_stage = transition.get("current_stage")
    if stages != list(TRANSITION_STAGES) or current_stage not in TRANSITION_STAGES:
        raise AssertionError("requirements transition stage")
    expected_count = 152

    registry = load_json(ROOT / "spec/requirements.json")
    validate(registry, resolve_sources=True, expected_count=expected_count)
    try:
        validate(
            registry,
            resolve_sources=True,
            expected_count=148,
        )
    except RegistryError as error:
        if error.code != "requirement_count_mismatch":
            raise AssertionError("wrong-stage requirement count diagnostic") from error
    else:
        raise AssertionError("wrong-stage requirement count unexpectedly passed")

    requirements = registry["requirements"]

    applicability = load_json(ROOT / "spec/requirements_applicability.json")
    if tuple(applicability) != ("schema", "reviewed", "classifications"):
        raise AssertionError("applicability shape")
    classifications = applicability.get("classifications")
    if not isinstance(classifications, dict):
        raise AssertionError("applicability classifications")
    validate_v11_append(requirements, classifications)

    append_mutations: list[tuple[list[object], dict[str, object]]] = []
    for mutate in (
        lambda rows: rows.pop(),
        lambda rows: rows.__setitem__(slice(148, None), reversed(rows[148:])),
        lambda rows: rows[148].update(text="changed"),
        lambda rows: rows.__setitem__(149, copy.deepcopy(rows[148])),
        lambda rows: rows[0].update(section="changed"),
    ):
        candidate = copy.deepcopy(requirements)
        mutate(candidate)
        append_mutations.append((candidate, dict(classifications)))
    for mutate in (
        lambda values: values.pop("NCRDT-OWNERSHIP-001"),
        lambda values: values.update({"NCRDT-OWNERSHIP-001": "rust-and-typescript"}),
        lambda values: values.__setitem__("NCRDT-SCOPE-001", "rust-only"),
    ):
        candidate = dict(classifications)
        mutate(candidate)
        append_mutations.append((copy.deepcopy(requirements), candidate))
    for candidate_requirements, candidate_classifications in append_mutations:
        try:
            validate_v11_append(candidate_requirements, candidate_classifications)
        except AssertionError:
            continue
        raise AssertionError("v11 append mutation unexpectedly passed")

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
    print(f"- append_negative_mutations={len(append_mutations)}")
    print("- source_references=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
