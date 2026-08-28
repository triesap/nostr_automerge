#!/usr/bin/env python3
"""Validate the closed RCLD-112 epoch work-ownership gate."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess


ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/remediation_v12_ancestry_authorization_gate.json"
SCHEMA = ROOT / "tools/validation/remediation_v12_ancestry_authorization_gate.schema.json"
SCHEMA_SHA256 = "d9cc7d8102f0e261ed23d9ca312ef3309eda1a38594841fc332b044fe271a3d6"
FINAL = "0dc4160ea4f419cbab8ac2523717c7ce4d3644b5"
CANDIDATES = (
    ("step_1388", "43884403ee71c5a0b6fbf7a9b91b4617dd53b43c"),
    ("step_1389", "89009e315cfe8596f3a639a0af9e359a7c0a40d7"),
    ("step_1390", "4f4d43c3aca9d4d959edb2464039d50a983e70a0"),
    ("step_1391", "d3b1d462ee4691741821067fb51d33d6d8eb24d6"),
    ("step_1392", "b7a72c9c0be884fa821cd4224fe523fa02e03426"),
    ("step_1393", "6de8d68c83996009962b315306ada3c339f12844"),
    ("step_1394", "6659ca2e5186af9447592e296eb375e17b62ae67"),
    ("step_1395", "c59f25b09576aa595e0ce97aadb0d159e33a1a8c"),
    ("step_1396", FINAL),
)
REQUIREMENTS = (
    "NCRDT-RESOURCE-017", "NCRDT-RESOURCE-018",
    "NCRDT-RESOURCE-019", "NCRDT-EVIDENCE-007",
)
OPERATION_KEYS = (
    "id", "family", "source_path", "source_symbol", "owner_mode",
    "test", "command", "candidate", "artifact_sha256",
)
OPERATION_IDS = (
    "actor_predecessor_expected_sequence",
    "causal_next_operation_projection",
    "empty_frontier_validation",
    "combined_candidate_semantics",
    "epoch_ancestry_classification",
    "epoch_writer_authorization",
    "dependency_closure",
    "candidate_schedule",
    "quarantine_traversal",
    "quarantine_overlay_publication",
    "candidate_storage_projection",
    "epoch_result_publication",
    "zero_post_stop",
)
TOP_KEYS = (
    "schema", "status", "rcld", "candidate_chain", "requirements",
    "operations", "work_contract", "reproductions", "findings", "holds", "result",
)
HOLDS = (
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
)


class GateError(RuntimeError):
    pass


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def git(*args: str, binary: bool = False) -> str | bytes:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)
    if result.returncode:
        raise GateError("git:" + ":".join(args))
    return result.stdout if binary else result.stdout.decode().strip()


def require_keys(value: object, keys: tuple[str, ...], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or tuple(value) != keys:
        raise GateError(label + ":keys")
    return value


def validate_record(value: object) -> None:
    record = require_keys(value, TOP_KEYS, "gate")
    if record["schema"] != "nostr_automerge.remediation_v12_ancestry_authorization_gate.v1":
        raise GateError("gate:schema")
    if record["status"] != "rcld_112_complete" or record["rcld"] != 112:
        raise GateError("gate:status")
    chain = record["candidate_chain"]
    if not isinstance(chain, list) or tuple(
        (row.get("step"), row.get("candidate")) if isinstance(row, dict) else None
        for row in chain
    ) != CANDIDATES:
        raise GateError("gate:candidates")
    for row in chain:
        require_keys(row, ("step", "candidate"), "gate:candidate")
    if tuple(record["requirements"]) != REQUIREMENTS:
        raise GateError("gate:requirements")
    operations = record["operations"]
    if not isinstance(operations, list) or len(operations) != len(OPERATION_IDS):
        raise GateError("gate:operations")
    if tuple(row.get("id") for row in operations if isinstance(row, dict)) != OPERATION_IDS:
        raise GateError("gate:operation_ids")
    seen_tests: set[str] = set()
    for row in operations:
        operation = require_keys(row, OPERATION_KEYS, "gate:operation")
        if operation["owner_mode"] not in {"item_metered", "exact_reserved", "sealed_constant_time"}:
            raise GateError("gate:owner")
        if operation["candidate"] != FINAL:
            raise GateError("gate:operation_candidate")
        test = operation["test"]
        if not isinstance(test, str) or test in seen_tests:
            raise GateError("gate:test")
        seen_tests.add(test)
        expected_command = chr(99) + f"argo test -p nostr_automerge --lib {test} --locked"
        if operation["command"] != expected_command:
            raise GateError("gate:command")
    work = require_keys(
        record["work_contract"],
        ("operation_rows", "item_metered", "sealed_constant_time", "exact_reserved",
         "unowned_operations", "budget_matrix", "cancellation_matrix",
         "first_stop_preserved", "zero_post_stop_work", "production_bypasses"),
        "gate:work",
    )
    if work != {
        "operation_rows": 13, "item_metered": 12, "sealed_constant_time": 1,
        "exact_reserved": 0, "unowned_operations": 0, "budget_matrix": "pass",
        "cancellation_matrix": "pass", "first_stop_preserved": True,
        "zero_post_stop_work": True, "production_bypasses": 0,
    }:
        raise GateError("gate:work")
    reproductions = require_keys(
        record["reproductions"],
        ("fixed_families", "remaining_finding_100_families", "finding_100_status"),
        "gate:reproductions",
    )
    if reproductions != {
        "fixed_families": 10,
        "remaining_finding_100_families": 0,
        "finding_100_status": "closed",
    }:
        raise GateError("gate:reproductions")
    findings = require_keys(record["findings"], ("open", "closed", "held"), "gate:findings")
    if findings != {
        "open": ["FINDING_101", "FINDING_102", "FINDING_103"],
        "closed": ["FINDING_100"], "held": ["FINDING_080"],
    }:
        raise GateError("gate:findings")
    if tuple(record["holds"]) != HOLDS or record["result"] != "pass":
        raise GateError("gate:result")


def test_is_enabled(source: str, test: str) -> bool:
    declaration = f"fn {test}()"
    if source.count(declaration) != 1:
        return False
    prefix = source[:source.index(declaration)]
    attribute = prefix.rsplit("#[test]", 1)
    return len(attribute) == 2 and "#[ignore" not in attribute[-1]


def validate_operation_sources(operations: list[object], overrides: dict[str, str] | None = None) -> None:
    overrides = overrides or {}
    for raw in operations:
        operation = require_keys(raw, OPERATION_KEYS, "source:operation")
        path = str(operation["source_path"])
        source = overrides.get(path, (ROOT / path).read_text())
        symbol = str(operation["source_symbol"])
        if source.count(symbol) < 1:
            raise GateError("source:symbol:" + symbol)
        if not test_is_enabled(source, str(operation["test"])):
            raise GateError("source:test:" + str(operation["test"]))
        committed = git("show", f"{FINAL}:{path}", binary=True)
        if sha256_bytes(committed) != operation["artifact_sha256"]:
            raise GateError("source:artifact:" + path)
        if sha256_bytes(source.encode()) != operation["artifact_sha256"]:
            raise GateError("source:worktree:" + path)


def validate_sources(record: dict[str, object]) -> None:
    prior = str(git("rev-parse", f"{CANDIDATES[0][1]}^"))
    for step, candidate in CANDIDATES:
        parents = str(git("rev-list", "--parents", "-n", "1", candidate)).split()
        if parents != [candidate, prior]:
            raise GateError("candidate:parent:" + step)
        prior = candidate
    operations = record["operations"]
    if not isinstance(operations, list):
        raise GateError("source:operations")
    validate_operation_sources(operations)
    requirements = json.loads((ROOT / "spec/requirements.json").read_text())
    requirement_ids = {row.get("id") for row in requirements.get("requirements", [])}
    if not set(REQUIREMENTS).issubset(requirement_ids):
        raise GateError("requirements:inventory")
    reproductions = json.loads(str(git("show", f"{FINAL}:spec/remediation_v12_reproductions.json")))
    cases = reproductions.get("cases")
    if not isinstance(cases, list) or len(cases) != 10:
        raise GateError("reproductions:inventory")
    if any(row.get("finding") != "FINDING_100" or row.get("expected") != "fixed_pass" for row in cases):
        raise GateError("reproductions:status")
    if sha256_bytes(SCHEMA.read_bytes()) != SCHEMA_SHA256:
        raise GateError("schema:sha256")


def mutation_self_test(record: dict[str, object]) -> tuple[int, int]:
    mutators = (
        lambda value: value.update(status="implementation_in_progress"),
        lambda value: value.update(rcld=111),
        lambda value: value["candidate_chain"].pop(),
        lambda value: value["candidate_chain"].reverse(),
        lambda value: value["requirements"].pop(),
        lambda value: value["operations"].pop(),
        lambda value: value["operations"].reverse(),
        lambda value: value["operations"].append(copy.deepcopy(value["operations"][0])),
        lambda value: value["operations"][0].update(owner_mode="unmetered"),
        lambda value: value["operations"][0].update(candidate="0" * 40),
        lambda value: value["operations"][0].update(artifact_sha256="0" * 64),
        lambda value: value["operations"][0].update(command="test command"),
        lambda value: value["operations"][0].update(test="wrong_test"),
        lambda value: value["work_contract"].update(unowned_operations=1),
        lambda value: value["work_contract"].update(first_stop_preserved=False),
        lambda value: value["work_contract"].update(production_bypasses=1),
        lambda value: value["reproductions"].update(remaining_finding_100_families=1),
        lambda value: value["reproductions"].update(finding_100_status="open"),
        lambda value: value["findings"]["open"].insert(0, "FINDING_100"),
        lambda value: value["findings"]["closed"].clear(),
        lambda value: value["holds"].pop(),
        lambda value: value.update(result="fail"),
        lambda value: value.update(unapproved=False),
    )
    caught = 0
    for mutation_index, mutate in enumerate(mutators):
        changed = copy.deepcopy(record)
        mutate(changed)
        try:
            validate_record(changed)
            validate_sources(changed)
        except GateError:
            caught += 1
            continue
        raise GateError(f"mutation:record:{mutation_index}")
    reordered = copy.deepcopy(record)
    reordered["schema"] = reordered.pop("schema")
    try:
        validate_record(reordered)
    except GateError:
        caught += 1
    else:
        raise GateError("mutation:record_order")

    operations = record["operations"]
    if not isinstance(operations, list):
        raise GateError("mutation:operations")
    source_caught = 0
    for raw in operations:
        operation = require_keys(raw, OPERATION_KEYS, "mutation:operation")
        path = str(operation["source_path"])
        source = (ROOT / path).read_text()
        symbol = str(operation["source_symbol"])
        changed = source.replace(symbol, "removed_operation_symbol", 1)
        try:
            validate_operation_sources(operations, {path: changed})
        except GateError:
            source_caught += 1
            continue
        raise GateError("mutation:source:" + str(operation["id"]))
    return caught, source_caught


def main() -> None:
    record = json.loads(REPORT.read_text())
    validate_record(record)
    validate_sources(record)
    record_mutations, source_mutations = mutation_self_test(record)
    print("PASS: remediation v12 ancestry and authorization gate")
    print(f"- candidates={len(CANDIDATES)}")
    print(f"- operations={len(OPERATION_IDS)} unowned=0")
    print(f"- record_mutations={record_mutations}")
    print(f"- source_mutations={source_mutations}")
    print("- finding_100=closed fixed_families=10 remaining=0")


if __name__ == "__main__":
    main()
