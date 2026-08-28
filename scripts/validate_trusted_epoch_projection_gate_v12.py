#!/usr/bin/env python3
"""Validate the closed RCLD-110 trusted epoch projection gate."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess


ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/trusted_epoch_projection_gate_v12.json"
SCHEMA = ROOT / "tools/validation/trusted_epoch_projection_gate_v12.schema.json"
SCHEMA_SHA256 = "8451ac4a647b8f2f12eca0bdbddf37becc1757deac765bf3558dc0f6cbce4577"
SOURCE = "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_SHA256 = "6e327e8f1300a0f2e62e211375365e7c57d20924a93af22ae9399b6f7341ace4"
CANDIDATES = (
    ("step_1372", "fd9ed9103879d4933832766c0c8dadb57262a49f"),
    ("step_1373", "25b540e176d291c9de823e8106e074a5d4eff48b"),
    ("step_1374", "eafa932ff4cfa7a4356827f2b97037a8d35c89f3"),
    ("step_1375", "6e7d13e735017ce310670ca70d56bd4e5225ac61"),
    ("step_1376", "bb2f600aeb08c74dd7c8556c1bfc14baa4568ce6"),
    ("step_1377", "41be17ed694f1e9848c47acd99a79f4513dfc2e4"),
    ("step_1378", "5b3a386160e3310071e644a7030ade80248640d5"),
)
REQUIREMENTS = (
    "NCRDT-RESOURCE-017",
    "NCRDT-RESOURCE-018",
    "NCRDT-RESOURCE-019",
    "NCRDT-EVIDENCE-007",
)
VALIDATORS = (
    (
        "scripts/validate_remediation_v12.py",
        "0b204212805005720c1a8f79f247a59f7aa09981272f5ec58dbb9ce5e606bcd7",
    ),
    (
        "scripts/validate_resource_operation_inventory_v10.py",
        "4359d156e5d5b750d462967d82d1d579438e16e0ad282d1288a799b9fa4909cc",
    ),
)
OPEN_FINDINGS = ("FINDING_100", "FINDING_101", "FINDING_102", "FINDING_103")
HOLDS = (
    "external_assurance",
    "event_kind_allocation",
    "nip_submission",
    "production_qualification",
    "publication",
    "release",
    "remote_mutation",
)
TOP_KEYS = (
    "schema",
    "status",
    "rcld",
    "candidate_chain",
    "requirements",
    "projection",
    "work_contract",
    "validators",
    "findings",
    "holds",
    "result",
)
PROOF_TESTS = (
    "trusted_epoch_projection_shape_and_construction_are_sealed",
    "charged_projection_traversal_stops_before_every_source_read",
    "projection_lookups_and_semantic_comparisons_are_immediately_charged",
    "projection_allocation_insertion_and_publication_are_charged_before_work",
    "projection_semantic_matrix_is_complete_and_order_invariant",
    "projection_work_contract_preserves_first_stop_and_predecessor_output",
)


class GateError(RuntimeError):
    pass


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        raise GateError("git:" + ":".join(args))
    return result.stdout.strip()


def git_file_sha(candidate: str, path: str) -> str:
    result = subprocess.run(
        ["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False
    )
    if result.returncode != 0:
        raise GateError("git:file:" + path)
    return hashlib.sha256(result.stdout).hexdigest()


def require_keys(value: object, keys: tuple[str, ...], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or tuple(value) != keys:
        raise GateError(label + ":keys")
    return value


def validate_record(value: object) -> None:
    record = require_keys(value, TOP_KEYS, "gate")
    if record["schema"] != "nostr_automerge.trusted_epoch_projection_gate.v12.v1":
        raise GateError("gate:schema")
    if record["status"] != "rcld_110_complete" or record["rcld"] != 110:
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
    projection = require_keys(
        record["projection"],
        (
            "source",
            "source_sha256",
            "sealed_constructors",
            "semantic_cases",
            "lookup_operations",
            "publication_operations",
        ),
        "gate:projection",
    )
    if projection != {
        "source": SOURCE,
        "source_sha256": SOURCE_SHA256,
        "sealed_constructors": 1,
        "semantic_cases": 8,
        "lookup_operations": 9,
        "publication_operations": 19,
    }:
        raise GateError("gate:projection")
    work = require_keys(
        record["work_contract"],
        (
            "total_charges",
            "graph_node_charges",
            "graph_edge_charges",
            "budget_matrix",
            "cancellation_matrix",
            "first_stop_preserved",
            "zero_post_stop_target_work",
            "predecessor_output_bytes",
            "unexpected_identity",
        ),
        "gate:work",
    )
    if work != {
        "total_charges": 41,
        "graph_node_charges": 32,
        "graph_edge_charges": 9,
        "budget_matrix": "pass",
        "cancellation_matrix": "pass",
        "first_stop_preserved": True,
        "zero_post_stop_target_work": True,
        "predecessor_output_bytes": "equal",
        "unexpected_identity": "preserved",
    }:
        raise GateError("gate:work")
    validators = record["validators"]
    if not isinstance(validators, list) or tuple(
        (row.get("path"), row.get("sha256")) if isinstance(row, dict) else None
        for row in validators
    ) != VALIDATORS:
        raise GateError("gate:validators")
    for row in validators:
        require_keys(row, ("path", "sha256"), "gate:validator")
    findings = require_keys(record["findings"], ("open", "held"), "gate:findings")
    if tuple(findings["open"]) != OPEN_FINDINGS or findings["held"] != ["FINDING_080"]:
        raise GateError("gate:findings")
    if tuple(record["holds"]) != HOLDS or record["result"] != "pass":
        raise GateError("gate:result")


def validate_sources() -> None:
    if sha256(ROOT / SOURCE) != SOURCE_SHA256:
        raise GateError("source:sha256")
    if git_file_sha(CANDIDATES[-1][1], SOURCE) != SOURCE_SHA256:
        raise GateError("source:candidate_sha256")
    for path, expected in VALIDATORS:
        if git_file_sha(CANDIDATES[-1][1], path) != expected:
            raise GateError("validator:sha256:" + path)
    prior = "4a5abe6f0bff2dbe147d9805f4cd3de844874ab6"
    for step, candidate in CANDIDATES:
        parents = git("rev-list", "--parents", "-n", "1", candidate).split()
        if parents != [candidate, prior]:
            raise GateError("candidate:parent:" + step)
        prior = candidate
    source = (ROOT / SOURCE).read_text()
    for test in PROOF_TESTS:
        declaration = f"fn {test}()"
        if source.count(declaration) != 1:
            raise GateError("proof:test:" + test)
        before = source[: source.index(declaration)]
        attributes = before[max(0, len(before) - 240) :]
        if "#[test]" not in attributes or "#[ignore" in attributes:
            raise GateError("proof:attributes:" + test)
    requirements = json.loads((ROOT / "spec/requirements.json").read_text())
    ids = tuple(row.get("id") for row in requirements.get("requirements", []))
    if ids[-4:] != REQUIREMENTS or len(ids) != 156:
        raise GateError("requirements:inventory")
    if sha256(SCHEMA) != SCHEMA_SHA256:
        raise GateError("schema:sha256")


def mutation_self_test(record: object) -> int:
    mutations: list[object] = []
    mutators = (
        lambda value: value.update(status="implementation_in_progress"),
        lambda value: value.update(rcld=109),
        lambda value: value["candidate_chain"].pop(),
        lambda value: value["candidate_chain"].reverse(),
        lambda value: value["candidate_chain"][0].update(candidate="0" * 40),
        lambda value: value["requirements"].pop(),
        lambda value: value["requirements"].reverse(),
        lambda value: value["projection"].update(source_sha256="0" * 64),
        lambda value: value["projection"].update(semantic_cases=7),
        lambda value: value["work_contract"].update(total_charges=40),
        lambda value: value["work_contract"].update(graph_node_charges=31),
        lambda value: value["work_contract"].update(first_stop_preserved=False),
        lambda value: value["validators"].reverse(),
        lambda value: value["validators"][0].update(sha256="0" * 64),
        lambda value: value["findings"]["open"].pop(),
        lambda value: value["findings"]["held"].clear(),
        lambda value: value["holds"].pop(),
        lambda value: value.update(result="fail"),
        lambda value: value.update(unapproved=False),
    )
    for mutate in mutators:
        candidate = copy.deepcopy(record)
        mutate(candidate)
        mutations.append(candidate)
    reordered = copy.deepcopy(record)
    reordered["schema"] = reordered.pop("schema")
    mutations.append(reordered)
    for index, candidate in enumerate(mutations):
        try:
            validate_record(candidate)
        except GateError:
            continue
        raise GateError(f"mutation:{index}")
    return len(mutations)


def main() -> None:
    record = json.loads(REPORT.read_text())
    validate_record(record)
    validate_sources()
    mutations = mutation_self_test(record)
    print("PASS: trusted epoch projection gate v12")
    print(f"- candidates={len(CANDIDATES)}")
    print(f"- proofs={len(PROOF_TESTS)}")
    print(f"- mutations={mutations}")
    print("- work_total=41 nodes=32 edges=9")


if __name__ == "__main__":
    main()
