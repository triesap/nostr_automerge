#!/usr/bin/env python3
"""Validate the exact causal-projection builder body with a Rust lexical view."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
CONTROL_STATE = ROOT / "crates/nostr_automerge/src/control/epoch_state.rs"
EPOCH_ENGINE = ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
REFERENCE_EVALUATE = ROOT / "crates/nostr_automerge/src/reference/evaluate.rs"
HISTORICAL_CANDIDATE = "9cdd8665b68499c4975c08fd1fac07dd5eed999f"
FUNCTION = "build_trusted_epoch_projection_observed"
MAXIMUM_SEQUENCE = """causal_next_op = perform_projection_build_operation(
            WorkCounter::GraphNode,
            ProjectionBuildOperation::CausalMaximumCompare,
            &mut charge,
            &mut built,
            || causal_next_op.max(advanced),
        )?;"""
CONSTANT_VALIDATION_SEQUENCE = """let (member_count, input_is_canonical) = perform_projection_build_operation(
        WorkCounter::GraphNode,
        ProjectionBuildOperation::ConstantCandidateValidation,"""
RESULT_PUBLICATION_SEQUENCE = """let projection = perform_projection_build_operation(
        WorkCounter::GraphNode,
        ProjectionBuildOperation::ResultPublication,"""
FINAL_PUBLICATION = """published(ProjectionPublicationOperation::Projection);
    Ok(projection)"""
READY_SEQUENCE = """let has_ready = perform_projection_build_operation(
            WorkCounter::GraphNode,
            ProjectionBuildOperation::ReadinessTransition,"""

sys.path.insert(0, str(ROOT / "scripts"))
from validate_report_contract_v9 import ReportSuiteError, rust_code_view  # noqa: E402


class SourceAuditError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SourceAuditError(label)


def function_bounds(source: str, name: str = FUNCTION) -> tuple[str, int, int]:
    try:
        code = rust_code_view(source)
    except ReportSuiteError as error:
        raise SourceAuditError("lexical_view") from error
    declaration = re.compile(rf"(?m)^[ \t]*fn[ \t]+{re.escape(name)}(?:[ \t\r\n]*<[^{{;]+>)?[^{{;]*\{{")
    matches = tuple(declaration.finditer(code))
    require(len(matches) == 1, "function:cardinality")
    opening = matches[0].end() - 1
    depth = 0
    for cursor in range(opening, len(code)):
        if code[cursor] == "{":
            depth += 1
        elif code[cursor] == "}":
            depth -= 1
            if depth == 0:
                return code, opening + 1, cursor
    raise SourceAuditError("function:unclosed")


def function_body(source: str, name: str = FUNCTION) -> str:
    code, start, end = function_bounds(source, name)
    return code[start:end]


def validate(source: str) -> None:
    body = function_body(source)
    require("while !ready.is_empty()" not in body, "fixed:raw_ready_loop")
    require(READY_SEQUENCE in body, "partial:readiness_boundary")
    require("states\n        .values()" not in body, "fixed:final_state_scan")
    require(".map(|state| state.next_op)" not in body, "fixed:final_state_projection")
    require("let mut causal_next_op = 1_u64;" in body, "fixed:causal_accumulator")
    require(body.count(MAXIMUM_SEQUENCE) == 1, "fixed:causal_maximum_boundary")
    require("ProjectionBuildOperation" in body, "partial:operation_boundary")
    require("perform_projection_build_operation" in body, "partial:dispatch")
    require(body.count(CONSTANT_VALIDATION_SEQUENCE) == 1, "fixed:constant_validation")
    require(body.count(RESULT_PUBLICATION_SEQUENCE) == 1, "fixed:result_publication")
    require(body.count("charge(WorkCounter::") == 0, "fixed:no_raw_charge")
    require(FINAL_PUBLICATION in body, "fixed:reviewed_final_publication")
    require(body.count("|| TrustedEpochProjection {") == 1, "fixed:constructor_cardinality")
    require(
        "#[cfg(test)]\npub(crate) fn initialize_actor_states(" in source,
        "fixed:reference_oracle_isolation",
    )


def production_module(source: str, marker: str) -> str:
    require(source.count(marker) == 1, "call_graph:test_module")
    return source.split(marker, 1)[0]


def validate_call_graph(
    source: str,
    control_state: str,
    epoch_engine: str,
    reference_evaluate: str,
) -> None:
    actor_production = production_module(source, "#[cfg(test)]\npub(crate) mod tests")
    control_production = production_module(control_state, "#[cfg(test)]\nmod tests")
    epoch_production = production_module(epoch_engine, "#[cfg(test)]\nmod tests")
    evaluation_production = production_module(reference_evaluate, "#[cfg(test)]\nmod tests")
    require(
        actor_production.count("pub(crate) fn initialize_actor_states_metered") == 1
        and actor_production.count("fn build_trusted_epoch_projection<") == 1
        and actor_production.count("fn build_trusted_epoch_projection_observed<") == 1,
        "call_graph:constructors",
    )
    require(
        "#[cfg(test)]\nuse crate::graph::actor_state::initialize_actor_states;"
        in control_production
        and "#[cfg(test)]\n    pub(crate) fn new(" in control_production,
        "call_graph:accepted_state_oracle",
    )
    require(
        "impl EpochEvaluationResult {\n    #[cfg(test)]\n    pub(crate) fn new("
        in epoch_production,
        "call_graph:epoch_result_oracle",
    )
    require(
        control_production.count("initialize_actor_states_metered(") == 1
        and epoch_production.count("initialize_actor_states_metered(") == 1
        and evaluation_production.count("AcceptedEpochState::new_metered(") == 2,
        "call_graph:metered_constructors",
    )
    require(
        epoch_production.count(".candidate_semantics_decision_metered(") == 1,
        "call_graph:semantic_consumer",
    )


def call_graph_mutation_self_test(
    source: str,
    control_state: str,
    epoch_engine: str,
    reference_evaluate: str,
) -> int:
    mutations = (
        (source.replace("#[cfg(test)]\npub(crate) fn initialize_actor_states(", "pub(crate) fn initialize_actor_states(", 1), control_state, epoch_engine, reference_evaluate),
        (source, control_state.replace("#[cfg(test)]\n    pub(crate) fn new(", "    pub(crate) fn new(", 1), epoch_engine, reference_evaluate),
        (source, control_state, epoch_engine.replace("impl EpochEvaluationResult {\n    #[cfg(test)]", "impl EpochEvaluationResult {", 1), reference_evaluate),
        (source, control_state, epoch_engine.replace("initialize_actor_states_metered(", "initialize_actor_states(", 1), reference_evaluate),
        (source, control_state, epoch_engine.replace(".candidate_semantics_decision_metered(", ".candidate_semantics_decision_metered(\n                    &candidate, input.accepted_base().frontier_heads(), |counter| Ok(counter),\n                );\n                projection.candidate_semantics_decision_metered(", 1), reference_evaluate),
        (source, control_state, epoch_engine, reference_evaluate.replace("AcceptedEpochState::new_metered(", "AcceptedEpochState::new(", 1)),
    )
    for index, values in enumerate(mutations):
        try:
            validate(values[0])
            validate_call_graph(*values)
        except SourceAuditError:
            continue
        raise SourceAuditError(f"call_graph_mutation_survived:{index}")
    return len(mutations)


def replace_in_function(source: str, before: str, after: str) -> str:
    _, start, end = function_bounds(source)
    body = source[start:end]
    require(body.count(before) == 1, "mutation:function_anchor")
    return source[:start] + body.replace(before, after, 1) + source[end:]


def mutation_self_test(source: str) -> int:
    attacks = []
    attacks.append("// fn build_trusted_epoch_projection_observed() {}\n" + source)
    attacks.append('const DECOY: &str = r#"fn build_trusted_epoch_projection_observed() {}"#;\n' + source)
    attacks.append(source.replace("    loop {", "    #[cfg(test)]\n    loop {", 1))
    attacks.append(source + "\nfn nearby() { charge(WorkCounter::GraphNode); }\n")
    for index, attacked in enumerate(attacks):
        try:
            validate(attacked)
        except SourceAuditError as error:
            raise SourceAuditError(f"benign_attack_rejected:{index}:{error}") from error

    mutations = (
        source.replace("fn build_trusted_epoch_projection_observed", "fn stale_projection_builder", 1),
        replace_in_function(source, "let mut causal_next_op = 1_u64;", "let mut causal_next_op = 0_u64;"),
        replace_in_function(
            source,
            MAXIMUM_SEQUENCE,
            MAXIMUM_SEQUENCE.replace(".max(advanced)", ".min(advanced)"),
        ),
        replace_in_function(
            source,
            READY_SEQUENCE,
            READY_SEQUENCE.replace(
                "ProjectionBuildOperation::ReadinessTransition",
                "ProjectionBuildOperation::StateLookup",
            ),
        ),
        replace_in_function(source, "    loop {", "    while !ready.is_empty() {"),
        replace_in_function(
            source,
            CONSTANT_VALIDATION_SEQUENCE,
            CONSTANT_VALIDATION_SEQUENCE.replace(
                "ProjectionBuildOperation::ConstantCandidateValidation",
                "ProjectionBuildOperation::StateLookup",
            ),
        ),
        replace_in_function(
            source,
            RESULT_PUBLICATION_SEQUENCE,
            RESULT_PUBLICATION_SEQUENCE.replace(
                "ProjectionBuildOperation::ResultPublication",
                "ProjectionBuildOperation::MapInsertion",
            ),
        ),
        replace_in_function(
            source,
            FINAL_PUBLICATION,
            "charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;\n    "
            + FINAL_PUBLICATION,
        ),
        source.replace(
            "#[cfg(test)]\npub(crate) fn initialize_actor_states(",
            "pub(crate) fn initialize_actor_states(",
            1,
        ),
        source.replace("|| TrustedEpochProjection {", "|| make_projection {", 1),
        source.replace("perform_projection_build_operation", "unsealed_projection_operation"),
    )
    for index, mutation in enumerate(mutations):
        require(mutation != source, f"mutation:{index}:applied")
        try:
            validate(mutation)
        except SourceAuditError:
            continue
        raise SourceAuditError(f"mutation_survived:{index}")
    return len(attacks) + len(mutations)


def historical_text(path: Path) -> str:
    relative = path.relative_to(ROOT).as_posix()
    completed = subprocess.run(
        ["git", "show", f"{HISTORICAL_CANDIDATE}:{relative}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(completed.returncode == 0, "historical:" + relative)
    return completed.stdout


def main() -> int:
    source = historical_text(SOURCE)
    control_state = historical_text(CONTROL_STATE)
    epoch_engine = historical_text(EPOCH_ENGINE)
    reference_evaluate = historical_text(REFERENCE_EVALUATE)
    validate(source)
    validate_call_graph(source, control_state, epoch_engine, reference_evaluate)
    mutations = mutation_self_test(source) + call_graph_mutation_self_test(
        source, control_state, epoch_engine, reference_evaluate
    )
    print(
        "PASS: causal-projection lexical source audit "
        f"function={FUNCTION} mutations={mutations} status=complete_refactor"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
