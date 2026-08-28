#!/usr/bin/env python3
"""Validate the closed RCLD-117 causal projection implementation gate."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_implementation_gate_v13.json"
SCHEMA = ROOT / "tools/validation/causal_projection_implementation_gate_v13.schema.json"
SCHEMA_SHA256 = "444cc5a45e68efbb71394617d6d9033282336a6e4827de240a8e293f4c41e729"
SOURCE = "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_SHA256 = "c8d04545c7f330806dd86a09b23e1318afde88c594de7860357d7ca42f400970"
REVIEWED_PREDECESSOR = "fbb3fd31bd0d37ff4976f733aa574e185d5280b6"
CANDIDATES = (
    ("step_1426", "e3e8c0eca50800a53462fd90ad306f51223f2173"),
    ("step_1427", "5c65022c86f3931d2df16d71b334be17cd8483ad"),
    ("step_1428", "f4efd6b4bfff04a0d2cce19d61c7487421113f06"),
    ("step_1429", "c875ca6b234a5d97b5427d9382b628000bc1392e"),
    ("step_1430", "2bb7dd7f241db00767aa66402e14a03e2a151b58"),
    ("step_1431", "9cdd8665b68499c4975c08fd1fac07dd5eed999f"),
)
REQUIREMENTS = (
    "NCRDT-RESOURCE-017",
    "NCRDT-RESOURCE-018",
    "NCRDT-RESOURCE-019",
    "NCRDT-EVIDENCE-007",
)
OPERATION_COUNTS = (4, 7, 3, 4, 2, 11, 6, 6, 12, 4, 0, 2, 1, 1)
VALIDATORS = (
    ("scripts/validate_causal_projection_source_v13.py", "a14d78b12bef9a9d91e591088fea1bd4d73fdad4028cad42af7ae16453492506"),
    ("scripts/reproduce_remediation_v13.py", "4ec2cdbf604df8a33b2ab3cb1a16c94973ac177e47618fd2e7baa09d3fb814e1"),
    ("scripts/run_causal_projection_mutations_v13.py", "44eba6855d134353ea2f58699993abfa8ed5bf59cfc8aa53b1ebe03dd22091b3"),
)
PROOF_TESTS = (
    "projection_build_operation_boundary_is_sealed_exhaustive_and_immediate",
    "projection_source_operations_use_the_sealed_boundary",
    "projection_causal_maximum_is_charged_once_per_accepted_change",
    "complete_candidate_semantics_preserve_precedence_and_every_stop_boundary",
    "projection_work_contract_preserves_first_stop_and_predecessor_output",
    "trusted_epoch_projection_shape_and_construction_are_sealed",
    "finding_104_projection_causal_maximum_has_no_final_state_scan",
    "finding_108_projection_operations_use_one_closed_boundary",
)
OPEN = tuple(f"FINDING_{value:03d}" for value in range(104, 113))
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
    "proof_tests",
    "findings",
    "next_step",
    "holds",
    "result",
)


class GateError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise GateError(label)


def keys(value: object, expected: tuple[str, ...], label: str) -> dict[str, object]:
    require(type(value) is dict and tuple(value) == expected, label + ":shape")
    return value


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_text(*args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    )
    require(result.returncode == 0 and result.stderr == "", "git:" + ":".join(args))
    return result.stdout.strip()


def git_file_sha(candidate: str, path: str) -> str:
    result = subprocess.run(
        ["git", "show", f"{candidate}:{path}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(result.returncode == 0 and result.stderr == b"", "git_file:" + path)
    return hashlib.sha256(result.stdout).hexdigest()


def validate_record(value: object) -> None:
    record = keys(value, TOP_KEYS, "gate")
    require(
        record["schema"]
        == "nostr_automerge.causal_projection_implementation_gate.v13.v1"
        and record["status"] == "rcld_117_complete"
        and record["rcld"] == 117,
        "gate:state",
    )
    chain = record["candidate_chain"]
    require(
        type(chain) is list
        and tuple(
            (row.get("step"), row.get("candidate")) if type(row) is dict else None
            for row in chain
        )
        == CANDIDATES,
        "gate:candidates",
    )
    for row in chain:
        keys(row, ("step", "candidate"), "gate:candidate")
    require(tuple(record["requirements"]) == REQUIREMENTS, "gate:requirements")
    projection = keys(
        record["projection"],
        (
            "source",
            "source_sha256",
            "sealed_constructors",
            "operation_families",
            "operation_counts",
            "publication_operations",
            "semantic_consumer_stages",
            "raw_builder_charges",
            "unmetered_production_bypasses",
        ),
        "gate:projection",
    )
    require(
        projection
        == {
            "source": SOURCE,
            "source_sha256": SOURCE_SHA256,
            "sealed_constructors": 1,
            "operation_families": 14,
            "operation_counts": list(OPERATION_COUNTS),
            "publication_operations": 19,
            "semantic_consumer_stages": 3,
            "raw_builder_charges": 0,
            "unmetered_production_bypasses": 0,
        },
        "gate:projection",
    )
    work = keys(
        record["work_contract"],
        (
            "total_charges",
            "graph_node_charges",
            "graph_edge_charges",
            "budget_matrix",
            "cancellation_matrix",
            "first_stop_preserved",
            "zero_post_stop_target_work",
            "reference_oracle_output",
            "signed_fixture_inputs",
            "ample_reports",
            "unexpected_identity",
        ),
        "gate:work",
    )
    require(
        work
        == {
            "total_charges": 72,
            "graph_node_charges": 56,
            "graph_edge_charges": 16,
            "budget_matrix": "pass",
            "cancellation_matrix": "pass",
            "first_stop_preserved": True,
            "zero_post_stop_target_work": True,
            "reference_oracle_output": "equal",
            "signed_fixture_inputs": "immutable",
            "ample_reports": "byte_identical",
            "unexpected_identity": "preserved",
        },
        "gate:work",
    )
    validators = record["validators"]
    require(
        type(validators) is list
        and tuple(
            (row.get("path"), row.get("sha256")) if type(row) is dict else None
            for row in validators
        )
        == VALIDATORS,
        "gate:validators",
    )
    for row in validators:
        keys(row, ("path", "sha256"), "gate:validator")
    require(tuple(record["proof_tests"]) == PROOF_TESTS, "gate:proofs")
    findings = keys(record["findings"], ("open", "held"), "gate:findings")
    require(
        tuple(findings["open"]) == OPEN and findings["held"] == ["FINDING_080"],
        "gate:findings",
    )
    require(
        record["next_step"] == "step_1433"
        and tuple(record["holds"]) == HOLDS
        and record["result"] == "pass",
        "gate:result",
    )


def validate_sources() -> None:
    prior = REVIEWED_PREDECESSOR
    for step, candidate in CANDIDATES:
        require(git_text("rev-parse", "--verify", f"{candidate}^{{commit}}") == candidate, "candidate:" + step)
        require(git_text("rev-parse", "--verify", f"{candidate}^") == prior, "candidate:parent:" + step)
        prior = candidate
    final = CANDIDATES[-1][1]
    require(git_file_sha(final, SOURCE) == SOURCE_SHA256, "source:sha256")
    for path, expected in VALIDATORS:
        require(git_file_sha(final, path) == expected, "validator:sha256:" + path)
    require(sha256(SCHEMA) == SCHEMA_SHA256, "schema:sha256")
    contract = json.loads((ROOT / "spec/causal_projection_operation_contract_v13.json").read_text())
    require(
        contract["final_operation_count"] == 14
        and tuple(row["id"] for row in contract["families"])
        == (
            "canonical_source_pull",
            "canonical_order_compare",
            "membership_lookup",
            "candidate_lookup",
            "dependency_lookup",
            "state_lookup",
            "readiness_transition",
            "checked_arithmetic",
            "map_insertion",
            "set_insertion",
            "shared_reference_clone",
            "causal_maximum_compare",
            "result_publication",
            "constant_candidate_validation",
        ),
        "contract:families",
    )
    fixture_drift = git_text(
        "diff", "--name-only", REVIEWED_PREDECESSOR, final, "--", "fixtures"
    )
    require(fixture_drift == "", "fixtures:immutable")
    actor_source = (ROOT / SOURCE).read_text()
    reproduction_source = (
        ROOT / "crates/nostr_automerge/tests/remediation_v13_reproductions.rs"
    ).read_text()
    for index, test in enumerate(PROOF_TESTS):
        source = actor_source if index < 6 else reproduction_source
        require(source.count(f"fn {test}()") == 1, "proof:" + test)
        declaration = source.index(f"fn {test}()")
        attributes = source[max(0, declaration - 180) : declaration]
        require("#[test]" in attributes and "#[ignore" not in attributes, "proof:enabled:" + test)


def mutation_self_test(record: object) -> int:
    mutators = (
        lambda value: value.update(status="implementation_active"),
        lambda value: value.update(rcld=118),
        lambda value: value["candidate_chain"].pop(),
        lambda value: value["candidate_chain"].reverse(),
        lambda value: value["candidate_chain"][0].update(candidate="0" * 40),
        lambda value: value["requirements"].pop(),
        lambda value: value["requirements"].reverse(),
        lambda value: value["projection"].update(source_sha256="0" * 64),
        lambda value: value["projection"].update(sealed_constructors=2),
        lambda value: value["projection"].update(operation_families=13),
        lambda value: value["projection"]["operation_counts"].pop(),
        lambda value: value["projection"]["operation_counts"].reverse(),
        lambda value: value["projection"]["operation_counts"].__setitem__(0, 5),
        lambda value: value["projection"].update(raw_builder_charges=1),
        lambda value: value["projection"].update(unmetered_production_bypasses=1),
        lambda value: value["work_contract"].update(total_charges=71),
        lambda value: value["work_contract"].update(graph_node_charges=55),
        lambda value: value["work_contract"].update(first_stop_preserved=False),
        lambda value: value["work_contract"].update(reference_oracle_output="different"),
        lambda value: value["validators"].reverse(),
        lambda value: value["validators"][0].update(sha256="0" * 64),
        lambda value: value["proof_tests"].pop(),
        lambda value: value["proof_tests"].reverse(),
        lambda value: value["findings"]["open"].pop(),
        lambda value: value["findings"]["held"].clear(),
        lambda value: value.update(next_step="step_1432"),
        lambda value: value["holds"].pop(),
        lambda value: value.update(result="fail"),
        lambda value: value.update(unapproved=False),
    )
    mutations = []
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
        raise GateError(f"mutation_survived:{index}")
    return len(mutations)


def main() -> int:
    record = json.loads(REPORT.read_text())
    validate_record(record)
    validate_sources()
    mutations = mutation_self_test(record)
    print(
        "PASS: causal-projection implementation gate "
        f"rcld=117 families=14 operations={sum(OPERATION_COUNTS)} mutations={mutations}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
