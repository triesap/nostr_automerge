#!/usr/bin/env python3
"""Validate the closed v10 authority and signed-distribution transition."""

from __future__ import annotations

import copy
import hashlib
import json
import re
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
STATE_PATH = "spec/authority_transition_v10.json"
AUTHORITY_SCHEMA_PATH = "tools/validation/authority_transition_v10.schema.json"
DISTRIBUTION_SCHEMA_PATH = "fixtures/schema/distribution.schema.v10.json"
BASE_MANIFEST_PATH = "fixtures/distribution/manifest_v9.json"
V10_MANIFEST_PATH = "fixtures/distribution/manifest_v10.json"

STAGES = (
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
STAGE_COUNTS = {
    "transition_installed": (139, 180, 0, 0),
    "companion_authority_installed": (139, 180, 0, 0),
    "requirements_appended": (148, 180, 0, 0),
    "checkpoint_expectations_corrected": (148, 180, 4, 0),
    "distribution_locked": (148, 180, 4, 0),
    "checkpoint_control_fixtures_added": (148, 183, 4, 3),
    "carrier_independence_fixtures_added": (148, 186, 4, 6),
    "interruption_fixtures_added": (148, 189, 4, 9),
    "target_work_fixtures_added": (148, 192, 4, 12),
    "distribution_complete": (148, 192, 4, 12),
}
APPENDED_REQUIREMENTS = (
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
APPENDED_REQUIREMENT_ROWS = (
    {
        "id": "NCRDT-CPAUTH-001",
        "section": "Checkpoint control resolution precedence",
        "text": "A checkpoint descriptor control reference MUST be resolved and authorized before chunk assembly, carrier-history coverage, accepted-at-control lookup, snapshot loading, or history verification is attempted.",
        "source": "spec/CHECKPOINT_PROFILE.md",
    },
    {
        "id": "NCRDT-CPAUTH-002",
        "section": "Recoverable checkpoint control states",
        "text": "Only a missing or statefully pending referenced control may produce a pending checkpoint descriptor. A noncanonical, wrong-kind, wrong-coordinate, statically invalid, dynamically invalid, unsupported, or role-denied control MUST produce an invalid draft-v1 descriptor outcome.",
        "source": "spec/CHECKPOINT_PROFILE.md",
    },
    {
        "id": "NCRDT-DISPOSITION-006",
        "section": "Independent change-carrier outcomes",
        "text": "A change-carrier Event disposition MUST be derived from that carrier claim and its referenced control or branch. An aggregate ChangeHash disposition MUST NOT convert a carrier with a known-invalid reference into accepted, pending, or excluded.",
        "source": "spec/REPORT_CONTRACT.md",
    },
    {
        "id": "NCRDT-INTERRUPT-001",
        "section": "No-progress interruption reports",
        "text": "A public evaluation that ends in `budget_exhausted` or `cancelled` MUST return a constant-size no-progress report. It MUST NOT expose canonical controls, protocol dispositions, evidence, checkpoints, an available or resolved manifest, integrity alerts, heads, or materialized document state.",
        "source": "spec/REPORT_CONTRACT.md",
    },
    {
        "id": "NCRDT-RESOURCE-013",
        "section": "Two-tier finalization reservation",
        "text": "The evaluator MUST reserve fixed no-progress fallback capacity separately from complete-report capacity. Actual complete-report passes are consumed immediately before their work; on interruption, complete-report capacity is forfeited and only fixed fallback passes are consumed.",
        "source": "spec/REPORT_CONTRACT.md",
    },
    {
        "id": "NCRDT-RESOURCE-014",
        "section": "Target-local deterministic work",
        "text": "Every target-proportional preparation collection, raw-byte copy or shared-reference operation, branch memo traversal, canonical derivation pass, alert copy, and disposition copy MUST be bounded, charged, cancellation-aware, or eliminated.",
        "source": "spec/REPORT_CONTRACT.md",
    },
    {
        "id": "NCRDT-VERSION-002",
        "section": "Unsupported change identity",
        "text": "An unsupported change carrier whose canonical Change Chunk and ChangeHash were not verified receives only an Event `unsupported_revision` outcome. Its unverified `x` tag MUST NOT create a semantic ChangeHash disposition in draft v1.",
        "source": "spec/REPORT_CONTRACT.md",
    },
    {
        "id": "NCRDT-CONF-010",
        "section": "Signed conformance v10",
        "text": "The checksum-bound signed v10 distribution MUST contain exactly 192 scenarios, including the corrected checkpoint expectations and new carrier, interruption, and work-boundary cases. Both implementations MUST execute all scenarios twice and under all eight delivery permutations with byte-identical canonical output and deliberate mismatch rejection.",
        "source": "spec/CONFORMANCE.md",
    },
    {
        "id": "NCRDT-EVIDENCE-006",
        "section": "Semantically exact proof catalog",
        "text": "Every passing requirement row MUST bind to a semantically matching exact signed fixture or named assertion through a validated proof catalog. Broad command-only proof, unrelated assertion categories, stale expectations, and missing opaque TypeScript evidence identifiers MUST be rejected.",
        "source": "spec/CONFORMANCE.md",
    },
)
APPENDED_APPLICABILITY = (
    "rust-and-typescript",
    "rust-and-typescript",
    "rust-and-typescript",
    "rust-and-typescript",
    "rust-and-typescript",
    "rust-and-typescript",
    "rust-and-typescript",
    "rust-and-typescript",
    "rust-only-evidence-with-opaque-typescript-overlay",
)
APPLICABILITY_VALUES = {
    "rust-and-typescript",
    "rust-only",
    "rust-only-evidence-with-opaque-typescript-overlay",
    "out-of-core",
    "explicitly-deferred",
}
BASELINE_REQUIREMENT_ROWS_PROJECTION_SHA256 = (
    "c2b145769ebdc9615872ed3b8f3bd03282b753da91db56a44f63ce3aadfa9347"
)
BASELINE_APPLICABILITY_PROJECTION_SHA256 = (
    "f07219b520128b6352cdf49d3df4ce55c515506b1bec16795383ccda55edd38e"
)
CORRECTION_BINDINGS = (
    (
        "checkpoint_descriptor_references_invalid_control",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_invalid_control.fixture.json",
        "6cc48b1dfebf97885b333f8e6bd52a4a73e3537c20f68e148800e50df2ab35ed",
        "136a365adfcdfc66583d705a0fab7676526fb1532fec089c121e1d98cac56c9a",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_invalid_control.expected.json",
        "65e8b7629623f29f7c7a980780a7f3da110f9dbd1069840d65affea45db11713",
    ),
    (
        "checkpoint_descriptor_references_unsupported_control",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_unsupported_control.fixture.json",
        "61076b811f530bc6cd80e625c48a8090f90ce846013970d0ea5f7e4ef81fbf52",
        "5e5e54a2d5a66737dec2232cc13fd9e385131545bd65dc379227dbaadcd5b062",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_unsupported_control.expected.json",
        "ec510d417e37d55c4fff14268edee5435774d1027aa3c24f463839ab63b60fa0",
    ),
    (
        "checkpoint_descriptor_references_wrong_kind_control",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_kind_control.fixture.json",
        "e815bbc47509df08915cf15c3a3dea8e61ad993c137caa45e2d0bdebe23063f8",
        "fbdcc180b46d7474b7dac53e2382192531a0a6eada596f1fed5d5634c810de21",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_kind_control.expected.json",
        "a42a12d8222b25b4dae8303726bf3df1b9d49bb15bffe007cba49e7e10fb2a06",
    ),
    (
        "checkpoint_descriptor_references_wrong_coordinate_control",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_coordinate_control.fixture.json",
        "33d51ad73eb8aba4966924a4526c6e6b29d144edeb81859597b6d6d6eef66a89",
        "5e6f552e368d0f8ed809fd7eabe0263a752be37c4410d3d27d7d3aa9b469bd68",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_coordinate_control.expected.json",
        "81d73336b0358d53deb30e3865833be319f9d019db801d624fafbedd72f0ec3c",
    ),
)
CORRECTED_REPORTS = tuple(binding[0] for binding in CORRECTION_BINDINGS)
NEW_FIXTURE_GROUPS = (
    (
        "checkpoint_control",
        (
            "checkpoint_descriptor_references_noncanonical_control",
            "checkpoint_descriptor_references_dynamic_invalid_control",
            "checkpoint_descriptor_references_canonical_without_checkpoint_role",
        ),
    ),
    (
        "carrier_independence",
        (
            "excluded_hash_with_dynamic_invalid_duplicate_carrier",
            "pruned_hash_with_invalid_control_carrier",
            "equivocation_excluded_hash_with_invalid_control_carrier",
        ),
    ),
    (
        "interruption_no_progress",
        (
            "interrupted_after_branch_evaluation_returns_no_progress",
            "interrupted_after_claim_reduction_returns_no_progress",
            "interrupted_after_checkpoint_resolution_returns_no_progress",
        ),
    ),
    (
        "target_work",
        (
            "target_preparation_exact_budget",
            "target_raw_memo_exact_budget",
            "canonical_derivation_exact_budget",
        ),
    ),
)
NEW_FIXTURES = tuple(
    fixture_id for _, fixture_ids in NEW_FIXTURE_GROUPS for fixture_id in fixture_ids
)
PROFILE_NAMES = ("checkpoint", "core", "malformed", "property", "resource")
PLAN_IMMUTABLE_PROJECTION_SHA256 = (
    "9164aaedc374703ea7bb31307a3ff8c959ef8ee390605a57eef8f2887b86b5e7"
)
RCLD_STEP_RANGES = (
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
PLAN_PROGRESS_FIELDS = (
    "Status: ",
    "Active RCLD: ",
    "Active checkpoint: ",
    "Next RCLD: ",
    "Next checkpoint: ",
)
PLAN_PROGRESS_HEADINGS = {
    "## Unfinished RCLDs",
    "## Completed RCLDs",
    "## RCLD Progress",
}
TRANSITION_BASELINE = (
    (
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md",
        "ddd58da3450108d9e73f0d6872ee4399419ebeeba35aa63c1c2c464a56552029",
    ),
    (
        "implementation/deviations/step_1158.md",
        "b787b9fa8787604fa11abad37e727b5e15544ac4b5abf38bb75f8c0e18ad8891",
    ),
    (
        "reports/remediation_v9_baseline.json",
        "e5b6567712364e57dfd99a2fd653ba63f51865622b902c4b9c26a50e0001f127",
    ),
)
V9_EVIDENCE = (
    ("reports/evidence_supersession_v8.json", "8306063cd8ff11ed86bf9faaba5efd04c46c98b133d5239cd154d067e8458c4c"),
    ("reports/external_holds_v8.json", "69c04d7183042c9b3935e4f2df3d6335ae76fbdaebb2dc249a021d227f172942"),
    ("reports/final_candidate_identity_v8.json", "b3d03079fc5418a7d8295c4196ff43669f3689a106fafa00f38c3b0a1bc3e0ae"),
    ("reports/interop_combined_v9.json", "cd501c2e15010b17e3a58dc977ae6ea1e5022c3b42b1c24e01512e10be0c3b0f"),
    ("reports/interop_evidence_mutations_v9.json", "89a2ea147028887af3128d40a869f6fa3946216b16980296c0f25b3a245ae022"),
    ("reports/interop_rust_v9.json", "42668996e5aab5d58ebc8ae4b8831b77fa648f9bf727c4a8650316b034789a81"),
    ("reports/interop_typescript_v9.json", "a8dd711a9064a2455bd20423f39f7644f0d76fd9ff24f53a85ef52a220d2ccf2"),
    ("reports/ordinary_assurance_v9.json", "e3f99dea04bd744829aadd6a6bceff4a75c7b81440e0820bf9cfd26defb6b955"),
    ("reports/private_assurance_v9.json", "565018940e4bde29bc8e755cf3248fe77900f2f53d850e28713ccecfec083bea"),
    ("reports/remediation_v8_final.json", "e484b87f7a8e1e034a63e25a80493aa9146b78e19ccc46a483d0bcd568c84e8f"),
    ("reports/requirements_authority_v9.json", "96e466abee8f33d8f86183283f544d85b8e123d7c678f7bd61f343665bd5564c"),
    ("reports/requirements_coverage_v9.json", "996f1f1b480170d1308427d40833105fd1432fcd6b656abc98892f9458cbcdcb"),
    ("reports/requirements_evidence_mutations_v9.json", "99fe9bf4dc862e74efdc2f0a7b3e36292d599e8fe17202278e9c3562834bed2c"),
    ("reports/requirements_typescript_overlay_v9.json", "e4ce85caa05ce85676bc08ae8375c20a22f8fd863478c71163263d242dd2563d"),
    ("reports/resource_qualification_v9.json", "4d82877e114a5f9206b9d5858bea3f0a8eab3f8b70d32e57e642046101172c4e"),
    ("reports/rust_conformance_v9.json", "bdde3b0312c9a08da78aa00930dc43496c71ed6cc2107bde41ae1bc4412ed932"),
)
BASELINE = {
    "nip_sha256": "0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1",
    "companion_sha256": "58177c31eb06086d76297bbb0fc15343a8e34c15499d6e03636c63df7604bb10",
    "requirements_sha256": "a97103be86946c15d81b3fc585efa36f4884da09f91cb51a8c5adfa27b7fe8f0",
    "applicability_sha256": "7cda8e59da0d8caf1f9a9985ba27c9367018c572824f092106fe5e5a8d823793",
    "ordered_requirement_ids_sha256": "1763ff189bde8a7cb2743a13450105b40faa7575eb9b4fb2730aa49a0836d071",
    "manifest_sha256": "7b4ab5d2146939d142eb92d43060ef2183c95d1fc574132894b3c01c874c7c56",
    "ordered_fixture_ids_sha256": "06f35baa3f56013232ab0c708bccaabe09348865038374d28c629d1689139082",
    "signed_events_sha256": "329c6946e3c56f94da3159c3e3d38b685f818da9c632f02fd868b0ccf05d401b",
}
MANIFEST_KEYS = {
    "distribution_schema",
    "distribution_id",
    "protocol_revision",
    "transition_stage",
    "status",
    "target_fixture_count",
    "fixture_count",
    "complete",
    "base_manifest_sha256",
    "preserved_v9_fixture_count",
    "preserved_v9_signed_events_sha256",
    "missing_v9_fixtures",
    "missing_v10_fixtures",
    "intentional_v9_report_changes",
    "requirements_sha256",
    "authority_sha256",
    "companion_sha256",
    "conformance_sha256",
    "supersedes",
    "v10_fixtures",
    "profiles",
    "fixtures",
    "files",
}
FIXTURE_ENTRY_KEYS = {
    "expected_path",
    "fixture_id",
    "input_paths",
    "metadata_path",
    "profile",
    "requirements",
}
FORBIDDEN_PUBLIC_MARKERS = (
    "/" + "Users/",
    "/" + "home/",
    "docs/" + "handoff",
    "domains/" + "triesap",
    "triesap/" + "dev",
    ".act" + "/",
    ".github/" + "workflows",
)


class TransitionError(ValueError):
    """One v10 transition invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise TransitionError(diagnostic)


def load_object(relative: str) -> dict[str, Any]:
    try:
        value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise TransitionError(f"json:{relative}") from error
    if not isinstance(value, dict):
        raise TransitionError(f"object:{relative}")
    return value


def digest(relative: str) -> str:
    try:
        return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()
    except OSError as error:
        raise TransitionError(f"file:{relative}") from error


def load_strict_lf_utf8(relative: str) -> str:
    try:
        value = (ROOT / relative).read_bytes().decode("utf-8", errors="strict")
    except (OSError, UnicodeDecodeError) as error:
        raise TransitionError(f"utf8:{relative}") from error
    require("\r" not in value and value.endswith("\n"), f"line_endings:{relative}")
    return value


def ordered_digest(values: list[str]) -> str:
    return hashlib.sha256(
        json.dumps(values, separators=(",", ":")).encode("utf-8")
    ).hexdigest()


def projection_digest(value: Any) -> str:
    """Return the exact canonical JSON projection identity for *value*."""

    return hashlib.sha256(
        json.dumps(
            value,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
    ).hexdigest()


def normalized_markdown(value: str) -> str:
    """Collapse Markdown wrapping while preserving exact token content."""

    return " ".join(value.split())


def metadata_invariant_sha256(relative: str) -> str:
    metadata = load_object(relative)
    expected = metadata.get("expected")
    require(isinstance(expected, dict), f"fixture_expected:{relative}")
    normalized = copy.deepcopy(metadata)
    normalized["expected"]["sha256"] = "<expected-report-sha256>"
    return hashlib.sha256(
        json.dumps(normalized, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()


def signed_event_set_sha256(entries: list[dict[str, Any]]) -> str:
    value = hashlib.sha256()
    for entry in entries:
        identifier = str(entry.get("fixture_id", "")).encode("utf-8")
        value.update(len(identifier).to_bytes(4, "big"))
        value.update(identifier)
        input_paths = entry.get("input_paths")
        require(isinstance(input_paths, list) and len(input_paths) == 1, "baseline_input_paths")
        scenario = load_object(str(input_paths[0]))
        require(
            scenario.get("scenario_schema") == "nostr_automerge.signed_scenario.v2",
            f"signed_scenario:{entry.get('fixture_id')}",
        )
        raw_events = scenario.get("raw_events")
        require(isinstance(raw_events, list), f"raw_events:{entry.get('fixture_id')}")
        for raw in raw_events:
            require(isinstance(raw, dict), f"raw_event:{entry.get('fixture_id')}")
            encoding = str(raw.get("encoding", "")).encode("utf-8")
            data = str(raw.get("data", "")).encode("utf-8")
            value.update(len(encoding).to_bytes(4, "big"))
            value.update(encoding)
            value.update(len(data).to_bytes(8, "big"))
            value.update(data)
    return value.hexdigest()


def validate_schema_contracts(
    authority_schema: dict[str, Any], distribution_schema: dict[str, Any]
) -> None:
    require(
        authority_schema.get("$schema") == "https://json-schema.org/draft/2020-12/schema",
        "authority_schema_version",
    )
    require(authority_schema.get("additionalProperties") is False, "authority_schema_open")
    definitions = authority_schema.get("$defs")
    require(isinstance(definitions, dict), "authority_schema_definitions")
    stage = definitions.get("stage", {})
    require(stage.get("enum") == list(STAGES), "authority_schema_stages")
    properties = authority_schema.get("properties")
    require(isinstance(properties, dict), "authority_schema_properties")
    require(
        set(authority_schema.get("required", []))
        == {
            "schema",
            "status",
            "protocol_revision",
            "current_stage",
            "stage_order",
            "transition_baseline",
            "authority",
            "distribution",
            "v9_evidence",
        },
        "authority_schema_required",
    )
    authority = properties.get("authority", {})
    distribution = properties.get("distribution", {})
    evidence = properties.get("v9_evidence", {})
    transition_baseline = properties.get("transition_baseline", {})
    require(
        transition_baseline.get("additionalProperties") is False,
        "transition_baseline_contract_open",
    )
    require(
        set(transition_baseline.get("required", []))
        == {"status", "plan_binding", "artifacts"},
        "transition_baseline_contract_required",
    )
    require(
        transition_baseline.get("properties", {}).get("plan_binding", {}).get("const")
        == "initial_exact_then_immutable_projection",
        "transition_baseline_plan_binding",
    )
    baseline_artifacts = transition_baseline.get("properties", {}).get("artifacts", {})
    require(
        baseline_artifacts.get("minItems")
        == len(TRANSITION_BASELINE)
        == baseline_artifacts.get("maxItems"),
        "transition_baseline_schema_count",
    )
    require(authority.get("additionalProperties") is False, "authority_contract_open")
    require(distribution.get("additionalProperties") is False, "distribution_contract_open")
    require(evidence.get("additionalProperties") is False, "evidence_contract_open")
    require(
        evidence.get("properties", {}).get("artifacts", {}).get("minItems")
        == len(V9_EVIDENCE)
        == evidence.get("properties", {}).get("artifacts", {}).get("maxItems"),
        "evidence_schema_count",
    )

    require(
        distribution_schema.get("$schema")
        == "https://json-schema.org/draft/2020-12/schema",
        "distribution_schema_version",
    )
    require(distribution_schema.get("additionalProperties") is False, "distribution_schema_open")
    dist_properties = distribution_schema.get("properties")
    require(isinstance(dist_properties, dict), "distribution_schema_properties")
    require(set(distribution_schema.get("required", [])) == MANIFEST_KEYS, "distribution_required")
    require(dist_properties.get("distribution_schema", {}).get("const") == "nostr_automerge.fixture_distribution.v10", "distribution_schema_identity")
    for name, value in (
        ("distribution_id", "draft_2026_08_signed_neutral_10"),
        ("protocol_revision", "draft_2026_08"),
        ("target_fixture_count", 192),
        ("preserved_v9_fixture_count", 180),
        ("supersedes", BASE_MANIFEST_PATH),
    ):
        require(dist_properties.get(name) == {"const": value}, f"distribution_constant:{name}")
    require(
        dist_properties.get("transition_stage", {}).get("enum")
        == list(STAGES[STAGES.index("distribution_locked") :]),
        "distribution_transition_stages",
    )
    require(
        dist_properties.get("status", {}).get("enum")
        == ["locked_transition", "canonical_signed_neutral_corpus"],
        "distribution_statuses",
    )
    require(dist_properties.get("complete") == {"type": "boolean"}, "distribution_complete_type")
    require(
        dist_properties.get("protocol_revision", {}).get("const") == "draft_2026_08",
        "distribution_protocol_revision",
    )
    require(
        dist_properties.get("target_fixture_count", {}).get("const") == 192,
        "distribution_target_count",
    )
    require(
        dist_properties.get("preserved_v9_fixture_count", {}).get("const") == 180,
        "distribution_preserved_count",
    )
    require(
        dist_properties.get("fixture_count", {}).get("enum") == [180, 183, 186, 189, 192],
        "distribution_stage_counts",
    )
    require(
        dist_properties.get("intentional_v9_report_changes", {}).get("minItems")
        == len(CORRECTED_REPORTS)
        == dist_properties.get("intentional_v9_report_changes", {}).get("maxItems"),
        "distribution_correction_count",
    )
    require(
        dist_properties.get("v10_fixtures", {}).get("minItems")
        == len(NEW_FIXTURES)
        == dist_properties.get("v10_fixtures", {}).get("maxItems"),
        "distribution_addition_count",
    )
    for name in ("missing_v10_fixtures", "intentional_v9_report_changes", "v10_fixtures"):
        collection = dist_properties.get(name, {})
        require(collection.get("type") == "array" and collection.get("uniqueItems") is True, f"distribution_inventory_type:{name}")
        require(collection.get("items") == {"$ref": "#/$defs/id"}, f"distribution_inventory_items:{name}")
    for name in (
        "base_manifest_sha256",
        "preserved_v9_signed_events_sha256",
        "requirements_sha256",
        "authority_sha256",
        "companion_sha256",
        "conformance_sha256",
    ):
        require(dist_properties.get(name) == {"$ref": "#/$defs/sha256"}, f"distribution_hash:{name}")
    require(
        dist_properties.get("missing_v9_fixtures")
        == {"type": "array", "maxItems": 0},
        "distribution_missing_v9_contract",
    )
    require(
        dist_properties.get("missing_v10_fixtures", {}).get("uniqueItems") is True,
        "distribution_missing_v10_unique",
    )
    definitions = distribution_schema.get("$defs")
    require(isinstance(definitions, dict), "distribution_definitions")
    require(set(definitions) == {"id", "requirement_id", "path", "sha256", "profile", "fixture", "file"}, "distribution_definition_inventory")
    require(definitions.get("id") == {"type": "string", "pattern": "^[a-z0-9][a-z0-9_]{2,127}$"}, "distribution_id_definition")
    require(definitions.get("requirement_id") == {"type": "string", "pattern": "^NCRDT-[A-Z0-9]+(?:-[A-Z0-9]+)*$"}, "distribution_requirement_definition")
    require(definitions.get("path") == {"type": "string", "pattern": "^(?:fixtures|spec)/[a-zA-Z0-9_./-]+$"}, "distribution_path_definition")
    require(definitions.get("sha256") == {"type": "string", "pattern": "^[0-9a-f]{64}$"}, "distribution_hash_definition")
    profile = definitions.get("profile", {})
    require(profile.get("type") == "array" and profile.get("uniqueItems") is True, "distribution_profile_type")
    require(profile.get("items") == {"$ref": "#/$defs/id"}, "distribution_profile_items")
    fixture = definitions.get("fixture", {})
    require(fixture.get("type") == "object" and fixture.get("additionalProperties") is False, "distribution_fixture_closed")
    require(set(fixture.get("required", [])) == FIXTURE_ENTRY_KEYS, "distribution_fixture_required")
    fixture_properties = fixture.get("properties", {})
    require(set(fixture_properties) == FIXTURE_ENTRY_KEYS, "distribution_fixture_properties")
    for name in ("expected_path", "metadata_path"):
        require(fixture_properties.get(name) == {"$ref": "#/$defs/path"}, f"distribution_fixture_path:{name}")
    require(fixture_properties.get("fixture_id") == {"$ref": "#/$defs/id"}, "distribution_fixture_id")
    require(fixture_properties.get("profile", {}).get("enum") == list(PROFILE_NAMES), "distribution_fixture_profiles")
    input_paths = fixture_properties.get("input_paths", {})
    require(input_paths.get("type") == "array" and input_paths.get("minItems") == input_paths.get("maxItems") == 1 and input_paths.get("uniqueItems") is True, "distribution_fixture_inputs")
    require(input_paths.get("items") == {"$ref": "#/$defs/path"}, "distribution_fixture_input_items")
    requirements = fixture_properties.get("requirements", {})
    require(requirements.get("type") == "array" and requirements.get("minItems") == 1 and requirements.get("uniqueItems") is True, "distribution_fixture_requirements")
    require(requirements.get("items") == {"$ref": "#/$defs/requirement_id"}, "distribution_fixture_requirement_items")
    file_definition = definitions.get("file", {})
    require(file_definition.get("type") == "object" and file_definition.get("additionalProperties") is False, "distribution_file_closed")
    require(set(file_definition.get("required", [])) == {"path", "sha256"}, "distribution_file_required")
    require(file_definition.get("properties") == {"path": {"$ref": "#/$defs/path"}, "sha256": {"$ref": "#/$defs/sha256"}}, "distribution_file_properties")
    profiles = dist_properties.get("profiles", {})
    require(profiles.get("type") == "object" and profiles.get("additionalProperties") is False, "distribution_profiles_closed")
    require(profiles.get("required") == list(PROFILE_NAMES), "distribution_profiles_required")
    require(set(profiles.get("properties", {})) == set(PROFILE_NAMES), "distribution_profiles_properties")
    for name in PROFILE_NAMES:
        require(profiles["properties"].get(name) == {"$ref": "#/$defs/profile"}, f"distribution_profile_ref:{name}")
    for name, reference in (("fixtures", "#/$defs/fixture"), ("files", "#/$defs/file")):
        collection = dist_properties.get(name, {})
        require(collection.get("type") == "array" and collection.get("uniqueItems") is True, f"distribution_{name}_type")
        require(collection.get("items") == {"$ref": reference}, f"distribution_{name}_items")
    require(dist_properties["fixtures"].get("minItems") == 180 and dist_properties["fixtures"].get("maxItems") == 192, "distribution_fixtures_bounds")
    require(dist_properties["files"].get("minItems") == 1, "distribution_files_bounds")


def validate_requirement_projection(
    registry: dict[str, Any],
    applicability: dict[str, Any],
    baseline_rows: list[Any],
    source_documents: dict[str, str],
    expected_count: int,
) -> None:
    """Validate the immutable prefix and exact staged v10 append projection."""

    require(
        set(registry) == {"schema", "project", "requirement_count", "requirements"},
        "requirement_registry_keys",
    )
    require(
        registry.get("schema") == "nostr_automerge.requirements.v6",
        "requirement_registry_schema",
    )
    require(registry.get("project") == "nostr_automerge_v1_spec", "requirement_project")
    require(
        set(applicability) == {"schema", "reviewed", "classifications"},
        "applicability_keys",
    )
    require(
        applicability.get("schema")
        == "nostr_automerge.requirements_applicability.v6",
        "applicability_schema",
    )
    expected_reviewed = "2026-08-20" if expected_count == 139 else "2026-08-21"
    require(applicability.get("reviewed") == expected_reviewed, "applicability_reviewed")

    rows = registry.get("requirements")
    classifications = applicability.get("classifications")
    require(isinstance(rows, list), "requirement_rows")
    require(isinstance(classifications, dict), "applicability_rows")
    require(
        len(rows) == registry.get("requirement_count") == expected_count,
        "live_requirement_count",
    )
    require(len(classifications) == expected_count, "applicability_count")
    require(
        all(
            isinstance(row, dict)
            and set(row) == {"id", "section", "text", "source"}
            for row in rows
        ),
        "requirement_row_shape",
    )
    identifiers = [str(row["id"]) for row in rows]
    require(len(identifiers) == len(set(identifiers)), "requirement_ids")
    require(list(classifications) == identifiers, "applicability_order")
    require(
        set(classifications.values()).issubset(APPLICABILITY_VALUES),
        "applicability_value",
    )

    require(
        projection_digest(rows[:139]) == BASELINE_REQUIREMENT_ROWS_PROJECTION_SHA256,
        "requirement_prefix_projection",
    )
    require(
        projection_digest(list(classifications.items())[:139])
        == BASELINE_APPLICABILITY_PROJECTION_SHA256,
        "applicability_prefix_projection",
    )
    require(
        isinstance(baseline_rows, list) and len(baseline_rows) == 139,
        "baseline_evidence_rows",
    )
    for current, baseline in zip(rows[:139], baseline_rows, strict=True):
        require(isinstance(baseline, dict), "baseline_row_shape")
        baseline_authority = baseline.get("authority")
        require(isinstance(baseline_authority, dict), "baseline_row_authority")
        identifier = current["id"]
        require(identifier == baseline.get("id"), f"requirement_prefix_order:{identifier}")
        require(current["source"] == baseline_authority.get("source"), f"requirement_source:{identifier}")
        require(current["section"] == baseline_authority.get("section"), f"requirement_section:{identifier}")
        require(
            hashlib.sha256(current["text"].encode("utf-8")).hexdigest()
            == baseline_authority.get("text_sha256"),
            f"requirement_text:{identifier}",
        )
        require(
            classifications.get(identifier) == baseline.get("applicability"),
            f"requirement_applicability:{identifier}",
        )

    if expected_count == 139:
        require(rows[139:] == [], "early_requirement_append")
        return

    require(rows[139:] == list(APPENDED_REQUIREMENT_ROWS), "requirement_append_rows")
    require(
        tuple(classifications[identifier] for identifier in APPENDED_REQUIREMENTS)
        == APPENDED_APPLICABILITY,
        "requirement_append_applicability",
    )
    for row in APPENDED_REQUIREMENT_ROWS:
        source = source_documents.get(row["source"])
        require(isinstance(source, str), f"requirement_anchor_source:{row['id']}")
        require(
            source.count(f"## {row['section']}\n") == 1,
            f"requirement_anchor_heading:{row['id']}",
        )
        require(
            normalized_markdown(row["text"]) in normalized_markdown(source),
            f"requirement_anchor_text:{row['id']}",
        )


def requirement_projection_self_test(
    registry: dict[str, Any],
    applicability: dict[str, Any],
    baseline_rows: list[Any],
    source_documents: dict[str, str],
) -> int:
    """Prove exact v10 row, source, class, order, and count failures close."""

    mutations: list[
        tuple[str, dict[str, Any], dict[str, Any], dict[str, str]]
    ] = []

    def copies() -> tuple[dict[str, Any], dict[str, Any], dict[str, str]]:
        return copy.deepcopy(registry), copy.deepcopy(applicability), source_documents.copy()

    candidate, classes, sources = copies()
    candidate["requirements"][139]["source"] = "spec/REPORT_CONTRACT.md"
    mutations.append(("wrong_source", candidate, classes, sources))
    candidate, classes, sources = copies()
    candidate["requirements"][140]["section"] = "Checkpoint control resolution precedence"
    mutations.append(("wrong_section", candidate, classes, sources))
    candidate, classes, sources = copies()
    candidate["requirements"][142]["text"] = candidate["requirements"][142]["text"].replace(
        "MUST return", "MAY return", 1
    )
    mutations.append(("wrong_text", candidate, classes, sources))
    candidate, classes, sources = copies()
    classes["classifications"]["NCRDT-EVIDENCE-006"] = "rust-only"
    mutations.append(("wrong_applicability", candidate, classes, sources))
    candidate, classes, sources = copies()
    classes["classifications"]["NCRDT-EVIDENCE-006"] = (
        "rust-only-evidence-with-opaque-typescript-overlays"
    )
    mutations.append(("near_miss_applicability", candidate, classes, sources))
    candidate, classes, sources = copies()
    candidate["requirements"][139], candidate["requirements"][140] = (
        candidate["requirements"][140],
        candidate["requirements"][139],
    )
    reordered = list(classes["classifications"].items())
    reordered[139], reordered[140] = reordered[140], reordered[139]
    classes["classifications"] = dict(reordered)
    mutations.append(("coordinated_order", candidate, classes, sources))
    candidate, classes, sources = copies()
    candidate["requirements"].pop()
    candidate["requirement_count"] = 147
    classes["classifications"].pop("NCRDT-EVIDENCE-006")
    mutations.append(("coordinated_count", candidate, classes, sources))
    candidate, classes, sources = copies()
    old = candidate["requirements"][141]["text"]
    new = old.replace("MUST NOT", "MAY", 1)
    candidate["requirements"][141]["text"] = new
    sources["spec/REPORT_CONTRACT.md"] = sources["spec/REPORT_CONTRACT.md"].replace(
        old, new, 1
    )
    mutations.append(("coordinated_authority_drift", candidate, classes, sources))
    candidate, classes, sources = copies()
    sources["spec/CONFORMANCE.md"] = sources["spec/CONFORMANCE.md"].replace(
        "## Signed conformance v10\n", "## Deferred signed conformance\n", 1
    )
    mutations.append(("missing_source_anchor", candidate, classes, sources))

    caught = 0
    for name, candidate, classes, sources in mutations:
        try:
            validate_requirement_projection(
                candidate,
                classes,
                baseline_rows,
                sources,
                148,
            )
        except TransitionError:
            caught += 1
            continue
        raise TransitionError(f"requirement_mutation_survived:{name}")
    return caught


def validate_requirements(state: dict[str, Any], stage: str) -> None:
    authority = state.get("authority")
    require(isinstance(authority, dict), "authority")
    expected_keys = {
        "schema_path",
        "schema_sha256",
        "nip_sha256",
        "baseline_companion_sha256",
        "baseline_requirements_sha256",
        "baseline_applicability_sha256",
        "baseline_ordered_requirement_ids_sha256",
        "baseline_requirement_count",
        "preserved_prefix_count",
        "target_requirement_count",
        "appended_ids",
        "live",
    }
    require(set(authority) == expected_keys, "authority_keys")
    require(authority.get("schema_path") == AUTHORITY_SCHEMA_PATH, "authority_schema_path")
    require(authority.get("schema_sha256") == digest(AUTHORITY_SCHEMA_PATH), "authority_schema_hash")
    require(authority.get("nip_sha256") == BASELINE["nip_sha256"], "nip_binding")
    require(digest("spec/NIP_DRAFT.md") == BASELINE["nip_sha256"], "nip_changed")
    require(
        authority.get("baseline_companion_sha256") == BASELINE["companion_sha256"],
        "baseline_companion",
    )
    require(
        authority.get("baseline_requirements_sha256") == BASELINE["requirements_sha256"],
        "baseline_requirements",
    )
    require(
        authority.get("baseline_applicability_sha256") == BASELINE["applicability_sha256"],
        "baseline_applicability",
    )
    require(
        authority.get("baseline_ordered_requirement_ids_sha256")
        == BASELINE["ordered_requirement_ids_sha256"],
        "baseline_requirement_order",
    )
    require(
        authority.get("baseline_requirement_count")
        == authority.get("preserved_prefix_count")
        == 139,
        "baseline_requirement_count",
    )
    require(authority.get("target_requirement_count") == 148, "target_requirement_count")
    require(authority.get("appended_ids") == list(APPENDED_REQUIREMENTS), "appended_ids")

    registry = load_object("spec/requirements.json")
    applicability = load_object("spec/requirements_applicability.json")
    expected_count = STAGE_COUNTS[stage][0]
    baseline_rows = load_object("reports/requirements_coverage_v9.json").get("rows")
    require(isinstance(baseline_rows, list), "baseline_evidence_rows")
    source_documents = {
        relative: load_strict_lf_utf8(relative)
        for relative in {
            row["source"] for row in APPENDED_REQUIREMENT_ROWS
        }
    }
    validate_requirement_projection(
        registry,
        applicability,
        baseline_rows,
        source_documents,
        expected_count,
    )
    rows = registry["requirements"]
    identifiers = [str(row["id"]) for row in rows]
    require(
        ordered_digest(identifiers[:139]) == BASELINE["ordered_requirement_ids_sha256"],
        "requirement_prefix_digest",
    )
    if expected_count == 139:
        require(digest("spec/requirements.json") == BASELINE["requirements_sha256"], "early_requirements_changed")
        require(
            digest("spec/requirements_applicability.json") == BASELINE["applicability_sha256"],
            "early_applicability_changed",
        )

    live = authority.get("live")
    require(isinstance(live, dict), "live_authority")
    require(
        set(live) == {"requirements_sha256", "applicability_sha256", "companion_sha256"},
        "live_authority_keys",
    )
    require(live.get("requirements_sha256") == digest("spec/requirements.json"), "live_requirements_hash")
    require(
        live.get("applicability_sha256") == digest("spec/requirements_applicability.json"),
        "live_applicability_hash",
    )
    require(live.get("companion_sha256") == digest("spec/NOSTR_AUTOMERGE_V1_SPEC.md"), "live_companion_hash")
    if STAGES.index(stage) < STAGES.index("companion_authority_installed"):
        require(digest("spec/NOSTR_AUTOMERGE_V1_SPEC.md") == BASELINE["companion_sha256"], "early_companion_changed")
    else:
        require(
            digest("spec/NOSTR_AUTOMERGE_V1_SPEC.md") != BASELINE["companion_sha256"],
            "companion_authority_not_installed",
        )


def discover_fixture_metadata() -> dict[str, Path]:
    result: dict[str, Path] = {}
    root = ROOT / "fixtures/v1_draft/scenarios"
    for path in sorted(root.rglob("*.fixture.json"), key=lambda item: item.as_posix().encode()):
        try:
            value = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            raise TransitionError(f"fixture_metadata:{path.relative_to(ROOT)}") from error
        require(isinstance(value, dict), f"fixture_metadata_object:{path.relative_to(ROOT)}")
        identifier = value.get("fixture_id")
        require(isinstance(identifier, str), f"fixture_id:{path.relative_to(ROOT)}")
        require(identifier not in result, f"duplicate_fixture_id:{identifier}")
        result[identifier] = path
    return result


def validate_new_signed_fixture(identifier: str, metadata_path: Path) -> None:
    metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    require(metadata.get("fixture_schema") == "nostr_automerge.fixture.v1", f"new_fixture_schema:{identifier}")
    inputs = metadata.get("inputs")
    require(isinstance(inputs, list) and len(inputs) == 1, f"new_fixture_inputs:{identifier}")
    item = inputs[0]
    require(isinstance(item, dict) and item.get("name") == "signed_scenario", f"new_fixture_input:{identifier}")
    input_path = metadata_path.parent / str(item.get("path", ""))
    require(input_path.is_file(), f"new_fixture_input_file:{identifier}")
    require(hashlib.sha256(input_path.read_bytes()).hexdigest() == item.get("sha256"), f"new_fixture_input_hash:{identifier}")
    scenario = json.loads(input_path.read_text(encoding="utf-8"))
    require(
        isinstance(scenario, dict)
        and scenario.get("scenario_schema") == "nostr_automerge.signed_scenario.v2"
        and scenario.get("fixture_id") == identifier
        and isinstance(scenario.get("raw_events"), list),
        f"new_signed_scenario:{identifier}",
    )


def fixture_profile(identifier: str, metadata_path: Path) -> str:
    resource_fixtures = {
        "unrelated_control_flood_exact_budget",
        *NEW_FIXTURE_GROUPS[-1][1],
    }
    if identifier in resource_fixtures:
        return "resource"
    return {
        "checkpoint": "checkpoint",
        "checkpoints": "checkpoint",
        "projection": "property",
        "versioning": "malformed",
    }.get(metadata_path.parent.name, "core")


def expected_fixture_entry(identifier: str, metadata_path: Path) -> dict[str, Any]:
    relative_metadata = metadata_path.relative_to(ROOT).as_posix()
    metadata = load_object(relative_metadata)
    require(metadata.get("fixture_id") == identifier, f"manifest_metadata_id:{identifier}")
    inputs = metadata.get("inputs")
    require(isinstance(inputs, list) and len(inputs) == 1, f"manifest_metadata_inputs:{identifier}")
    input_row = inputs[0]
    require(isinstance(input_row, dict), f"manifest_metadata_input:{identifier}")
    input_path = metadata_path.parent / str(input_row.get("path", ""))
    expected = metadata.get("expected")
    require(isinstance(expected, dict), f"manifest_metadata_expected:{identifier}")
    expected_path = metadata_path.parent / str(expected.get("report_path", ""))
    requirements = metadata.get("requirements")
    require(
        isinstance(requirements, list)
        and requirements
        and all(isinstance(item, str) for item in requirements)
        and requirements == sorted(set(requirements), key=str.encode),
        f"manifest_metadata_requirements:{identifier}",
    )
    require(input_path.is_file() and expected_path.is_file(), f"manifest_fixture_files:{identifier}")
    require(input_row.get("sha256") == digest(input_path.relative_to(ROOT).as_posix()), f"manifest_fixture_input_hash:{identifier}")
    require(expected.get("sha256") == digest(expected_path.relative_to(ROOT).as_posix()), f"manifest_fixture_expected_hash:{identifier}")
    return {
        "expected_path": expected_path.relative_to(ROOT).as_posix(),
        "fixture_id": identifier,
        "input_paths": [input_path.relative_to(ROOT).as_posix()],
        "metadata_path": relative_metadata,
        "profile": fixture_profile(identifier, metadata_path),
        "requirements": requirements,
    }


def expected_manifest_inventory(
    discovered: dict[str, Path],
) -> tuple[list[dict[str, Any]], dict[str, list[str]], list[dict[str, str]]]:
    entries = [
        expected_fixture_entry(identifier, discovered[identifier])
        for identifier in sorted(discovered, key=str.encode)
    ]
    requirement_rows = load_object("spec/requirements.json").get("requirements")
    require(isinstance(requirement_rows, list), "manifest_requirement_rows")
    known_requirements = {
        str(row.get("id")) for row in requirement_rows if isinstance(row, dict)
    }
    require(len(known_requirements) == len(requirement_rows), "manifest_requirement_ids")
    for entry in entries:
        require(
            set(str(item) for item in entry["requirements"]).issubset(known_requirements),
            f"manifest_unknown_requirement:{entry['fixture_id']}",
        )
    profiles = {name: [] for name in PROFILE_NAMES}
    paths = set((ROOT / "fixtures/schema").glob("*.json"))
    paths.update(
        ROOT / relative
        for relative in (
            "spec/requirements.json",
            "spec/NIP_DRAFT.md",
            "spec/NOSTR_AUTOMERGE_V1_SPEC.md",
            "spec/CONFORMANCE.md",
        )
    )
    for entry in entries:
        profiles[str(entry["profile"])].append(str(entry["fixture_id"]))
        paths.add(ROOT / str(entry["metadata_path"]))
        paths.add(ROOT / str(entry["expected_path"]))
        paths.update(ROOT / str(relative) for relative in entry["input_paths"])
    files = [
        {"path": path.relative_to(ROOT).as_posix(), "sha256": digest(path.relative_to(ROOT).as_posix())}
        for path in sorted(paths, key=lambda item: item.relative_to(ROOT).as_posix().encode())
    ]
    return entries, profiles, files


def expected_v10_manifest(
    stage: str, discovered: dict[str, Path]
) -> dict[str, Any]:
    _, expected_fixture_count, _, new_count = STAGE_COUNTS[stage]
    entries, profiles, files = expected_manifest_inventory(discovered)
    complete = stage == "distribution_complete"
    return {
        "distribution_schema": "nostr_automerge.fixture_distribution.v10",
        "distribution_id": "draft_2026_08_signed_neutral_10",
        "protocol_revision": "draft_2026_08",
        "transition_stage": stage,
        "status": "canonical_signed_neutral_corpus" if complete else "locked_transition",
        "target_fixture_count": 192,
        "fixture_count": expected_fixture_count,
        "complete": complete,
        "base_manifest_sha256": BASELINE["manifest_sha256"],
        "preserved_v9_fixture_count": 180,
        "preserved_v9_signed_events_sha256": BASELINE["signed_events_sha256"],
        "missing_v9_fixtures": [],
        "missing_v10_fixtures": list(NEW_FIXTURES[new_count:]),
        "intentional_v9_report_changes": list(CORRECTED_REPORTS),
        "requirements_sha256": digest("spec/requirements.json"),
        "authority_sha256": digest(STATE_PATH),
        "companion_sha256": digest("spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
        "conformance_sha256": digest("spec/CONFORMANCE.md"),
        "supersedes": BASE_MANIFEST_PATH,
        "v10_fixtures": list(NEW_FIXTURES),
        "profiles": profiles,
        "fixtures": entries,
        "files": files,
    }


def validate_v10_manifest(
    stage: str,
    manifest: dict[str, Any],
    discovered: dict[str, Path],
) -> None:
    require(set(manifest) == MANIFEST_KEYS, "v10_manifest_keys")
    expected = expected_v10_manifest(stage, discovered)
    for name in (
        "distribution_schema",
        "distribution_id",
        "protocol_revision",
        "transition_stage",
        "status",
        "base_manifest_sha256",
        "preserved_v9_signed_events_sha256",
        "requirements_sha256",
        "authority_sha256",
        "companion_sha256",
        "conformance_sha256",
        "supersedes",
    ):
        require(type(manifest.get(name)) is str, f"v10_manifest_type:{name}")
        require(manifest.get(name) == expected[name], f"v10_manifest_value:{name}")
    for name in ("target_fixture_count", "fixture_count", "preserved_v9_fixture_count"):
        require(type(manifest.get(name)) is int, f"v10_manifest_type:{name}")
        require(manifest.get(name) == expected[name], f"v10_manifest_value:{name}")
    require(type(manifest.get("complete")) is bool, "v10_manifest_type:complete")
    require(manifest.get("complete") == expected["complete"], "v10_manifest_value:complete")
    for name in (
        "missing_v9_fixtures",
        "missing_v10_fixtures",
        "intentional_v9_report_changes",
        "v10_fixtures",
    ):
        value = manifest.get(name)
        require(isinstance(value, list), f"v10_manifest_type:{name}")
        require(all(type(item) is str for item in value), f"v10_manifest_item_type:{name}")
        require(value == expected[name], f"v10_manifest_value:{name}")

    fixtures = manifest.get("fixtures")
    require(isinstance(fixtures, list), "v10_manifest_entries")
    for entry in fixtures:
        require(isinstance(entry, dict) and set(entry) == FIXTURE_ENTRY_KEYS, "v10_manifest_entry")
        require(all(type(entry[name]) is str for name in ("expected_path", "fixture_id", "metadata_path", "profile")), "v10_manifest_entry_string")
        for name in ("input_paths", "requirements"):
            values = entry.get(name)
            require(isinstance(values, list) and values and all(type(item) is str for item in values), f"v10_manifest_entry_list:{name}")
            require(len(values) == len(set(values)), f"v10_manifest_entry_unique:{name}")
        require(len(entry["input_paths"]) == 1, "v10_manifest_entry_input_count")
    identifiers = [str(entry["fixture_id"]) for entry in fixtures]
    require(identifiers == sorted(identifiers, key=str.encode), "v10_manifest_fixture_order")
    require(len(identifiers) == len(set(identifiers)), "v10_manifest_fixture_unique")
    require(fixtures == expected["fixtures"], "v10_manifest_fixture_inventory")

    baseline_entries = load_object(BASE_MANIFEST_PATH).get("fixtures")
    require(isinstance(baseline_entries, list), "v10_manifest_baseline_entries")
    entry_by_id = {str(entry["fixture_id"]): entry for entry in fixtures}
    require(
        all(
            isinstance(entry, dict)
            and entry_by_id.get(str(entry.get("fixture_id"))) == entry
            for entry in baseline_entries
        ),
        "v10_manifest_v9_entry_preservation",
    )

    profiles = manifest.get("profiles")
    require(isinstance(profiles, dict) and set(profiles) == set(PROFILE_NAMES), "v10_manifest_profiles")
    assigned: list[str] = []
    for name in PROFILE_NAMES:
        values = profiles.get(name)
        require(isinstance(values, list) and values, f"v10_manifest_profile:{name}")
        require(all(type(item) is str for item in values), f"v10_manifest_profile_type:{name}")
        require(values == sorted(set(values), key=str.encode), f"v10_manifest_profile_order:{name}")
        assigned.extend(values)
    require(len(assigned) == len(set(assigned)), "v10_manifest_profile_unique")
    require(profiles == expected["profiles"], "v10_manifest_profile_inventory")

    files = manifest.get("files")
    require(isinstance(files, list) and files, "v10_manifest_files")
    for row in files:
        require(isinstance(row, dict) and set(row) == {"path", "sha256"}, "v10_manifest_file")
        require(type(row["path"]) is str and type(row["sha256"]) is str, "v10_manifest_file_type")
    file_paths = [str(row["path"]) for row in files]
    require(file_paths == sorted(file_paths, key=str.encode), "v10_manifest_file_order")
    require(len(file_paths) == len(set(file_paths)), "v10_manifest_file_unique")
    require(files == expected["files"], "v10_manifest_file_inventory")


def validate_distribution(state: dict[str, Any], stage: str) -> None:
    distribution = state.get("distribution")
    require(isinstance(distribution, dict), "distribution")
    expected_keys = {
        "schema_path",
        "schema_sha256",
        "baseline_manifest",
        "baseline_manifest_sha256",
        "baseline_fixture_count",
        "preserved_fixture_count",
        "target_fixture_count",
        "baseline_ordered_fixture_ids_sha256",
        "baseline_signed_events_sha256",
        "corrected_expected_reports",
        "new_fixture_groups",
    }
    require(set(distribution) == expected_keys, "distribution_keys")
    require(distribution.get("schema_path") == DISTRIBUTION_SCHEMA_PATH, "distribution_schema_path")
    require(distribution.get("schema_sha256") == digest(DISTRIBUTION_SCHEMA_PATH), "distribution_schema_hash")
    require(distribution.get("baseline_manifest") == BASE_MANIFEST_PATH, "baseline_manifest_path")
    require(distribution.get("baseline_manifest_sha256") == BASELINE["manifest_sha256"], "baseline_manifest_binding")
    require(digest(BASE_MANIFEST_PATH) == BASELINE["manifest_sha256"], "baseline_manifest_changed")
    require(
        distribution.get("baseline_fixture_count")
        == distribution.get("preserved_fixture_count")
        == 180,
        "baseline_fixture_count",
    )
    require(distribution.get("target_fixture_count") == 192, "target_fixture_count")
    require(
        distribution.get("baseline_ordered_fixture_ids_sha256")
        == BASELINE["ordered_fixture_ids_sha256"],
        "baseline_fixture_order",
    )
    require(
        distribution.get("baseline_signed_events_sha256") == BASELINE["signed_events_sha256"],
        "baseline_signed_events",
    )
    corrections = distribution.get("corrected_expected_reports")
    require(isinstance(corrections, list), "corrected_reports")
    actual_corrections = tuple(
        (
            row.get("fixture_id"),
            row.get("metadata_path"),
            row.get("baseline_metadata_sha256"),
            row.get("metadata_invariant_sha256"),
            row.get("expected_path"),
            row.get("baseline_expected_sha256"),
        )
        for row in corrections
        if isinstance(row, dict)
    )
    require(actual_corrections == CORRECTION_BINDINGS, "corrected_report_bindings")
    groups = distribution.get("new_fixture_groups")
    require(isinstance(groups, list), "new_fixture_groups")
    actual_groups = tuple(
        (row.get("group"), tuple(row.get("fixture_ids", [])))
        for row in groups
        if isinstance(row, dict)
    )
    require(actual_groups == NEW_FIXTURE_GROUPS, "new_fixture_inventory")

    manifest = load_object(BASE_MANIFEST_PATH)
    entries = manifest.get("fixtures")
    files = manifest.get("files")
    require(isinstance(entries, list) and len(entries) == 180, "baseline_manifest_entries")
    require(isinstance(files, list), "baseline_manifest_files")
    baseline_ids = [entry.get("fixture_id") for entry in entries if isinstance(entry, dict)]
    require(len(baseline_ids) == len(entries) == len(set(baseline_ids)), "baseline_fixture_ids")
    require(ordered_digest([str(identifier) for identifier in baseline_ids]) == BASELINE["ordered_fixture_ids_sha256"], "baseline_fixture_ids_digest")
    require(signed_event_set_sha256(entries) == BASELINE["signed_events_sha256"], "baseline_signed_inputs_changed")
    file_hashes = {
        row.get("path"): row.get("sha256")
        for row in files
        if isinstance(row, dict) and isinstance(row.get("path"), str)
    }
    require(len(file_hashes) == len(files), "baseline_file_inventory")
    entry_by_id = {str(entry["fixture_id"]): entry for entry in entries}
    changed_reports = []
    correction_by_id = {binding[0]: binding for binding in CORRECTION_BINDINGS}
    for identifier, entry in entry_by_id.items():
        metadata_path = str(entry.get("metadata_path", ""))
        input_paths = entry.get("input_paths")
        expected_path = str(entry.get("expected_path", ""))
        require(isinstance(input_paths, list) and len(input_paths) == 1, f"baseline_inputs:{identifier}")
        require(
            file_hashes.get(str(input_paths[0])) == digest(str(input_paths[0])),
            f"preserved_fixture_input:{identifier}",
        )
        binding = correction_by_id.get(identifier)
        if binding is None:
            require(file_hashes.get(metadata_path) == digest(metadata_path), f"preserved_fixture_metadata:{identifier}")
            require(file_hashes.get(expected_path) == digest(expected_path), f"preserved_fixture_report:{identifier}")
        else:
            _, bound_metadata, baseline_metadata, invariant, bound_expected, baseline_expected = binding
            require(metadata_path == bound_metadata and expected_path == bound_expected, f"correction_paths:{identifier}")
            require(file_hashes.get(metadata_path) == baseline_metadata, f"correction_baseline_metadata:{identifier}")
            require(file_hashes.get(expected_path) == baseline_expected, f"correction_baseline_report:{identifier}")
            require(metadata_invariant_sha256(metadata_path) == invariant, f"correction_metadata_invariant:{identifier}")
        if file_hashes.get(expected_path) != digest(expected_path):
            changed_reports.append(identifier)
    changed_reports.sort(key=str.encode)
    expected_requirement_count, expected_fixture_count, correction_count, new_count = STAGE_COUNTS[stage]
    del expected_requirement_count
    expected_changed = sorted(CORRECTED_REPORTS[:correction_count], key=str.encode)
    require(changed_reports == expected_changed, "authorized_report_changes")
    for identifier in CORRECTED_REPORTS:
        metadata_path = Path(str(entry_by_id[identifier]["metadata_path"]))
        metadata = load_object(metadata_path.as_posix())
        expected_path = str(entry_by_id[identifier]["expected_path"])
        current_expected_sha256 = digest(expected_path)
        if correction_count == 0:
            require(digest(metadata_path.as_posix()) == file_hashes[metadata_path.as_posix()], f"early_correction_metadata:{identifier}")
        else:
            require(metadata.get("expected", {}).get("sha256") == current_expected_sha256, f"corrected_metadata_hash:{identifier}")
            require(digest(metadata_path.as_posix()) != file_hashes[metadata_path.as_posix()], f"unchanged_correction_metadata:{identifier}")

    discovered = discover_fixture_metadata()
    expected_new = set(NEW_FIXTURES[:new_count])
    actual_ids = set(discovered)
    require(actual_ids == set(str(identifier) for identifier in baseline_ids) | expected_new, "live_fixture_inventory")
    require(len(actual_ids) == expected_fixture_count, "live_fixture_count")
    for identifier in expected_new:
        validate_new_signed_fixture(identifier, discovered[identifier])

    v10_manifest_path = ROOT / V10_MANIFEST_PATH
    if STAGES.index(stage) < STAGES.index("distribution_locked"):
        require(not v10_manifest_path.exists(), "early_v10_manifest")
    else:
        validate_v10_manifest(stage, load_object(V10_MANIFEST_PATH), discovered)


def normalized_plan_projection(plan: str) -> str:
    require("\r" not in plan and plan.endswith("\n"), "transition_plan_line_endings")
    lines = plan[:-1].split("\n")
    progress_indexes = [
        index for index, line in enumerate(lines) if line in PLAN_PROGRESS_HEADINGS
    ]
    require(bool(progress_indexes), "transition_plan_progress_heading")
    progress_index = progress_indexes[0]
    immutable_lines = lines[:progress_index]
    progress_lines = lines[progress_index:]

    normalized: list[str] = []
    top_fields: set[str] = set()
    rcld_statuses: set[int] = set()
    current_rcld: int | None = None
    first_section_seen = False
    rcld_titles: dict[int, str] = {}
    for line in immutable_lines:
        heading = re.fullmatch(r"## RCLD (\d+) — ([^\n]+)", line)
        if heading is not None:
            current_rcld = int(heading.group(1))
            require(current_rcld not in rcld_titles, "transition_plan_duplicate_rcld")
            rcld_titles[current_rcld] = heading.group(2)
            first_section_seen = True
            normalized.append(line)
            continue
        if line.startswith("## "):
            current_rcld = None
            first_section_seen = True
        top_field = next(
            (prefix for prefix in PLAN_PROGRESS_FIELDS if line.startswith(prefix)),
            None,
        )
        if top_field is not None and not first_section_seen:
            require(top_field not in top_fields, "transition_plan_duplicate_progress_field")
            require(bool(line[len(top_field) :].strip()), "transition_plan_empty_progress_field")
            top_fields.add(top_field)
            normalized.append(f"{top_field}<progress>")
            continue
        if line.startswith("Status: ") and current_rcld is not None:
            require(current_rcld not in rcld_statuses, "transition_plan_duplicate_rcld_status")
            require(bool(line.removeprefix("Status: ").strip()), "transition_plan_empty_rcld_status")
            rcld_statuses.add(current_rcld)
            normalized.append("Status: <progress>")
            continue
        normalized.append(line)
    require(top_fields == set(PLAN_PROGRESS_FIELDS), "transition_plan_progress_fields")
    require(
        rcld_statuses == {rcld for rcld, _, _ in RCLD_STEP_RANGES},
        "transition_plan_rcld_statuses",
    )

    progress_headings = 0
    progress_rows: dict[int, str] = {}
    progress_summaries = 0
    for line in progress_lines:
        if not line:
            continue
        if line in PLAN_PROGRESS_HEADINGS:
            progress_headings += 1
            continue
        row = re.fullmatch(r"- RCLD (\d+) — ([^\n]+)", line)
        if row is not None:
            rcld = int(row.group(1))
            require(rcld not in progress_rows, "transition_plan_duplicate_progress_rcld")
            progress_rows[rcld] = row.group(2)
            continue
        require(
            re.fullmatch(
                r"All 126 checkpoints from `step_1158` through `step_1283` "
                r"(?:remain unfinished|are in progress|are complete)\.",
                line,
            )
            is not None,
            "transition_plan_progress_line",
        )
        progress_summaries += 1
    require(progress_headings >= 1, "transition_plan_progress_headings")
    require(progress_summaries == 1, "transition_plan_progress_summary")
    require(progress_rows == rcld_titles, "transition_plan_progress_inventory")
    normalized.extend(("## RCLD Progress", ""))
    normalized.extend(
        f"- RCLD {rcld} — {rcld_titles[rcld]}" for rcld in sorted(rcld_titles)
    )
    normalized.extend(
        (
            "",
            "All 126 checkpoints from `step_1158` through `step_1283` <progress>.",
        )
    )
    return "\n".join(normalized) + "\n"


def validate_plan_semantics(plan: str) -> None:
    require(
        plan.startswith("# nostr_automerge Draft V1 Follow-up Remediation V9 Multi-RCLD\n"),
        "transition_plan_title",
    )
    status = re.search(r"^Status: ([^\n]+)$", plan, flags=re.MULTILINE)
    require(status is not None and bool(status.group(1).strip()), "transition_plan_status")
    require(
        "Steps: `step_1158` through `step_1283` (126 contiguous checkpoints)"
        in plan,
        "transition_plan_range",
    )
    headers = list(
        re.finditer(r"^## RCLD (\d+) — ([^\n]+)$", plan, flags=re.MULTILINE)
    )
    require(
        [int(match.group(1)) for match in headers]
        == [rcld for rcld, _, _ in RCLD_STEP_RANGES],
        "transition_plan_rclds",
    )
    all_steps: list[int] = []
    for index, (rcld, first, last) in enumerate(RCLD_STEP_RANGES):
        start = headers[index].end()
        end = headers[index + 1].start() if index + 1 < len(headers) else len(plan)
        section = plan[start:end]
        steps = [
            int(value)
            for value in re.findall(
                r"^\| `step_(\d+)` \|", section, flags=re.MULTILINE
            )
        ]
        expected_steps = list(range(first, last + 1))
        require(steps == expected_steps, f"transition_plan_steps:RCLD_{rcld}")
        require(
            f"Steps: `step_{first}` through `step_{last}`" in section,
            f"transition_plan_declared_range:RCLD_{rcld}",
        )
        require(
            re.search(r"^Status: [^\n]+$", section, flags=re.MULTILINE)
            is not None,
            f"transition_plan_rcld_status:RCLD_{rcld}",
        )
        require(
            re.search(r"^Gate: `?[A-Z0-9_]+`?$", section, flags=re.MULTILINE)
            is not None,
            f"transition_plan_gate:RCLD_{rcld}",
        )
        all_steps.extend(steps)
    require(all_steps == list(range(1158, 1284)), "transition_plan_contiguous_steps")

    for required_text in (
        "## Approved Planning Deviation",
        "The execution replacement is approved as",
        "replacement action:",
        "execute the 126 checkpoints in RCLD 81 through RCLD 94",
        "retain FINDING_080 as held",
        "preserve exactly 148 requirements and 192 signed",
        "with no wire revision",
        "The durable runtime deviation record created in `step_1158`",
        "This sequence does not edit `spec/NIP_DRAFT.md`",
        "No checkpoint authorizes push, pull request, tag, release, deployment,",
        "No checkpoint adds networking, persistence, async runtime, FFI, application",
        "`code_complete_publication_held`",
    ):
        require(required_text in plan, f"transition_plan_semantics:{required_text}")
    for marker in (
        "/" + "Users/",
        "/" + "home/",
        "domains/" + "triesap",
        "triesap/" + "dev",
        "docs/" + "handoff",
    ):
        require(marker not in plan, "transition_plan_scope_leak")
    require(
        hashlib.sha256(normalized_plan_projection(plan).encode("utf-8")).hexdigest()
        == PLAN_IMMUTABLE_PROJECTION_SHA256,
        "transition_plan_immutable_projection",
    )


def plan_header_fields(plan: str) -> dict[str, str]:
    boundary = plan.find("\n## Outcome\n")
    require(boundary >= 0, "transition_plan_progress_header")
    header = plan[:boundary]
    values: dict[str, str] = {}
    for prefix in PLAN_PROGRESS_FIELDS:
        matches = re.findall(
            rf"^{re.escape(prefix)}([^\n]+)$",
            header,
            flags=re.MULTILINE,
        )
        require(len(matches) == 1, f"transition_plan_progress_field:{prefix}")
        values[prefix] = matches[0]
    return values


def step_rcld(number: int) -> int:
    for rcld, first, last in RCLD_STEP_RANGES:
        if first <= number <= last:
            return rcld
    raise TransitionError("transition_plan_progress_step_range")


def plan_rcld_statuses(plan: str) -> dict[int, str]:
    headers = list(re.finditer(r"^## RCLD (\d+) — [^\n]+$", plan, re.MULTILINE))
    statuses: dict[int, str] = {}
    for index, header in enumerate(headers):
        start = header.end()
        end = headers[index + 1].start() if index + 1 < len(headers) else len(plan)
        matches = re.findall(r"^Status: ([^\n]+)$", plan[start:end], re.MULTILINE)
        require(len(matches) == 1, "transition_plan_progress_rcld_status")
        statuses[int(header.group(1))] = matches[0]
    return statuses


def plan_progress_groups(plan: str) -> dict[str, tuple[int, ...]]:
    lines = plan.splitlines()
    indexes = [index for index, line in enumerate(lines) if line in PLAN_PROGRESS_HEADINGS]
    require(indexes, "transition_plan_progress_groups")
    groups: dict[str, list[int]] = {}
    current = ""
    for line in lines[indexes[0] :]:
        if line in PLAN_PROGRESS_HEADINGS:
            current = line
            require(current not in groups, "transition_plan_progress_group_duplicate")
            groups[current] = []
            continue
        row = re.fullmatch(r"- RCLD (\d+) — [^\n]+", line)
        if row is not None:
            require(bool(current), "transition_plan_progress_group_missing")
            groups[current].append(int(row.group(1)))
    return {name: tuple(values) for name, values in groups.items()}


def plan_execution_rows(plan: str) -> dict[str, tuple[str, str]]:
    rows: dict[str, tuple[str, str]] = {}
    for line in plan.splitlines():
        if not line.startswith("| `step_"):
            continue
        fields = [field.strip() for field in line.split("|")[1:-1]]
        require(len(fields) == 5, "transition_plan_progress_execution_shape")
        step = fields[0].strip("`")
        require(step not in rows, "transition_plan_progress_execution_duplicate")
        rows[step] = (fields[1], fields[4].strip("`"))
    return rows


def validate_progress_predecessors(plan: str, runtime: dict[str, Any]) -> int:
    predecessors = runtime.get("predecessors")
    require(isinstance(predecessors, list), "transition_plan_runtime_predecessors")
    require(10 <= len(predecessors) <= 125, "transition_plan_runtime_predecessor_count")
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
    execution = plan_execution_rows(plan)
    candidates: list[str] = []
    for index, row in enumerate(predecessors):
        require(
            isinstance(row, dict) and set(row) == expected_keys,
            "transition_plan_runtime_predecessor_shape",
        )
        step = f"step_{1158 + index}"
        require(row.get("step") == step, "transition_plan_runtime_predecessor_step")
        candidate = row.get("candidate")
        require(
            isinstance(candidate, str)
            and re.fullmatch(r"[0-9a-f]{40}", candidate) is not None,
            "transition_plan_runtime_predecessor_candidate",
        )
        candidates.append(candidate)
        require(step in execution, "transition_plan_runtime_predecessor_execution")
        owner, gate = execution[step]
        require(
            row.get("owner_class")
            == {"public Rust": "public", "private TypeScript": "opaque_private"}.get(owner),
            "transition_plan_runtime_predecessor_owner",
        )
        require(row.get("gate_ids") == [gate], "transition_plan_runtime_predecessor_gate")
        for field in ("requirement_ids", "finding_ids", "deviation_ids"):
            values = row.get(field)
            require(
                isinstance(values, list)
                and all(isinstance(value, str) for value in values),
                f"transition_plan_runtime_predecessor_{field}_type",
            )
            require(
                len(values) == len(set(values)),
                f"transition_plan_runtime_predecessor_{field}",
            )
        require(row.get("result") == "pass", "transition_plan_runtime_predecessor_result")
    require(len(candidates) == len(set(candidates)), "transition_plan_runtime_candidates")
    return len(predecessors)


def validate_current_plan_progress(
    plan: str, runtime: dict[str, Any], stage: str
) -> None:
    status = runtime.get("status")
    require(
        status in {"in_progress", "code_complete_publication_held"},
        "transition_plan_runtime_status",
    )
    terminal = status == "code_complete_publication_held"
    cursor = runtime.get("cursor")
    projection = runtime.get("authority_projection")
    require(isinstance(cursor, dict), "transition_plan_runtime_cursor")
    require(isinstance(projection, dict), "transition_plan_runtime_projection")
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
        "transition_plan_runtime_cursor_shape",
    )
    active = cursor.get("active_step")
    following = cursor.get("next_step")
    require(isinstance(active, str), "transition_plan_runtime_active")
    require(isinstance(following, str), "transition_plan_runtime_next")
    active_match = re.fullmatch(r"step_(\d{4})", active)
    following_match = re.fullmatch(r"step_(\d{4})", following)
    require(active_match is not None, "transition_plan_runtime_active_shape")
    require(following_match is not None, "transition_plan_runtime_next_shape")
    active_number = int(active_match.group(1))
    following_number = int(following_match.group(1))
    predecessor_count = validate_progress_predecessors(plan, runtime)
    require(
        active_number == 1158 + predecessor_count,
        "transition_plan_runtime_predecessor_binding",
    )
    require(1168 <= active_number <= 1283, "transition_plan_runtime_active_range")
    require(following_number == active_number + 1, "transition_plan_runtime_contiguous")
    require(cursor.get("last_step") == "step_1283", "transition_plan_runtime_last_step")
    active_rcld = step_rcld(active_number)
    following_rcld = (
        step_rcld(following_number) if following_number <= 1283 else active_rcld
    )
    require(runtime.get("rcld") == active_rcld, "transition_plan_runtime_rcld")
    require(cursor.get("first_rcld") == active_rcld, "transition_plan_runtime_first_rcld")
    require(cursor.get("last_rcld") == 94, "transition_plan_runtime_last_rcld")
    require(
        cursor.get("remaining_checkpoint_count")
        == (0 if terminal else 1283 - active_number + 1),
        "transition_plan_runtime_remaining_checkpoints",
    )
    require(
        cursor.get("remaining_rcld_count")
        == (0 if terminal else 94 - active_rcld + 1),
        "transition_plan_runtime_remaining_rclds",
    )
    require(not terminal or active_number == 1283, "transition_plan_runtime_terminal_step")
    require(
        not terminal or stage == "distribution_complete",
        "transition_plan_runtime_terminal_stage",
    )
    findings = runtime.get("findings")
    require(isinstance(findings, dict), "transition_plan_runtime_findings")
    require(
        findings.get("status")
        == (
            "code_complete_publication_held"
            if terminal
            else "implementation_remediation_required"
        ),
        "transition_plan_runtime_finding_status",
    )
    require(projection.get("current_stage") == stage, "transition_plan_runtime_stage")
    requirement_count, fixture_count, _, _ = STAGE_COUNTS[stage]
    require(
        projection.get("requirement_count") == requirement_count,
        "transition_plan_runtime_requirements",
    )
    require(
        projection.get("signed_fixture_count") == fixture_count,
        "transition_plan_runtime_fixtures",
    )

    fields = plan_header_fields(plan)
    require(
        fields
        == {
            "Status: ": (
                "code complete — publication held"
                if terminal
                else "in progress — approved for execution"
            ),
            "Active RCLD: ": f"RCLD {active_rcld}",
            "Active checkpoint: ": f"`{active}`",
            "Next RCLD: ": f"RCLD {following_rcld}",
            "Next checkpoint: ": f"`{following}`",
        },
        "transition_plan_progress_header_binding",
    )

    completed = tuple(
        rcld
        for rcld, _, last in RCLD_STEP_RANGES
        if last <= active_number
    )
    unfinished = tuple(
        rcld for rcld, _, last in RCLD_STEP_RANGES if last > active_number
    )
    statuses = plan_rcld_statuses(plan)
    expected_statuses = {
        rcld: ("complete" if rcld in completed else "planned")
        for rcld, _, _ in RCLD_STEP_RANGES
    }
    require(statuses == expected_statuses, "transition_plan_progress_statuses")
    require(
        plan_progress_groups(plan)
        == {
            "## Completed RCLDs": completed,
            "## Unfinished RCLDs": unfinished,
        },
        "transition_plan_progress_groups_binding",
    )
    require(
        plan.count(
            "All 126 checkpoints from `step_1158` through `step_1283` "
            + ("are complete." if terminal else "are in progress.")
        )
        == 1,
        "transition_plan_progress_summary_binding",
    )


def current_plan_progress_self_test(plan: str, runtime: dict[str, Any], stage: str) -> int:
    future_runtime = copy.deepcopy(runtime)
    future_runtime["predecessors"].append(
        {
            "step": "step_1174",
            "candidate": "f" * 40,
            "owner_class": "public",
            "gate_ids": ["V-RUST"],
            "requirement_ids": [],
            "finding_ids": [],
            "deviation_ids": [],
            "result": "pass",
        }
    )
    future_runtime["cursor"].update(
        {
            "active_step": "step_1175",
            "next_step": "step_1176",
            "remaining_checkpoint_count": 109,
            "remaining_rcld_count": 13,
        }
    )
    future_plan = plan.replace(
        "Active checkpoint: `step_1174`",
        "Active checkpoint: `step_1175`",
        1,
    ).replace(
        "Next checkpoint: `step_1175`",
        "Next checkpoint: `step_1176`",
        1,
    )
    validate_plan_semantics(future_plan)
    validate_current_plan_progress(future_plan, future_runtime, stage)

    coordinated_runtime = copy.deepcopy(future_runtime)
    coordinated_runtime["predecessors"].pop()
    appended_without_cursor = copy.deepcopy(runtime)
    appended_without_cursor["predecessors"].append(
        copy.deepcopy(future_runtime["predecessors"][-1])
    )
    stale_checkpoint_count = copy.deepcopy(future_runtime)
    stale_checkpoint_count["cursor"]["remaining_checkpoint_count"] = 110
    stale_rcld_count = copy.deepcopy(future_runtime)
    stale_rcld_count["cursor"]["remaining_rcld_count"] = 14
    stale_stage_count = copy.deepcopy(future_runtime)
    stale_stage_count["authority_projection"]["signed_fixture_count"] = 183
    stale_finding_status = copy.deepcopy(future_runtime)
    stale_finding_status["findings"]["status"] = "code_complete_publication_held"
    stale_groups = future_plan.replace(
        "## Completed RCLDs\n\n- RCLD 81 — Authority, Deviation, And Reproducible Baseline\n\n"
        "## Unfinished RCLDs\n\n",
        "## Completed RCLDs\n\n## Unfinished RCLDs\n\n"
        "- RCLD 81 — Authority, Deviation, And Reproducible Baseline\n",
        1,
    )
    stale_summary = future_plan.replace(
        "All 126 checkpoints from `step_1158` through `step_1283` are in progress.",
        "All 126 checkpoints from `step_1158` through `step_1283` remain unfinished.",
        1,
    )
    mutations = (
        (
            "active_step",
            plan.replace(
                "Active checkpoint: `step_1174`",
                "Active checkpoint: `step_9999`",
                1,
            ),
            runtime,
        ),
        (
            "next_step",
            plan.replace(
                "Next checkpoint: `step_1175`",
                "Next checkpoint: `step_9998`",
                1,
            ),
            runtime,
        ),
        (
            "active_rcld",
            plan.replace("Active RCLD: RCLD 82", "Active RCLD: RCLD 81", 1),
            runtime,
        ),
        (
            "next_rcld",
            plan.replace("Next RCLD: RCLD 82", "Next RCLD: RCLD 81", 1),
            runtime,
        ),
        (
            "status",
            plan.replace(
                "Status: in progress — approved for execution",
                "Status: planned — approved for execution",
                1,
            ),
            runtime,
        ),
        (
            "rcld_status",
            plan.replace(
                "## RCLD 81 — Authority, Deviation, And Reproducible Baseline\n\nStatus: complete",
                "## RCLD 81 — Authority, Deviation, And Reproducible Baseline\n\nStatus: planned",
                1,
            ),
            runtime,
        ),
        (
            "runtime_rcld",
            plan,
            {**runtime, "rcld": 81},
        ),
        (
            "runtime_status",
            plan,
            {**runtime, "status": "code_complete_publication_held"},
        ),
        (
            "partial_plan_advance",
            future_plan,
            runtime,
        ),
        (
            "coordinated_without_predecessor",
            future_plan,
            coordinated_runtime,
        ),
        (
            "appended_without_cursor",
            plan,
            appended_without_cursor,
        ),
        (
            "stale_checkpoint_count",
            future_plan,
            stale_checkpoint_count,
        ),
        (
            "stale_rcld_count",
            future_plan,
            stale_rcld_count,
        ),
        (
            "stale_stage_count",
            future_plan,
            stale_stage_count,
        ),
        (
            "stale_finding_status",
            future_plan,
            stale_finding_status,
        ),
        (
            "stale_groups",
            stale_groups,
            future_runtime,
        ),
        (
            "stale_summary",
            stale_summary,
            future_runtime,
        ),
    )
    caught = 0
    for name, candidate, candidate_runtime in mutations:
        require(candidate != plan or candidate_runtime != runtime, f"progress_mutation:{name}")
        try:
            validate_current_plan_progress(candidate, candidate_runtime, stage)
        except TransitionError:
            caught += 1
            continue
        raise TransitionError(f"progress_mutation_survived:{name}")
    return caught


def validate_transition_baseline(state: dict[str, Any], stage: str) -> None:
    baseline = state.get("transition_baseline")
    require(isinstance(baseline, dict), "transition_baseline")
    require(
        set(baseline) == {"status", "plan_binding", "artifacts"},
        "transition_baseline_keys",
    )
    require(baseline.get("status") == "step_1158_bound", "transition_baseline_status")
    require(
        baseline.get("plan_binding") == "initial_exact_then_immutable_projection",
        "transition_baseline_plan_binding",
    )
    artifacts = baseline.get("artifacts")
    require(isinstance(artifacts, list), "transition_baseline_artifacts")
    rows = tuple(
        (row.get("path"), row.get("sha256"))
        for row in artifacts
        if isinstance(row, dict) and set(row) == {"path", "sha256"}
    )
    require(rows == TRANSITION_BASELINE, "transition_baseline_inventory")
    plan_path, plan_sha256 = TRANSITION_BASELINE[0]
    plan = load_strict_lf_utf8(plan_path)
    validate_plan_semantics(plan)
    validate_current_plan_progress(
        plan,
        load_object("implementation/runtime_ledger_v9.json"),
        stage,
    )
    if stage == "transition_installed":
        require(digest(plan_path) == plan_sha256, "transition_plan_initial_hash")
    for relative, expected_sha256 in TRANSITION_BASELINE[1:]:
        require(digest(relative) == expected_sha256, f"transition_baseline_hash:{relative}")

    remediation = load_object("reports/remediation_v9_baseline.json")
    require(remediation.get("schema") == "nostr_automerge.remediation_v9_baseline.v1", "transition_remediation_schema")
    require(remediation.get("status") == "implementation_remediation_required", "transition_remediation_status")
    planning = remediation.get("planning_approval")
    require(isinstance(planning, dict), "transition_planning")
    require(
        planning
        == {
            "status": "planned_approved_for_execution",
            "mode": "rcl-durable",
            "active_rcld": 81,
            "active_checkpoint": "step_1158",
            "first_rcld": 81,
            "last_rcld": 94,
            "rcld_count": 14,
            "first_checkpoint": "step_1158",
            "last_checkpoint": "step_1283",
            "checkpoint_count": 126,
            "deviation_record": "implementation/deviations/step_1158.md",
        },
        "transition_planning_binding",
    )
    authority = remediation.get("authority")
    require(isinstance(authority, dict), "transition_remediation_authority")
    require(authority.get("nip_sha256") == BASELINE["nip_sha256"], "transition_remediation_nip")
    require(authority.get("companion_sha256") == BASELINE["companion_sha256"], "transition_remediation_companion")
    require(authority.get("requirements_sha256") == BASELINE["requirements_sha256"], "transition_remediation_requirements")
    require(authority.get("applicability_sha256") == BASELINE["applicability_sha256"], "transition_remediation_applicability")
    require(authority.get("distribution_v9_manifest_sha256") == BASELINE["manifest_sha256"], "transition_remediation_manifest")
    require(authority.get("nip_edit_authorized") is False, "transition_remediation_nip_scope")
    require(remediation.get("counts") == {"requirements": 139, "signed_fixtures": 180}, "transition_remediation_counts")
    require(remediation.get("maximum_local_claim") == "code_complete_publication_held", "transition_remediation_claim")
    require(remediation.get("remote_actions_authorized") is False, "transition_remediation_remote")
    require(remediation.get("publication_authorized") is False, "transition_remediation_publication")


def validate_embedded_hashes(value: Any, diagnostic: str) -> None:
    if isinstance(value, dict):
        rows = list(value.items())
    elif isinstance(value, list):
        rows = [
            (row.get("path"), row.get("sha256"))
            for row in value
            if isinstance(row, dict) and set(row) == {"path", "sha256"}
        ]
        require(len(rows) == len(value), f"{diagnostic}_rows")
    else:
        raise TransitionError(f"{diagnostic}_type")
    require(rows, f"{diagnostic}_empty")
    require(len(rows) == len({str(path) for path, _ in rows}), f"{diagnostic}_unique")
    for path, expected_sha256 in rows:
        require(type(path) is str and type(expected_sha256) is str, f"{diagnostic}_row")
        require(digest(path) == expected_sha256, f"{diagnostic}_hash:{path}")


def validate_historical_transitive_evidence() -> None:
    supersession = load_object("reports/evidence_supersession_v8.json")
    require(supersession.get("schema") == "nostr_automerge.evidence_supersession.v8", "historical_supersession_schema")
    require(supersession.get("status") == "v9_authoritative", "historical_supersession_status")
    validate_embedded_hashes(supersession.get("authoritative"), "historical_authoritative")
    validate_embedded_hashes(supersession.get("superseded"), "historical_superseded")

    final_identity = load_object("reports/final_candidate_identity_v8.json")
    require(final_identity.get("schema") == "nostr_automerge.final_candidate_identity.v8", "historical_identity_schema")
    require(final_identity.get("status") == "code_complete_publication_held", "historical_identity_status")
    require(final_identity.get("result") == "bound", "historical_identity_result")
    require(final_identity.get("publication_authorized") is False, "historical_identity_publication")
    validate_embedded_hashes(final_identity.get("evidence"), "historical_identity_evidence")
    identity_authority = final_identity.get("authority")
    require(isinstance(identity_authority, dict), "historical_identity_authority")
    require(identity_authority.get("nip_sha256") == BASELINE["nip_sha256"], "historical_identity_nip")
    require(identity_authority.get("companion_sha256") == BASELINE["companion_sha256"], "historical_identity_companion")
    require(identity_authority.get("requirements_sha256") == BASELINE["requirements_sha256"], "historical_identity_requirements")
    require(identity_authority.get("applicability_sha256") == BASELINE["applicability_sha256"], "historical_identity_applicability")
    require(identity_authority.get("fixture_distribution_sha256") == BASELINE["manifest_sha256"], "historical_identity_manifest")

    remediation = load_object("reports/remediation_v8_final.json")
    require(remediation.get("schema") == "nostr_automerge.remediation_v8_final.v1", "historical_remediation_schema")
    require(remediation.get("status") == "code_complete_publication_held", "historical_remediation_status")
    require(remediation.get("local_implementation") == "pass", "historical_remediation_local")
    require(remediation.get("publication_authorized") is False, "historical_remediation_publication")
    require(remediation.get("remote_actions_performed") is False, "historical_remediation_remote")
    validate_embedded_hashes(remediation.get("evidence"), "historical_remediation_evidence")

    holds = load_object("reports/external_holds_v8.json")
    require(holds.get("schema") == "nostr_automerge.external_holds.v8", "historical_holds_schema")
    require(holds.get("status") == "code_complete_publication_held", "historical_holds_status")
    require(holds.get("remote_actions_performed") is False, "historical_holds_remote")
    hold_rows = holds.get("holds")
    require(isinstance(hold_rows, list) and hold_rows, "historical_holds")
    require(
        all(
            isinstance(row, dict)
            and row.get("executed") is False
            and row.get("result_claimed") is False
            for row in hold_rows
        ),
        "historical_hold_execution",
    )


def validate_supersession(state: dict[str, Any]) -> None:
    evidence = state.get("v9_evidence")
    require(isinstance(evidence, dict), "v9_evidence")
    require(set(evidence) == {"status", "superseded_by", "artifacts"}, "v9_evidence_keys")
    require(evidence.get("status") == "historical_superseded_non_current", "v9_evidence_status")
    require(evidence.get("superseded_by") == STATE_PATH, "v9_evidence_successor")
    artifacts = evidence.get("artifacts")
    require(isinstance(artifacts, list), "v9_evidence_artifacts")
    rows = tuple(
        (row.get("path"), row.get("sha256"))
        for row in artifacts
        if isinstance(row, dict) and set(row) == {"path", "sha256"}
    )
    require(rows == V9_EVIDENCE, "v9_evidence_inventory")
    for relative, expected_sha256 in V9_EVIDENCE:
        require(digest(relative) == expected_sha256, f"v9_evidence_hash:{relative}")
    validate_historical_transitive_evidence()


def validate_public_boundary() -> None:
    public_text = "\n".join(
        (ROOT / relative).read_text(encoding="utf-8")
        for relative in (
            STATE_PATH,
            AUTHORITY_SCHEMA_PATH,
            DISTRIBUTION_SCHEMA_PATH,
            "scripts/validate_authority_transition_v10.py",
        )
    )
    require(
        not any(marker in public_text for marker in FORBIDDEN_PUBLIC_MARKERS),
        "private_or_workflow_material",
    )


def validate_state(state: dict[str, Any]) -> None:
    require(
        set(state)
        == {
            "schema",
            "status",
            "protocol_revision",
            "current_stage",
            "stage_order",
            "transition_baseline",
            "authority",
            "distribution",
            "v9_evidence",
        },
        "state_keys",
    )
    require(state.get("schema") == "nostr_automerge.authority_transition.v10", "state_schema")
    require(state.get("status") == "in_progress", "state_status")
    require(state.get("protocol_revision") == "draft_2026_08", "state_revision")
    require(state.get("stage_order") == list(STAGES), "stage_order")
    stage = state.get("current_stage")
    require(isinstance(stage, str) and stage in STAGES, "current_stage")
    validate_transition_baseline(state, stage)
    validate_requirements(state, stage)
    validate_distribution(state, stage)
    validate_supersession(state)


def mutation_self_test(state: dict[str, Any]) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    current_stage = str(state["current_stage"])
    current_index = STAGES.index(current_stage)
    caught = 0
    if current_index >= STAGES.index("requirements_appended"):
        registry = load_object("spec/requirements.json")
        applicability = load_object("spec/requirements_applicability.json")
        baseline_rows = load_object("reports/requirements_coverage_v9.json").get("rows")
        require(isinstance(baseline_rows, list), "mutation_baseline_evidence_rows")
        source_documents = {
            relative: load_strict_lf_utf8(relative)
            for relative in {row["source"] for row in APPENDED_REQUIREMENT_ROWS}
        }
        caught += requirement_projection_self_test(
            registry,
            applicability,
            baseline_rows,
            source_documents,
        )
    claimed_index = current_index + 1 if current_index + 1 < len(STAGES) else current_index - 1
    skipped = copy.deepcopy(state)
    skipped["current_stage"] = STAGES[claimed_index]
    mutations.append(("unbacked_stage_claim", skipped))
    regressed = copy.deepcopy(state)
    regressed["current_stage"] = "transition_installed"
    mutations.append(("regressed_stage_with_companion_authority", regressed))
    stale_companion = copy.deepcopy(state)
    stale_companion["authority"]["live"]["companion_sha256"] = BASELINE[
        "companion_sha256"
    ]
    mutations.append(("companion_stage_stale_live_binding", stale_companion))
    nip = copy.deepcopy(state)
    nip["authority"]["nip_sha256"] = "0" * 64
    mutations.append(("nip_binding", nip))
    requirements = copy.deepcopy(state)
    requirements["authority"]["live"]["requirements_sha256"] = "0" * 64
    mutations.append(("requirements_hash", requirements))
    appended = copy.deepcopy(state)
    appended["authority"]["appended_ids"].reverse()
    mutations.append(("requirement_append_order", appended))
    fixtures = copy.deepcopy(state)
    fixtures["distribution"]["target_fixture_count"] = 193
    mutations.append(("fixture_target", fixtures))
    corrections = copy.deepcopy(state)
    corrections["distribution"]["corrected_expected_reports"].pop()
    mutations.append(("fixture_correction_inventory", corrections))
    current_v9 = copy.deepcopy(state)
    current_v9["v9_evidence"]["status"] = "current"
    mutations.append(("v9_marked_current", current_v9))
    missing_evidence = copy.deepcopy(state)
    missing_evidence["v9_evidence"]["artifacts"].pop()
    mutations.append(("v9_evidence_omission", missing_evidence))
    rewritten_evidence = copy.deepcopy(state)
    rewritten_evidence["v9_evidence"]["artifacts"][0]["sha256"] = "0" * 64
    mutations.append(("v9_evidence_coordinated_hash", rewritten_evidence))
    rewritten_baseline = copy.deepcopy(state)
    rewritten_baseline["transition_baseline"]["artifacts"][0]["sha256"] = "0" * 64
    mutations.append(("transition_baseline_coordinated_hash", rewritten_baseline))

    for name, mutation in mutations:
        try:
            validate_state(mutation)
        except TransitionError:
            caught += 1
            continue
        raise TransitionError(f"mutation_survived:{name}")

    authority_schema = load_object(AUTHORITY_SCHEMA_PATH)
    distribution_schema = load_object(DISTRIBUTION_SCHEMA_PATH)
    weak_authority = copy.deepcopy(authority_schema)
    weak_authority["$defs"]["stage"]["enum"].append("unreviewed_stage")
    weak_distribution = copy.deepcopy(distribution_schema)
    weak_distribution["properties"]["fixture_count"]["enum"].append(191)
    open_fixture = copy.deepcopy(distribution_schema)
    open_fixture["$defs"]["fixture"]["additionalProperties"] = True
    weak_file = copy.deepcopy(distribution_schema)
    weak_file["$defs"]["file"]["required"].pop()
    for name, first, second in (
        ("authority_schema_stage", weak_authority, distribution_schema),
        ("distribution_schema_count", authority_schema, weak_distribution),
        ("distribution_schema_open_fixture", authority_schema, open_fixture),
        ("distribution_schema_weak_file", authority_schema, weak_file),
    ):
        try:
            validate_schema_contracts(first, second)
        except TransitionError:
            caught += 1
            continue
        raise TransitionError(f"mutation_survived:{name}")

    discovered = discover_fixture_metadata()
    manifest = expected_v10_manifest("distribution_locked", discovered)
    validate_v10_manifest("distribution_locked", manifest, discovered)
    manifest_mutations: list[tuple[str, dict[str, Any]]] = []
    extra_key = copy.deepcopy(manifest)
    extra_key["unreviewed"] = True
    manifest_mutations.append(("manifest_extra_key", extra_key))
    wrong_hash = copy.deepcopy(manifest)
    wrong_hash["requirements_sha256"] = "0" * 64
    manifest_mutations.append(("manifest_authority_hash", wrong_hash))
    missing_fixture = copy.deepcopy(manifest)
    missing_fixture["missing_v10_fixtures"].pop()
    manifest_mutations.append(("manifest_missing_inventory", missing_fixture))
    profile_duplicate = copy.deepcopy(manifest)
    profile_duplicate["profiles"]["core"].append(profile_duplicate["profiles"]["core"][0])
    manifest_mutations.append(("manifest_profile_duplicate", profile_duplicate))
    entry_order = copy.deepcopy(manifest)
    entry_order["fixtures"].reverse()
    manifest_mutations.append(("manifest_entry_order", entry_order))
    entry_shape = copy.deepcopy(manifest)
    entry_shape["fixtures"][0]["unreviewed"] = True
    manifest_mutations.append(("manifest_entry_shape", entry_shape))
    file_hash = copy.deepcopy(manifest)
    file_hash["files"][0]["sha256"] = "0" * 64
    manifest_mutations.append(("manifest_file_hash", file_hash))
    file_shape = copy.deepcopy(manifest)
    file_shape["files"][0]["unreviewed"] = True
    manifest_mutations.append(("manifest_file_shape", file_shape))
    for name, mutation in manifest_mutations:
        try:
            validate_v10_manifest("distribution_locked", mutation, discovered)
        except TransitionError:
            caught += 1
            continue
        raise TransitionError(f"mutation_survived:{name}")

    plan = load_strict_lf_utf8(TRANSITION_BASELINE[0][0])
    advanced_plan = plan.replace(
        "Status: planned — approved for execution",
        "Status: in progress — approved for execution",
        1,
    ).replace("Active checkpoint: `step_1158`", "Active checkpoint: `step_1159`", 1)
    advanced_plan = advanced_plan.replace("Active RCLD: RCLD 81", "Active RCLD: RCLD 82", 1)
    advanced_plan = advanced_plan.replace("Next RCLD: RCLD 81", "Next RCLD: RCLD 82", 1)
    advanced_plan = advanced_plan.replace(
        "Next checkpoint: `step_1158`", "Next checkpoint: `step_1160`", 1
    ).replace("\nStatus: planned\n", "\nStatus: complete\n")
    advanced_plan = advanced_plan.replace(
        "## Unfinished RCLDs", "## Completed RCLDs", 1
    ).replace(
        "All 126 checkpoints from `step_1158` through `step_1283` remain unfinished.",
        "All 126 checkpoints from `step_1158` through `step_1283` are complete.",
        1,
    )
    require(
        hashlib.sha256(advanced_plan.encode("utf-8")).hexdigest()
        != TRANSITION_BASELINE[0][1],
        "plan_advance_must_change_hash",
    )
    validate_plan_semantics(advanced_plan)
    plan_mutations = (
        ("plan_missing_checkpoint", plan.replace("| `step_1200` |", "| `removed_1200` |", 1)),
        ("plan_missing_deviation", plan.replace("## Approved Planning Deviation", "## Removed Planning Deviation", 1)),
        ("plan_scope_leak", plan + "\n/" + "Users/example/private-input\n"),
        ("plan_crlf", plan.replace("\n", "\r\n")),
        ("plan_bare_cr", plan.replace("\n", "\r")),
        ("plan_nel", plan.replace("\n", "\u0085", 1)),
        ("plan_unicode_line_separator", plan.replace("\n", "\u2028", 1)),
        ("plan_unicode_paragraph_separator", plan.replace("\n", "\u2029", 1)),
        ("plan_vertical_tab", plan.replace("\n", "\x0b", 1)),
        ("plan_form_feed", plan.replace("\n", "\x0c", 1)),
        (
            "plan_rcld_title",
            plan.replace(
                "## RCLD 82 — Rust Checkpoint Control Precedence",
                "## RCLD 82 — Altered Checkpoint Control Precedence",
                1,
            ),
        ),
        (
            "plan_gate",
            plan.replace("Gate: `GATE_V9_AUTHORITY`", "Gate: `GATE_V9_OTHER`", 1),
        ),
        (
            "plan_dependency",
            plan.replace("Depends on: completed RCLD 80", "Depends on: completed RCLD 79", 1),
        ),
        (
            "plan_step_git_identity",
            plan.replace(
                "| `step_1159` | public Rust |",
                "| `step_1159` | public Other |",
                1,
            ),
        ),
        (
            "plan_step_scope",
            plan.replace(
                "| `step_1159` | public Rust | Install staged v10 authority/distribution schemas",
                "| `step_1159` | public Rust | Alter staged v10 authority/distribution schemas",
                1,
            ),
        ),
        (
            "plan_step_green",
            plan.replace(
                "The current 139/180 tree is an exact allowed initial v10-transition stage",
                "The current 139/180 tree is a relaxed initial v10-transition stage",
                1,
            ),
        ),
        (
            "plan_step_lane",
            plan.replace(
                "stale v9 evidence is not treated as current. | `V-AUTH` |",
                "stale v9 evidence is not treated as current. | `V-RUST` |",
                1,
            ),
        ),
    )
    for name, mutation in plan_mutations:
        require(mutation != plan, f"mutation_not_applied:{name}")
        try:
            validate_plan_semantics(mutation)
        except TransitionError:
            caught += 1
            continue
        raise TransitionError(f"mutation_survived:{name}")
    caught += current_plan_progress_self_test(
        plan,
        load_object("implementation/runtime_ledger_v9.json"),
        current_stage,
    )
    return caught


def main() -> int:
    state = load_object(STATE_PATH)
    validate_schema_contracts(
        load_object(AUTHORITY_SCHEMA_PATH), load_object(DISTRIBUTION_SCHEMA_PATH)
    )
    validate_state(state)
    validate_public_boundary()
    mutations = mutation_self_test(state)
    stage = str(state["current_stage"])
    requirement_count, fixture_count, _, _ = STAGE_COUNTS[stage]
    print(f"PASS: authority transition v10 {stage}")
    print(f"- requirements={requirement_count}")
    print(f"- signed_fixtures={fixture_count}")
    print("- plan_binding=initial_exact_then_immutable_projection")
    print("- v9_evidence=historical_superseded_non_current")
    print(f"- negative_mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
