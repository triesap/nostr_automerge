#!/usr/bin/env python3
"""Validate the closed cross-implementation report-parity checkpoint."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/report_parity_v9.json"
SCHEMA = ROOT / "tools/validation/report_parity_v9.schema.json"
MANIFEST = ROOT / "fixtures/distribution/manifest_v9.json"
REPORT_SCHEMA = ROOT / "fixtures/schema/report.schema.json"
RUNNER = ROOT / "tools/nostr_automerge_conformance/src/runner.rs"

APPROVED_REPORT_SHA256 = "7c11f5c32cb8c7494cbc50a4132e0ee0a05b9b233ba52b01f2702019725ed51c"
APPROVED_SCHEMA_SHA256 = "e8753412f0741604155a0d8ab31efe0f65ce85343f4881d0c24ff0fed43e91f0"
APPROVED_RUNNER_SHA256 = "19ea36eeb55711004b7a1470e6adeff15980ea5952d00ff9f28e347f082fef33"
APPROVED_RESULT_IDENTITY = "aaf76821bb0fa463c4b71c1f27d6c194dea1b5c9790b505e04d3c810b898059d"
APPROVED_PUBLIC_MANIFEST = "bbb3802490ac758614fecad9ef7c37586da5af54c150a472f4ea7611e8eaa659"
RESOURCE_BUDGET_BASE = "fec9ef4c38c4044902285d9bcfadf2f078dc3a6e"
APPROVED_RESOURCE_MANIFEST = "4c6866b91bffbeba9610c4602b99abfc7e5a16c9d262d6e4d624a4e3a9537f9a"
RESOURCE_BUDGET_TRANSITIONS = (
    (
        "fixtures/v1_draft/scenarios/resource/parent_propagation_exact_budget.input.json",
        2_288,
        6_912,
    ),
    (
        "fixtures/v1_draft/scenarios/resource/unrelated_control_flood_exact_budget.input.json",
        68,
        110,
    ),
    (
        "fixtures/v1_draft/scenarios/scope/foreign_claim_flood_exact_budget.input.json",
        68,
        110,
    ),
    (
        "fixtures/v1_draft/scenarios/scope/unrelated_valid_checkpoints_exact_budget.input.json",
        140,
        264,
    ),
)
APPROVED_REPORT_SCHEMA = "08a88d5ad7049203bb766dc763601a6c5311a70e631fa35ab62c164203cd8e1c"
APPROVED_CANONICAL_OUTPUT = "cfb32cbf0f2248470ae07d7e42f78301df9014afc2822d622e2c260c8c60b5c6"
APPROVED_SERIALIZED_OUTPUT = "edd05b0ee5f09f8b4fda87b3bf15a1988141a371cd4b13f504a49b27ad345ed4"
APPROVED_CORRECTED_IDS_SHA256 = "3531071426c8c1a55be31dbf116bb1a6d62a431f58462cd52e528676a39fc566"
HISTORICAL_EVENT_ID = "9a0701b37736afc4c28c82bfdc94ddf53a3b054fecfa191f18ed94c14982ac7f"
ACCEPTED_CHANGE_HASH = "66be06a76d30b453372abdd246e6ea8aecf8e2dd9c134264b3cce7d57bbda43f"

CORRECTED_CHECKPOINT_IDS = (
    "checkpoints_chunk_author_mismatch",
    "checkpoints_merkle_mismatch",
    "checkpoints_missing_chunk",
    "checkpoints_multichunk",
    "checkpoints_partial_multichunk_dynamic",
    "checkpoints_single_chunk",
    "checkpoints_snapshot_mismatch",
    "checkpoints_unauthorized",
)
TOP_LEVEL_KEYS = (
    "schema",
    "checkpoint",
    "gate_id",
    "stage",
    "protocol_revision",
    "public_predecessor",
    "opaque_candidates",
    "report_schema_authority",
    "opaque_evidence",
    "public_evidence",
    "parity_results",
    "report_negative_mutations",
    "binding_negative_mutations",
    "schema_negative_mutations",
    "publication_status",
    "result_identity_sha256",
)
PARITY_CLASSES = (
    "schema_identity",
    "mandatory_execution",
    "canonical_byte_equality",
    "malformed_output_rejection",
    "structurally_valid_mismatch_rejection",
    "opaque_boundary",
)
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
CANDIDATE_RE = re.compile(r"^[0-9a-f]{40}$")


class ParityError(RuntimeError):
    pass


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ParityError(diagnostic)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def load_object(path: Path) -> tuple[bytes, dict[str, Any]]:
    raw = path.read_bytes()
    value = json.loads(raw)
    require(isinstance(value, dict), f"report_parity:not_object:{path.name}")
    return raw, value


def candidate_bytes(candidate: str, relative: str) -> bytes:
    result = subprocess.run(
        ["git", "show", f"{candidate}:{relative}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(result.returncode == 0, f"report_parity:candidate_path:{relative}")
    return result.stdout


def canonical_bytes(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def ordered_identity(values: tuple[str, ...]) -> str:
    digest = hashlib.sha256()
    for value in values:
        encoded = value.encode()
        digest.update(len(encoded).to_bytes(4, "big"))
        digest.update(encoded)
    return digest.hexdigest()


def validate_report(raw: bytes, report: dict[str, Any]) -> None:
    require(sha256_bytes(raw) == APPROVED_REPORT_SHA256, "report_parity:report_sha256")
    require(tuple(report) == TOP_LEVEL_KEYS, "report_parity:top_level_shape")
    require(report["schema"] == "nostr_automerge.report_parity.v9.v1", "report_parity:schema")
    require(report["checkpoint"] == "step_1217", "report_parity:checkpoint")
    require(report["gate_id"] == "GATE_V9_REPORT_PARITY", "report_parity:gate")
    require(report["stage"] == "report_parity_closed", "report_parity:stage")
    require(report["protocol_revision"] == "draft_2026_08", "report_parity:revision")
    require(CANDIDATE_RE.fullmatch(report["public_predecessor"]) is not None, "report_parity:predecessor")
    candidates = report["opaque_candidates"]
    require(isinstance(candidates, list) and len(candidates) == 8, "report_parity:candidate_count")
    require(len(set(candidates)) == 8, "report_parity:candidate_unique")
    require(all(CANDIDATE_RE.fullmatch(value) is not None for value in candidates), "report_parity:candidate_shape")
    require(
        [row.get("class") for row in report["parity_results"]] == list(PARITY_CLASSES)
        and all(row == {"class": name, "result": "pass"} for row, name in zip(report["parity_results"], PARITY_CLASSES, strict=True)),
        "report_parity:results",
    )
    require(report["report_negative_mutations"] == 28, "report_parity:report_mutations")
    require(report["binding_negative_mutations"] == 12, "report_parity:binding_mutations")
    require(report["schema_negative_mutations"] == 8, "report_parity:schema_mutations")
    require(report["publication_status"] == "held", "report_parity:publication")
    identity_projection = dict(report)
    identity = identity_projection.pop("result_identity_sha256", None)
    require(identity == APPROVED_RESULT_IDENTITY, "report_parity:identity_pin")
    require(sha256_bytes(canonical_bytes(identity_projection)) == identity, "report_parity:identity")
    for section in (report["report_schema_authority"], report["opaque_evidence"], report["public_evidence"]):
        for key, value in section.items():
            if key.endswith("sha256"):
                require(isinstance(value, str) and SHA256_RE.fullmatch(value) is not None, f"report_parity:sha:{key}")
    require(report["opaque_evidence"]["result"] == "pass", "report_parity:opaque_result")
    require(report["public_evidence"]["result"] == "pass", "report_parity:public_result")
    require(
        report["opaque_evidence"]["canonical_output_sha256"]
        == report["public_evidence"]["canonical_output_sha256"]
        == APPROVED_CANONICAL_OUTPUT,
        "report_parity:canonical_equality",
    )


def validate_schema(raw: bytes, schema: dict[str, Any]) -> None:
    require(sha256_bytes(raw) == APPROVED_SCHEMA_SHA256, "report_parity:schema_sha256")
    require(schema.get("type") == "object", "report_parity:schema_type")
    require(schema.get("additionalProperties") is False, "report_parity:schema_closed")
    require(tuple(schema.get("required", ())) == TOP_LEVEL_KEYS, "report_parity:schema_required")
    require(tuple(schema.get("properties", {})) == TOP_LEVEL_KEYS, "report_parity:schema_properties")
    require(schema["properties"]["schema"] == {"const": "nostr_automerge.report_parity.v9.v1"}, "report_parity:schema_const")
    require(schema["properties"]["report_negative_mutations"] == {"const": 28}, "report_parity:schema_report_mutations")
    require(schema["properties"]["binding_negative_mutations"] == {"const": 12}, "report_parity:schema_binding_mutations")
    require(schema["properties"]["schema_negative_mutations"] == {"const": 8}, "report_parity:schema_schema_mutations")


def validate_neutral_schema(report: dict[str, Any]) -> None:
    raw, schema = load_object(REPORT_SCHEMA)
    authority = report["report_schema_authority"]
    require(sha256_bytes(raw) == APPROVED_REPORT_SCHEMA == authority["sha256"], "report_parity:neutral_schema_sha256")
    require(schema.get("additionalProperties") is False, "report_parity:neutral_schema_closed")
    require(len(schema.get("required", ())) == authority["required_fields"] == 18, "report_parity:neutral_required")
    require(len(schema["$defs"]["checkpoint_result"]["properties"]["status"]["enum"]) == authority["checkpoint_statuses"] == 22, "report_parity:checkpoint_statuses")
    require(len(schema["$defs"]["diagnostic_code"]["enum"]) == authority["diagnostic_codes"] == 50, "report_parity:diagnostics")
    require(authority["schema"] == "nostr_automerge.report.v1" and authority["result"] == "pass", "report_parity:neutral_authority")


def validate_resource_budget_transition(
    manifest_raw: bytes, manifest: dict[str, Any]
) -> None:
    require(
        sha256_bytes(manifest_raw) == APPROVED_RESOURCE_MANIFEST,
        "report_parity:resource_manifest_sha256",
    )
    historical_raw = candidate_bytes(
        RESOURCE_BUDGET_BASE, "fixtures/distribution/manifest_v9.json"
    )
    require(
        sha256_bytes(historical_raw) == APPROVED_PUBLIC_MANIFEST,
        "report_parity:historical_manifest_sha256",
    )
    historical = json.loads(historical_raw)
    require(isinstance(historical, dict), "report_parity:historical_manifest_shape")
    historical_files = {
        str(row["path"]): str(row["sha256"]) for row in historical["files"]
    }
    projected = copy.deepcopy(manifest)
    projected_files = {str(row["path"]): row for row in projected["files"]}
    changed_paths: set[str] = set()

    for input_path, old_budget, new_budget in RESOURCE_BUDGET_TRANSITIONS:
        metadata_path = input_path.replace(".input.json", ".fixture.json")
        old_input_raw = candidate_bytes(RESOURCE_BUDGET_BASE, input_path)
        new_input_raw = (ROOT / input_path).read_bytes()
        old_input = json.loads(old_input_raw)
        new_input = json.loads(new_input_raw)
        require(
            old_input["budget"]["max_items"] == old_budget
            and new_input["budget"]["max_items"] == new_budget,
            f"report_parity:resource_budget:{input_path}",
        )
        normalized_input = copy.deepcopy(new_input)
        normalized_input["budget"]["max_items"] = old_budget
        require(
            normalized_input == old_input,
            f"report_parity:resource_input_delta:{input_path}",
        )

        old_metadata = json.loads(candidate_bytes(RESOURCE_BUDGET_BASE, metadata_path))
        new_metadata_raw = (ROOT / metadata_path).read_bytes()
        new_metadata = json.loads(new_metadata_raw)
        require(
            new_metadata["inputs"][0]["sha256"] == sha256_bytes(new_input_raw),
            f"report_parity:resource_input_binding:{input_path}",
        )
        normalized_metadata = copy.deepcopy(new_metadata)
        normalized_metadata["inputs"][0]["sha256"] = old_metadata["inputs"][0][
            "sha256"
        ]
        require(
            normalized_metadata == old_metadata,
            f"report_parity:resource_metadata_delta:{metadata_path}",
        )
        for path in (input_path, metadata_path):
            require(path in projected_files, f"report_parity:resource_manifest_path:{path}")
            projected_files[path]["sha256"] = historical_files[path]
            changed_paths.add(path)

    require(
        projected == historical,
        "report_parity:resource_manifest_delta",
    )
    actual_changed = {
        path
        for path, digest in {
            str(row["path"]): str(row["sha256"]) for row in manifest["files"]
        }.items()
        if historical_files.get(path) != digest
    }
    require(
        actual_changed == changed_paths,
        "report_parity:resource_manifest_inventory",
    )


def distribution_projection() -> dict[str, Any]:
    manifest_raw, manifest = load_object(MANIFEST)
    validate_resource_budget_transition(manifest_raw, manifest)
    fixtures = manifest.get("fixtures")
    require(isinstance(fixtures, list) and len(fixtures) == 180, "report_parity:fixture_count")
    fixture_ids = tuple(str(row.get("fixture_id")) for row in fixtures)
    require(fixture_ids == tuple(sorted(fixture_ids, key=str.encode)), "report_parity:fixture_order")
    require(len(set(fixture_ids)) == 180, "report_parity:fixture_unique")
    files = manifest.get("files")
    require(isinstance(files, list), "report_parity:manifest_files")
    file_paths = tuple(str(row.get("path")) for row in files)
    require(file_paths == tuple(sorted(file_paths, key=str.encode)), "report_parity:file_order")
    require(len(set(file_paths)) == len(file_paths), "report_parity:file_unique")
    for row in files:
        relative = str(row.get("path"))
        require(relative and not relative.startswith(("/", "../")), "report_parity:file_path")
        require(sha256_file(ROOT / relative) == row.get("sha256"), f"report_parity:file_sha256:{relative}")

    aggregate = hashlib.sha256()
    report_rows: list[dict[str, str]] = []
    observed_corrected: list[str] = []
    for row in fixtures:
        fixture_id = str(row["fixture_id"])
        metadata_path = ROOT / str(row["metadata_path"])
        metadata = json.loads(metadata_path.read_bytes())
        require(metadata.get("fixture_id") == fixture_id, f"report_parity:metadata_id:{fixture_id}")
        expected_path = metadata_path.parent / str(metadata["expected"]["report_path"])
        input_path = metadata_path.parent / str(metadata["inputs"][0]["path"])
        expected = expected_path.read_bytes()
        input_bytes = input_path.read_bytes()
        require(sha256_bytes(expected) == metadata["expected"]["sha256"], f"report_parity:expected_sha:{fixture_id}")
        require(sha256_bytes(input_bytes) == metadata["inputs"][0]["sha256"], f"report_parity:input_sha:{fixture_id}")
        expected_value = json.loads(expected)
        input_value = json.loads(input_bytes)
        require(input_value.get("expected_report") == expected_value, f"report_parity:input_expected:{fixture_id}")
        if fixture_id in CORRECTED_CHECKPOINT_IDS:
            checkpoints = expected_value.get("checkpoints")
            require(isinstance(checkpoints, list) and len(checkpoints) == 1, f"report_parity:checkpoint_count:{fixture_id}")
            checkpoint = checkpoints[0]
            require(checkpoint.get("historical_carriers") == [HISTORICAL_EVENT_ID], f"report_parity:historical_event:{fixture_id}")
            require(checkpoint.get("accepted_at_control") == [ACCEPTED_CHANGE_HASH], f"report_parity:accepted_hash:{fixture_id}")
            require(checkpoint["historical_carriers"] != checkpoint["accepted_at_control"], f"report_parity:namespace_distinct:{fixture_id}")
            observed_corrected.append(fixture_id)
        identifier = fixture_id.encode()
        aggregate.update(len(identifier).to_bytes(8, "big"))
        aggregate.update(identifier)
        aggregate.update(len(expected).to_bytes(8, "big"))
        aggregate.update(expected)
        report_rows.append({"fixture_id": fixture_id, "report_sha256": sha256_bytes(expected)})
    require(tuple(observed_corrected) == CORRECTED_CHECKPOINT_IDS, "report_parity:corrected_inventory")
    require(ordered_identity(CORRECTED_CHECKPOINT_IDS) == APPROVED_CORRECTED_IDS_SHA256, "report_parity:corrected_identity")
    canonical_output = aggregate.hexdigest()
    distribution = {
        "canonical_output_sha256": canonical_output,
        "delivery_permutations": 8,
        "fixture_count": len(report_rows),
        "reports": report_rows,
        "schema": "nostr_automerge.distribution_run.v1",
        "status": "pass",
    }
    serialized = json.dumps(distribution, separators=(",", ":")).encode() + b"\n"
    return {
        "fixture_manifest_sha256": APPROVED_PUBLIC_MANIFEST,
        "fixture_count": len(report_rows),
        "delivery_permutations": 8,
        "processes": 2,
        "corrected_checkpoint_fixture_count": len(observed_corrected),
        "corrected_checkpoint_ids_sha256": ordered_identity(CORRECTED_CHECKPOINT_IDS),
        "historical_carrier_namespace": "event_id",
        "accepted_history_namespace": "change_hash",
        "canonical_output_sha256": canonical_output,
        "serialized_output_sha256": sha256_bytes(serialized),
        "result": "pass",
    }


def validate_repository_bindings(report: dict[str, Any]) -> None:
    require(sha256_file(RUNNER) == APPROVED_RUNNER_SHA256, "report_parity:runner_sha256")
    validate_neutral_schema(report)
    projection = distribution_projection()
    require(projection == report["public_evidence"], "report_parity:public_projection")
    require(projection["canonical_output_sha256"] == APPROVED_CANONICAL_OUTPUT, "report_parity:public_canonical")
    require(projection["serialized_output_sha256"] == APPROVED_SERIALIZED_OUTPUT, "report_parity:public_serialized")
    malformed = b'{"report_schema":}\n'
    try:
        json.loads(malformed)
    except json.JSONDecodeError:
        pass
    else:
        raise ParityError("report_parity:malformed_accepted")
    expected_path = ROOT / "fixtures/v1_draft/scenarios/checkpoints/checkpoints_single_chunk.expected.json"
    expected = json.loads(expected_path.read_bytes())
    mismatch = copy.deepcopy(expected)
    mismatch["checkpoints"][0]["historical_carriers"] = ["aa" * 32]
    require(mismatch != expected, "report_parity:mismatch_changed")
    require(tuple(mismatch) == tuple(expected), "report_parity:mismatch_shape")
    require(sha256_bytes(canonical_bytes(mismatch)) != sha256_bytes(canonical_bytes(expected)), "report_parity:mismatch_bytes")


def mutation_self_tests(report_raw: bytes, report: dict[str, Any], schema_raw: bytes, schema: dict[str, Any]) -> tuple[int, int, int]:
    report_mutations: list[dict[str, Any]] = []
    for index in range(28):
        mutation = copy.deepcopy(report)
        if index == 0:
            mutation.pop("stage")
        elif index == 1:
            mutation["extra"] = False
        elif index == 2:
            value = mutation.pop("schema"); mutation["schema"] = value
        elif index == 3:
            mutation["schema"] = "nostr_automerge.report_parity.v9.v2"
        elif index == 4:
            mutation["checkpoint"] = "step_1216"
        elif index == 5:
            mutation["gate_id"] = "GATE_V9_REPORT"
        elif index == 6:
            mutation["stage"] = "report_parity_candidate"
        elif index == 7:
            mutation["protocol_revision"] = "draft_2026_09"
        elif index == 8:
            mutation["public_predecessor"] = "0" * 40
        elif index == 9:
            mutation["opaque_candidates"].pop()
        elif index == 10:
            mutation["opaque_candidates"].append("0" * 40)
        elif index == 11:
            mutation["opaque_candidates"].reverse()
        elif index == 12:
            mutation["opaque_evidence"]["report_sha256"] = "0" * 64
        elif index == 13:
            mutation["opaque_evidence"]["schema_sha256"] = "0" * 64
        elif index == 14:
            mutation["opaque_evidence"]["result_identity_sha256"] = "0" * 64
        elif index == 15:
            mutation["opaque_evidence"]["canonical_output_sha256"] = "0" * 64
        elif index == 16:
            mutation["opaque_evidence"]["fixture_count"] = 179
        elif index == 17:
            mutation["opaque_evidence"]["clause_count"] = 17
        elif index == 18:
            mutation["public_evidence"]["fixture_manifest_sha256"] = "0" * 64
        elif index == 19:
            mutation["public_evidence"]["canonical_output_sha256"] = "0" * 64
        elif index == 20:
            mutation["public_evidence"]["serialized_output_sha256"] = "0" * 64
        elif index == 21:
            mutation["public_evidence"]["corrected_checkpoint_fixture_count"] = 7
        elif index == 22:
            mutation["public_evidence"]["corrected_checkpoint_ids_sha256"] = "0" * 64
        elif index == 23:
            mutation["public_evidence"]["historical_carrier_namespace"] = "change_hash"
        elif index == 24:
            mutation["parity_results"][0]["class"] = "opaque_boundary"
        elif index == 25:
            mutation["parity_results"][0]["result"] = "held"
        elif index == 26:
            mutation["publication_status"] = "released"
        else:
            mutation["result_identity_sha256"] = "0" * 64
        report_mutations.append(mutation)
    for mutation in report_mutations:
        mutated_raw = json.dumps(mutation, indent=2).encode() + b"\n"
        try:
            validate_report(mutated_raw, mutation)
        except ParityError:
            continue
        raise ParityError("report_parity:report_mutation_survived")

    binding = report["public_evidence"]
    binding_mutations = []
    binding_fields = (
        "fixture_manifest_sha256", "fixture_count", "delivery_permutations", "processes",
        "corrected_checkpoint_fixture_count", "corrected_checkpoint_ids_sha256",
        "historical_carrier_namespace", "accepted_history_namespace",
        "canonical_output_sha256", "serialized_output_sha256", "result",
    )
    for field in binding_fields:
        mutation = copy.deepcopy(binding)
        value = mutation[field]
        mutation[field] = (value + 1) if isinstance(value, int) else ("0" * 64 if field.endswith("sha256") else "held")
        binding_mutations.append(mutation)
    opaque_mismatch = copy.deepcopy(binding)
    opaque_mismatch["canonical_output_sha256"] = report["opaque_evidence"]["fixture_manifest_sha256"]
    binding_mutations.append(opaque_mismatch)
    require(len(binding_mutations) == 12, "report_parity:binding_mutation_count")
    for mutation in binding_mutations:
        try:
            require(mutation == distribution_projection(), "report_parity:binding_projection")
        except ParityError:
            continue
        raise ParityError("report_parity:binding_mutation_survived")

    schema_mutations = []
    for index in range(8):
        mutation = copy.deepcopy(schema)
        if index == 0:
            mutation["additionalProperties"] = True
        elif index == 1:
            mutation["required"].pop()
        elif index == 2:
            mutation["properties"]["extra"] = {"type": "boolean"}
        elif index == 3:
            mutation["properties"]["schema"]["const"] += ".future"
        elif index == 4:
            mutation["properties"]["opaque_candidates"]["maxItems"] = 9
        elif index == 5:
            mutation["$defs"]["sha256"]["pattern"] = ".*"
        elif index == 6:
            mutation["properties"]["public_evidence"]["additionalProperties"] = True
        else:
            mutation["properties"]["parity_results"]["items"]["properties"]["result"] = {"type": "string"}
        schema_mutations.append(mutation)
    for mutation in schema_mutations:
        mutated_raw = json.dumps(mutation, indent=2).encode() + b"\n"
        try:
            validate_schema(mutated_raw, mutation)
        except ParityError:
            continue
        raise ParityError("report_parity:schema_mutation_survived")
    return len(report_mutations), len(binding_mutations), len(schema_mutations)


def run_suite() -> None:
    test_name = "runner::tests::report_parity_rejects_malformed_and_structurally_valid_mismatch"
    test = subprocess.run(
        ["cargo", "test", "-p", "nostr_automerge_conformance", "--locked", test_name, "--", "--exact"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    transcript = test.stdout + test.stderr
    require(test.returncode == 0, "report_parity:test_exit")
    require(f"test {test_name} ... ok" in transcript, "report_parity:test_name")
    require("test result: ok. 1 passed; 0 failed" in transcript, "report_parity:test_result")
    command = [
        "cargo", "run", "-p", "nostr_automerge_conformance", "--locked", "--",
        "run_distribution", "fixtures/distribution/manifest_v9.json",
    ]
    first = subprocess.run(command, cwd=ROOT, capture_output=True, check=False)
    second = subprocess.run(command, cwd=ROOT, capture_output=True, check=False)
    require(first.returncode == second.returncode == 0, "report_parity:distribution_exit")
    require(first.stdout == second.stdout, "report_parity:distribution_identity")
    require(sha256_bytes(first.stdout) == APPROVED_SERIALIZED_OUTPUT, "report_parity:distribution_sha256")
    result = json.loads(first.stdout)
    require(result.get("canonical_output_sha256") == APPROVED_CANONICAL_OUTPUT, "report_parity:distribution_canonical")


def main() -> int:
    run = False
    if sys.argv[1:] == ["--run-suite"]:
        run = True
    elif sys.argv[1:]:
        raise ParityError("usage: validate_report_parity_v9.py [--run-suite]")
    report_raw, report = load_object(REPORT)
    schema_raw, schema = load_object(SCHEMA)
    validate_report(report_raw, report)
    validate_schema(schema_raw, schema)
    validate_repository_bindings(report)
    report_mutations, binding_mutations, schema_mutations = mutation_self_tests(
        report_raw, report, schema_raw, schema
    )
    if run:
        run_suite()
    print("PASS: report parity v9")
    print(f"- opaque_candidates={len(report['opaque_candidates'])}")
    print(f"- fixtures={report['public_evidence']['fixture_count']}")
    print(f"- corrected_checkpoints={len(CORRECTED_CHECKPOINT_IDS)}")
    print(f"- report_negative_mutations={report_mutations}")
    print(f"- binding_negative_mutations={binding_mutations}")
    print(f"- schema_negative_mutations={schema_mutations}")
    print(f"- result_identity_sha256={APPROVED_RESULT_IDENTITY}")
    print(f"- executed={int(run)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
