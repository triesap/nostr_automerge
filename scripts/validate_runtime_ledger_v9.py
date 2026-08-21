#!/usr/bin/env python3
"""Validate the stage-aware v9 runtime ledger and opaque reproduction import."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
import unicodedata
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT = "reports/opaque_reproduction_v9.json"
REPORT_SCHEMA = "tools/validation/opaque_reproduction_v9.schema.json"
LEDGER = "implementation/runtime_ledger_v9.json"
LEDGER_SCHEMA = "tools/validation/runtime_ledger_v9.schema.json"
PLAN = "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md"
APPROVED_CANDIDATE = "ad7f90268233418be95f4e640f2238a1d240858f"
APPROVED_RESULT_IDENTITY = (
    "5678ffbb08a87fc518c4518d7f348ee4743a89c3cb1c4549061fe62707eed936"
)
REPORT_SCHEMA_PROJECTION = (
    "5de6a509ec2cb50e618f3f1915a02931c03902a2d82d5462b0b55354df2a5a9d"
)
LEDGER_SCHEMA_PROJECTION = (
    "aa774adf2f8fdfd1da5370623f2f5deda68667516e057e0ce278af58d5b5e855"
)
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
STEP = re.compile(r"^step_(\d{4})$")
GATE = re.compile(r"^V-[A-Z]+(?:-[A-Z]+)*$")
FINDING_IDS = tuple(f"FINDING_{number:03d}" for number in range(73, 94))
REPRODUCED_IDS = tuple(identifier for identifier in FINDING_IDS if identifier != "FINDING_080")
OPAQUE_FINDING_IDS = tuple(f"FINDING_{number:03d}" for number in range(85, 94))
APPENDED_REQUIREMENT_IDS = (
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
AUTHORITY_STAGES = (
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
STAGE_FIXTURE_COUNTS = {
    "transition_installed": 180,
    "companion_authority_installed": 180,
    "requirements_appended": 180,
    "checkpoint_expectations_corrected": 180,
    "distribution_locked": 180,
    "checkpoint_control_fixtures_added": 183,
    "carrier_independence_fixtures_added": 186,
    "interruption_fixtures_added": 189,
    "target_work_fixtures_added": 192,
    "distribution_complete": 192,
}
RCLD_RANGES = (
    (81, 1158, 1168),
    (82, 1169, 1177),
    (83, 1178, 1186),
    (84, 1187, 1196),
    (85, 1197, 1206),
    (86, 1207, 1214),
    (87, 1215, 1223),
    (88, 1224, 1231),
    (89, 1232, 1241),
    (90, 1242, 1250),
    (91, 1251, 1259),
    (92, 1260, 1270),
    (93, 1271, 1278),
    (94, 1279, 1283),
)
PUBLIC_CANDIDATES = (
    "17f2b0bb57b12558f678b80e88da36962798762f",
    "361c49936d663ada8e10b4eaccea21ef85236ff9",
    "6a0bdcc93f87955b9557323f827fad5d6e3df6da",
    "9587b149c39b4a180cc0e43ae4e7196cf39bc963",
    "4c26318fe29e1cd0b018127c03278e4139448361",
    "030cccdc1763168cf2aec6571c733387a2f72a51",
    "892c1e31f9290340bd93b108cefa8c9542d83d91",
    "af3cbc1d865a2ed6491965193a045dcf1b267ba1",
)
EXPECTED_GATES = (
    ("V-AUTH",),
    ("V-AUTH",),
    ("V-AUTH",),
    ("V-AUTH",),
    ("V-AUTH",),
    ("V-AUTH",),
    ("V-RUST",),
    ("V-RUST",),
    ("V-TS",),
)
EXPECTED_REQUIREMENTS = (
    (),
    (),
    (),
    APPENDED_REQUIREMENT_IDS,
    APPENDED_REQUIREMENT_IDS,
    APPENDED_REQUIREMENT_IDS,
    (
        "NCRDT-CPAUTH-001",
        "NCRDT-CPAUTH-002",
        "NCRDT-DISPOSITION-006",
        "NCRDT-INTERRUPT-001",
        "NCRDT-RESOURCE-014",
        "NCRDT-VERSION-002",
        "NCRDT-CONF-010",
        "NCRDT-EVIDENCE-006",
    ),
    (
        "NCRDT-CPAUTH-001",
        "NCRDT-INTERRUPT-001",
        "NCRDT-RESOURCE-013",
        "NCRDT-RESOURCE-014",
        "NCRDT-CONF-010",
        "NCRDT-EVIDENCE-006",
    ),
    (
        "NCRDT-CPAUTH-001",
        "NCRDT-CPAUTH-002",
        "NCRDT-LIMIT-001",
        "NCRDT-RESOURCE-001",
        "NCRDT-RESOURCE-013",
        "NCRDT-RESOURCE-014",
        "NCRDT-INTERRUPT-001",
        "NCRDT-DISPOSITION-005",
        "NCRDT-STATE-002",
        "NCRDT-CONF-010",
        "NCRDT-EVIDENCE-006",
    ),
)
EXPECTED_FINDINGS = (
    (),
    (),
    FINDING_IDS,
    REPRODUCED_IDS,
    REPRODUCED_IDS,
    (),
    ("FINDING_073", "FINDING_074", "FINDING_079", "FINDING_083"),
    (
        "FINDING_075",
        "FINDING_076",
        "FINDING_077",
        "FINDING_078",
        "FINDING_081",
        "FINDING_082",
        "FINDING_084",
    ),
    OPAQUE_FINDING_IDS,
)
FORBIDDEN_KEY_WORDS = {
    "source",
    "test",
    "file",
    "path",
    "package",
    "case",
    "command",
    "log",
    "url",
    "workflow",
    "artifact",
    "root",
    "submodule",
    "detail",
}
URI_TEXT = re.compile(r"\b[a-z][a-z0-9+.-]{1,31}\x3a\x2f\x2f")
ABSOLUTE_PATH_TEXT = re.compile(
    r"(?:^|[\s\"'=])(?:\x2f[a-z0-9._-]+(?:\x2f[a-z0-9._-]+)+|[a-z]\x3a\\)",
    re.IGNORECASE,
)
RELATIVE_PATH_TEXT = re.compile(
    r"(?:^|[\s\"'=])(?:\.\.?\x2f)?[a-z0-9._-]+\x2f[a-z0-9._-]+",
    re.IGNORECASE,
)
LOG_TEXT = re.compile(r"(?:^|[\s._-])[a-z0-9_-]+\.log(?:$|[\s._-])", re.IGNORECASE)
PACKAGE_SUFFIX_TEXT = re.compile(
    r"[a-z0-9][_-](?:typescript|javascript)(?:$|[._-])", re.IGNORECASE
)
COMMAND_TEXT = re.compile(
    r"(?:^|\s)(?:cargo|pnpm|npm|yarn|node|python3?|deno|bun|just|make|git)(?:\s|$)",
    re.IGNORECASE,
)
CASE_TEXT = re.compile(r"(?:^|[^a-z0-9])f[0-9]{3}(?:$|[^a-z0-9])", re.IGNORECASE)
COMMIT_SUBJECT_TEXT = re.compile(
    r"(?:^|\n)(?:build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)"
    r"(?:\([a-z0-9._-]+\))?!?\x3a\s",
    re.IGNORECASE,
)


class LedgerError(ValueError):
    """One runtime-ledger or opaque-boundary invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise LedgerError(diagnostic)


