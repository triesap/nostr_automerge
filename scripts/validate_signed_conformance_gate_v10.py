#!/usr/bin/env python3
"""Validate the closed signed-conformance v10 gate."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import subprocess
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/signed_conformance_gate_v10.json"
SCHEMA_PATH = "tools/validation/signed_conformance_gate_v10.schema.json"
REPORT_SHA256 = "16577552f984f88f0e07cfe001ebae6591b02f8c3dff6d7949380c353bc4ec85"
SCHEMA_SHA256 = "3ce97b446042f51d86a1a9244962fbe5c29c81f03cf8877ea1683d2a59b14422"
RESULT_IDENTITY = "3aa7447b502c86de51d6e82a0ceb067df816a7ce956895b2aef54c3f76303b6e"
FIELDS = (
    "schema", "checkpoint", "gate_id", "status", "publication_status",
    "requirement_ids", "candidate_chain", "authority", "signatures",
    "execution", "mismatch_proofs", "validation", "result_identity_sha256",
)
CANDIDATES = (
    ("step_1264", "1bbb9b90fe0302c972dc0b9350d762667ac840df", "public"),
    ("step_1265", "957d0bbef4045afee2b125feda842b18f8c879ef", "public"),
    ("step_1266", "95f25100f5dc9234e97d67508439485d39d3d85c", "public"),
    ("step_1267", "a87c9c7ca4b5fb59b6ef68217a6b410375f7305d", "public"),
    ("step_1268", "43f71ad17e490fd42979723e45a58164d726884b", "public"),
    ("step_1269", "4dc5329d0d1fdcd4a7e3e2aee8e8f749c4ed72aa", "public"),
    ("step_1270", "20b786c5c3ff143786aaaca56ad19bd26739b67b", "public"),
    ("step_1271", "6e7084ae32b9d20e55e76b5496c126bd52974f0d", "public"),
    ("step_1272", "36db673b8e5b62df69a5ee321b2e13c040fc8237", "opaque_private"),
    ("step_1273", "fc256396c534adb90be4da4c9d172a14d3786f1d", "public"),
)
NESTED_FIELDS = {
    "authority": (
        "transition_stage", "distribution_schema_sha256", "generator_sha256",
        "corrected_expectation_validator_sha256", "manifest_sha256",
        "fixture_count", "preserved_fixture_count", "new_fixture_count",
        "corrected_expectation_count", "v9_status", "result",
    ),
    "signatures": (
        "signed_fixture_count", "invalid_signature_count",
        "missing_fixture_authority", "result",
    ),
    "execution": (
        "rust_process_count", "private_process_count", "delivery_permutations",
        "rust_report_count", "private_report_count", "canonical_output_sha256",
        "rust_distribution_sha256", "private_execution_sha256",
        "byte_mismatch_count", "result",
    ),
    "mismatch_proofs": (
        "malformed_report", "noncanonical_report",
        "structurally_valid_semantic_mismatch", "result",
    ),
    "validation": (
        "generator", "corrected_expectations", "rust_two_process",
        "opaque_private_two_process", "comparison", "leak_boundary", "full_public",
    ),
}
FILE_BINDINGS = {
    "fixtures/schema/distribution.schema.v10.json": "4c61490648e97f53ed074561794a7dfa5c39d60817846cfa6d6489cd8570b818",
    "scripts/generate_distribution_v10.py": "f63e8ed153e13cd2ea392e5c7d0d4901820e4a0a31afbd0a0a87b05ed8ca8ef4",
    "scripts/validate_corrected_checkpoint_expectations_v10.py": "4fde7eecf9e4eefe603a18210406f828ab59749a5feb2ffb1cb2f486faa08144",
    "fixtures/distribution/manifest_v10.json": "86ec32f34dd99ef0c1e5ea3531360a1f78bf07d62818375096e0bdf0f209b8e5",
}


class GateError(ValueError):
    """One signed-conformance gate invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise GateError(diagnostic)


def digest_file(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"object:{relative}")
    return value


