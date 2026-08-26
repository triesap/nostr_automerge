#!/usr/bin/env python3
"""Validate ADR numbering, staged authority, index mappings, and portability."""

from __future__ import annotations

import copy
import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ADR_ROOT = ROOT / "docs/adr"
BASE_ADR_COUNT = 71
BASELINE_REQUIREMENT_COUNT = 139
TARGET_REQUIREMENT_COUNT = 148
STAGED_REQUIREMENTS = (
    "NCRDT-CPAUTH-001",
    "NCRDT-CPAUTH-002",
    "NCRDT-DISPOSITION-006",
    "NCRDT-INTERRUPT-001",
    "NCRDT-RESOURCE-013",
    "NCRDT-RESOURCE-014",
    "NCRDT-VERSION-002",
    "NCRDT-CONF-010",
    "NCRDT-EVIDENCE-006",
)
NEW_ADRS = {
    65: (
        "adr_0065_checkpoint_control_precedence.md",
        ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002"),
    ),
    66: (
        "adr_0066_independent_carrier_and_semantic_identity.md",
        ("NCRDT-DISPOSITION-006", "NCRDT-VERSION-002"),
    ),
    67: ("adr_0067_revision_bound_no_progress_reports.md", ("NCRDT-INTERRUPT-001",)),
    68: ("adr_0068_two_tier_finalization_ledgers.md", ("NCRDT-RESOURCE-013",)),
    69: ("adr_0069_target_work_and_shared_raw_bytes.md", ("NCRDT-RESOURCE-014",)),
    70: (
        "adr_0070_independent_compatibility_limits_and_immutability.md",
        ("NCRDT-TS-001", "NCRDT-LIMIT-001", "NCRDT-LIMITS-001", "NCRDT-STATE-002"),
    ),
    71: (
        "adr_0071_signed_conformance_and_semantic_evidence_v10.md",
        ("NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ),
}
V11_STAGED_REQUIREMENTS = (
    "NCRDT-RESOURCE-015",
    "NCRDT-RESOURCE-016",
    "NCRDT-VERSION-003",
    "NCRDT-OWNERSHIP-001",
)
V11_ADRS = {
    72: ("adr_0072_metered_persistent_state.md", ("NCRDT-RESOURCE-015",)),
    73: ("adr_0073_no_post_stop_target_work.md", ("NCRDT-RESOURCE-016",)),
    74: ("adr_0074_unsupported_event_only_identity.md", ("NCRDT-VERSION-003",)),
    75: ("adr_0075_bounded_persistent_teardown.md", ("NCRDT-OWNERSHIP-001",)),
}
REQUIRED_NEW_HEADINGS = (
    "Status",
    "Authority transition",
    "Context",
    "Decision",
    "Rationale",
    "Consequences",
)
AUTHORITY_BINDINGS = {
    65: ("`transition_installed`", "`companion_authority_installed`", "ADR 0055"),
    66: ("`transition_installed`", "`companion_authority_installed`", "supersedes ADR 0063 only"),
    67: ("`transition_installed`", "`companion_authority_installed`", "supersedes ADR 0034"),
    68: ("`transition_installed`", "`companion_authority_installed`", "ADR 0044 and ADR 0062"),
    69: ("`transition_installed`", "`companion_authority_installed`", "ADR 0061"),
    70: ("`transition_installed`", "`companion_authority_installed`", "does not supersede"),
    71: (
        "`transition_installed`",
        "`requirements_appended`",
        "`distribution_complete`",
        "current\ncompanion's `### Signed conformance v9` section",
        "`spec/CONFORMANCE.md`",
        "`NCRDT-CONF-009`",
        "does not supersede ADR 0064's local-NIP reconciliation",
    ),
    72: (
        "counts the nodes actually visited",
        "caller owns local-delta preparation",
        "persistent boundary owns only inherited lookup and insertion",
    ),
    73: ("live-metered", "constant-time", "exact-reserved"),
    74: ("Event-only", "no semantic ChangeHash identity"),
    75: ("iterative", "deep unique", "wide shared"),
}
EXPECTED_INDEX_INTRO = """# Architecture decision records

All imported decisions through ADR 0064 are approved for the current draft-v1
implementation baseline. Consensus-affecting changes require a new superseding
ADR and the complete change-control process.

ADRs 0065 through 0071 are approved staged candidate decisions, not effective
current protocol authority at `transition_installed`. The unchanged NIP and
current companion remain controlling. ADRs 0065 through 0070 become effective
only for the staged local implementation candidate at
`companion_authority_installed`; they do not override contrary NIP text. ADR
0071's signed-v10 distribution becomes current only at
`distribution_complete`, and its semantic-evidence pass requires the later
proof-catalog evidence gate. Candidate closure, release, and NIP-conformance
remain held wherever unchanged NIP text is unresolved.

The nine future requirement mappings are staged by
`spec/authority_transition_v10.json`. Before `requirements_appended`, those
identifiers are planned mappings and are not live rows in
`spec/requirements.json`.

ADRs 0072 through 0075 are the approved staged decisions for remediation v11.
They become effective only through their ordered implementation and evidence
gates. They do not override unchanged NIP text or authorize publication.

| ADR | Status | Primary requirements |
| --- | --- | --- |
"""
INDEX_TAIL_ANCHOR = (
    "| [0064](adr_0064_local_nip_reconciliation.md) | Approved | "
    "`NCRDT-NIP-003`, `NCRDT-CONF-009` |\n"
)
INDEX_ROW = re.compile(
    r"^\| \[(\d{4})\]\((adr_\d{4}_[a-z0-9_]+\.md)\) "
    r"\| ([^|]+?) \| (.*?) \|$",
    re.MULTILINE,
)
MAPPED_IDENTIFIER = re.compile(r"`([A-Z][A-Z0-9_-]{2,})`")
REQUIREMENT_IDENTIFIER = re.compile(r"^NCRDT-[A-Z0-9]+(?:-[A-Z0-9]+)*$")
POSIX_ABSOLUTE_PATH = re.compile(
    r"(?:^|[\s`('\"])/(?:[A-Za-z0-9._-]+/)+[A-Za-z0-9._-]+",
    re.MULTILINE,
)
WINDOWS_ABSOLUTE_PATH = re.compile(r"(?:^|[\s`('\"])[A-Za-z]:[\\/]")
GENERIC_SCOPE_MARKERS = ("://", ".github/workflows/", ".act/")


@dataclass
class AdrRecord:
    """One parsed ADR file."""

    number: int
    path: str
    text: str


@dataclass
class IndexRecord:
    """One parsed ADR index row."""

    number: int
    path: str
    status: str
    requirement_cell: str
    identifiers: tuple[str, ...]


@dataclass(frozen=True)
class AuthorityContext:
    """Validated live and staged requirement authority."""

    live_identifiers: tuple[str, ...]
    staged_identifiers: tuple[str, ...]
    before_requirements_appended: bool
    baseline_order_digest: str


def require(condition: bool, diagnostic: str) -> None:
    """Fail with a stable diagnostic when one ADR invariant is false."""

    if not condition:
        raise AssertionError(diagnostic)


def load_json(relative: str) -> dict[str, object]:
    """Load one required JSON object."""

    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"expected object: {relative}")
    return value


def file_digest(relative: str) -> str:
    """Return one repository-relative file's SHA-256 digest."""

    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def ordered_digest(identifiers: tuple[str, ...]) -> str:
    """Return the transition contract's ordered-ID digest."""

    encoded = json.dumps(identifiers, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def validate_registry_projection(
    identifiers: tuple[str, ...],
    declared_count: int,
    before_requirements_appended: bool,
    baseline_order_digest: str,
) -> None:
    """Validate exact early and appended requirement projections."""

    require(declared_count == len(identifiers), "requirement projection count field")
    require(len(set(identifiers)) == len(identifiers), "requirement projection duplicate ID")
    require(
        all(REQUIREMENT_IDENTIFIER.fullmatch(identifier) for identifier in identifiers),
        "requirement projection malformed ID",
    )
    require(
        ordered_digest(identifiers[:BASELINE_REQUIREMENT_COUNT]) == baseline_order_digest,
        "requirement projection baseline order",
    )
    if before_requirements_appended:
        require(len(identifiers) == BASELINE_REQUIREMENT_COUNT, "early requirement projection count")
        require(
            set(identifiers).isdisjoint(STAGED_REQUIREMENTS),
            "staged requirements are prematurely live",
        )
    else:
        require(len(identifiers) == TARGET_REQUIREMENT_COUNT, "appended requirement projection count")
        require(
            identifiers[BASELINE_REQUIREMENT_COUNT:] == STAGED_REQUIREMENTS,
            "appended requirement projection tail",
        )


def requirement_authority() -> AuthorityContext:
    """Validate and return live plus transition-staged requirement authority."""

    registry = load_json("spec/requirements.json")
    rows = registry.get("requirements")
    require(isinstance(rows, list), "requirements registry rows are missing")
    require(all(isinstance(row, dict) for row in rows), "requirements registry row shape")
    identifiers = tuple(str(row.get("id", "")) for row in rows)
    declared_count = registry.get("requirement_count")
    require(type(declared_count) is int, "requirements registry count is invalid")

    transition = load_json("spec/authority_transition_v10.json")
    require(
        transition.get("schema") == "nostr_automerge.authority_transition.v10"
        and transition.get("status") == "in_progress",
        "v10 authority transition is not active",
    )
    stage = transition.get("current_stage")
    stage_order = transition.get("stage_order")
    authority = transition.get("authority")
    require(
        isinstance(stage, str)
        and isinstance(stage_order, list)
        and stage in stage_order
        and "requirements_appended" in stage_order,
        "v10 authority transition stage is invalid",
    )
    require(isinstance(authority, dict), "v10 transition authority is missing")
    require(
        authority.get("baseline_requirement_count") == BASELINE_REQUIREMENT_COUNT
        and authority.get("preserved_prefix_count") == BASELINE_REQUIREMENT_COUNT
        and authority.get("target_requirement_count") == TARGET_REQUIREMENT_COUNT,
        "v10 transition requirement counts changed",
    )
    appended = authority.get("appended_ids")
    require(isinstance(appended, list), "v10 staged requirement IDs are invalid")
    staged = tuple(str(identifier) for identifier in appended)
    require(staged == STAGED_REQUIREMENTS, "v10 staged requirement mapping changed")
    baseline_order_digest = authority.get("baseline_ordered_requirement_ids_sha256")
    require(
        isinstance(baseline_order_digest, str) and len(baseline_order_digest) == 64,
        "v10 baseline requirement order digest is invalid",
    )
    before_live = stage_order.index(stage) < stage_order.index("requirements_appended")
    validate_registry_projection(identifiers, declared_count, before_live, baseline_order_digest)

    live = authority.get("live")
    require(isinstance(live, dict), "v10 live authority is missing")
    require(
        live.get("requirements_sha256") == file_digest("spec/requirements.json"),
        "v10 live requirement hash mismatch",
    )
    if before_live:
        require(
            authority.get("baseline_requirements_sha256") == file_digest("spec/requirements.json"),
            "early requirement registry differs from baseline",
        )
    return AuthorityContext(identifiers, staged, before_live, baseline_order_digest)


def parse_records() -> list[AdrRecord]:
    """Load the numbered ADR files."""

    return [
        AdrRecord(int(adr_path.name[4:8]), adr_path.name, adr_path.read_text(encoding="utf-8"))
        for adr_path in sorted(ADR_ROOT.glob("adr_[0-9][0-9][0-9][0-9]_*.md"))
    ]


def parse_index(index: str) -> list[IndexRecord]:
    """Parse every numbered ADR index row."""

    return [
        IndexRecord(
            int(match.group(1)),
            match.group(2),
            match.group(3),
            match.group(4),
            tuple(MAPPED_IDENTIFIER.findall(match.group(4))),
        )
        for match in INDEX_ROW.finditer(index)
    ]


def expected_new_index_tail(present_v11: tuple[int, ...]) -> str:
    """Build the exact staged index tail."""

    rows = []
    for number, (adr_path, requirements) in NEW_ADRS.items():
        requirement_cell = ", ".join(f"`{identifier}`" for identifier in requirements)
        rows.append(
            f"| [{number:04d}]({adr_path}) | Approved staged | {requirement_cell} |"
        )
    for number in present_v11:
        adr_path, requirements = V11_ADRS[number]
        requirement_cell = ", ".join(f"`{identifier}`" for identifier in requirements)
        rows.append(
            f"| [{number:04d}]({adr_path}) | Approved staged | {requirement_cell} |"
        )
    return "\n".join(rows) + "\n"


def validate_portable_text(text: str, diagnostic: str) -> None:
    """Reject generic absolute locations, URLs, and workflow-owned directories."""

    folded = text.casefold()
    require(
        not any(marker in folded for marker in GENERIC_SCOPE_MARKERS),
        f"{diagnostic}:generic_marker",
    )
    require(POSIX_ABSOLUTE_PATH.search(text) is None, f"{diagnostic}:absolute_posix")
    require(WINDOWS_ABSOLUTE_PATH.search(text) is None, f"{diagnostic}:absolute_windows")


def validate_index_contract(index: str, present_v11: tuple[int, ...]) -> None:
    """Exact-bind the staged index explanation and complete new tail."""

    require(index.startswith(EXPECTED_INDEX_INTRO), "ADR index staged introduction changed")
    require(index.count(INDEX_TAIL_ANCHOR) == 1, "ADR index tail anchor changed")
    _, new_tail = index.split(INDEX_TAIL_ANCHOR, 1)
    require(new_tail == expected_new_index_tail(present_v11), "ADR index staged tail changed")
    validate_portable_text(index, "ADR index scope")


def section_body(text: str, heading: str) -> str:
    """Return one second-level section body or an empty string."""

    match = re.search(
        rf"^## {re.escape(heading)}\n\n(.*?)(?=^## |\Z)",
        text,
        re.MULTILINE | re.DOTALL,
    )
    return "" if match is None else match.group(1).strip()


def validate_records(
    records: list[AdrRecord],
    indexed: list[IndexRecord],
    index_text: str,
    authority: AuthorityContext,
) -> None:
    """Validate one complete in-memory ADR and index projection."""

    present_v11 = tuple(record.number for record in records if record.number in V11_ADRS)
    expected_v11 = tuple(range(72, 72 + len(present_v11)))
    require(present_v11 == expected_v11, "v11 ADR sequence is not an exact prefix")
    expected_adr_count = BASE_ADR_COUNT + len(present_v11)
    validate_index_contract(index_text, present_v11)
    require(len(records) == expected_adr_count, f"expected {expected_adr_count} ADRs")
    require(
        [record.number for record in records] == list(range(1, expected_adr_count + 1)),
        "ADR numbering gap",
    )
    require(len(indexed) == expected_adr_count, "ADR index row count mismatch")
    require(
        [record.number for record in indexed] == list(range(1, expected_adr_count + 1)),
        "ADR index numbering gap",
    )
    index_by_number = {record.number: record for record in indexed}
    require(len(index_by_number) == len(indexed), "duplicate ADR index number")

    live = set(authority.live_identifiers)
    staged = set(authority.staged_identifiers)
    allowed_requirements = live | staged | set(V11_STAGED_REQUIREMENTS)
    mapped_staged: set[str] = set()
    for record in records:
        require(
            record.path.startswith(f"adr_{record.number:04d}_"),
            f"ADR path mismatch: {record.path}",
        )
        require(
            re.search(rf"^# ADR {record.number:04d}:", record.text, re.MULTILINE) is not None,
            f"ADR title mismatch: {record.path}",
        )
        row = index_by_number[record.number]
        require(row.path == record.path, f"ADR index path mismatch: {record.path}")
        expected_status = "Approved staged" if record.number in NEW_ADRS or record.number in V11_ADRS else "Approved"
        require(row.status == expected_status, f"ADR index status mismatch: {record.path}")
        canonical_cell = ", ".join(f"`{identifier}`" for identifier in row.identifiers)
        require(
            row.requirement_cell == canonical_cell and bool(row.identifiers),
            f"ADR index requirement cell is not canonical: {record.path}",
        )
        canonical_requirements = {
            identifier for identifier in row.identifiers if identifier.startswith("NCRDT-")
        }
        require(
            canonical_requirements.issubset(allowed_requirements),
            f"ADR index references unauthorized requirement: {record.path}",
        )
        mapped_staged.update(canonical_requirements & staged)

    for number, (expected_path, expected_requirements) in NEW_ADRS.items():
        record = records[number - 1]
        row = index_by_number[number]
        require(record.path == expected_path, f"new ADR filename mismatch: {number:04d}")
        headings = tuple(re.findall(r"^## (.+)$", record.text, re.MULTILINE))
        require(headings == REQUIRED_NEW_HEADINGS, f"new ADR heading order: {record.path}")
        bodies = {heading: section_body(record.text, heading) for heading in headings}
        require(
            all(body for body in bodies.values()),
            f"new ADR section body is empty: {record.path}",
        )
        require(
            bodies["Status"] == "Approved staged candidate for remediation v9.",
            f"new ADR status mismatch: {record.path}",
        )
        require(
            all(fragment in bodies["Authority transition"] for fragment in AUTHORITY_BINDINGS[number]),
            f"new ADR authority transition mismatch: {record.path}",
        )
        require(
            re.search(r"not effective\s+current", bodies["Authority transition"]) is not None
            and "unchanged NIP" in bodies["Authority transition"]
            and re.search(
                r"NIP-conformance\s+remain\s+held", bodies["Authority transition"]
            )
            is not None,
            f"new ADR authority hold mismatch: {record.path}",
        )
        require(row.identifiers == expected_requirements, f"new ADR mapping mismatch: {record.path}")
        validate_portable_text(record.text, f"new ADR scope:{record.path}")

    for number in present_v11:
        expected_path, expected_requirements = V11_ADRS[number]
        record = records[number - 1]
        row = index_by_number[number]
        require(record.path == expected_path, f"v11 ADR filename mismatch: {number:04d}")
        headings = tuple(re.findall(r"^## (.+)$", record.text, re.MULTILINE))
        require(headings == REQUIRED_NEW_HEADINGS, f"v11 ADR heading order: {record.path}")
        bodies = {heading: section_body(record.text, heading) for heading in headings}
        require(all(bodies.values()), f"v11 ADR section body is empty: {record.path}")
        require(
            bodies["Status"] == "Approved staged candidate for remediation v11.",
            f"v11 ADR status mismatch: {record.path}",
        )
        normalized_decision = " ".join(bodies["Decision"].split())
        require(
            all(fragment in normalized_decision for fragment in AUTHORITY_BINDINGS[number]),
            f"v11 ADR decision mismatch: {record.path}",
        )
        require(
            "not\neffective current" in bodies["Authority transition"]
            and "unchanged NIP" in bodies["Authority transition"]
            and "NIP-conformance remains\nheld" in bodies["Authority transition"],
            f"v11 ADR authority hold mismatch: {record.path}",
        )
        require(row.identifiers == expected_requirements, f"v11 ADR mapping mismatch: {record.path}")
        validate_portable_text(record.text, f"v11 ADR scope:{record.path}")

    require(mapped_staged == staged, "staged ADR requirement coverage mismatch")
    mapped_v11 = {
        identifier
        for number in present_v11
        for identifier in index_by_number[number].identifiers
        if identifier.startswith("NCRDT-")
    }
    expected_mapped_v11 = {
        identifier
        for number in present_v11
        for identifier in V11_ADRS[number][1]
    }
    require(mapped_v11 == expected_mapped_v11, "v11 staged ADR requirement coverage mismatch")
    if authority.before_requirements_appended:
        future_references = {
            identifier
            for row in indexed
            for identifier in row.identifiers
            if identifier.startswith("NCRDT-") and identifier not in live
        }
        require(future_references == staged, "future ADR IDs are not transition-staged")


def mutation_self_test(
    records: list[AdrRecord],
    indexed: list[IndexRecord],
    index_text: str,
    authority: AuthorityContext,
) -> int:
    """Prove exact rejection of ADR, index, scope, and registry mutations."""

    mutations: list[tuple[str, list[AdrRecord], list[IndexRecord], str]] = []
    gap = copy.deepcopy(records)
    gap.pop(64)
    mutations.append(("gap", gap, copy.deepcopy(indexed), index_text))
    missing_index = copy.deepcopy(indexed)
    missing_index.pop(64)
    mutations.append(("index", copy.deepcopy(records), missing_index, index_text))
    status = copy.deepcopy(indexed)
    status[64].status = "Draft"
    mutations.append(("status", copy.deepcopy(records), status, index_text))
    mapping = copy.deepcopy(indexed)
    mapping[64].identifiers = ("NCRDT-CPAUTH-001",)
    mapping[64].requirement_cell = "`NCRDT-CPAUTH-001`"
    mutations.append(("mapping", copy.deepcopy(records), mapping, index_text))
    unparsed = copy.deepcopy(indexed)
    unparsed[64].requirement_cell += " explanatory text"
    mutations.append(("unparsed_mapping", copy.deepcopy(records), unparsed, index_text))
    empty_decision = copy.deepcopy(records)
    empty_decision[64].text = re.sub(
        r"^## Decision\n\n.*?(?=^## Rationale$)",
        "## Decision\n\n",
        empty_decision[64].text,
        count=1,
        flags=re.MULTILINE | re.DOTALL,
    )
    mutations.append(("empty_decision", empty_decision, copy.deepcopy(indexed), index_text))
    wrong_attribution = copy.deepcopy(records)
    wrong_attribution[70].text = wrong_attribution[70].text.replace(
        "current\ncompanion's `### Signed conformance v9` section",
        "ADR 0064",
        1,
    )
    mutations.append(
        ("wrong_conformance_attribution", wrong_attribution, copy.deepcopy(indexed), index_text)
    )
    extra_index = index_text.replace(
        INDEX_TAIL_ANCHOR,
        INDEX_TAIL_ANCHOR + "Unreviewed staged index content.\n",
        1,
    )
    mutations.append(("extra_index", copy.deepcopy(records), copy.deepcopy(indexed), extra_index))
    for name, leaked in (
        ("absolute_scope", "/opt/example/undisclosed"),
        ("url_scope", "https" + "://example.invalid/resource"),
        ("workflow_scope", ".github/workflows/check.yml"),
    ):
        changed = copy.deepcopy(records)
        changed[64].text += f"\n{leaked}\n"
        mutations.append((name, changed, copy.deepcopy(indexed), index_text))

    if len(records) > BASE_ADR_COUNT:
        wrong_v11 = copy.deepcopy(records)
        wrong_v11[BASE_ADR_COUNT].text = wrong_v11[BASE_ADR_COUNT].text.replace(
            "counts the nodes actually visited", "counts only the outer call", 1
        )
        mutations.append(("v11_decision", wrong_v11, copy.deepcopy(indexed), index_text))

    caught = 0
    for name, changed_records, changed_index, changed_text in mutations:
        try:
            validate_records(changed_records, changed_index, changed_text, authority)
        except AssertionError:
            caught += 1
            continue
        raise AssertionError(f"ADR mutation survived: {name}")

    future = authority.live_identifiers[:BASELINE_REQUIREMENT_COUNT] + STAGED_REQUIREMENTS
    registry_mutations = (
        (
            "future_reordered_registry",
            future[:BASELINE_REQUIREMENT_COUNT] + tuple(reversed(STAGED_REQUIREMENTS)),
        ),
        ("future_149_registry", future + ("NCRDT-UNREVIEWED-001",)),
    )
    for name, identifiers in registry_mutations:
        try:
            validate_registry_projection(
                identifiers,
                len(identifiers),
                False,
                authority.baseline_order_digest,
            )
        except AssertionError:
            caught += 1
            continue
        raise AssertionError(f"ADR mutation survived: {name}")
    return caught


def main() -> int:
    """Validate the complete approved and staged ADR set."""

    index = (ADR_ROOT / "README.md").read_text(encoding="utf-8")
    records = parse_records()
    indexed = parse_index(index)
    authority = requirement_authority()
    validate_records(records, indexed, index, authority)
    mutations = mutation_self_test(records, indexed, index, authority)

    print("PASS: architecture decision records")
    print(f"- decisions={len(records)}")
    print("- current_decisions=64")
    print(f"- staged_candidate_decisions={len(records) - 64}")
    print(f"- v11_staged_decisions={max(0, len(records) - BASE_ADR_COUNT)}")
    print(f"- staged_requirement_mappings={len(authority.staged_identifiers)}")
    print(
        "- staged_requirements_live="
        f"{'no' if authority.before_requirements_appended else 'yes'}"
    )
    print(f"- negative_mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
