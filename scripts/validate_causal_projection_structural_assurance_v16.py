#!/usr/bin/env python3
"""Validate v16 causal-projection structure independently from identity."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_structural_assurance_v16.json"
SCHEMA = ROOT / "tools/validation/causal_projection_structural_assurance_v16.schema.json"
CONTRACT = ROOT / "spec/causal_projection_contracts_v16.json"
INVENTORY = ROOT / "reports/causal_projection_operation_inventory_v16.json"
PROOFS = ROOT / "reports/causal_projection_proof_catalog_v16.json"
SOURCE_PATHS = {
    "actor": "crates/nostr_automerge/src/graph/actor_state.rs",
    "control": "crates/nostr_automerge/src/control/epoch_state.rs",
    "consumer": "crates/nostr_automerge/src/reference/epoch_engine.rs",
}
SOURCE_CANDIDATE = "bbb17083b4110e912a672f30b329f7799e2df1a5"
SOURCE_HASHES = {
    "actor": "101e9502101d7c08d11dadafc46c679a084bfe88b8ea8614c79682565c3bbc0e",
    "control": "734f70b9eed8f4281d719b0581153db1175bbe1401c11fcd0c0ef59b36343221",
    "consumer": "0f7e948b27b6cc0d7b921596bde5bba496ef72fca43c6f4e485a68a1919c4315",
}
INVENTORY_SHA256 = "95562a0f032c6fcedf3e397f82f42072fa2179b30a48b7424e38c2bf39403de1"
PROOF_SHA256 = "486dd1f70a108166a5380ef533f707f1aeebac6b4f5b2d1f20708a9a4e0f4ca0"
CONTRACT_SHA256 = "bbd58073a7dab83d7a96541ba7d1a90e0ceb5c4876bb4533d7b196058b5e7b3b"
PROPERTY_CODES = [
    "UNWRAPPED_ACTOR_SEQUENCE_DECISION",
    "CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS",
    "DUPLICATE_CAUSAL_START_COMPARISON",
    "UNMETERED_FINAL_TRAVERSAL",
    "STATE_WRITE_BEFORE_CHARGE",
    "CHARGE_AFTER_OPERATION",
    "POST_STOP_TARGET_WORK",
    "PUBLICATION_BEFORE_CHARGE",
    "ALTERNATE_CONSUMER_BYPASS",
    "COUNTER_MISMATCH",
]
TOP_FIELDS = [
    "schema", "status", "source_candidate", "source_bindings",
    "inventory_sha256", "proof_catalog_sha256", "contracts_sha256",
    "modes", "full_order", "property_codes", "structural_summary",
    "neutral_comment_result", "result_identity_sha256", "result",
]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_operation_inventory_v16 import (  # noqa: E402
    InventoryError,
    derive_rows,
)
from validate_report_contract_v9 import ReportSuiteError, rust_code_view  # noqa: E402


class OwnershipError(RuntimeError):
    def __init__(self, code: str):
        super().__init__(code)
        self.code = code


def require(condition: bool, code: str) -> None:
    if not condition:
        raise OwnershipError(code)


def sha_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def code_view(source: str) -> str:
    try:
        return rust_code_view(source)
    except ReportSuiteError as error:
        raise OwnershipError("ALTERNATE_CONSUMER_BYPASS") from error


def body(source: str, symbol: str, code: str) -> str:
    lexical = code_view(source)
    declaration = re.compile(
        rf"(?m)^[ \t]*(?:pub\s*\(\s*crate\s*\)\s+)?fn\s+{re.escape(symbol)}\b"
        rf"(?:[ \t\r\n]*<[^{{;]+>)?[^{{;]*\{{"
    )
    matches = tuple(declaration.finditer(lexical))
    require(len(matches) == 1, code)
    opening = matches[0].end() - 1
    depth = 0
    for cursor in range(opening, len(lexical)):
        if lexical[cursor] == "{":
            depth += 1
        elif lexical[cursor] == "}":
            depth -= 1
            if depth == 0:
                return lexical[opening + 1:cursor]
    raise OwnershipError(code)


def production(source: str) -> str:
    marker = "\n#[cfg(test)]\npub(crate) mod tests {"
    require(source.count(marker) == 1, "ALTERNATE_CONSUMER_BYPASS")
    return source.split(marker, 1)[0] + "\n"


def call_spans(value: str, name: str) -> list[tuple[int, int]]:
    spans: list[tuple[int, int]] = []
    for match in re.finditer(rf"\b{re.escape(name)}\s*\(", value):
        opening = value.find("(", match.start())
        depth = 0
        for cursor in range(opening, len(value)):
            if value[cursor] == "(":
                depth += 1
            elif value[cursor] == ")":
                depth -= 1
                if depth == 0:
                    spans.append((match.start(), cursor + 1))
                    break
        else:
            raise OwnershipError("UNMETERED_FINAL_TRAVERSAL")
    return spans


def masked(value: str, spans: list[tuple[int, int]]) -> str:
    result = list(value)
    for start, end in spans:
        for cursor in range(start, end):
            if result[cursor] not in "\r\n":
                result[cursor] = " "
    return "".join(result)


def validate_actor_sequence(actor: str) -> None:
    value = body(actor, "actor_sequence_decision_metered_observed", "UNWRAPPED_ACTOR_SEQUENCE_DECISION")
    expected = ["ActorStateRead", "PredecessorCandidateRead", "ActorIdentityDecision", "SequenceRelationDecision"]
    observed = re.findall(r"observed\s*\(\s*ActorDecisionOperation::([A-Za-z0-9_]+)\s*\)", value)
    charges = list(re.finditer(r"charge\s*\(\s*WorkCounter::GraphNode\s*\).*?\?\s*;", value))
    require(observed == expected and len(charges) == 4, "UNWRAPPED_ACTOR_SEQUENCE_DECISION")
    identity = value.find("value.actor == candidate.actor")
    stop = value.find("actor_relation == ActorIdentityRelation::InvalidPredecessor")
    sequence = value.find("let sequence_relation =")
    require(identity >= 0 and value.count("value.actor == candidate.actor") == 1, "UNWRAPPED_ACTOR_SEQUENCE_DECISION")
    require(identity < stop < sequence, "UNWRAPPED_ACTOR_SEQUENCE_DECISION")


def validate_stage_order(actor: str) -> None:
    value = body(actor, "candidate_semantics_decision_metered_observed", "CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS")
    anchors = [
        "self.actor_sequence_decision_metered(candidate, &mut charge)?;",
        "observed(CandidateSemanticStage::ActorSequence);",
        "self.causal_next_decision_metered(candidate, &mut charge)?;",
        "observed(CandidateSemanticStage::CausalCounter);",
        "self.empty_frontier_decision_metered(candidate, base_frontier, charge)?;",
        "observed(CandidateSemanticStage::EmptyFrontier);",
    ]
    positions = [value.find(anchor) for anchor in anchors]
    require(all(position >= 0 for position in positions), "CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS")
    require(positions == sorted(positions) and all(value.count(anchor) == 1 for anchor in anchors), "CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS")


def validate_causal_comparison(actor: str) -> None:
    value = body(actor, "causal_next_decision_metered_observed", "DUPLICATE_CAUSAL_START_COMPARISON")
    require(value.count("candidate.start_op == causal_next_op") == 1, "DUPLICATE_CAUSAL_START_COMPARISON")
    require(value.count("CausalNextOperation::ExpectedStartComparison") == 1, "DUPLICATE_CAUSAL_START_COMPARISON")


def validate_wrapper(actor: str) -> None:
    value = body(actor, "perform_projection_build_operation", "POST_STOP_TARGET_WORK")
    charge = value.find("charge(counter)")
    perform = value.find("let result = perform();")
    observed = value.find("observed(operation);")
    require(charge >= 0 and perform >= 0 and charge < perform, "CHARGE_AFTER_OPERATION")
    require(perform < observed and value.count("perform()") == 1 and value.count("observed(operation)") == 1, "CHARGE_AFTER_OPERATION")
    require(
        "charge(counter).map_err(MeteredActorStateError::Work)?;" in value
        and value.rstrip().endswith("Ok(result)"),
        "POST_STOP_TARGET_WORK",
    )


def validate_build_ownership(actor: str) -> None:
    value = body(actor, "build_trusted_epoch_projection_observed", "UNMETERED_FINAL_TRAVERSAL")
    spans = call_spans(value, "perform_projection_build_operation")
    source_calls = {
        "member_count": 1,
        "next_member": 1,
        "accepted_member": 2,
        "candidate": 2,
        "dependency_count": 1,
        "dependency": 1,
    }
    for symbol, count in source_calls.items():
        require(value.count(f"source.{symbol}(") == count, "UNMETERED_FINAL_TRAVERSAL")
    outside = masked(value, spans)
    require(len(spans) == 50, "UNMETERED_FINAL_TRAVERSAL")
    require(
        re.search(r"\bsource\.(?:member_count|next_member|accepted_member|candidate|dependency_count|dependency)\s*\(", outside) is None,
        "UNMETERED_FINAL_TRAVERSAL",
    )
    require("states.values()" not in outside, "UNMETERED_FINAL_TRAVERSAL")
    writes = re.compile(
        r"\b(?:dependencies|depended_on|remaining_dependencies|dependants|ready|states|frontier_heads|writer_contributions|causal_next_by_change)"
        r"\s*\.(?:entry|insert|remove|pop_first|get_mut)\s*\(|\*\s*remaining\s*="
    )
    require(writes.search(outside) is None, "STATE_WRITE_BEFORE_CHARGE")
    publications = list(re.finditer(r"\bpublished\s*\(\s*ProjectionPublicationOperation::", value))
    require(len(publications) == 14, "PUBLICATION_BEFORE_CHARGE")
    for publication in publications:
        prior = [span for span in spans if span[1] < publication.start()]
        require(bool(prior), "PUBLICATION_BEFORE_CHARGE")
        between = value[prior[-1][1]:publication.start()]
        require(re.fullmatch(r"\s*\?\s*;\s*", between) is not None, "PUBLICATION_BEFORE_CHARGE")


def validate_consumers(sources: dict[str, str]) -> None:
    actor = production(sources["actor"])
    consumer = body(sources["consumer"], "evaluate_epoch", "ALTERNATE_CONSUMER_BYPASS")
    control = body(sources["control"], "new_metered", "ALTERNATE_CONSUMER_BYPASS")
    require(actor.count(".actor_sequence_decision_metered(") == 1, "ALTERNATE_CONSUMER_BYPASS")
    require(actor.count(".causal_next_decision_metered(") == 1, "ALTERNATE_CONSUMER_BYPASS")
    require(actor.count(".empty_frontier_decision_metered(") == 1, "ALTERNATE_CONSUMER_BYPASS")
    require(consumer.count(".candidate_semantics_decision_metered(") == 1, "ALTERNATE_CONSUMER_BYPASS")
    require(".actor_sequence_decision_metered(" not in consumer, "ALTERNATE_CONSUMER_BYPASS")
    require(".causal_next_decision_metered(" not in consumer, "ALTERNATE_CONSUMER_BYPASS")
    require(".empty_frontier_decision_metered(" not in consumer, "ALTERNATE_CONSUMER_BYPASS")
    require(consumer.count("initialize_actor_states_metered(") == 1, "ALTERNATE_CONSUMER_BYPASS")
    require(control.count("initialize_actor_states_metered(") == 1, "ALTERNATE_CONSUMER_BYPASS")


def validate_rows(actor: str, inventory: dict[str, Any], proofs: dict[str, Any]) -> None:
    try:
        rows = derive_rows(actor)
    except InventoryError as error:
        raise OwnershipError("UNMETERED_FINAL_TRAVERSAL") from error
    expected = inventory["rows"]
    require(len(rows) == len(expected) == len(proofs["rows"]) == 68, "ALTERNATE_CONSUMER_BYPASS")
    for actual, recorded, proof in zip(rows, expected, proofs["rows"], strict=True):
        require(actual["id"] == recorded["id"] == proof["id"], "ALTERNATE_CONSUMER_BYPASS")
        require(actual["source_site"] == recorded["source_site"] == proof["source_site"], "ALTERNATE_CONSUMER_BYPASS")
        require(actual["counter"] == recorded["counter"] == proof["counter"], "COUNTER_MISMATCH")
        require(proof["result"] == "pass" and proof["test"] == recorded["test"], "ALTERNATE_CONSUMER_BYPASS")
    dependency = [row for row in rows if row["abstract_family"] == "projection_construction.dependency_count_read"]
    require(len(dependency) == 1 and dependency[0]["counter"] == "graph_node", "COUNTER_MISMATCH")


def validate_structural(sources: dict[str, str], contract: dict[str, Any], inventory: dict[str, Any], proofs: dict[str, Any]) -> None:
    require(contract["structural_identity"]["property_codes"] == PROPERTY_CODES, "ALTERNATE_CONSUMER_BYPASS")
    actor = sources["actor"]
    validate_actor_sequence(actor)
    validate_stage_order(actor)
    validate_causal_comparison(actor)
    validate_wrapper(actor)
    validate_build_ownership(actor)
    validate_consumers(sources)
    validate_rows(actor, inventory, proofs)


def source_bindings() -> list[dict[str, str]]:
    return [
        {"role": role, "path": SOURCE_PATHS[role], "sha256": SOURCE_HASHES[role]}
        for role in ["actor", "control", "consumer"]
    ]


def expected_report() -> dict[str, Any]:
    value: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_structural_assurance.v16.v1",
        "status": "pass",
        "source_candidate": SOURCE_CANDIDATE,
        "source_bindings": source_bindings(),
        "inventory_sha256": INVENTORY_SHA256,
        "proof_catalog_sha256": PROOF_SHA256,
        "contracts_sha256": CONTRACT_SHA256,
        "modes": ["structural", "identity", "full"],
        "full_order": ["structural", "identity"],
        "property_codes": PROPERTY_CODES,
        "structural_summary": {
            "rows": 68,
            "phases": {
                "projection_construction": 50,
                "actor_sequence": 4,
                "causal_counter_consumer": 3,
                "frontier_comparison": 11,
            },
            "consumer_bindings": 3,
            "checks": [{"code": code, "result": "pass"} for code in PROPERTY_CODES],
        },
        "neutral_comment_result": ["structural_pass", "identity_fail"],
        "result_identity_sha256": "",
        "result": "pass",
    }
    projection = {key: item for key, item in value.items() if key != "result_identity_sha256"}
    value["result_identity_sha256"] = sha_bytes(canonical(projection))
    return value


def validate_schema(schema: object) -> None:
    require(type(schema) is dict and schema.get("additionalProperties") is False, "SCHEMA_IDENTITY")
    assert isinstance(schema, dict)
    require(schema.get("required") == TOP_FIELDS and list(schema.get("properties", {})) == TOP_FIELDS, "SCHEMA_IDENTITY")
    definitions = schema.get("$defs", {})
    require(list(definitions) == ["source_binding", "phase_counts", "check", "structural_summary"], "SCHEMA_IDENTITY")
    for definition in definitions.values():
        require(type(definition) is dict and definition.get("additionalProperties") is False, "SCHEMA_IDENTITY")


def validate_identity(report: object, schema: object, sources: dict[str, str]) -> None:
    require(type(report) is dict and list(report) == TOP_FIELDS and report == expected_report(), "REPORT_IDENTITY")
    validate_schema(schema)
    resolved = subprocess.run(["git", "rev-parse", f"{SOURCE_CANDIDATE}^{{commit}}"], cwd=ROOT, capture_output=True, text=True, check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == SOURCE_CANDIDATE, "SOURCE_CANDIDATE_IDENTITY")
    for role, expected in SOURCE_HASHES.items():
        require(sha_bytes(sources[role].encode()) == expected, "SOURCE_IDENTITY")
        committed = subprocess.run(
            ["git", "show", f"{SOURCE_CANDIDATE}:{SOURCE_PATHS[role]}"],
            cwd=ROOT,
            capture_output=True,
            check=False,
        )
        require(committed.returncode == 0 and sha_bytes(committed.stdout) == expected, "SOURCE_CANDIDATE_IDENTITY")
    require(sha(INVENTORY) == INVENTORY_SHA256, "INVENTORY_IDENTITY")
    require(sha(PROOFS) == PROOF_SHA256, "PROOF_IDENTITY")
    require(sha(CONTRACT) == CONTRACT_SHA256, "CONTRACT_IDENTITY")


def replace_once(value: str, old: str, new: str) -> str:
    require(value.count(old) == 1, "SELF_TEST_SETUP")
    return value.replace(old, new, 1)


def structural_mutations(sources: dict[str, str]) -> list[tuple[str, dict[str, str]]]:
    actor = sources["actor"]
    cases: list[tuple[str, dict[str, str]]] = []

    def actor_case(code: str, changed: str) -> None:
        value = dict(sources)
        value["actor"] = changed
        cases.append((code, value))

    actor_case(
        "UNWRAPPED_ACTOR_SEQUENCE_DECISION",
        replace_once(
            actor,
            "        charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;\n        let actor_relation =",
            "        let actor_relation =",
        ),
    )
    actor_case(
        "CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS",
        replace_once(
            actor,
            "        self.actor_sequence_decision_metered(candidate, &mut charge)?;\n        observed(CandidateSemanticStage::ActorSequence);\n        self.causal_next_decision_metered(candidate, &mut charge)?;",
            "        self.causal_next_decision_metered(candidate, &mut charge)?;\n        observed(CandidateSemanticStage::ActorSequence);\n        self.actor_sequence_decision_metered(candidate, &mut charge)?;",
        ),
    )
    actor_case(
        "DUPLICATE_CAUSAL_START_COMPARISON",
        replace_once(actor, "candidate.start_op == causal_next_op", "candidate.start_op == causal_next_op && candidate.start_op == causal_next_op"),
    )
    actor_case(
        "UNMETERED_FINAL_TRAVERSAL",
        replace_once(actor, "    let projection = perform_projection_build_operation(\n        WorkCounter::GraphNode,\n        ProjectionBuildOperation::ResultPublication,", "    let _ = source.member_count();\n    let projection = perform_projection_build_operation(\n        WorkCounter::GraphNode,\n        ProjectionBuildOperation::ResultPublication,"),
    )
    actor_case(
        "STATE_WRITE_BEFORE_CHARGE",
        replace_once(actor, "        let previous_state = perform_projection_build_operation(", "        ready.insert(hash);\n        let previous_state = perform_projection_build_operation("),
    )
    actor_case(
        "CHARGE_AFTER_OPERATION",
        replace_once(actor, "    charge(counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();", "    let result = perform();\n    charge(counter).map_err(MeteredActorStateError::Work)?;"),
    )
    actor_case(
        "POST_STOP_TARGET_WORK",
        replace_once(actor, "    charge(counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();", "    let charged = charge(counter).map_err(MeteredActorStateError::Work);\n    let result = perform();\n    charged?;"),
    )
    actor_case(
        "PUBLICATION_BEFORE_CHARGE",
        replace_once(actor, "    let member_count = perform_projection_build_operation(", "    published(ProjectionPublicationOperation::Projection);\n    let member_count = perform_projection_build_operation("),
    )
    alternate = dict(sources)
    alternate["consumer"] = replace_once(
        alternate["consumer"],
        ".candidate_semantics_decision_metered(\n",
        ".actor_sequence_decision_metered(candidate, |_| Ok::<_, ()>(()))?;\n                match projection.candidate_semantics_decision_metered(\n",
    )
    cases.append(("ALTERNATE_CONSUMER_BYPASS", alternate))
    actor_case(
        "COUNTER_MISMATCH",
        replace_once(
            actor,
            "WorkCounter::GraphNode,\n            ProjectionBuildOperation::DependencyCountRead",
            "WorkCounter::GraphEdge,\n            ProjectionBuildOperation::DependencyCountRead",
        ),
    )
    return cases


def self_test(report: dict[str, Any], schema: dict[str, Any], sources: dict[str, str], contract: dict[str, Any], inventory: dict[str, Any], proofs: dict[str, Any]) -> int:
    caught = 0
    for expected, changed_sources in structural_mutations(sources):
        try:
            validate_structural(changed_sources, contract, inventory, proofs)
        except OwnershipError as error:
            require(error.code == expected, "SELF_TEST_CODE:" + expected)
            caught += 1
            continue
        raise OwnershipError("SELF_TEST_SURVIVED:" + expected)

    neutral = dict(sources)
    neutral["actor"] = replace_once(
        neutral["actor"],
        "use std::collections::{BTreeMap, BTreeSet};",
        "// neutral ownership comment\nuse std::collections::{BTreeMap, BTreeSet};",
    )
    validate_structural(neutral, contract, inventory, proofs)
    try:
        validate_identity(report, schema, neutral)
    except OwnershipError as error:
        require(error.code == "SOURCE_IDENTITY", "SELF_TEST_NEUTRAL_IDENTITY")
        caught += 1
    else:
        raise OwnershipError("SELF_TEST_NEUTRAL_SURVIVED")

    lexical = dict(sources)
    lexical["actor"] = replace_once(
        lexical["actor"],
        "use std::collections::{BTreeMap, BTreeSet};",
        "// source.member_count(); published(ProjectionPublicationOperation::Projection);\nuse std::collections::{BTreeMap, BTreeSet};",
    )
    validate_structural(lexical, contract, inventory, proofs)
    caught += 1

    coordinated = copy.deepcopy(report)
    coordinated["source_bindings"][0]["sha256"] = sha_bytes(neutral["actor"].encode())
    coordinated["result_identity_sha256"] = sha_bytes(canonical({key: coordinated[key] for key in TOP_FIELDS if key != "result_identity_sha256"}))
    try:
        validate_identity(coordinated, schema, neutral)
    except OwnershipError:
        caught += 1
    else:
        raise OwnershipError("SELF_TEST_COORDINATED_SURVIVED")

    changed_schema = copy.deepcopy(schema)
    changed_schema["additionalProperties"] = True
    try:
        validate_identity(report, changed_schema, sources)
    except OwnershipError as error:
        require(error.code == "SCHEMA_IDENTITY", "SELF_TEST_SCHEMA_CODE")
        caught += 1
    else:
        raise OwnershipError("SELF_TEST_SCHEMA_SURVIVED")
    doubled = dict(sources)
    doubled["actor"] = replace_once(
        doubled["actor"],
        "|| source.next_member(),",
        "|| { let member = source.next_member(); let _ = source.next_member(); member },",
    )
    try:
        validate_structural(doubled, contract, inventory, proofs)
    except OwnershipError as error:
        require(error.code == "UNMETERED_FINAL_TRAVERSAL", "SELF_TEST_DOUBLE_OPERATION")
        caught += 1
    else:
        raise OwnershipError("SELF_TEST_DOUBLE_OPERATION_SURVIVED")
    require(caught == 15, "SELF_TEST_COUNT")
    return caught


def load_sources(source_root: Path = ROOT) -> dict[str, str]:
    return {role: (source_root / path).read_text() for role, path in SOURCE_PATHS.items()}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=["structural", "identity", "full"], default="full")
    parser.add_argument("--write-report", action="store_true")
    parser.add_argument("--source-root", type=Path, default=ROOT)
    args = parser.parse_args()
    if args.write_report:
        REPORT.write_text(json.dumps(expected_report(), ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    contract = json.loads(CONTRACT.read_text())
    inventory = json.loads(INVENTORY.read_text())
    proofs = json.loads(PROOFS.read_text())
    sources = load_sources(args.source_root.resolve())
    try:
        if args.mode in {"structural", "full"}:
            validate_structural(sources, contract, inventory, proofs)
        if args.mode in {"identity", "full"}:
            validate_identity(report, schema, sources)
        mutations = self_test(report, schema, sources, contract, inventory, proofs)
    except OwnershipError as error:
        print(f"FAIL: causal projection structural assurance v16 code={error.code}", file=sys.stderr)
        return 1
    print(f"PASS: causal projection structural assurance v16 mode={args.mode} rows=68 property_codes=10 mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
