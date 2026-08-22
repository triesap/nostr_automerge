#!/usr/bin/env python3
"""Validate and optionally execute the closed remediation-v9 report suite."""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import pathlib
import re
import subprocess
import sys
from collections.abc import Mapping, Sequence

sys.dont_write_bytecode = True

ROOT = pathlib.Path(__file__).resolve().parents[1]


class ReportSuiteError(Exception):
    """The report-contract proof inventory is stale or incomplete."""


@dataclasses.dataclass(frozen=True)
class ReportProof:
    clause: str
    target: str
    source: str
    test: str
    anchor: str


@dataclasses.dataclass(frozen=True)
class RustTest:
    attributes: tuple[str, ...]
    body: str
    body_code: str
    body_start: int
    declaration_start: int
    body_end: int


REPORT_PROOFS = (
    ReportProof(
        "revision_public_getter",
        "public_api",
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "evaluation_report_exposes_its_sealed_protocol_revision",
        "assert_eq!(report.revision(), revision);",
    ),
    ReportProof(
        "constructor_families_closed",
        "rust_lib",
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "engine::evaluation_report::tests::report_construction_inventory_is_closed_and_ordered",
        "assert_eq!(identifiers, [",
    ),
    ReportProof(
        "incomplete_empty_shape_and_digests",
        "rust_lib",
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "engine::evaluation_report::tests::incomplete_report_shape_rejects_every_nonempty_or_mismatched_field",
        "mutations.push((",
    ),
    ReportProof(
        "incomplete_typed_stop_compatibility",
        "rust_lib",
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "engine::evaluation_report::tests::budget_and_cancel_no_progress_reports_differ_only_by_typed_stop",
        "Some(super::EvaluationFailure::Cancelled)",
    ),
    ReportProof(
        "complete_partitions_controls_heads_document",
        "rust_lib",
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "engine::evaluation_report::tests::complete_report_rejects_every_partition_control_and_head_mutation",
        "mutation.document = None;",
    ),
    ReportProof(
        "carrier_event_change_hash_coverage_and_dominance",
        "rust_lib",
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "engine::evaluation_report::tests::complete_report_carrier_coverage_and_namespaces_are_exact",
        "assert_eq!(\n            complete_report(mutation, &forged_authority),",
    ),
    ReportProof(
        "digests_evidence_checkpoints_alerts_manifest_document",
        "rust_lib",
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "engine::evaluation_report::tests::complete_report_rejects_exact_field_and_coordinated_rewrite_mutations",
        "mutation.checkpoints[0] = changed_checkpoint;",
    ),
    ReportProof(
        "materialized_assertion_authority",
        "conformance",
        "tools/nostr_automerge_conformance/src/runner.rs",
        "runner::tests::signed_requirements_and_materialized_state_reject_assertion_mutations",
        "StateAssertionPolicy::None",
    ),
    ReportProof(
        "reevaluation_prior_only_alert_authority",
        "public_api",
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "late_lower_control_id_reorganizes_and_replays_signed_state",
        "assert_eq!(reorganization.affected_changes(), exact_affected);",
    ),
    ReportProof(
        "hybrid_incomplete_path_deleted",
        "rust_lib",
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "reference::evaluate::tests::finding_075_interrupted_batch_discards_all_canonical_progress",
        "assert_no_progress_batch(&interrupted, Completion::BudgetExhausted);",
    ),
    ReportProof(
        "incomplete_reevaluation_early_return",
        "rust_lib",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "engine::reference_evaluator::tests::finding_082_reevaluation_stops_before_post_incomplete_alert_work",
        "observations, [0; 5]",
    ),
    ReportProof(
        "charged_complete_reevaluation",
        "rust_lib",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "engine::reference_evaluator::tests::complete_reevaluation_has_exact_final_budget_and_cancellation_boundaries",
        "assert_eq!(\n            short_observations[4],",
    ),
    ReportProof(
        "charged_canonical_alert_comparisons",
        "rust_lib",
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "engine::evaluation_report::tests::canonical_alert_comparisons_are_interleaved_with_successful_charges",
        "assert_eq!(comparisons, comparison_count.saturating_sub(1));",
    ),
    ReportProof(
        "expected_loader_revision_and_ordering",
        "conformance",
        "tools/nostr_automerge_conformance/src/expected.rs",
        "expected::tests::parse_expected_canonical_report_schema",
        "wrong_revision.revision =",
    ),
    ReportProof(
        "canonical_serializer_compatibility",
        "conformance",
        "tools/nostr_automerge_conformance/src/report_json.rs",
        "report_json::tests::implement_canonical_report_json_writer",
        "assert_eq!(first, Ok(expected_bytes));",
    ),
    ReportProof(
        "fixture_loader_revision_compatibility",
        "conformance",
        "tools/nostr_automerge_conformance/src/fixture.rs",
        "fixture::tests::implement_fixture_metadata_loader",
        "fixture.revision =",
    ),
    ReportProof(
        "signed_scenario_loader_is_input_only",
        "conformance",
        "tools/nostr_automerge_conformance/src/scenario.rs",
        "scenario::tests::signed_scenario_v2_schema_rejects_protocol_truth_inputs",
        "for abstract_field in [",
    ),
    ReportProof(
        "consumer_compatibility_pipeline",
        "conformance",
        "tools/nostr_automerge_conformance/src/runner.rs",
        "runner::tests::report_contract_compatibility_consumers_are_exact",
        "assert_eq!(actual, Ok(expected.clone()));",
    ),
    ReportProof(
        "expected_independent_conformance_output",
        "conformance",
        "tools/nostr_automerge_conformance/src/runner.rs",
        "runner::tests::expected_report_values_never_drive_engine_output",
        "poisoned_signed.expected_report = poisoned_declaration.clone();",
    ),
    ReportProof(
        "signed_complete_field_families",
        "conformance",
        "tools/nostr_automerge_conformance/src/runner.rs",
        "runner::tests::signed_complete_report_field_families_pass_from_independent_inputs",
        "for relative in [",
    ),
    ReportProof(
        "public_no_progress_getter_surface",
        "public_api",
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "evaluation_errors_are_noncanonical",
        "assert_exact_no_progress_report(&cancelled);",
    ),
)

