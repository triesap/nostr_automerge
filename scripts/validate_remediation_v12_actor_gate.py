#!/usr/bin/env python3
"""Validate the closed RCLD-111 actor, counter, and frontier gate."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess


ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/remediation_v12_actor_gate.json"
SCHEMA = ROOT / "tools/validation/remediation_v12_actor_gate.schema.json"
SCHEMA_SHA256 = "2819004af57e5f75a0cded5dc907ad8c710a74fda72b278a4a629091f14faa91"
CANDIDATES = (
    ("step_1379", "187a260858bdd41f9dd967e32279b9f21454ede2"),
    ("step_1380", "629747ab2537593ddf0a4f689a3e14ea6e576039"),
    ("step_1381", "3ced1446a04a422f5760d00d9786dd18ace76930"),
    ("step_1382", "63661f2f1df8394d7526ec2f5d2fa2be60e65efe"),
    ("step_1383", "287b0967751f7575faf3a8ee38c15c65aa428290"),
    ("step_1384", "5377b9d276b4deabd0fd3c6f6dac1734d213e74d"),
    ("step_1385", "502c5701d73c4452906ef92f0a324908b9d039a8"),
    ("step_1386", "6a190cefa11f84e069ece47644b66992dc82e8f3"),
)
REQUIREMENTS = (
    "NCRDT-RESOURCE-017",
    "NCRDT-RESOURCE-018",
    "NCRDT-RESOURCE-019",
    "NCRDT-EVIDENCE-007",
)
SOURCE_BINDINGS = (
    ("crates/nostr_automerge/src/graph/actor_state.rs", "b33733c9c84a7b7a6247967172a562c2c3f0da68a64514a50b27b715127c8290"),
    ("crates/nostr_automerge/src/reference/epoch_engine.rs", "3ffd16c4fb3b8d6de7c7a6aa49e2a7f8f57ef23df39e54f1a1f2f511b9bf68ac"),
    ("tools/nostr_automerge_conformance/src/runner.rs", "dcf1826785ff35fd636838e55b67a0b40bf40d4ba3ff5dca1875e64d56233b5a"),
    ("scripts/validate_remediation_v12.py", "be0e3cd28d9aa2d9e3973d4a950d53d9a39b804378a8fea8780afd18b1e7bd75"),
    ("scripts/reproduce_remediation_v12.py", "ad3e3c6df4f20963efb2bea22aceac373e7b014d9b11d91ec5028f39d65019a4"),
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
    "schema", "status", "rcld", "candidate_chain", "requirements",
    "decisions", "work_contract", "reproductions", "source_bindings",
    "findings", "holds", "result",
)
PROOF_TESTS = (
    ("crates/nostr_automerge/src/graph/actor_state.rs", "projected_actor_sequence_decision_is_nonmutating_and_complete"),
    ("crates/nostr_automerge/src/graph/actor_state.rs", "finding_100_actor_predecessor_scan_reproduction"),
    ("crates/nostr_automerge/src/graph/actor_state.rs", "projected_causal_next_decision_is_checked_constant_size_and_exactly_metered"),
    ("crates/nostr_automerge/src/graph/actor_state.rs", "finding_100_causal_next_op_scan_reproduction"),
    ("crates/nostr_automerge/src/graph/actor_state.rs", "empty_frontier_comparison_is_streaming_exact_and_immediately_metered"),
    ("crates/nostr_automerge/src/graph/actor_state.rs", "finding_100_empty_frontier_work_reproduction"),
    ("crates/nostr_automerge/src/graph/actor_state.rs", "complete_candidate_semantics_preserve_precedence_and_every_stop_boundary"),
    ("tools/nostr_automerge_conformance/src/runner.rs", "actor_counter_frontier_reports_match_predecessor_bytes"),
)


class GateError(RuntimeError):
    pass


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    )
    if result.returncode:
        raise GateError("git:" + ":".join(args))
    return result.stdout.strip()


def git_file_sha(candidate: str, path: str) -> str:
    result = subprocess.run(
        ["git", "show", f"{candidate}:{path}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        raise GateError("git:file:" + path)
    return hashlib.sha256(result.stdout).hexdigest()


def require_keys(value: object, keys: tuple[str, ...], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or tuple(value) != keys:
        raise GateError(label + ":keys")
    return value


def validate_record(value: object) -> None:
    record = require_keys(value, TOP_KEYS, "gate")
    if record["schema"] != "nostr_automerge.remediation_v12_actor_gate.v1":
        raise GateError("gate:schema")
    if record["status"] != "rcld_111_complete" or record["rcld"] != 111:
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
    decisions = require_keys(
        record["decisions"],
        ("actor_lookup_operations", "causal_counter_operations", "frontier_operations", "combined_stages", "signed_scenarios", "delivery_order_minimum", "typed_precedence"),
        "gate:decisions",
    )
    if decisions != {
        "actor_lookup_operations": 9,
        "causal_counter_operations": 3,
        "frontier_operations": 11,
        "combined_stages": 3,
        "signed_scenarios": 8,
        "delivery_order_minimum": 2,
        "typed_precedence": "actor_then_counter_then_frontier",
    }:
        raise GateError("gate:decisions")
    work = require_keys(
        record["work_contract"],
        ("budget_matrix", "cancellation_matrix", "first_stop_preserved", "zero_later_stage_work", "unexpected_identity", "predecessor_output_bytes", "production_bypasses"),
        "gate:work",
    )
    if work != {
        "budget_matrix": "pass",
        "cancellation_matrix": "pass",
        "first_stop_preserved": True,
        "zero_later_stage_work": True,
        "unexpected_identity": "preserved",
        "predecessor_output_bytes": "equal",
        "production_bypasses": 0,
    }:
        raise GateError("gate:work")
    reproductions = require_keys(
        record["reproductions"],
        ("fixed_families", "remaining_finding_100_families", "finding_100_status"),
        "gate:reproductions",
    )
    if reproductions != {
        "fixed_families": ["actor_predecessor", "causal_next_op", "empty_frontier"],
        "remaining_finding_100_families": 7,
        "finding_100_status": "open",
    }:
        raise GateError("gate:reproductions")
    bindings = record["source_bindings"]
    if not isinstance(bindings, list) or tuple(
        (row.get("path"), row.get("sha256")) if isinstance(row, dict) else None
        for row in bindings
    ) != SOURCE_BINDINGS:
        raise GateError("gate:bindings")
    for row in bindings:
        require_keys(row, ("path", "sha256"), "gate:binding")
    findings = require_keys(record["findings"], ("open", "held"), "gate:findings")
    if tuple(findings["open"]) != OPEN_FINDINGS or findings["held"] != ["FINDING_080"]:
        raise GateError("gate:findings")
    if tuple(record["holds"]) != HOLDS or record["result"] != "pass":
        raise GateError("gate:result")


def validate_runtime_sources(actor: str | None = None, engine: str | None = None) -> None:
    actor = actor or (ROOT / SOURCE_BINDINGS[0][0]).read_text()
    engine = engine or (ROOT / SOURCE_BINDINGS[1][0]).read_text()
    actor_production = actor.split("#[cfg(test)]\npub(crate) mod tests", 1)[0]
    engine_production = engine.split("#[cfg(test)]\nmod tests", 1)[0]
    ordered = (
        "self.actor_sequence_decision_metered(candidate, &mut charge)?;",
        "self.causal_next_decision_metered(candidate, &mut charge)?;",
        "self.empty_frontier_decision_metered(candidate, base_frontier, charge)?;",
    )
    positions = tuple(actor_production.find(token) for token in ordered)
    if any(position < 0 for position in positions) or positions != tuple(sorted(positions)):
        raise GateError("source:decision_order")
    if engine_production.count(".candidate_semantics_decision_metered(") != 1:
        raise GateError("source:combined_route")
    if any(
        token in engine_production
        for token in (
            ".actor_sequence_decision_metered(",
            ".causal_next_decision_metered(",
            ".empty_frontier_decision_metered(",
            "validate_actor_predecessor",
            "legacy_counter_is_valid",
        )
    ):
        raise GateError("source:bypass")


def validate_sources() -> None:
    final = CANDIDATES[-1][1]
    for path, expected in SOURCE_BINDINGS:
        if git_file_sha(final, path) != expected:
            raise GateError("source:sha256:" + path)
    prior = git("rev-parse", f"{CANDIDATES[0][1]}^")
    for step, candidate in CANDIDATES:
        parents = git("rev-list", "--parents", "-n", "1", candidate).split()
        if parents != [candidate, prior]:
            raise GateError("candidate:parent:" + step)
        prior = candidate
    for path, test in PROOF_TESTS:
        source = (ROOT / path).read_text()
        declaration = f"fn {test}()"
        if source.count(declaration) != 1:
            raise GateError("proof:test:" + test)
        attributes = source[: source.index(declaration)].rsplit("#[test]", 1)
        if len(attributes) != 2 or "#[ignore" in attributes[-1]:
            raise GateError("proof:attributes:" + test)
    requirements = json.loads((ROOT / "spec/requirements.json").read_text())
    ids = tuple(row.get("id") for row in requirements.get("requirements", []))
    if len(ids) != 156 or ids[-4:] != REQUIREMENTS:
        raise GateError("requirements:inventory")
    reproductions = json.loads(
        git("show", f"{CANDIDATES[-1][1]}:spec/remediation_v12_reproductions.json")
    )
    cases = reproductions.get("cases", [])
    if [row.get("family") for row in cases[:3]] != [
        "actor_predecessor", "causal_next_op", "empty_frontier"
    ] or any(row.get("expected") != "fixed_pass" for row in cases[:3]):
        raise GateError("reproductions:fixed")
    if len(cases) != 10 or any(row.get("expected") != "open_failure" for row in cases[3:]):
        raise GateError("reproductions:remaining")
    validate_runtime_sources()
    if sha256(SCHEMA) != SCHEMA_SHA256:
        raise GateError("schema:sha256")


def mutation_self_test(record: object) -> tuple[int, int]:
    mutators = (
        lambda value: value.update(status="implementation_in_progress"),
        lambda value: value.update(rcld=110),
        lambda value: value["candidate_chain"].pop(),
        lambda value: value["candidate_chain"].reverse(),
        lambda value: value["candidate_chain"][0].update(candidate="0" * 40),
        lambda value: value["requirements"].pop(),
        lambda value: value["decisions"].update(actor_lookup_operations=8),
        lambda value: value["decisions"].update(typed_precedence="counter_then_actor"),
        lambda value: value["work_contract"].update(first_stop_preserved=False),
        lambda value: value["work_contract"].update(production_bypasses=1),
        lambda value: value["reproductions"]["fixed_families"].pop(),
        lambda value: value["reproductions"].update(remaining_finding_100_families=6),
        lambda value: value["reproductions"].update(finding_100_status="closed"),
        lambda value: value["source_bindings"].reverse(),
        lambda value: value["source_bindings"][0].update(sha256="0" * 64),
        lambda value: value["findings"]["open"].pop(),
        lambda value: value["findings"]["held"].clear(),
        lambda value: value["holds"].pop(),
        lambda value: value.update(result="fail"),
        lambda value: value.update(unapproved=False),
    )
    caught = 0
    for mutate in mutators:
        candidate = copy.deepcopy(record)
        mutate(candidate)
        try:
            validate_record(candidate)
        except GateError:
            caught += 1
            continue
        raise GateError("mutation:record")
    reordered = copy.deepcopy(record)
    reordered["schema"] = reordered.pop("schema")
    try:
        validate_record(reordered)
    except GateError:
        caught += 1
    else:
        raise GateError("mutation:record_order")

    actor = (ROOT / SOURCE_BINDINGS[0][0]).read_text()
    engine = (ROOT / SOURCE_BINDINGS[1][0]).read_text()
    source_mutations = (
        (actor.replace("self.actor_sequence_decision_metered(candidate, &mut charge)?;", "Ok(())?;", 1), engine),
        (actor.replace("self.causal_next_decision_metered(candidate, &mut charge)?;", "Ok(())?;", 1), engine),
        (actor.replace("self.empty_frontier_decision_metered(candidate, base_frontier, charge)?;", "Ok(())?;", 1), engine),
        (actor, engine.replace(".candidate_semantics_decision_metered(", ".actor_sequence_decision_metered(", 1)),
    )
    source_caught = 0
    for changed_actor, changed_engine in source_mutations:
        try:
            validate_runtime_sources(changed_actor, changed_engine)
        except GateError:
            source_caught += 1
            continue
        raise GateError("mutation:source")
    return caught, source_caught


def main() -> None:
    record = json.loads(REPORT.read_text())
    validate_record(record)
    validate_sources()
    record_mutations, source_mutations = mutation_self_test(record)
    print("PASS: remediation v12 actor gate")
    print(f"- candidates={len(CANDIDATES)}")
    print(f"- proofs={len(PROOF_TESTS)}")
    print(f"- record_mutations={record_mutations}")
    print(f"- source_mutations={source_mutations}")
    print("- mapped_fixed=3 finding_100_remaining=7")


if __name__ == "__main__":
    main()