def load_object(relative: str) -> dict[str, Any]:
    try:
        value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise LedgerError(f"json:{relative}") from error
    require(isinstance(value, dict), f"object:{relative}")
    return value


def projection_digest(value: Any) -> str:
    return hashlib.sha256(
        json.dumps(
            value,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
    ).hexdigest()


def normalized_key_words(value: str) -> tuple[str, ...]:
    separated = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", value)
    return tuple(re.findall(r"[a-z0-9]+", separated.casefold()))


def scalar_texts(value: Any) -> tuple[list[str], list[str], list[str]]:
    keys: list[str] = []
    scalars: list[str] = []
    ordered_text: list[str] = []

    def visit(item: Any) -> None:
        if isinstance(item, dict):
            for key, child in item.items():
                require(isinstance(key, str), "boundary:key_type")
                normalized_key = unicodedata.normalize("NFKC", key)
                keys.append(normalized_key)
                ordered_text.append(normalized_key)
                visit(child)
        elif isinstance(item, list):
            for child in item:
                visit(child)
        elif isinstance(item, str):
            normalized_scalar = unicodedata.normalize("NFKC", item)
            scalars.append(normalized_scalar)
            ordered_text.append(normalized_scalar)

    visit(value)
    return keys, scalars, ordered_text


def validate_scalar_text(value: str, diagnostic: str) -> None:
    require(URI_TEXT.search(value) is None, f"{diagnostic}:uri")
    require(ABSOLUTE_PATH_TEXT.search(value) is None, f"{diagnostic}:absolute_path")
    require(RELATIVE_PATH_TEXT.search(value) is None, f"{diagnostic}:relative_path")
    require(LOG_TEXT.search(value) is None, f"{diagnostic}:log")
    require(PACKAGE_SUFFIX_TEXT.search(value) is None, f"{diagnostic}:package_suffix")
    require(COMMAND_TEXT.search(value) is None, f"{diagnostic}:command")
    require(CASE_TEXT.search(value) is None, f"{diagnostic}:case")
    require(COMMIT_SUBJECT_TEXT.search(value) is None, f"{diagnostic}:commit_subject")


def validate_no_leak(value: Any, diagnostic: str = "boundary") -> None:
    keys, scalars, ordered_text = scalar_texts(value)
    for key in keys:
        words = set(normalized_key_words(key))
        require(
            not words.intersection(FORBIDDEN_KEY_WORDS),
            f"{diagnostic}:key:{key}",
        )
    for index, scalar in enumerate(scalars):
        validate_scalar_text(scalar, f"{diagnostic}:scalar:{index}")
    for width in range(2, min(4, len(ordered_text)) + 1):
        for start in range(0, len(ordered_text) - width + 1):
            validate_scalar_text(
                "".join(ordered_text[start : start + width]),
                f"{diagnostic}:adjacent:{start}:{width}",
            )
    validate_scalar_text("".join(ordered_text), f"{diagnostic}:coordinated")


def validate_schema_contract(
    schema: dict[str, Any], diagnostic: str, expected_projection: str
) -> None:
    def visit(value: Any, location: str) -> None:
        if isinstance(value, dict):
            if value.get("type") == "object":
                require(value.get("additionalProperties") is False, f"{location}:open")
                properties = value.get("properties")
                required = value.get("required")
                require(isinstance(properties, dict), f"{location}:properties")
                require(isinstance(required, list), f"{location}:required")
                require(set(required) == set(properties), f"{location}:closed_shape")
            for key, child in value.items():
                visit(child, f"{location}:{key}")
        elif isinstance(value, list):
            for index, child in enumerate(value):
                visit(child, f"{location}:{index}")

    require(schema.get("type") == "object", f"{diagnostic}:root")
    visit(schema, diagnostic)
    require(projection_digest(schema) == expected_projection, f"{diagnostic}:projection")


def validate_opaque_reproduction(report: dict[str, Any]) -> None:
    expected_keys = {
        "schema",
        "checkpoint",
        "candidate",
        "status",
        "publication_status",
        "finding_ids",
        "gate_ids",
        "result_classes",
        "toolchain_classes",
        "result_identity_sha256",
    }
    require(set(report) == expected_keys, "opaque:keys")
    require(report.get("schema") == "nostr_automerge.opaque_reproduction.v9.v1", "opaque:schema")
    require(report.get("checkpoint") == "step_1166", "opaque:checkpoint")
    require(report.get("candidate") == APPROVED_CANDIDATE, "opaque:candidate")
    require(HEX40.fullmatch(str(report.get("candidate", ""))) is not None, "opaque:candidate_shape")
    require(report.get("status") == "pass", "opaque:status")
    require(report.get("publication_status") == "held", "opaque:publication")
    require(report.get("finding_ids") == list(OPAQUE_FINDING_IDS), "opaque:findings")
    require(report.get("gate_ids") == ["V-TS"], "opaque:gates")
    require(
        report.get("result_classes")
        == [
            {"class": "ordinary_check", "count": 1, "status": "pass"},
            {"class": "expected_failure_reproduction", "count": 23, "status": "pass"},
            {"class": "negative_mutation", "count": 276, "status": "pass"},
        ],
        "opaque:results",
    )
    require(
        report.get("toolchain_classes")
        == ["language_runtime", "package_manager", "static_analyzer", "unit_runner"],
        "opaque:toolchains",
    )
    identity = report.get("result_identity_sha256")
    require(isinstance(identity, str) and HEX64.fullmatch(identity) is not None, "opaque:identity_shape")
    projection = copy.deepcopy(report)
    projection.pop("result_identity_sha256")
    require(projection_digest(projection) == APPROVED_RESULT_IDENTITY, "opaque:projection")
    require(identity == APPROVED_RESULT_IDENTITY, "opaque:identity")
    validate_no_leak(report, "opaque:boundary")


def step_number(value: Any, diagnostic: str) -> int:
    require(isinstance(value, str), f"{diagnostic}:type")
    match = STEP.fullmatch(value)
    require(match is not None, f"{diagnostic}:shape")
    return int(match.group(1))


def rcld_for_step(number: int) -> int:
    for rcld, first, last in RCLD_RANGES:
        if first <= number <= last:
            return rcld
    raise LedgerError("cursor:step_range")


def is_public_ancestor(candidate: str) -> bool:
    return subprocess.run(
        ("git", "merge-base", "--is-ancestor", candidate, "HEAD"),
        cwd=ROOT,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode == 0


def public_head() -> str:
    result = subprocess.run(
        ("git", "rev-parse", "HEAD"),
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    require(result.returncode == 0 and result.stderr == "", "predecessors:head")
    value = result.stdout.strip()
    require(HEX40.fullmatch(value) is not None, "predecessors:head_shape")
    return value


def public_precedes(first: str, second: str) -> bool:
    return subprocess.run(
        ("git", "merge-base", "--is-ancestor", first, second),
        cwd=ROOT,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode == 0


def plan_execution_rows() -> dict[str, tuple[str, str]]:
    try:
        lines = (ROOT / PLAN).read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeDecodeError) as error:
        raise LedgerError("predecessors:plan") from error
    rows: dict[str, tuple[str, str]] = {}
    for line in lines:
        if not line.startswith("| `step_"):
            continue
        fields = [field.strip() for field in line.split("|")[1:-1]]
        require(len(fields) == 5, "predecessors:plan_shape")
        step = fields[0].strip("`")
        owner = fields[1]
        lane = fields[4].strip("`")
        require(step not in rows, "predecessors:plan_duplicate")
        rows[step] = (owner, lane)
    return rows


def expected_predecessor(index: int) -> dict[str, Any]:
    candidate = (*PUBLIC_CANDIDATES, APPROVED_CANDIDATE)[index]
    return {
        "step": f"step_{1158 + index}",
        "candidate": candidate,
        "owner_class": "public" if index < 8 else "opaque_private",
        "gate_ids": list(EXPECTED_GATES[index]),
        "requirement_ids": list(EXPECTED_REQUIREMENTS[index]),
        "finding_ids": list(EXPECTED_FINDINGS[index]),
        "deviation_ids": ["step_1158"] if index == 0 else [],
        "result": "pass",
    }


def known_requirement_ids() -> set[str]:
    rows = load_object("spec/requirements.json").get("requirements")
    require(isinstance(rows, list), "requirements:rows")
    identifiers = {
        row.get("id") for row in rows if isinstance(row, dict) and isinstance(row.get("id"), str)
    }
    require(len(identifiers) == len(rows), "requirements:ids")
    return identifiers


def validate_predecessors(
    rows: Any, active: int, report: dict[str, Any]
) -> None:
    require(isinstance(rows, list), "predecessors:type")
    approved = [expected_predecessor(index) for index in range(9)]
    require(rows == approved, "predecessors:approved_projection")
    require(active == 1158 + len(approved), "predecessors:approved_cursor")
    expected_keys = {
        "step",
        "candidate",
        "owner_class",
        "gate_ids",
        "requirement_ids",
        "finding_ids",
        "deviation_ids",
        "result",
    }
    plan = plan_execution_rows()
    requirements = known_requirement_ids()
    candidates: list[str] = []
    public_rows: list[tuple[str, str]] = []
    for index, row in enumerate(rows):
        require(isinstance(row, dict) and set(row) == expected_keys, f"predecessor:{index}:keys")
        require(row.get("step") == f"step_{1158 + index}", f"predecessor:{index}:step")
        candidate = row.get("candidate")
        require(isinstance(candidate, str) and HEX40.fullmatch(candidate) is not None, f"predecessor:{index}:candidate")
        candidates.append(candidate)
        owner = row.get("owner_class")
        require(owner in {"public", "opaque_private"}, f"predecessor:{index}:owner")
        plan_owner, plan_lane = plan[row["step"]]
        expected_owner = {
            "public Rust": "public",
            "private TypeScript": "opaque_private",
        }.get(plan_owner)
        require(owner == expected_owner, f"predecessor:{index}:plan_owner")
        require(row.get("gate_ids") == [plan_lane], f"predecessor:{index}:plan_lane")
        if owner == "public":
            require(is_public_ancestor(candidate), f"predecessor:{index}:ancestry")
            public_rows.append((row["step"], candidate))
        else:
            require(row["step"] == report["checkpoint"], f"predecessor:{index}:opaque_step")
            require(candidate == report["candidate"], f"predecessor:{index}:opaque_candidate")
            require(
                report["result_identity_sha256"] == APPROVED_RESULT_IDENTITY,
                f"predecessor:{index}:opaque_result",
            )
        for field, authorized in (
            ("gate_ids", None),
            ("requirement_ids", requirements),
            ("finding_ids", set(FINDING_IDS)),
            ("deviation_ids", None),
        ):
            values = row.get(field)
            require(isinstance(values, list), f"predecessor:{index}:{field}:type")
            require(len(values) == len(set(values)), f"predecessor:{index}:{field}:unique")
            require(all(isinstance(value, str) for value in values), f"predecessor:{index}:{field}:items")
            if authorized is not None:
                require(set(values).issubset(authorized), f"predecessor:{index}:{field}:authority")
        require(
            bool(row["gate_ids"]) and all(GATE.fullmatch(value) for value in row["gate_ids"]),
            f"predecessor:{index}:gates",
        )
        require(
            all(STEP.fullmatch(value) for value in row["deviation_ids"]),
            f"predecessor:{index}:deviations",
        )
        require(row.get("result") == "pass", f"predecessor:{index}:result")
    require(len(candidates) == len(set(candidates)), "predecessors:candidate_unique")
    require(
        all(
            public_precedes(first[1], second[1])
            for first, second in zip(public_rows, public_rows[1:], strict=False)
        ),
        "predecessors:public_order",
    )
    require(public_rows[-1][1] == public_head(), "predecessors:latest_public_head")


def validate_runtime_ledger(ledger: dict[str, Any], report: dict[str, Any]) -> None:
    expected_keys = {
        "schema",
        "status",
        "rcld",
        "cursor",
        "authority_projection",
        "requirements",
        "findings",
        "predecessors",
        "opaque_reproduction",
    }
    require(set(ledger) == expected_keys, "ledger:keys")
    require(ledger.get("schema") == "nostr_automerge.runtime_ledger.v9.v1", "ledger:schema")
    status = ledger.get("status")
    require(status in {"in_progress", "code_complete_publication_held"}, "ledger:status")
    terminal = status == "code_complete_publication_held"

    cursor = ledger.get("cursor")
    require(isinstance(cursor, dict), "cursor:type")
    require(
        set(cursor)
        == {
            "active_step",
            "next_step",
            "last_step",
            "remaining_checkpoint_count",
            "first_rcld",
            "last_rcld",
            "remaining_rcld_count",
        },
        "cursor:keys",
    )
    active = step_number(cursor.get("active_step"), "cursor:active")
    following = step_number(cursor.get("next_step"), "cursor:next")
    require(1167 <= active <= 1283, "cursor:active_range")
    require(following == active + 1, "cursor:next_value")
    require(cursor.get("last_step") == "step_1283", "cursor:last")
    expected_remaining = 0 if terminal else 1283 - active + 1
    require(cursor.get("remaining_checkpoint_count") == expected_remaining, "cursor:remaining")
    expected_rcld = rcld_for_step(active)
    require(ledger.get("rcld") == expected_rcld, "ledger:rcld")
    require(cursor.get("first_rcld") == expected_rcld, "cursor:first_rcld")
    require(cursor.get("last_rcld") == 94, "cursor:last_rcld")
    expected_remaining_rclds = 0 if terminal else 94 - expected_rcld + 1
    require(cursor.get("remaining_rcld_count") == expected_remaining_rclds, "cursor:rcld_count")
    require(not terminal or active == 1283, "cursor:terminal")

    authority = load_object("spec/authority_transition_v10.json")
    live_stage = authority.get("current_stage")
    require(isinstance(live_stage, str) and live_stage in AUTHORITY_STAGES, "authority:stage")
    projection = ledger.get("authority_projection")
    require(isinstance(projection, dict), "projection:type")
    require(
        set(projection)
        == {
            "binding",
            "current_stage",
            "minimum_stage",
            "target_stage",
            "requirement_count",
            "target_requirement_count",
            "signed_fixture_count",
            "target_signed_fixture_count",
        },
        "projection:keys",
    )
    require(projection.get("binding") == "monotonic_stage_projection", "projection:binding")
    require(projection.get("current_stage") == live_stage, "projection:current")
    require(projection.get("minimum_stage") == "requirements_appended", "projection:minimum")
    require(projection.get("target_stage") == "distribution_complete", "projection:target")
    require(AUTHORITY_STAGES.index(live_stage) >= AUTHORITY_STAGES.index("requirements_appended"), "projection:regression")
    require(not terminal or live_stage == "distribution_complete", "projection:terminal")
    require(projection.get("requirement_count") == 148, "projection:requirements")
    require(projection.get("target_requirement_count") == 148, "projection:requirement_target")
    require(projection.get("signed_fixture_count") == STAGE_FIXTURE_COUNTS[live_stage], "projection:fixtures")
    require(projection.get("target_signed_fixture_count") == 192, "projection:fixture_target")

    require(
        ledger.get("requirements")
        == {
            "preserved_prefix_count": 139,
            "current_count": 148,
            "target_count": 148,
            "appended_ids": list(APPENDED_REQUIREMENT_IDS),
        },
        "ledger:requirements",
    )
    require(
        ledger.get("findings")
        == {
            "registered_count": 21,
            "reproduced_ids": list(REPRODUCED_IDS),
            "held_ids": ["FINDING_080"],
            "status": (
                "code_complete_publication_held"
                if terminal
                else "implementation_remediation_required"
            ),
        },
        "ledger:findings",
    )
    validate_predecessors(ledger.get("predecessors"), active, report)
    require(
        ledger.get("opaque_reproduction")
        == {
            "checkpoint": report["checkpoint"],
            "candidate": report["candidate"],
            "result_identity_sha256": report["result_identity_sha256"],
            "finding_count": len(report["finding_ids"]),
            "reproduction_count": report["result_classes"][1]["count"],
            "negative_mutation_count": report["result_classes"][2]["count"],
            "result": report["status"],
            "publication_status": report["publication_status"],
        },
        "ledger:opaque_binding",
    )
    validate_no_leak(ledger, "ledger:boundary")


def mutation_self_test(report: dict[str, Any], ledger: dict[str, Any]) -> int:
    report_mutations: list[tuple[str, dict[str, Any]]] = []
    missing = copy.deepcopy(report)
    missing.pop("candidate")
    report_mutations.append(("opaque_missing", missing))
    duplicate = copy.deepcopy(report)
    duplicate["finding_ids"][1] = duplicate["finding_ids"][0]
    report_mutations.append(("opaque_duplicate", duplicate))
    reordered = copy.deepcopy(report)
    reordered["finding_ids"].reverse()
    report_mutations.append(("opaque_reordered", reordered))
    stale = copy.deepcopy(report)
    stale["candidate"] = "b7607280fec23cdf71b4a0f5b44a1a573ff16b83"
    report_mutations.append(("opaque_stale", stale))
    forged = copy.deepcopy(report)
    forged["result_identity_sha256"] = "f" * 64
    report_mutations.append(("opaque_forged", forged))
    generic = copy.deepcopy(report)
    generic["result_classes"][1]["class"] = "generic"
    report_mutations.append(("opaque_generic", generic))

    ledger_mutations: list[tuple[str, dict[str, Any]]] = []
    missing_predecessor = copy.deepcopy(ledger)
    missing_predecessor["predecessors"].pop()
    ledger_mutations.append(("ledger_missing", missing_predecessor))
    duplicate_predecessor = copy.deepcopy(ledger)
    duplicate_predecessor["predecessors"][-1] = copy.deepcopy(
        duplicate_predecessor["predecessors"][-2]
    )
    ledger_mutations.append(("ledger_duplicate", duplicate_predecessor))
    reordered_predecessor = copy.deepcopy(ledger)
    reordered_predecessor["predecessors"].reverse()
    ledger_mutations.append(("ledger_reordered", reordered_predecessor))
    stale_private = copy.deepcopy(ledger)
    stale_private["predecessors"][-1]["candidate"] = "0" * 40
    ledger_mutations.append(("ledger_stale_private", stale_private))
    forged_private = copy.deepcopy(ledger)
    forged_private["opaque_reproduction"]["result_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_forged_private", forged_private))
    stale_cursor = copy.deepcopy(ledger)
    stale_cursor["cursor"]["active_step"] = "step_1166"
    ledger_mutations.append(("ledger_stale_cursor", stale_cursor))
    regressed_stage = copy.deepcopy(ledger)
    regressed_stage["authority_projection"]["current_stage"] = "transition_installed"
    ledger_mutations.append(("ledger_regressed_stage", regressed_stage))
    missing_requirement = copy.deepcopy(ledger)
    missing_requirement["requirements"]["appended_ids"].pop()
    ledger_mutations.append(("ledger_missing_requirement", missing_requirement))
    false_hold = copy.deepcopy(ledger)
    false_hold["findings"]["held_ids"] = []
    ledger_mutations.append(("ledger_false_hold", false_hold))
    premature_terminal = copy.deepcopy(ledger)
    premature_terminal["status"] = "code_complete_publication_held"
    premature_terminal["findings"]["status"] = "code_complete_publication_held"
    ledger_mutations.append(("ledger_premature_terminal", premature_terminal))
    fabricated_opaque = copy.deepcopy(ledger)
    fabricated_opaque["predecessors"].append(
        {
            "step": "step_1167",
            "candidate": "0" * 40,
            "owner_class": "opaque_private",
            "gate_ids": ["V-BOGUS"],
            "requirement_ids": [],
            "finding_ids": [],
            "deviation_ids": [],
            "result": "pass",
        }
    )
    fabricated_opaque["cursor"]["active_step"] = "step_1168"
    fabricated_opaque["cursor"]["next_step"] = "step_1169"
    fabricated_opaque["cursor"]["remaining_checkpoint_count"] = 116
    ledger_mutations.append(("ledger_unapproved_opaque_future", fabricated_opaque))
    fabricated_public = copy.deepcopy(fabricated_opaque)
    fabricated_public["predecessors"][-1]["candidate"] = (
        "291bcd978fa765077b69fcaec66d9b96305b2553"
    )
    fabricated_public["predecessors"][-1]["owner_class"] = "public"
    ledger_mutations.append(("ledger_unapproved_public_future", fabricated_public))

    caught = 0
    for name, mutation in report_mutations:
        try:
            validate_opaque_reproduction(mutation)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"mutation_survived:{name}")
    for name, mutation in ledger_mutations:
        try:
            validate_runtime_ledger(mutation, report)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"mutation_survived:{name}")

    report_schema = load_object(REPORT_SCHEMA)
    ledger_schema = load_object(LEDGER_SCHEMA)
    schema_mutations = []
    open_report = copy.deepcopy(report_schema)
    open_report["additionalProperties"] = True
    schema_mutations.append(("schema_open_report", open_report, ledger_schema))
    weak_report = copy.deepcopy(report_schema)
    weak_report["required"].pop()
    schema_mutations.append(("schema_weak_report", weak_report, ledger_schema))
    open_ledger = copy.deepcopy(ledger_schema)
    open_ledger["properties"]["predecessors"]["items"]["additionalProperties"] = True
    schema_mutations.append(("schema_open_predecessor", report_schema, open_ledger))
    weak_ledger = copy.deepcopy(ledger_schema)
    weak_ledger["properties"]["cursor"]["required"].pop()
    schema_mutations.append(("schema_weak_cursor", report_schema, weak_ledger))
    for name, first, second in schema_mutations:
        try:
            validate_schema_contract(first, "opaque_schema", REPORT_SCHEMA_PROJECTION)
            validate_schema_contract(second, "ledger_schema", LEDGER_SCHEMA_PROJECTION)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"mutation_survived:{name}")
    return caught


def main() -> int:
    report = load_object(REPORT)
    ledger = load_object(LEDGER)
    validate_schema_contract(
        load_object(REPORT_SCHEMA), "opaque_schema", REPORT_SCHEMA_PROJECTION
    )
    validate_schema_contract(
        load_object(LEDGER_SCHEMA), "ledger_schema", LEDGER_SCHEMA_PROJECTION
    )
    validate_opaque_reproduction(report)
    validate_runtime_ledger(ledger, report)
    mutations = mutation_self_test(report, ledger)
    print("PASS: remediation-v9 runtime ledger and opaque reproduction import")
    print(f"- predecessors={len(ledger['predecessors'])}")
    print(f"- opaque_reproductions={report['result_classes'][1]['count']}")
    print(f"- negative_mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