def identity(value: dict[str, Any]) -> str:
    projection = {key: value[key] for key in FIELDS[:-1]}
    return hashlib.sha256(
        json.dumps(projection, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def validate_schema(schema: dict[str, Any]) -> None:
    require(
        tuple(schema) == ("title", "type", "required", "properties", "additionalProperties"),
        "schema:keys",
    )
    require(schema.get("type") == "object", "schema:type")
    require(schema.get("required") == list(FIELDS), "schema:required")
    require(tuple(schema.get("properties", {})) == FIELDS, "schema:properties")
    require(schema.get("additionalProperties") is False, "schema:closed")
    for field, keys in NESTED_FIELDS.items():
        nested = schema["properties"][field]
        require(nested.get("required") == list(keys), f"schema:{field}:required")
        require(tuple(nested.get("properties", {})) == keys, f"schema:{field}:properties")
        require(nested.get("additionalProperties") is False, f"schema:{field}:closed")


def validate(value: dict[str, Any], schema: dict[str, Any], *, bind_files: bool = True) -> None:
    require(tuple(value) == FIELDS, "report:keys")
    require(value.get("schema") == "nostr_automerge.signed_conformance_gate.v10.v1", "report:schema")
    require(value.get("checkpoint") == "step_1274", "report:checkpoint")
    require(value.get("gate_id") == "GATE_V9_CONFORMANCE", "report:gate")
    require(value.get("status") == "pass", "report:status")
    require(value.get("publication_status") == "held", "report:publication")
    require(value.get("requirement_ids") == ["NCRDT-CONF-010", "NCRDT-EVIDENCE-006"], "report:requirements")
    expected_chain = [
        {"checkpoint": step, "candidate": candidate, "owner_class": owner, "result": "pass"}
        for step, candidate, owner in CANDIDATES
    ]
    require(value.get("candidate_chain") == expected_chain, "report:candidates")
    for field, keys in NESTED_FIELDS.items():
        require(tuple(value.get(field, {})) == keys, f"report:{field}:keys")
    require(value["authority"] == {
        "transition_stage": "distribution_complete",
        "distribution_schema_sha256": FILE_BINDINGS["fixtures/schema/distribution.schema.v10.json"],
        "generator_sha256": FILE_BINDINGS["scripts/generate_distribution_v10.py"],
        "corrected_expectation_validator_sha256": FILE_BINDINGS["scripts/validate_corrected_checkpoint_expectations_v10.py"],
        "manifest_sha256": FILE_BINDINGS["fixtures/distribution/manifest_v10.json"],
        "fixture_count": 192, "preserved_fixture_count": 180, "new_fixture_count": 12,
        "corrected_expectation_count": 4, "v9_status": "historical_superseded_non_current",
        "result": "pass",
    }, "report:authority")
    require(value["signatures"] == {
        "signed_fixture_count": 192, "invalid_signature_count": 0,
        "missing_fixture_authority": "rejected", "result": "pass",
    }, "report:signatures")
    require(value["execution"] == {
        "rust_process_count": 2, "private_process_count": 2,
        "delivery_permutations": 8, "rust_report_count": 192,
        "private_report_count": 192,
        "canonical_output_sha256": "c9f28deb32dfedce674a6871b0eb949f38b5a5f977a67ca993f7ed639df1e112",
        "rust_distribution_sha256": "377b0fe6ae2916b829b3ada84f7adf760d874123ce8be14130999a076c8578c6",
        "private_execution_sha256": "4a663141aa5d122fd388e8c08e115d8ceb58efe8e36408abc0339f9aeba4a958",
        "byte_mismatch_count": 0, "result": "pass",
    }, "report:execution")
    require(value["mismatch_proofs"] == {
        "malformed_report": "rejected_by_parser",
        "noncanonical_report": "rejected_by_parser",
        "structurally_valid_semantic_mismatch": "rejected_by_comparison",
        "result": "pass",
    }, "report:mismatch")
    require(set(value["validation"].values()) == {"pass"}, "report:validation")
    require(identity(value) == RESULT_IDENTITY, "report:identity")
    require(value.get("result_identity_sha256") == RESULT_IDENTITY, "report:identity_field")
    validate_schema(schema)
    if not bind_files:
        return
    require(digest_file(REPORT_PATH) == REPORT_SHA256, "report:file")
    require(digest_file(SCHEMA_PATH) == SCHEMA_SHA256, "schema:file")
    for relative, expected in FILE_BINDINGS.items():
        require(digest_file(relative) == expected, f"file:{relative}")
    manifest = load("fixtures/distribution/manifest_v10.json")
    require(manifest.get("transition_stage") == "distribution_complete", "manifest:stage")
    require(manifest.get("fixture_count") == 192, "manifest:count")
    require(manifest.get("preserved_v9_fixture_count") == 180, "manifest:preserved")
    require(len(manifest.get("v10_fixtures", [])) == 12, "manifest:new")
    require(len(manifest.get("intentional_v9_report_changes", [])) == 4, "manifest:corrected")
    rust = load("reports/rust_conformance_v10.json")
    opaque = load("reports/opaque_conformance_v10.json")
    require(rust.get("scenario_count") == 192 and rust.get("process_count") == 2, "rust:counts")
    require(rust.get("permutations_per_fixture") == 8, "rust:permutations")
    require(rust.get("canonical_output_sha256") == value["execution"]["canonical_output_sha256"], "rust:canonical")
    require(rust.get("distribution_run_sha256") == value["execution"]["rust_distribution_sha256"], "rust:distribution")
    require(opaque.get("private_candidate") == CANDIDATES[8][1], "opaque:candidate")
    require(opaque.get("comparison", {}).get("byte_mismatch_count") == 0, "opaque:mismatch")
    require(opaque.get("comparison", {}).get("canonical_output_sha256") == value["execution"]["canonical_output_sha256"], "opaque:canonical")
    for _, candidate, owner in CANDIDATES:
        if owner == "public":
            result = subprocess.run(("git", "cat-file", "-e", f"{candidate}^{{commit}}"), cwd=ROOT)
            require(result.returncode == 0, f"candidate:{candidate}")


def expect_rejected(work: Any, diagnostic: str) -> int:
    try:
        work()
    except GateError:
        return 1
    raise GateError(f"mutation_survived:{diagnostic}")


def mutation_self_test(value: dict[str, Any], schema: dict[str, Any]) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    for field, replacement in (
        ("checkpoint", "step_1273"), ("status", "held"),
        ("publication_status", "published"), ("result_identity_sha256", "0" * 64),
    ):
        changed = copy.deepcopy(value); changed[field] = replacement; mutations.append((field, changed))
    for index in (0, 8, 9):
        changed = copy.deepcopy(value); changed["candidate_chain"][index]["candidate"] = "0" * 40
        mutations.append((f"candidate:{index}", changed))
    for field, replacement in (
        ("fixture_count", 191), ("preserved_fixture_count", 179),
        ("new_fixture_count", 13), ("corrected_expectation_count", 5),
        ("manifest_sha256", "0" * 64),
    ):
        changed = copy.deepcopy(value); changed["authority"][field] = replacement; mutations.append((f"authority:{field}", changed))
    for field, replacement in (
        ("signed_fixture_count", 191), ("invalid_signature_count", 1),
        ("missing_fixture_authority", "skipped"),
    ):
        changed = copy.deepcopy(value); changed["signatures"][field] = replacement; mutations.append((f"signatures:{field}", changed))
    for field, replacement in (
        ("rust_process_count", 1), ("private_process_count", 1),
        ("delivery_permutations", 7), ("byte_mismatch_count", 1),
        ("canonical_output_sha256", "0" * 64),
    ):
        changed = copy.deepcopy(value); changed["execution"][field] = replacement; mutations.append((f"execution:{field}", changed))
    for field in ("malformed_report", "noncanonical_report", "structurally_valid_semantic_mismatch"):
        changed = copy.deepcopy(value); changed["mismatch_proofs"][field] = "accepted"; mutations.append((f"mismatch:{field}", changed))
    changed = copy.deepcopy(value); changed["validation"]["leak_boundary"] = "failed"; mutations.append(("validation:leak_boundary", changed))
    missing = copy.deepcopy(value); missing.pop("status"); mutations.append(("missing", missing))
    extra = copy.deepcopy(value); extra["extra"] = False; mutations.append(("extra", extra))
    reordered = {"status": value["status"], **value}; mutations.append(("reordered", reordered))
    coordinated = copy.deepcopy(value); coordinated["execution"]["canonical_output_sha256"] = "f" * 64
    coordinated["result_identity_sha256"] = identity(coordinated); mutations.append(("coordinated", coordinated))
    caught = sum(expect_rejected(lambda item=item: validate(item, schema, bind_files=False), name) for name, item in mutations)
    for name, mutate in (
        ("schema_open", lambda item: item.__setitem__("additionalProperties", True)),
        ("schema_required", lambda item: item["required"].pop()),
        ("schema_nested_open", lambda item: item["properties"]["execution"].__setitem__("additionalProperties", True)),
    ):
        changed = copy.deepcopy(schema); mutate(changed)
        caught += expect_rejected(lambda item=changed: validate(value, item, bind_files=False), name)
    require(caught == 31, "mutation_count")
    return caught


def run_suite() -> None:
    commands = (
        ("python3", "scripts/generate_distribution_v10.py"),
        ("python3", "scripts/validate_corrected_checkpoint_expectations_v10.py"),
        ("python3", "scripts/validate_rust_conformance_v10.py", "--run"),
        ("python3", "scripts/validate_opaque_conformance_v10.py", "--run"),
        ("python3", "scripts/validate_private_reproduction_boundary_v9.py"),
    )
    for command in commands:
        subprocess.run(
            command,
            cwd=ROOT,
            check=True,
            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"},
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run", action="store_true")
    arguments = parser.parse_args()
    value = load(REPORT_PATH)
    schema = load(SCHEMA_PATH)
    validate(value, schema)
    mutations = mutation_self_test(value, schema)
    if arguments.run:
        run_suite()
    print("PASS: signed conformance-v10 gate")
    print("- candidate_count=10")
    print("- signed_fixture_count=192")
    print("- delivery_permutations=8")
    print("- processes_per_implementation=2")
    print(f"- negative_mutations={mutations}")
    print(f"- executed={int(arguments.run)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
