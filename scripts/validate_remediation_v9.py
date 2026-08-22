#!/usr/bin/env python3
"""Fail-closed validation for the remediation-v9 finding registry."""

from __future__ import annotations

import copy
import hashlib
import json
import re
from pathlib import Path, PurePosixPath
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = "spec/remediation_findings_v9.json"
PLAN_PATH = "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md"
REGISTRY_PROJECTION_SHA256 = "7e7c99525cae6e29dc8b818ec821e20fe8133c698118e56b6e55c1bbc9402a95"
FINDING_IDS = tuple(f"FINDING_{number:03d}" for number in range(73, 94))
SEVERITIES = {
    "high",
    "medium-high",
    "medium",
    "high-assurance",
    "specification",
    "release-hold",
}
ANCHOR_KINDS = {
    "public_source",
    "public_contract",
    "public_evidence",
    "execution_authority",
}
ALLOWED_ANCHOR_ROOTS = ("crates/", "docs/", "fixtures/", "reports/", "scripts/", "spec/")
FORBIDDEN_PUBLIC_MARKERS = (
    "/" + "users/",
    "/" + "home/",
    "docs/" + "hand" + "off",
    "domains" + "/",
    "triesap" + "/dev",
    "nostr_automerge" + "_typescript",
    "file:" + "//",
    "http:" + "//",
    "https:" + "//",
    "git" + "@",
    ".github/" + "workflows",
    ".act" + "/",
    "private " + "tooling",
    "private " + "source",
    "private " + "repository",
)
CONTRACTS = (
    ("V9-CHECKPOINT-RESOLUTION", "### Checkpoint resolution"),
    ("V9-CARRIER-IDENTITY", "### Carrier and semantic identity"),
    ("V9-CANONICAL-REPORT", "### Canonical report contract"),
    ("V9-TWO-TIER-FINALIZATION", "### Two-tier finalization"),
    ("V9-TARGET-WORK-OWNERSHIP", "### Target work and ownership"),
    ("V9-SEALED-LIMITS-ORDERING", "### Sealed limits and ordering"),
    ("V9-SIGNED-DISTRIBUTION", "## Signed Distribution V10"),
    ("V9-SEMANTIC-EVIDENCE", "## RCLD 93 — Semantic Proof Catalog V10"),
    ("V9-TRUTHFUL-CLOSURE", "## Final Status Rule"),
)
RCLD_MAP = {
    "FINDING_073": [82, 83],
    "FINDING_074": [84],
    "FINDING_075": [85, 86],
    "FINDING_076": [87, 88],
    "FINDING_077": [89, 90, 91],
    "FINDING_078": [93],
    "FINDING_079": [84],
    "FINDING_080": [94],
    "FINDING_081": [85, 86],
    "FINDING_082": [85, 87],
    "FINDING_083": [84, 89],
    "FINDING_084": [82, 89],
    "FINDING_085": [83, 90],
    "FINDING_086": [83],
    "FINDING_087": [83, 90],
    "FINDING_088": [91],
    "FINDING_089": [88, 90],
    "FINDING_090": [86],
    "FINDING_091": [90],
    "FINDING_092": [90],
    "FINDING_093": [92, 93, 94],
}
SEVERITY_MAP = {
    "FINDING_073": "high",
    "FINDING_074": "high",
    "FINDING_075": "medium-high",
    "FINDING_076": "medium-high",
    "FINDING_077": "medium",
    "FINDING_078": "high-assurance",
    "FINDING_079": "specification",
    "FINDING_080": "release-hold",
    "FINDING_081": "high",
    "FINDING_082": "medium-high",
    "FINDING_083": "high",
    "FINDING_084": "high",
    "FINDING_085": "high",
    "FINDING_086": "high",
    "FINDING_087": "high",
    "FINDING_088": "high",
    "FINDING_089": "high",
    "FINDING_090": "high",
    "FINDING_091": "high",
    "FINDING_092": "high",
    "FINDING_093": "high-assurance",
}
REQUIREMENT_MAP = {
    "FINDING_073": ["NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"],
    "FINDING_074": ["NCRDT-DISPOSITION-006", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"],
    "FINDING_075": ["NCRDT-INTERRUPT-001", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"],
    "FINDING_076": ["NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"],
    "FINDING_077": ["NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"],
    "FINDING_078": ["NCRDT-EVIDENCE-006"],
    "FINDING_079": ["NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"],
    "FINDING_080": [],
    "FINDING_081": ["NCRDT-INTERRUPT-001", "NCRDT-EVIDENCE-006"],
    "FINDING_082": ["NCRDT-INTERRUPT-001", "NCRDT-RESOURCE-013", "NCRDT-RESOURCE-014"],
    "FINDING_083": ["NCRDT-INTERRUPT-001", "NCRDT-RESOURCE-014"],
    "FINDING_084": ["NCRDT-CPAUTH-001", "NCRDT-RESOURCE-014"],
    "FINDING_085": ["NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-LIMIT-001", "NCRDT-RESOURCE-001"],
    "FINDING_086": ["NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-CONF-010"],
    "FINDING_087": ["NCRDT-LIMIT-001", "NCRDT-RESOURCE-001", "NCRDT-RESOURCE-014"],
    "FINDING_088": ["NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"],
    "FINDING_089": ["NCRDT-RESOURCE-013", "NCRDT-RESOURCE-014", "NCRDT-CONF-010"],
    "FINDING_090": ["NCRDT-INTERRUPT-001", "NCRDT-DISPOSITION-005", "NCRDT-CONF-010"],
    "FINDING_091": ["NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"],
    "FINDING_092": ["NCRDT-STATE-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"],
    "FINDING_093": ["NCRDT-CONF-010", "NCRDT-EVIDENCE-006"],
}
CONTRACT_MAP = {
    "FINDING_073": ["V9-CHECKPOINT-RESOLUTION", "V9-SIGNED-DISTRIBUTION", "V9-SEMANTIC-EVIDENCE"],
    "FINDING_074": ["V9-CARRIER-IDENTITY", "V9-SIGNED-DISTRIBUTION", "V9-SEMANTIC-EVIDENCE"],
    "FINDING_075": ["V9-CANONICAL-REPORT", "V9-SIGNED-DISTRIBUTION", "V9-SEMANTIC-EVIDENCE"],
    "FINDING_076": ["V9-TWO-TIER-FINALIZATION", "V9-SIGNED-DISTRIBUTION", "V9-SEMANTIC-EVIDENCE"],
    "FINDING_077": ["V9-TARGET-WORK-OWNERSHIP", "V9-SEALED-LIMITS-ORDERING", "V9-SIGNED-DISTRIBUTION", "V9-SEMANTIC-EVIDENCE"],
    "FINDING_078": ["V9-SEMANTIC-EVIDENCE"],
    "FINDING_079": ["V9-CARRIER-IDENTITY", "V9-SIGNED-DISTRIBUTION", "V9-SEMANTIC-EVIDENCE"],
    "FINDING_080": ["V9-TRUTHFUL-CLOSURE"],
    "FINDING_081": ["V9-CANONICAL-REPORT", "V9-SEMANTIC-EVIDENCE"],
    "FINDING_082": ["V9-CANONICAL-REPORT", "V9-TWO-TIER-FINALIZATION", "V9-TARGET-WORK-OWNERSHIP"],
    "FINDING_083": ["V9-CARRIER-IDENTITY", "V9-TARGET-WORK-OWNERSHIP"],
    "FINDING_084": ["V9-CHECKPOINT-RESOLUTION", "V9-TARGET-WORK-OWNERSHIP"],
    "FINDING_085": ["V9-CHECKPOINT-RESOLUTION", "V9-SEALED-LIMITS-ORDERING"],
    "FINDING_086": ["V9-CHECKPOINT-RESOLUTION", "V9-SIGNED-DISTRIBUTION"],
    "FINDING_087": ["V9-SEALED-LIMITS-ORDERING", "V9-TARGET-WORK-OWNERSHIP"],
    "FINDING_088": ["V9-TARGET-WORK-OWNERSHIP", "V9-SIGNED-DISTRIBUTION", "V9-SEMANTIC-EVIDENCE"],
    "FINDING_089": ["V9-TWO-TIER-FINALIZATION", "V9-SEALED-LIMITS-ORDERING", "V9-SIGNED-DISTRIBUTION"],
    "FINDING_090": ["V9-CANONICAL-REPORT", "V9-SIGNED-DISTRIBUTION"],
    "FINDING_091": ["V9-TARGET-WORK-OWNERSHIP", "V9-SEALED-LIMITS-ORDERING"],
    "FINDING_092": ["V9-SEALED-LIMITS-ORDERING", "V9-SIGNED-DISTRIBUTION"],
    "FINDING_093": ["V9-SIGNED-DISTRIBUTION", "V9-SEMANTIC-EVIDENCE", "V9-TRUTHFUL-CLOSURE"],
}


class FindingError(ValueError):
    """One finding registry invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise FindingError(diagnostic)


def load(relative: str) -> dict[str, Any]:
    try:
        value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise FindingError(f"json:{relative}") from error
    require(isinstance(value, dict), f"object:{relative}")
    return value


def validate_anchor(anchor: Any, diagnostic: str) -> None:
    require(isinstance(anchor, dict), f"{diagnostic}:object")
    require(set(anchor) == {"kind", "path", "locator"}, f"{diagnostic}:keys")
    kind = anchor.get("kind")
    relative = anchor.get("path")
    locator = anchor.get("locator")
    require(kind in ANCHOR_KINDS, f"{diagnostic}:kind")
    require(isinstance(relative, str) and relative, f"{diagnostic}:path")
    path = PurePosixPath(relative)
    require(not path.is_absolute() and ".." not in path.parts, f"{diagnostic}:scope")
    require(relative.startswith(ALLOWED_ANCHOR_ROOTS), f"{diagnostic}:root")
    require(isinstance(locator, str) and locator.strip() == locator and locator, f"{diagnostic}:locator")
    require(not re.search(r":\d+(?:$|[-:])", relative + locator), f"{diagnostic}:line_number")
    try:
        source = (ROOT / relative).read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as error:
        raise FindingError(f"{diagnostic}:unreadable") from error
    require(locator in source, f"{diagnostic}:missing_locator")


def validate_contract_catalog(value: Any) -> set[str]:
    require(isinstance(value, list), "contract_catalog:type")
    expected = [identifier for identifier, _ in CONTRACTS]
    require([row.get("id") for row in value if isinstance(row, dict)] == expected, "contract_catalog:order")
    for index, ((identifier, locator), row) in enumerate(zip(CONTRACTS, value, strict=True)):
        require(isinstance(row, dict) and set(row) == {"id", "anchor"}, f"contract:{index}:keys")
        require(row.get("id") == identifier, f"contract:{index}:id")
        anchor = row.get("anchor")
        require(isinstance(anchor, dict), f"contract:{index}:anchor")
        require(anchor == {"path": PLAN_PATH, "locator": locator}, f"contract:{index}:binding")
        validate_anchor(
            {"kind": "execution_authority", "path": anchor["path"], "locator": anchor["locator"]},
            f"contract:{index}",
        )
    return set(expected)


def authorized_requirement_ids() -> set[str]:
    live = load("spec/requirements.json").get("requirements")
    require(isinstance(live, list), "requirements:rows")
    live_ids = {row.get("id") for row in live if isinstance(row, dict)}
    transition = load("spec/authority_transition_v10.json").get("authority")
    require(isinstance(transition, dict), "transition:authority")
    appended = transition.get("appended_ids")
    require(isinstance(appended, list), "transition:appended")
    return {identifier for identifier in live_ids | set(appended) if isinstance(identifier, str)}


def validate_plan_mapping() -> None:
    plan = (ROOT / PLAN_PATH).read_text(encoding="utf-8")
    rows = re.findall(
        r"^\| `(FINDING_\d{3})` \| .+ \| ([0-9]+(?:, [0-9]+)*) \|$",
        plan,
        re.MULTILINE,
    )
    mapping = {
        identifier: [int(value) for value in rclds.split(", ")]
        for identifier, rclds in rows
        if identifier in FINDING_IDS
    }
    require(mapping == RCLD_MAP, "plan:rcld_mapping")


def validate_registry(value: dict[str, Any]) -> None:
    require(
        set(value) == {"schema", "status", "finding_count", "contract_catalog", "findings"},
        "registry:keys",
    )
    require(value.get("schema") == "nostr_automerge.remediation_findings.v9.v1", "registry:schema")
    require(value.get("status") == "implementation_remediation_required", "registry:status")
    require(value.get("finding_count") == len(FINDING_IDS), "registry:count")
    validate_plan_mapping()
    contract_ids = validate_contract_catalog(value.get("contract_catalog"))
    findings = value.get("findings")
    require(isinstance(findings, list), "findings:type")
    identifiers = [row.get("id") for row in findings if isinstance(row, dict)]
    require(len(findings) == len(FINDING_IDS), "findings:count")
    require(identifiers == list(FINDING_IDS), "findings:order")
    require(len(set(identifiers)) == len(identifiers), "findings:unique")
    requirements = authorized_requirement_ids()
    expected_keys = {
        "id",
        "severity",
        "title",
        "cause",
        "source_anchors",
        "governing_requirement_ids",
        "governing_contract_ids",
        "closure_criteria",
        "status",
        "rclds",
    }
    for index, row in enumerate(findings):
        require(isinstance(row, dict) and set(row) == expected_keys, f"finding:{index}:keys")
        identifier = row["id"]
        require(
            row.get("severity") in SEVERITIES
            and row.get("severity") == SEVERITY_MAP[identifier],
            f"finding:{identifier}:severity",
        )
        for field in ("title", "cause"):
            item = row.get(field)
            require(isinstance(item, str) and item.strip() == item and item, f"finding:{identifier}:{field}")
        anchors = row.get("source_anchors")
        require(isinstance(anchors, list) and anchors, f"finding:{identifier}:anchors")
        anchor_keys = []
        for anchor_index, anchor in enumerate(anchors):
            validate_anchor(anchor, f"finding:{identifier}:anchor:{anchor_index}")
            anchor_keys.append((anchor["kind"], anchor["path"], anchor["locator"]))
        require(len(anchor_keys) == len(set(anchor_keys)), f"finding:{identifier}:anchor_unique")
        requirement_ids = row.get("governing_requirement_ids")
        contract_mapping = row.get("governing_contract_ids")
        require(requirement_ids == REQUIREMENT_MAP[identifier], f"finding:{identifier}:requirement_mapping")
        require(all(item in requirements for item in requirement_ids), f"finding:{identifier}:requirement_authority")
        require(contract_mapping == CONTRACT_MAP[identifier], f"finding:{identifier}:contract_mapping")
        require(all(item in contract_ids for item in contract_mapping), f"finding:{identifier}:contract_authority")
        closure = row.get("closure_criteria")
        require(isinstance(closure, list) and len(closure) >= 3, f"finding:{identifier}:closure")
        require(
            all(isinstance(item, str) and item.strip() == item and item for item in closure)
            and len(set(closure)) == len(closure),
            f"finding:{identifier}:closure_items",
        )
        expected_status = "held" if identifier == "FINDING_080" else "implementation_remediation_required"
        require(row.get("status") == expected_status, f"finding:{identifier}:status")
        require(row.get("rclds") == RCLD_MAP[identifier], f"finding:{identifier}:rcld_mapping")

    serialized = json.dumps(value, ensure_ascii=False, sort_keys=True).casefold()
    require(
        not any(marker in serialized for marker in FORBIDDEN_PUBLIC_MARKERS),
        "registry:scope_leakage",
    )
    projection = json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    require(
        hashlib.sha256(projection).hexdigest() == REGISTRY_PROJECTION_SHA256,
        "registry:exact_projection",
    )


def mutation_self_test(value: dict[str, Any]) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    missing = copy.deepcopy(value)
    missing["findings"].pop()
    mutations.append(("missing", missing))
    reordered = copy.deepcopy(value)
    reordered["findings"].reverse()
    mutations.append(("order", reordered))
    duplicate = copy.deepcopy(value)
    duplicate["findings"][1]["id"] = duplicate["findings"][0]["id"]
    mutations.append(("duplicate", duplicate))
    status = copy.deepcopy(value)
    status["findings"][0]["status"] = "closed"
    mutations.append(("status", status))
    mapping = copy.deepcopy(value)
    mapping["findings"][0]["rclds"] = [82]
    mutations.append(("mapping", mapping))
    anchor = copy.deepcopy(value)
    anchor["findings"][0]["source_anchors"][0]["locator"] = "missing_anchor_symbol"
    mutations.append(("anchor", anchor))
    leakage = copy.deepcopy(value)
    leakage["findings"][0]["source_anchors"][0]["path"] = "/" + "Users/example/review.rs"
    mutations.append(("scope_leakage", leakage))

    caught = 0
    for name, mutation in mutations:
        try:
            validate_registry(mutation)
        except FindingError:
            caught += 1
            continue
        raise FindingError(f"mutation_survived:{name}")
    return caught


def main() -> int:
    value = load(REGISTRY_PATH)
    validate_registry(value)
    mutations = mutation_self_test(value)
    print("PASS: remediation-v9 finding registry")
    print(f"- findings={len(FINDING_IDS)}")
    print("- implementation_remediation_required=20")
    print("- held=1")
    print(f"- negative_mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
