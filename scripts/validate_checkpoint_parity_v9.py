#!/usr/bin/env python3
"""Validate the closed checkpoint state-table parity record."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

from validate_runtime_ledger_v9 import (
    APPROVED_CHECKPOINT_RESULT_IDENTITY,
    LedgerError,
    load_object,
    projection_digest,
    require,
    validate_no_leak,
    validate_opaque_checkpoint,
    validate_schema_contract,
)


ROOT = Path(__file__).resolve().parents[1]
REPORT = "reports/checkpoint_parity_v9.json"
SCHEMA = "tools/validation/checkpoint_parity_v9.schema.json"
OPAQUE_CHECKPOINT = "reports/opaque_checkpoint_v9.json"
FIXTURE_ROOTS = (
    "fixtures/v1_draft/scenarios/checkpoint",
    "fixtures/v1_draft/scenarios/checkpoints",
)
SCHEMA_PROJECTION = (
    "8a39124411fb6683315ba4a6d705045114a8ac76d27daf3b323d5a001337ce14"
)
APPROVED_TABLE_IDENTITY = (
    "91d2d98780f7f68948892a5f70905657e621631e24977953641bc84246d45b2c"
)
APPROVED_CHECKPOINT_CANDIDATE = "d956d20699508ec8e54b660fa634ff68df323846"
PUBLIC_PARITY_CANDIDATE = "2addba148fecc8039ee26084ae499e0602c5f4ed"
APPROVED_CHECKPOINT_REPORT_PROJECTION = (
    "631759d0441b25f4c99d91406fca386eb4b29a23c86521071274ad293345c00d"
)
APPROVED_CORRECTED_EXPECTATION_PROJECTION = (
    "170d72de39705b0a3aa71cb9c2a7b22a27f6597b1bc5ae8f12d965f0cf30a908"
)
APPROVED_ATTRIBUTION_IDENTITY = (
    "300ebc22ef62c57cdcd0c4408ee42a5ab68b90655de53177f5c7d618eebbab2c"
)
APPROVED_PUBLIC_INPUT_IDENTITY = (
    "17a31eafbd3db5c56fc58dd06f250817bebd092983995a324797e40dc905f194"
)
APPROVED_RESULT_IDENTITY = (
    "b55220e99db3bf33ff9473c820a7fc4a59fb60d3fb90e847a903d94a5939606b"
)
REQUIREMENT_IDS = (
    "NCRDT-CPAUTH-001",
    "NCRDT-CPAUTH-002",
    "NCRDT-CONF-010",
    "NCRDT-EVIDENCE-006",
)


def state_row(
    state: str,
    decision: str,
    checkpoint_status: str | None,
    disposition: str | None,
    diagnostic: str | None,
    downstream_work: str,
) -> dict[str, Any]:
    return {
        "state": state,
        "decision": decision,
        "checkpoint_status": checkpoint_status,
        "disposition": disposition,
        "diagnostic": diagnostic,
        "downstream_work": downstream_work,
    }


EXPECTED_TABLE = (
    state_row("canonical_authorized", "authorized", None, None, None, "permitted"),
    state_row("missing", "refused", "pending_control", "pending", None, "prohibited"),
    state_row("pending", "refused", "pending_control", "pending", None, "prohibited"),
    state_row(
        "noncanonical",
        "refused",
        "unauthorized",
        "invalid",
        "checkpoint.history",
        "prohibited",
    ),
    state_row(
        "wrong_kind",
        "refused",
        "unauthorized",
        "invalid",
        "checkpoint.history",
        "prohibited",
    ),
    state_row(
        "wrong_coordinate",
        "refused",
        "unauthorized",
        "invalid",
        "checkpoint.history",
        "prohibited",
    ),
    state_row(
        "static_invalid",
        "refused",
        "unauthorized",
        "invalid",
        "checkpoint.history",
        "prohibited",
    ),
    state_row(
        "dynamic_invalid",
        "refused",
        "unauthorized",
        "invalid",
        "checkpoint.history",
        "prohibited",
    ),
    state_row(
        "unsupported_revision",
        "refused",
        "unauthorized",
        "invalid",
        "checkpoint.history",
        "prohibited",
    ),
    state_row(
        "role_denied",
        "refused",
        "unauthorized",
        "invalid",
        "checkpoint.history",
        "prohibited",
    ),
    state_row(
        "internal_outcome_absent",
        "refused",
        "unauthorized",
        "invalid",
        "checkpoint.history",
        "prohibited",
    ),
)


def load_json(path: Path, diagnostic: str) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise LedgerError(diagnostic) from error


def file_sha256(path: Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        raise LedgerError("parity_inventory:read") from error


def git_bytes(*arguments: str) -> bytes:
    result = subprocess.run(
        ("git", *arguments), cwd=ROOT, check=False, capture_output=True
    )
    require(result.returncode == 0 and result.stderr == b"", "parity_inventory:git")
    return result.stdout


def candidate_bytes(relative: str) -> bytes:
    return git_bytes("show", f"{PUBLIC_PARITY_CANDIDATE}:{relative}")


def candidate_json(relative: str, diagnostic: str) -> Any:
    try:
        return json.loads(candidate_bytes(relative))
    except json.JSONDecodeError as error:
        raise LedgerError(diagnostic) from error


def validate_companion_inventory(observed: set[str], expected: set[str]) -> None:
    require(observed == expected, "parity_inventory:companions_exact")


def checkpoint_inventory() -> tuple[int, int, str]:
    tree_paths = tuple(
        value.decode("utf-8")
        for value in git_bytes(
            "ls-tree", "-r", "--name-only", "-z", PUBLIC_PARITY_CANDIDATE,
            "--", *FIXTURE_ROOTS,
        )[:-1].split(b"\0")
    )
    metadata_paths = tuple(
        relative for relative in tree_paths if relative.endswith(".fixture.json")
    )
    require(len(metadata_paths) == 22, "parity_inventory:scenario_count")
    expected_companions = {
        path
        for metadata_path in metadata_paths
        for path in (
            metadata_path,
            metadata_path.replace(".fixture.json", ".input.json"),
            metadata_path.replace(".fixture.json", ".expected.json"),
        )
    }
    observed_companions = {
        relative for relative in tree_paths if relative.endswith(".json")
    }
    validate_companion_inventory(observed_companions, expected_companions)
    require(len(observed_companions) == 66, "parity_inventory:companion_count")
    projection: list[dict[str, Any]] = []
    signed_event_count = 0
    observed_names: set[str] = set()
    for index, metadata_path in enumerate(metadata_paths):
        metadata = candidate_json(metadata_path, f"parity_inventory:{index}:metadata")
        require(isinstance(metadata, dict), f"parity_inventory:{index}:metadata_shape")
        fixture_id = metadata.get("fixture_id")
        require(
            isinstance(fixture_id, str)
            and Path(metadata_path).name == f"{fixture_id}.fixture.json",
            f"parity_inventory:{index}:fixture_id",
        )
        require(fixture_id not in observed_names, f"parity_inventory:{index}:unique")
        observed_names.add(fixture_id)
        inputs = metadata.get("inputs")
        expected = metadata.get("expected")
        require(
            isinstance(inputs, list)
            and len(inputs) == 1
            and isinstance(inputs[0], dict),
            f"parity_inventory:{index}:inputs",
        )
        require(isinstance(expected, dict), f"parity_inventory:{index}:expected")
        input_row = inputs[0]
        input_name = input_row.get("path")
        expected_name = expected.get("report_path")
        require(
            input_name == f"{fixture_id}.input.json"
            and expected_name == f"{fixture_id}.expected.json",
            f"parity_inventory:{index}:companions",
        )
        parent = Path(metadata_path).parent.as_posix()
        input_path = f"{parent}/{input_name}"
        expected_path = f"{parent}/{expected_name}"
        input_sha256 = input_row.get("sha256")
        expected_sha256 = expected.get("sha256")
        require(
            isinstance(input_sha256, str)
            and hashlib.sha256(candidate_bytes(input_path)).hexdigest() == input_sha256,
            f"parity_inventory:{index}:input_identity",
        )
        require(
            isinstance(expected_sha256, str)
            and hashlib.sha256(candidate_bytes(expected_path)).hexdigest() == expected_sha256,
            f"parity_inventory:{index}:expected_identity",
        )
        input_record = candidate_json(input_path, f"parity_inventory:{index}:input")
        require(isinstance(input_record, dict), f"parity_inventory:{index}:input_shape")
        raw_events = input_record.get("raw_events")
        require(isinstance(raw_events, list), f"parity_inventory:{index}:events")
        signed_event_count += len(raw_events)
        projection.append(
            {
                "fixture_id": fixture_id,
                "input_sha256": input_sha256,
                "expected_sha256": expected_sha256,
                "signed_event_count": len(raw_events),
            }
        )
    require(signed_event_count == 75, "parity_inventory:event_count")
    identity = projection_digest(projection)
    require(identity == APPROVED_PUBLIC_INPUT_IDENTITY, "parity_inventory:projection")
    return len(metadata_paths), signed_event_count, identity


def validate_checkpoint_parity(
    report: dict[str, Any], opaque_checkpoint: dict[str, Any]
) -> None:
    expected_keys = {
        "schema",
        "checkpoint",
        "gate_id",
        "authority_stage",
        "status",
        "publication_status",
        "requirement_ids",
        "imported_checkpoint_identity_sha256",
        "public_table",
        "opaque_table",
        "opaque_attribution",
        "comparison",
        "conformance",
        "result_identity_sha256",
    }
    require(set(report) == expected_keys, "parity:keys")
    require(report.get("schema") == "nostr_automerge.checkpoint_parity.v9.v1", "parity:schema")
    require(report.get("checkpoint") == "step_1186", "parity:checkpoint")
    require(report.get("gate_id") == "GATE_V9_PRIVATE_CHECKPOINT", "parity:gate")
    require(
        report.get("authority_stage") == "checkpoint_expectations_corrected",
        "parity:stage",
    )
    require(report.get("status") == "pass", "parity:status")
    require(report.get("publication_status") == "held", "parity:publication")
    require(report.get("requirement_ids") == list(REQUIREMENT_IDS), "parity:requirements")
    validate_opaque_checkpoint(opaque_checkpoint)
    require(
        report.get("imported_checkpoint_identity_sha256")
        == opaque_checkpoint["result_identity_sha256"]
        == APPROVED_CHECKPOINT_RESULT_IDENTITY,
        "parity:imported_identity",
    )
    public_table = report.get("public_table")
    opaque_table = report.get("opaque_table")
    require(public_table == list(EXPECTED_TABLE), "parity:public_table")
    require(opaque_table == list(EXPECTED_TABLE), "parity:opaque_table")
    require(public_table == opaque_table, "parity:comparison")
    table_identity = projection_digest(public_table)
    require(table_identity == APPROVED_TABLE_IDENTITY, "parity:table_identity")
    identity_rows = {
        row["class"]: row["sha256"] for row in opaque_checkpoint["result_identities"]
    }
    attribution = report.get("opaque_attribution")
    expected_attribution = {
        "candidate": APPROVED_CHECKPOINT_CANDIDATE,
        "checkpoint_result_identity_sha256": APPROVED_CHECKPOINT_RESULT_IDENTITY,
        "checkpoint_report_projection_sha256": APPROVED_CHECKPOINT_REPORT_PROJECTION,
        "corrected_expectation_projection_sha256": APPROVED_CORRECTED_EXPECTATION_PROJECTION,
        "engine_vector_count": 11,
        "table_identity_sha256": APPROVED_TABLE_IDENTITY,
        "binding_identity_sha256": APPROVED_ATTRIBUTION_IDENTITY,
        "result": "bound",
    }
    require(attribution == expected_attribution, "parity:opaque_attribution")
    require(
        opaque_checkpoint["candidate_chain"][-1]["candidate"]
        == attribution["candidate"],
        "parity:attribution_candidate",
    )
    require(
        identity_rows.get("checkpoint_report_projection")
        == attribution["checkpoint_report_projection_sha256"],
        "parity:attribution_checkpoint_projection",
    )
    require(
        identity_rows.get("corrected_expectation_projection")
        == attribution["corrected_expectation_projection_sha256"],
        "parity:attribution_expectation_projection",
    )
    attribution_projection = copy.deepcopy(attribution)
    attribution_projection.pop("binding_identity_sha256")
    attribution_projection["opaque_table"] = opaque_table
    require(
        projection_digest(attribution_projection) == APPROVED_ATTRIBUTION_IDENTITY,
        "parity:attribution_binding",
    )
    require(
        report.get("comparison")
        == {
            "projection_class": "external_checkpoint_contract",
            "state_count": len(EXPECTED_TABLE),
            "table_identity_sha256": APPROVED_TABLE_IDENTITY,
            "result": "exact",
        },
        "parity:comparison_record",
    )
    scenario_count, event_count, public_input_identity = checkpoint_inventory()
    opaque_counts = opaque_checkpoint["result_counts"]
    require(
        report.get("conformance")
        == {
            "signed_scenario_count": scenario_count,
            "signed_event_count": event_count,
            "public_input_identity_sha256": public_input_identity,
            "engine_vector_count": opaque_counts["engine_vectors"],
            "delivery_order_count": opaque_counts["delivery_orders"],
            "fixed_regression_count": opaque_counts["fixed_regressions"],
            "open_regression_count": opaque_counts["open_regressions"],
            "public_result": "pass",
            "opaque_result": opaque_checkpoint["execution_result"],
        },
        "parity:conformance",
    )
    require(scenario_count == opaque_counts["signed_scenarios"], "parity:scenario_parity")
    require(event_count == opaque_counts["signed_events"], "parity:event_parity")
    identity = report.get("result_identity_sha256")
    require(identity == APPROVED_RESULT_IDENTITY, "parity:result_identity")
    projection = copy.deepcopy(report)
    projection.pop("result_identity_sha256")
    require(projection_digest(projection) == identity, "parity:result_projection")
    validate_no_leak(report, "parity:boundary")


def mutation_self_test(
    report: dict[str, Any], opaque_checkpoint: dict[str, Any]
) -> int:
    mutations: list[tuple[str, dict[str, Any], dict[str, Any]]] = []
    missing = copy.deepcopy(report)
    missing.pop("comparison")
    mutations.append(("missing", missing, opaque_checkpoint))
    extra = copy.deepcopy(report)
    extra["note"] = "held"
    mutations.append(("extra", extra, opaque_checkpoint))
    requirement_order = copy.deepcopy(report)
    requirement_order["requirement_ids"].reverse()
    mutations.append(("requirement_order", requirement_order, opaque_checkpoint))
    for table_name in ("public_table", "opaque_table"):
        reordered = copy.deepcopy(report)
        reordered[table_name].reverse()
        mutations.append((f"{table_name}_order", reordered, opaque_checkpoint))
        missing_row = copy.deepcopy(report)
        missing_row[table_name].pop()
        mutations.append((f"{table_name}_missing", missing_row, opaque_checkpoint))
        extra_row = copy.deepcopy(report)
        extra_row[table_name].append(copy.deepcopy(extra_row[table_name][-1]))
        mutations.append((f"{table_name}_extra", extra_row, opaque_checkpoint))
    for field, value in (
        ("state", "unknown"),
        ("decision", "authorized"),
        ("checkpoint_status", "pending_control"),
        ("disposition", "pending"),
        ("diagnostic", None),
        ("downstream_work", "permitted"),
    ):
        changed = copy.deepcopy(report)
        changed["public_table"][-1][field] = value
        mutations.append((f"row_{field}", changed, opaque_checkpoint))
    for field in (
        "signed_scenario_count",
        "signed_event_count",
        "engine_vector_count",
        "delivery_order_count",
        "fixed_regression_count",
        "open_regression_count",
    ):
        changed = copy.deepcopy(report)
        changed["conformance"][field] += 1
        mutations.append((f"conformance_{field}", changed, opaque_checkpoint))
    for field in (
        "imported_checkpoint_identity_sha256",
        "result_identity_sha256",
    ):
        changed = copy.deepcopy(report)
        changed[field] = "f" * 64
        mutations.append((field, changed, opaque_checkpoint))
    table_identity = copy.deepcopy(report)
    table_identity["comparison"]["table_identity_sha256"] = "f" * 64
    mutations.append(("table_identity", table_identity, opaque_checkpoint))
    input_identity = copy.deepcopy(report)
    input_identity["conformance"]["public_input_identity_sha256"] = "f" * 64
    mutations.append(("input_identity", input_identity, opaque_checkpoint))
    imported = copy.deepcopy(opaque_checkpoint)
    imported["result_counts"]["engine_vectors"] += 1
    mutations.append(("imported_count", report, imported))
    imported_identity = copy.deepcopy(opaque_checkpoint)
    imported_identity["result_identity_sha256"] = "f" * 64
    mutations.append(("imported_identity", report, imported_identity))
    missing_attribution = copy.deepcopy(report)
    missing_attribution["opaque_attribution"].pop("checkpoint_report_projection_sha256")
    mutations.append(("opaque_attribution_missing", missing_attribution, opaque_checkpoint))
    extra_attribution = copy.deepcopy(report)
    extra_attribution["opaque_attribution"]["note"] = "held"
    mutations.append(("opaque_attribution_extra", extra_attribution, opaque_checkpoint))
    for field in (
        "candidate",
        "checkpoint_result_identity_sha256",
        "checkpoint_report_projection_sha256",
        "corrected_expectation_projection_sha256",
        "engine_vector_count",
        "table_identity_sha256",
        "binding_identity_sha256",
        "result",
    ):
        changed = copy.deepcopy(report)
        value = changed["opaque_attribution"][field]
        if isinstance(value, int):
            changed["opaque_attribution"][field] = value + 1
        elif field == "result":
            changed["opaque_attribution"][field] = "exact"
        else:
            changed["opaque_attribution"][field] = "f" * len(value)
        mutations.append((f"opaque_attribution_{field}", changed, opaque_checkpoint))
    coordinated_table = copy.deepcopy(report)
    coordinated_table["opaque_table"][0]["decision"] = "refused"
    coordinated_projection = copy.deepcopy(coordinated_table["opaque_attribution"])
    coordinated_projection["table_identity_sha256"] = projection_digest(
        coordinated_table["opaque_table"]
    )
    coordinated_projection.pop("binding_identity_sha256")
    coordinated_projection["opaque_table"] = coordinated_table["opaque_table"]
    coordinated_table["opaque_attribution"]["binding_identity_sha256"] = projection_digest(
        coordinated_projection
    )
    mutations.append(("coordinated_opaque_table", coordinated_table, opaque_checkpoint))
    coordinated_import = copy.deepcopy(opaque_checkpoint)
    coordinated_report = copy.deepcopy(report)
    coordinated_import["result_identities"][3]["sha256"] = "e" * 64
    coordinated_report["opaque_attribution"]["checkpoint_report_projection_sha256"] = (
        "e" * 64
    )
    coordinated_projection = copy.deepcopy(coordinated_report["opaque_attribution"])
    coordinated_projection.pop("binding_identity_sha256")
    coordinated_projection["opaque_table"] = coordinated_report["opaque_table"]
    coordinated_report["opaque_attribution"]["binding_identity_sha256"] = projection_digest(
        coordinated_projection
    )
    mutations.append(("coordinated_opaque_identity", coordinated_report, coordinated_import))

    caught = 0
    for name, candidate, imported_candidate in mutations:
        try:
            validate_checkpoint_parity(candidate, imported_candidate)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"parity_mutation_survived:{name}")

    schema = load_object(SCHEMA)
    schema_mutations: list[tuple[str, dict[str, Any]]] = []
    open_root = copy.deepcopy(schema)
    open_root["additionalProperties"] = True
    schema_mutations.append(("schema_open_root", open_root))
    open_row = copy.deepcopy(schema)
    open_row["$defs"]["state_table"]["items"]["additionalProperties"] = True
    schema_mutations.append(("schema_open_row", open_row))
    weak_row = copy.deepcopy(schema)
    weak_row["$defs"]["state_table"]["items"]["required"].pop()
    schema_mutations.append(("schema_weak_row", weak_row))
    open_attribution = copy.deepcopy(schema)
    open_attribution["properties"]["opaque_attribution"]["additionalProperties"] = True
    schema_mutations.append(("schema_open_attribution", open_attribution))
    for name, candidate in schema_mutations:
        try:
            validate_schema_contract(candidate, "parity_schema", SCHEMA_PROJECTION)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"parity_mutation_survived:{name}")
    companion_inventory = {
        "checkpoint.example.fixture.json",
        "checkpoint.example.input.json",
        "checkpoint.example.expected.json",
    }
    for name, candidate in (
        ("inventory_extra", companion_inventory | {"checkpoint.extra.json"}),
        ("inventory_missing", companion_inventory - {"checkpoint.example.expected.json"}),
    ):
        try:
            validate_companion_inventory(candidate, companion_inventory)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"parity_mutation_survived:{name}")
    return caught


def main() -> int:
    report = load_object(REPORT)
    opaque_checkpoint = load_object(OPAQUE_CHECKPOINT)
    validate_schema_contract(load_object(SCHEMA), "parity_schema", SCHEMA_PROJECTION)
    validate_checkpoint_parity(report, opaque_checkpoint)
    mutations = mutation_self_test(report, opaque_checkpoint)
    print("PASS: checkpoint parity v9")
    print(f"- abstract_states={len(report['public_table'])}")
    print(f"- signed_scenarios={report['conformance']['signed_scenario_count']}")
    print(f"- signed_events={report['conformance']['signed_event_count']}")
    print(f"- negative_mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
