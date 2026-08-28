#!/usr/bin/env python3
"""Validate the exact causal-projection builder body with a Rust lexical view."""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
FUNCTION = "build_trusted_epoch_projection_observed"
MAXIMUM_SEQUENCE = """causal_next_op = perform_projection_build_operation(
            WorkCounter::GraphNode,
            ProjectionBuildOperation::CausalMaximumCompare,
            &mut charge,
            &mut built,
            || causal_next_op.max(advanced),
        )?;"""
FINAL_PUBLICATION = """charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
    published(ProjectionPublicationOperation::Projection);"""
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
    require(body.count("charge(WorkCounter::") == 1, "partial:single_raw_publication_charge")
    require(FINAL_PUBLICATION in body, "open:reviewed_final_publication")


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
            "    charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;",
            "    charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;\n"
            "    charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;",
        ),
        replace_in_function(
            source,
            FINAL_PUBLICATION,
            "charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;\n    "
            + FINAL_PUBLICATION.replace(
                "\n    charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;",
                "",
            ),
        ),
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


def main() -> int:
    source = SOURCE.read_text(encoding="utf-8")
    validate(source)
    mutations = mutation_self_test(source)
    print(
        "PASS: causal-projection lexical source audit "
        f"function={FUNCTION} mutations={mutations} status=partial_refactor"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