EXPECTED_CLAUSES = tuple(proof.clause for proof in REPORT_PROOFS)
EXPECTED_TARGETS = frozenset({"rust_lib", "public_api", "conformance"})
APPROVED_INVENTORY_SHA256 = "f911bcb863106be48017734dce12d398fa66794c73d3ca7d1d692d897d42b7ca"
PASS_RESULT = re.compile(
    r"^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; "
    r"\d+ filtered out; finished in [^\r\n]+$",
    re.MULTILINE,
)


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ReportSuiteError(diagnostic)


def inventory_digest(proofs: Sequence[ReportProof]) -> str:
    encoded = json.dumps(
        [dataclasses.asdict(proof) for proof in proofs],
        ensure_ascii=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def load_sources(proofs: Sequence[ReportProof]) -> dict[str, str]:
    sources: dict[str, str] = {}
    for relative in dict.fromkeys(proof.source for proof in proofs):
        try:
            sources[relative] = (ROOT / relative).read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as error:
            raise ReportSuiteError(f"source:{relative}") from error
    return sources


def blank_non_newlines(view: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if view[index] not in "\r\n":
            view[index] = " "


def raw_string_end(source: str, start: int) -> int | None:
    if start > 0 and (source[start - 1].isalnum() or source[start - 1] == "_"):
        return None
    for prefix in ("br", "rb", "cr", "rc", "r"):
        if not source.startswith(prefix, start):
            continue
        cursor = start + len(prefix)
        while cursor < len(source) and source[cursor] == "#":
            cursor += 1
        if cursor >= len(source) or source[cursor] != '"':
            continue
        terminator = '"' + "#" * (cursor - start - len(prefix))
        end = source.find(terminator, cursor + 1)
        require(end >= 0, "rust_lex:raw_string")
        return end + len(terminator)
    return None


def quoted_string_end(source: str, start: int) -> int:
    cursor = start + 1
    while cursor < len(source):
        if source[cursor] == "\\":
            cursor += 2
            continue
        if source[cursor] == '"':
            return cursor + 1
        cursor += 1
    raise ReportSuiteError("rust_lex:string")


def char_literal_end(source: str, start: int) -> int | None:
    cursor = start + 1
    if cursor >= len(source) or source[cursor] in "\r\n":
        return None
    if source[cursor] == "\\":
        cursor += 1
        if cursor >= len(source):
            return None
        if source[cursor] == "u" and cursor + 1 < len(source) and source[cursor + 1] == "{":
            closing = source.find("}", cursor + 2)
            if closing < 0:
                return None
            cursor = closing + 1
        elif source[cursor] == "x":
            cursor += 3
        else:
            cursor += 1
    else:
        cursor += 1
    if cursor < len(source) and source[cursor] == "'":
        return cursor + 1
    return None


def rust_code_view(source: str) -> str:
    view = list(source)
    cursor = 0
    while cursor < len(source):
        if source.startswith("//", cursor):
            end = source.find("\n", cursor + 2)
            if end < 0:
                end = len(source)
            blank_non_newlines(view, cursor, end)
            cursor = end
            continue
        if source.startswith("/*", cursor):
            depth = 1
            end = cursor + 2
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            require(depth == 0, "rust_lex:block_comment")
            blank_non_newlines(view, cursor, end)
            cursor = end
            continue
        raw_end = raw_string_end(source, cursor)
        if raw_end is not None:
            blank_non_newlines(view, cursor, raw_end)
            cursor = raw_end
            continue
        if source[cursor] == '"':
            end = quoted_string_end(source, cursor)
            blank_non_newlines(view, cursor, end)
            cursor = end
            continue
        if source[cursor] == "'":
            end = char_literal_end(source, cursor)
            if end is not None:
                blank_non_newlines(view, cursor, end)
                cursor = end
                continue
        cursor += 1
    return "".join(view)


def matching_open_bracket(code: str, end: int) -> int:
    depth = 0
    for cursor in range(end - 1, -1, -1):
        if code[cursor] == "]":
            depth += 1
        elif code[cursor] == "[":
            depth -= 1
            if depth == 0:
                return cursor
    raise ReportSuiteError("test_attribute:bracket")


def attached_attributes(source: str, code: str, declaration_start: int) -> tuple[str, ...]:
    attributes: list[str] = []
    cursor = declaration_start
    while True:
        while cursor > 0 and code[cursor - 1].isspace():
            cursor -= 1
        if cursor == 0 or code[cursor - 1] != "]":
            break
        opening = matching_open_bracket(code, cursor)
        require(opening > 0 and code[opening - 1] == "#", "test_attribute:outer")
        attributes.append(code[opening - 1 : cursor])
        cursor = opening - 1
    attributes.reverse()
    return tuple(attributes)


def extract_rust_test(source: str, proof: ReportProof) -> RustTest:
    name = proof.test.rsplit("::", 1)[-1]
    code = rust_code_view(source)
    declaration = re.compile(
        rf"(?m)^[ \t]*fn[ \t]+{re.escape(name)}[ \t]*"
        rf"\([ \t\r\n]*\)[ \t\r\n]*\{{"
    )
    matches = tuple(declaration.finditer(code))
    require(len(matches) == 1, f"test_declaration:{proof.clause}")
    match = matches[0]
    attributes = attached_attributes(source, code, match.start())
    normalized = tuple(re.sub(r"\s+", "", attribute[2:-1]) for attribute in attributes)
    require(normalized.count("test") == 1, f"test_attribute:{proof.clause}")
    require(
        all(re.search(r"\bignore\b", attribute) is None for attribute in attributes),
        f"test_ignored:{proof.clause}",
    )
    opening = match.end() - 1
    depth = 0
    closing = -1
    for cursor in range(opening, len(code)):
        if code[cursor] == "{":
            depth += 1
        elif code[cursor] == "}":
            depth -= 1
            if depth == 0:
                closing = cursor
                break
    require(closing >= 0, f"test_body:{proof.clause}")
    return RustTest(
        attributes=attributes,
        body=source[opening + 1 : closing],
        body_code=code[opening + 1 : closing],
        body_start=opening + 1,
        declaration_start=match.start(),
        body_end=closing + 1,
    )


def anchor_is_executable(test: RustTest, anchor: str) -> bool:
    cursor = 0
    while True:
        occurrence = test.body.find(anchor, cursor)
        if occurrence < 0:
            return False
        span = test.body_code[occurrence : occurrence + len(anchor)]
        if all(
            character.isspace() or span[index] == character
            for index, character in enumerate(anchor)
        ):
            return True
        cursor = occurrence + 1


def validate_inventory(
    proofs: Sequence[ReportProof],
    sources: Mapping[str, str],
    *,
    expected_digest: str = APPROVED_INVENTORY_SHA256,
) -> None:
    require(tuple(proof.clause for proof in proofs) == EXPECTED_CLAUSES, "clauses")
    require(len({proof.clause for proof in proofs}) == len(proofs), "clause_unique")
    require(len({proof.test for proof in proofs}) == len(proofs), "test_unique")
    require(all(proof.target in EXPECTED_TARGETS for proof in proofs), "target")
    require(inventory_digest(proofs) == expected_digest, "inventory_identity")
    require(
        set(sources) == {proof.source for proof in proofs},
        "source_inventory",
    )
    for proof in proofs:
        test = extract_rust_test(sources[proof.source], proof)
        require(anchor_is_executable(test, proof.anchor), f"behavior_anchor:{proof.clause}")


def mutation_self_test(sources: Mapping[str, str]) -> int:
    extra = dataclasses.replace(REPORT_PROOFS[-1], clause="extra_clause", test="extra_test")
    inventory_mutations = (
        REPORT_PROOFS[:-1],
        (*REPORT_PROOFS, extra),
        (*REPORT_PROOFS, REPORT_PROOFS[-1]),
        tuple(reversed(REPORT_PROOFS)),
        (
            dataclasses.replace(REPORT_PROOFS[0], anchor="stale revision anchor"),
            *REPORT_PROOFS[1:],
        ),
        (
            dataclasses.replace(REPORT_PROOFS[0], test="stale_test_name"),
            *REPORT_PROOFS[1:],
        ),
    )
    caught = 0
    for mutation in inventory_mutations:
        try:
            validate_inventory(mutation, sources)
        except ReportSuiteError:
            caught += 1
            continue
        raise ReportSuiteError("inventory_mutation_survived")

    proof = REPORT_PROOFS[0]
    name = proof.test.rsplit("::", 1)[-1]
    marker = f"fn {name}()"
    source = sources[proof.source]
    test = extract_rust_test(source, proof)
    indentation_end = source.find("fn", test.declaration_start)
    require(indentation_end >= 0, "self_test_indentation")
    indentation = source[test.declaration_start:indentation_end]

    source_mutations: list[dict[str, str]] = []
    for attribute_value in (
        '#[ignore = "stale report proof"]',
        "#[proof::ignore]",
        "#[cfg_attr(all(), ignore)]",
        "#[cfg_attr(all(), proof::ignore)]",
    ):
        mutation = dict(sources)
        mutation[proof.source] = (
            source[: test.declaration_start]
            + indentation
            + attribute_value
            + "\n"
            + source[test.declaration_start :]
        )
        source_mutations.append(mutation)

    missing_test_attribute = dict(sources)
    attribute = source.rfind("#[test]", 0, source.index(marker))
    require(attribute >= 0, "self_test_attribute")
    missing_test_attribute[proof.source] = (
        missing_test_attribute[proof.source][:attribute]
        + missing_test_attribute[proof.source][attribute + len("#[test]\n") :]
    )
    source_mutations.append(missing_test_attribute)

    anchor_offset = test.body.find(proof.anchor)
    require(anchor_offset >= 0, "self_test_behavior_anchor")
    anchor_start = test.body_start + anchor_offset
    missing_anchor = dict(sources)
    missing_anchor[proof.source] = (
        source[:anchor_start] + source[anchor_start + len(proof.anchor) :]
    )
    source_mutations.append(missing_anchor)

    commented_anchor = dict(sources)
    commented_anchor[proof.source] = (
        source[:anchor_start] + "// " + source[anchor_start:]
    )
    source_mutations.append(commented_anchor)

    string_anchor = dict(sources)
    string_anchor[proof.source] = (
        source[:anchor_start]
        + f'let _proof_anchor = r####"{proof.anchor}"####;'
        + source[anchor_start + len(proof.anchor) :]
    )
    source_mutations.append(string_anchor)

    between_tests = dict(sources)
    without_anchor = missing_anchor[proof.source]
    adjusted_body_end = test.body_end - len(proof.anchor)
    between_tests[proof.source] = (
        without_anchor[:adjusted_body_end]
        + "\n"
        + proof.anchor
        + "\n"
        + without_anchor[adjusted_body_end:]
    )
    source_mutations.append(between_tests)

    missing_source = dict(sources)
    missing_source.pop(proof.source)
    source_mutations.append(missing_source)
    for mutation in source_mutations:
        try:
            validate_inventory(REPORT_PROOFS, mutation)
        except ReportSuiteError:
            caught += 1
            continue
        raise ReportSuiteError("source_mutation_survived")

    for split_name, split_anchor, split_source in (
        (
            "split_string_anchor",
            'let proof = "value";',
            '#[test]\nfn split_string_anchor() { let proof = "value"; }\n',
        ),
        (
            "split_comment_anchor",
            "assert!(true); // proof",
            "#[test]\nfn split_comment_anchor() { assert!(true); // proof\n}\n",
        ),
    ):
        split_proof = dataclasses.replace(
            proof,
            test=split_name,
            anchor=split_anchor,
        )
        split_test = extract_rust_test(split_source, split_proof)
        require(
            not anchor_is_executable(split_test, split_anchor),
            "split_anchor_mutation_survived",
        )
        caught += 1

    for prefix in ("identifierr", "identifierbr"):
        boundary_probe = (
            f'{prefix}#"masked" + SHOULD_REMAIN_CODE + "#; "tail'
        )
        require(
            "SHOULD_REMAIN_CODE" in rust_code_view(boundary_probe),
            "rust_lex:raw_string_boundary",
        )
        caught += 1
    return caught


def test_command(proof: ReportProof) -> list[str]:
    command = ["cargo", "extbuild", "run", "--", "cargo", "test"]
    if proof.target == "conformance":
        command.extend(("-p", "nostr_automerge_conformance", "--bin", "nostr_automerge_conformance"))
    elif proof.target == "public_api":
        command.extend(("-p", "nostr_automerge", "--test", "public_engine_api"))
    else:
        command.extend(("-p", "nostr_automerge", "--lib"))
    command.extend(("--locked", proof.test, "--", "--exact"))
    return command


def validate_test_transcript(
    proof: ReportProof,
    command: Sequence[str],
    returncode: int,
    stdout: str,
    stderr: str,
) -> None:
    require(tuple(command) == tuple(test_command(proof)), f"test_command:{proof.clause}")
    require(returncode == 0, f"test_exit:{proof.clause}")
    lines = stdout.splitlines()
    expected_line = f"test {proof.test} ... ok"
    test_lines = tuple(
        line for line in lines if line.startswith("test ") and not line.startswith("test result:")
    )
    require(lines.count("running 1 test") == 1, f"test_count:{proof.clause}")
    require(test_lines == (expected_line,), f"test_identity:{proof.clause}")
    require(len(PASS_RESULT.findall(stdout)) == 1, f"test_result:{proof.clause}")
    require("test result:" not in stderr, f"test_result_stream:{proof.clause}")


def transcript_mutation_self_test() -> int:
    proof = REPORT_PROOFS[0]
    command = test_command(proof)
    expected_line = f"test {proof.test} ... ok"
    transcript = (
        "running 1 test\n"
        f"{expected_line}\n\n"
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; "
        "0 filtered out; finished in 0.01s\n"
    )
    validate_test_transcript(proof, command, 0, transcript, "")

    wrong_name = transcript.replace(proof.test, "wrong::test", 1)
    substring_name = transcript.replace(proof.test, f"{proof.test}_stale", 1)
    ignored = transcript.replace(
        f"{expected_line}\n\n"
        "test result: ok. 1 passed; 0 failed; 0 ignored;",
        f"test {proof.test} ... ignored\n\n"
        "test result: ok. 0 passed; 0 failed; 1 ignored;",
    )
    zero = transcript.replace("running 1 test", "running 0 tests").replace(
        f"{expected_line}\n", ""
    )
    two = transcript.replace("running 1 test", "running 2 tests").replace(
        f"{expected_line}\n",
        f"{expected_line}\ntest stale::second ... ok\n",
    ).replace("1 passed", "2 passed")
    failed = transcript.replace(f"{proof.test} ... ok", f"{proof.test} ... FAILED").replace(
        "test result: ok. 1 passed; 0 failed;",
        "test result: FAILED. 0 passed; 1 failed;",
    )
    duplicate = transcript.replace(f"{expected_line}\n", f"{expected_line}\n{expected_line}\n")
    stale_target = list(command)
    stale_target[stale_target.index("nostr_automerge")] = "nostr_automerge_stale"
    stale_tool = list(command)
    stale_tool[0] = "rustc"
    mutations = (
        (command, 0, wrong_name, ""),
        (command, 0, substring_name, ""),
        (command, 0, ignored, ""),
        (command, 0, zero, ""),
        (command, 0, two, ""),
        (command, 101, failed, ""),
        (command, 0, duplicate, ""),
        (stale_target, 0, transcript, ""),
        (stale_tool, 0, transcript, ""),
        (command, 0, "", "Finished test profile\n"),
    )
    caught = 0
    for mutated_command, returncode, stdout, stderr in mutations:
        try:
            validate_test_transcript(
                proof,
                mutated_command,
                returncode,
                stdout,
                stderr,
            )
        except ReportSuiteError:
            caught += 1
            continue
        raise ReportSuiteError("transcript_mutation_survived")
    return caught


def run_suite(proofs: Sequence[ReportProof]) -> int:
    executed = 0
    for proof in proofs:
        command = test_command(proof)
        result = subprocess.run(
            command,
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        try:
            validate_test_transcript(
                proof,
                command,
                result.returncode,
                result.stdout,
                result.stderr,
            )
        except ReportSuiteError as error:
            raise ReportSuiteError(
                f"{error}\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}"
            ) from error
        executed += 1
    return executed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-suite", action="store_true")
    arguments = parser.parse_args()
    sources = load_sources(REPORT_PROOFS)
    validate_inventory(REPORT_PROOFS, sources)
    mutations = mutation_self_test(sources)
    transcript_mutations = transcript_mutation_self_test()
    executed = run_suite(REPORT_PROOFS) if arguments.run_suite else 0
    print("PASS: remediation-v9 report contract suite")
    print(f"- clauses={len(REPORT_PROOFS)}")
    print(f"- source_files={len(sources)}")
    print(f"- negative_mutations={mutations}")
    print(f"- transcript_negative_mutations={transcript_mutations}")
    print(f"- inventory_sha256={APPROVED_INVENTORY_SHA256}")
    print(f"- executed={executed}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ReportSuiteError as error:
        raise SystemExit(f"FAIL: {error}") from error
