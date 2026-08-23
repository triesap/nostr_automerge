#!/usr/bin/env python3
"""Verify fixed regressions and reproduce still-open remediation-v9 Rust defects."""

from __future__ import annotations

import argparse
import copy
import json
import os
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXED_CASES = (
    (
        "public_engine_api",
        "finding_073_checkpoint_authorization_precedes_history",
    ),
    (
        "public_engine_api",
        "finding_074_invalid_carrier_is_independent_of_excluded_hash",
    ),
    (
        "lib",
        "reference::evaluate::tests::finding_075_interrupted_batch_discards_all_canonical_progress",
    ),
    (
        "public_engine_api",
        "finding_079_unsupported_carrier_does_not_create_semantic_hash_state",
    ),
    (
        "public_engine_api",
        "finding_083_budget_stop_is_not_relabelled_by_cancellation_requery",
    ),
    (
        "lib",
        "engine::evaluation_report::tests::finding_081_incomplete_report_rejects_canonical_cross_view_state",
    ),
    (
        "lib",
        "engine::reference_evaluator::tests::finding_082_reevaluation_stops_before_post_incomplete_alert_work",
    ),
    (
        "lib",
        "engine::reference_evaluator::tests::finding_076_finalization_rejects_reordered_named_passes",
    ),
)
OPEN_CASES = (
    (
        "lib",
        "reference::evaluate::tests::finding_077_canonical_raw_bytes_share_one_allocation",
        "FINDING_077 reproduced: canonical raw bytes are copied without byte accounting",
        "left == right",
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "(false, 0)",
        "(true, 0)",
    ),
    (
        "lib",
        "checkpoint::assemble::tests::finding_084_checkpoint_sort_stops_before_cancelled_work",
        "FINDING_084 reproduced: checkpoint chunks are sorted before cancellation is observed",
        "left == right",
        "crates/nostr_automerge/src/checkpoint/assemble.rs",
        "[0, 1]",
        "[1, 0]",
    ),
)
SEMANTIC_DIAGNOSTIC = (
    "FINDING_078 reproduced: a semantically unrelated named assertion "
    "passes requirement validation"
)
SEMANTIC_STDOUT = (
    f"{SEMANTIC_DIAGNOSTIC}\n"
    "observed=accepted:NCRDT-NIP01-002:"
    "invalid_raw_corpus_has_exact_stable_diagnostics\n"
    "desired=rejected:semantic-category-mismatch\n"
)
REVISION_MESSAGE = (
    "no method named `revision` found for reference `&EvaluationReport` in the current scope"
)
REVISION_LABEL = "method not found in `&EvaluationReport`"
REVISION_PACKAGE = "remediation_v9_report_revision_probe"
REVISION_VERSION = "0.0.0"
REVISION_SOURCE = "src/main.rs"
REVISION_ROOT = ROOT / "tests/compile_fail/remediation_v9_report_revision"
REVISION_MANIFEST = str(REVISION_ROOT / "Cargo.toml")
REVISION_TARGET_SOURCE = str(REVISION_ROOT / REVISION_SOURCE)
REVISION_PACKAGE_ID = (
    f"path+{REVISION_ROOT.as_uri()}#{REVISION_PACKAGE}@{REVISION_VERSION}"
)
STRUCTURED_FAILURE_MARKERS = (
    "could not compile",
    "could not execute process",
    "failed to run custom build command",
    "linker failure",
    "linker failed",
    "launcher failure",
    "process abort",
    "extbuild: error",
    "command not found",
)
TOOL_FAILURE_MARKERS = (
    "error: could not compile",
    "error: could not execute process",
    "failed to run custom build command",
    "error: linking with",
    "error: linker",
    "could not find `cargo.toml`",
    "no test target named",
    "unexpected argument",
    "extbuild: error",
    "extbuild doctor failed",
    "command not found",
    "launcher",
    "compiler",
    "process abort",
)


class ReproductionError(AssertionError):
    """The command output was not the exact expected libtest failure."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ReproductionError(diagnostic)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--verify-remediation-state", action="store_true")
    mode.add_argument(
        "--expect-baseline-fail",
        action="store_true",
        help="legacy spelling for --verify-remediation-state",
    )
    mode.add_argument("--self-test-only", action="store_true")
    return parser.parse_args()


def command(target: str, test_name: str, *, ignored: bool) -> tuple[str, ...]:
    selection = ("--lib",) if target == "lib" else ("--test", target)
    arguments = (
        "cargo",
        "extbuild",
        "run",
        "--",
        "cargo",
        "test",
        "-p",
        "nostr_automerge",
        *selection,
        "--locked",
        "--",
        "--exact",
        test_name,
    )
    if ignored:
        return (*arguments[:-2], "--ignored", *arguments[-2:])
    return arguments


def expected_stdout_pattern(
    test_name: str,
    diagnostic: str,
    assertion_expression: str,
    panic_source: str,
    observed_left: str,
    expected_right: str,
) -> re.Pattern[str]:
    return re.compile(
        rf"\n"
        rf"running 1 test\n"
        rf"test {re.escape(test_name)} \.\.\. FAILED\n"
        rf"\n"
        rf"failures:\n"
        rf"\n"
        rf"---- {re.escape(test_name)} stdout ----\n"
        rf"\n"
        rf"thread '{re.escape(test_name)}' \([^\n]+\) panicked at "
        rf"{re.escape(panic_source)}:\d+:\d+:\n"
        rf"assertion `{re.escape(assertion_expression)}` failed: "
        rf"{re.escape(diagnostic)}\n"
        rf"  left: {re.escape(observed_left)}\n"
        rf" right: {re.escape(expected_right)}\n"
        rf"note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n"
        rf"\n\n"
        rf"failures:\n"
        rf"    {re.escape(test_name)}\n"
        rf"\n"
        rf"test result: FAILED\. 0 passed; 1 failed; 0 ignored; 0 measured; "
        rf"\d+ filtered out; finished in [^\s]+\n"
        rf"\n"
    )


def expected_stderr_pattern(target: str) -> re.Pattern[str]:
    runner = (
        r"Running unittests src/lib\.rs \([^\n]+\)"
        if target == "lib"
        else rf"Running tests/{re.escape(target)}\.rs \([^\n]+\)"
    )
    rerun = (
        "error: test failed, to rerun pass `-p nostr_automerge --lib`"
        if target == "lib"
        else "error: test failed, to rerun pass "
        f"`-p nostr_automerge --test {target}`"
    )
    progress = r"[ \t]*(?:Blocking|Checking|Compiling|Finished|Fresh|Waiting) [^\n]+\n"
    return re.compile(
        rf"(?:{progress})*"
        rf"[ \t]*{runner}\n"
        rf"{re.escape(rerun)}\n?"
    )


def expected_success_stdout_pattern(test_name: str) -> re.Pattern[str]:
    return re.compile(
        rf"\n"
        rf"running 1 test\n"
        rf"test {re.escape(test_name)} \.\.\. ok\n"
        rf"\n"
        rf"test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; "
        rf"\d+ filtered out; finished in [^\s]+\n"
        rf"\n"
    )


def expected_success_stderr_pattern(target: str) -> re.Pattern[str]:
    runner = (
        r"Running unittests src/lib\.rs \([^\n]+\)"
        if target == "lib"
        else rf"Running tests/{re.escape(target)}\.rs \([^\n]+\)"
    )
    progress = r"[ \t]*(?:Blocking|Checking|Compiling|Finished|Fresh|Waiting) [^\n]+\n"
    return re.compile(rf"(?:{progress})*[ \t]*{runner}\n?")


def semantic_command() -> tuple[str, ...]:
    return ("python3", "scripts/reproduce_requirement_matrix_v9_weakness.py")


def revision_command() -> tuple[str, ...]:
    return (
        "cargo",
        "extbuild",
        "run",
        "--",
        "cargo",
        "check",
        "--manifest-path",
        "tests/compile_fail/remediation_v9_report_revision/Cargo.toml",
        "--locked",
        "--offline",
        "--message-format=json",
    )


def validate_semantic_failure(returncode: int, stdout: str, stderr: str) -> None:
    output = stdout + stderr
    folded = output.casefold()
    require(returncode == 78, "wrong semantic-validator reproduction exit")
    require(stdout == SEMANTIC_STDOUT, "malformed semantic-validator reproduction stdout")
    require(stderr == "", "semantic-validator reproduction wrote stderr")
    require(
        not any(marker in folded for marker in TOOL_FAILURE_MARKERS),
        "tool or launcher failure in semantic-validator reproduction",
    )
    require(
        output.count(SEMANTIC_DIAGNOSTIC) == 1,
        "wrong semantic-validator diagnostic cardinality",
    )


TARGET_KEYS = {
    "crate_types",
    "doc",
    "doctest",
    "edition",
    "kind",
    "name",
    "src_path",
    "test",
}
PROFILE_KEYS = {
    "debug_assertions",
    "debuginfo",
    "opt_level",
    "overflow_checks",
    "test",
}


def strings(value: object) -> bool:
    return isinstance(value, list) and all(isinstance(item, str) for item in value)


def cargo_target_is_closed(value: object) -> bool:
    return (
        isinstance(value, dict)
        and set(value) == TARGET_KEYS
        and strings(value.get("crate_types"))
        and isinstance(value.get("doc"), bool)
        and isinstance(value.get("doctest"), bool)
        and isinstance(value.get("edition"), str)
        and strings(value.get("kind"))
        and isinstance(value.get("name"), str)
        and isinstance(value.get("src_path"), str)
        and isinstance(value.get("test"), bool)
    )


def cargo_profile_is_closed(value: object) -> bool:
    return (
        isinstance(value, dict)
        and set(value) == PROFILE_KEYS
        and isinstance(value.get("debug_assertions"), bool)
        and isinstance(value.get("debuginfo"), (str, int, type(None)))
        and isinstance(value.get("opt_level"), (str, int))
        and isinstance(value.get("overflow_checks"), bool)
        and isinstance(value.get("test"), bool)
    )


def compiler_artifact_is_closed(record: dict[str, object]) -> bool:
    return (
        set(record)
        == {
            "executable",
            "features",
            "filenames",
            "fresh",
            "manifest_path",
            "package_id",
            "profile",
            "reason",
            "target",
        }
        and record.get("reason") == "compiler-artifact"
        and isinstance(record.get("package_id"), str)
        and isinstance(record.get("manifest_path"), str)
        and cargo_target_is_closed(record.get("target"))
        and cargo_profile_is_closed(record.get("profile"))
        and strings(record.get("features"))
        and strings(record.get("filenames"))
        and isinstance(record.get("executable"), (str, type(None)))
        and isinstance(record.get("fresh"), bool)
    )


def build_script_record_is_closed(record: dict[str, object]) -> bool:
    environment = record.get("env")
    return (
        set(record)
        == {
            "cfgs",
            "env",
            "linked_libs",
            "linked_paths",
            "out_dir",
            "package_id",
            "reason",
        }
        and record.get("reason") == "build-script-executed"
        and isinstance(record.get("package_id"), str)
        and strings(record.get("linked_libs"))
        and strings(record.get("linked_paths"))
        and strings(record.get("cfgs"))
        and isinstance(environment, list)
        and all(
            isinstance(item, list)
            and len(item) == 2
            and all(isinstance(part, str) for part in item)
            for item in environment
        )
        and isinstance(record.get("out_dir"), str)
    )


def expected_revision_target() -> dict[str, object]:
    return {
        "kind": ["bin"],
        "crate_types": ["bin"],
        "name": REVISION_PACKAGE,
        "src_path": REVISION_TARGET_SOURCE,
        "edition": "2024",
        "doc": True,
        "doctest": False,
        "test": True,
    }


def compiler_message_record_is_exact(record: dict[str, object]) -> bool:
    return (
        set(record)
        == {"manifest_path", "message", "package_id", "reason", "target"}
        and record.get("reason") == "compiler-message"
        and record.get("package_id") == REVISION_PACKAGE_ID
        and record.get("manifest_path") == REVISION_MANIFEST
        and record.get("target") == expected_revision_target()
        and isinstance(record.get("message"), dict)
    )


def validate_revision_failure(returncode: int, stdout: str, stderr: str) -> None:
    require(returncode == 101, "wrong report-revision probe exit")
    require(
        re.fullmatch(
            r"(?:[ \t]*(?:Blocking|Checking|Compiling|Finished|Fresh|Waiting) [^\n]+\n)*"
            r"error: could not compile `remediation_v9_report_revision_probe` "
            r"\(bin \"remediation_v9_report_revision_probe\"\) due to 1 previous error\n?",
            stderr,
        )
        is not None,
        "malformed report-revision cargo stderr",
    )
    try:
        records = [json.loads(line) for line in stdout.splitlines()]
    except json.JSONDecodeError as error:
        raise ReproductionError("non-JSON report-revision cargo stdout") from error
    require(records, "empty report-revision cargo stdout")
    require(
        not any(
            marker
            in json.dumps(records, sort_keys=True, separators=(",", ":")).casefold()
            for marker in STRUCTURED_FAILURE_MARKERS
        ),
        "unrelated structured report-revision failure",
    )
    require(
        records[-1] == {"reason": "build-finished", "success": False},
        "report-revision build did not finish with one failure",
    )
    require(
        len(records) >= 3
        and all(isinstance(record, dict) for record in records)
        and all(
            compiler_artifact_is_closed(record)
            or build_script_record_is_closed(record)
            for record in records[:-3]
        )
        and compiler_message_record_is_exact(records[-3])
        and compiler_message_record_is_exact(records[-2]),
        "foreign or malformed report-revision Cargo record sequence",
    )
    error_record, note_record = records[-3:-1]
    error = error_record.get("message")
    note = note_record.get("message")
    require(isinstance(error, dict), "missing report-revision error")
    require(isinstance(note, dict), "missing report-revision failure note")
    require(
        set(error)
        == {"rendered", "$message_type", "children", "level", "message", "spans", "code"}
        and error.get("$message_type") == "diagnostic",
        "wrong report-revision error schema",
    )
    require(
        set(note)
        == {"rendered", "$message_type", "children", "level", "message", "spans", "code"}
        and note.get("$message_type") == "diagnostic",
        "wrong report-revision note schema",
    )
    require(error.get("level") == "error", "wrong report-revision error level")
    code = error.get("code")
    require(
        isinstance(code, dict)
        and set(code) == {"code", "explanation"}
        and code.get("code") == "E0599"
        and isinstance(code.get("explanation"), str),
        "wrong report-revision error code",
    )
    require(error.get("message") == REVISION_MESSAGE, "wrong report-revision message")
    require(error.get("children") == [], "unexpected report-revision child diagnostic")
    require(
        error.get("rendered")
        == "error[E0599]: no method named `revision` found for reference "
        "`&EvaluationReport` in the current scope\n"
        " --> src/main.rs:4:12\n"
        "  |\n"
        "4 |     report.revision()\n"
        "  |            ^^^^^^^^ method not found in `&EvaluationReport`\n\n",
        "wrong rendered report-revision diagnostic",
    )
    spans = error.get("spans")
    require(isinstance(spans, list) and len(spans) == 1, "wrong report-revision span count")
    span = spans[0]
    require(
        set(span)
        == {
            "byte_end",
            "byte_start",
            "column_end",
            "column_start",
            "expansion",
            "file_name",
            "is_primary",
            "label",
            "line_end",
            "line_start",
            "suggested_replacement",
            "suggestion_applicability",
            "text",
        }
        and span.get("file_name") == REVISION_SOURCE
        and span.get("line_start") == 4
        and span.get("line_end") == 4
        and span.get("column_start") == 12
        and span.get("column_end") == 20
        and span.get("byte_start") == 139
        and span.get("byte_end") == 147
        and span.get("is_primary") is True
        and span.get("expansion") is None
        and span.get("label") == REVISION_LABEL
        and span.get("suggested_replacement") is None
        and span.get("suggestion_applicability") is None,
        "wrong report-revision primary span",
    )
    require(
        span.get("text")
        == [
            {
                "highlight_end": 20,
                "highlight_start": 12,
                "text": "    report.revision()",
            }
        ],
        "wrong report-revision source line",
    )
    require(
        note.get("level") == "failure-note"
        and note.get("code") is None
        and note.get("children") == []
        and note.get("spans") == []
        and note.get("message")
        == "For more information about this error, try `rustc --explain E0599`.",
        "wrong report-revision failure note",
    )
    require(
        note.get("rendered")
        == "For more information about this error, try `rustc --explain E0599`.\n",
        "wrong rendered report-revision failure note",
    )


def revision_artifact_is_exact(record: dict[str, object]) -> bool:
    return (
        compiler_artifact_is_closed(record)
        and record.get("package_id") == REVISION_PACKAGE_ID
        and record.get("manifest_path") == REVISION_MANIFEST
        and record.get("target") == expected_revision_target()
        and record.get("features") == []
        and record.get("executable") is None
        and isinstance(record.get("filenames"), list)
        and len(record["filenames"]) == 1
    )


def validate_revision_success(returncode: int, stdout: str, stderr: str) -> None:
    require(returncode == 0, "wrong fixed report-revision probe exit")
    require(
        re.fullmatch(
            r"(?:[ \t]*(?:Blocking|Checking|Compiling|Finished|Fresh|Waiting) [^\n]+\n?)*",
            stderr,
        )
        is not None,
        "malformed fixed report-revision cargo stderr",
    )
    try:
        records = [json.loads(line) for line in stdout.splitlines()]
    except json.JSONDecodeError as error:
        raise ReproductionError("non-JSON fixed report-revision cargo stdout") from error
    require(records, "empty fixed report-revision cargo stdout")
    require(
        records[-1] == {"reason": "build-finished", "success": True},
        "fixed report-revision build did not finish successfully",
    )
    require(
        all(
            compiler_artifact_is_closed(record)
            or build_script_record_is_closed(record)
            for record in records[:-1]
        ),
        "foreign fixed report-revision Cargo record",
    )
    require(
        sum(revision_artifact_is_exact(record) for record in records[:-1]) == 1,
        "fixed report-revision artifact cardinality",
    )
    require(
        not any(
            marker
            in json.dumps(records, sort_keys=True, separators=(",", ":")).casefold()
            for marker in STRUCTURED_FAILURE_MARKERS
        ),
        "unrelated structured fixed report-revision failure",
    )


def validate_expected_failure(
    target: str,
    test_name: str,
    diagnostic: str,
    assertion_expression: str,
    panic_source: str,
    observed_left: str,
    expected_right: str,
    returncode: int,
    stdout: str,
    stderr: str,
) -> None:
    """Accept only the ordered stdout and stderr records for one exact failure."""

    require(returncode == 101, f"wrong cargo test exit for {test_name}")
    output = stdout + stderr
    folded = output.casefold()
    require(
        not any(marker in folded for marker in TOOL_FAILURE_MARKERS),
        f"tool, compiler, or launcher failure for {test_name}",
    )
    require(
        not any(line.lstrip().casefold().startswith("fatal:") for line in output.splitlines()),
        f"fatal tool or launcher failure for {test_name}",
    )
    require(
        expected_stdout_pattern(
            test_name,
            diagnostic,
            assertion_expression,
            panic_source,
            observed_left,
            expected_right,
        ).fullmatch(stdout)
        is not None,
        f"malformed or noncanonical libtest stdout for {test_name}",
    )
    require(
        expected_stderr_pattern(target).fullmatch(stderr) is not None,
        f"malformed or noncanonical cargo stderr for {test_name}",
    )
    require(output.count(diagnostic) == 1, f"wrong diagnostic cardinality for {test_name}")


def validate_expected_success(
    target: str,
    test_name: str,
    returncode: int,
    stdout: str,
    stderr: str,
) -> None:
    """Accept only one exact enabled passing regression and no stale ignore."""

    require(returncode == 0, f"wrong cargo test exit for fixed {test_name}")
    output = stdout + stderr
    folded = output.casefold()
    require(
        not any(marker in folded for marker in TOOL_FAILURE_MARKERS),
        f"tool, compiler, or launcher failure for fixed {test_name}",
    )
    require(
        expected_success_stdout_pattern(test_name).fullmatch(stdout) is not None,
        f"malformed, ignored, or noncanonical libtest stdout for fixed {test_name}",
    )
    require(
        expected_success_stderr_pattern(target).fullmatch(stderr) is not None,
        f"malformed cargo stderr for fixed {test_name}",
    )
    require(
        output.count(test_name) == 1,
        f"wrong fixed-test diagnostic cardinality for {test_name}",
    )


def canonical_self_test_output(
    test_name: str,
    diagnostic: str,
    assertion_expression: str,
    panic_source: str,
    observed_left: str,
    expected_right: str,
) -> tuple[str, str]:
    stdout = f"""
running 1 test
test {test_name} ... FAILED

failures:

---- {test_name} stdout ----

thread '{test_name}' (1234) panicked at {panic_source}:1:1:
assertion `{assertion_expression}` failed: {diagnostic}
  left: {observed_left}
 right: {expected_right}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    {test_name}

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

"""
    stderr = """    Finished `test` profile [unoptimized] target(s) in 0.01s
     Running tests/public_engine_api.rs (/tmp/public_engine_api-hash)
error: test failed, to rerun pass `-p nostr_automerge --test public_engine_api`
"""
    return stdout, stderr


def canonical_success_self_test_output(test_name: str) -> tuple[str, str]:
    stdout = f"""
running 1 test
test {test_name} ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

"""
    stderr = """    Finished `test` profile [unoptimized] target(s) in 0.01s
     Running tests/public_engine_api.rs (/tmp/public_engine_api-hash)
"""
    return stdout, stderr


def canonical_revision_self_test_output() -> tuple[str, str]:
    artifact = {
        "reason": "compiler-artifact",
        "package_id": "registry+https://github.com/rust-lang/crates.io-index#safe@1.0.0",
        "manifest_path": "/tmp/safe/Cargo.toml",
        "target": {
            "kind": ["lib"],
            "crate_types": ["lib"],
            "name": "safe",
            "src_path": "/tmp/safe/src/lib.rs",
            "edition": "2024",
            "doc": True,
            "doctest": True,
            "test": True,
        },
        "profile": {
            "opt_level": "0",
            "debuginfo": "line-tables-only",
            "debug_assertions": True,
            "overflow_checks": True,
            "test": False,
        },
        "features": [],
        "filenames": ["/tmp/libsafe.rmeta"],
        "executable": None,
        "fresh": True,
    }
    build_script = {
        "reason": "build-script-executed",
        "package_id": "registry+https://github.com/rust-lang/crates.io-index#safe-sys@1.0.0",
        "linked_libs": [],
        "linked_paths": [],
        "cfgs": [],
        "env": [],
        "out_dir": "/tmp/safe/out",
    }
    rendered = (
        "error[E0599]: no method named `revision` found for reference "
        "`&EvaluationReport` in the current scope\n"
        " --> src/main.rs:4:12\n"
        "  |\n"
        "4 |     report.revision()\n"
        "  |            ^^^^^^^^ method not found in `&EvaluationReport`\n\n"
    )
    error = {
        "reason": "compiler-message",
        "package_id": REVISION_PACKAGE_ID,
        "manifest_path": REVISION_MANIFEST,
        "target": expected_revision_target(),
        "message": {
            "rendered": rendered,
            "$message_type": "diagnostic",
            "children": [],
            "level": "error",
            "message": REVISION_MESSAGE,
            "spans": [
                {
                    "byte_end": 147,
                    "byte_start": 139,
                    "column_end": 20,
                    "column_start": 12,
                    "expansion": None,
                    "file_name": REVISION_SOURCE,
                    "is_primary": True,
                    "label": REVISION_LABEL,
                    "line_end": 4,
                    "line_start": 4,
                    "suggested_replacement": None,
                    "suggestion_applicability": None,
                    "text": [
                        {
                            "highlight_end": 20,
                            "highlight_start": 12,
                            "text": "    report.revision()",
                        }
                    ],
                }
            ],
            "code": {"code": "E0599", "explanation": "exact E0599 explanation"},
        },
    }
    note_text = "For more information about this error, try `rustc --explain E0599`."
    note = {
        "reason": "compiler-message",
        "package_id": REVISION_PACKAGE_ID,
        "manifest_path": REVISION_MANIFEST,
        "target": expected_revision_target(),
        "message": {
            "rendered": f"{note_text}\n",
            "$message_type": "diagnostic",
            "children": [],
            "level": "failure-note",
            "message": note_text,
            "spans": [],
            "code": None,
        },
    }
    finished = {"reason": "build-finished", "success": False}
    stdout = "".join(
        json.dumps(record, separators=(",", ":")) + "\n"
        for record in [artifact, build_script, error, note, finished]
    )
    stderr = (
        'error: could not compile `remediation_v9_report_revision_probe` '
        '(bin "remediation_v9_report_revision_probe") due to 1 previous error\n'
    )
    return stdout, stderr


def canonical_revision_success_self_test_output() -> tuple[str, str]:
    artifact = {
        "reason": "compiler-artifact",
        "package_id": REVISION_PACKAGE_ID,
        "manifest_path": REVISION_MANIFEST,
        "target": expected_revision_target(),
        "profile": {
            "opt_level": "0",
            "debuginfo": "line-tables-only",
            "debug_assertions": True,
            "overflow_checks": True,
            "test": False,
        },
        "features": [],
        "filenames": ["/tmp/libremediation_v9_report_revision_probe.rmeta"],
        "executable": None,
        "fresh": False,
    }
    finished = {"reason": "build-finished", "success": True}
    stdout = "".join(
        json.dumps(record, separators=(",", ":")) + "\n"
        for record in [artifact, finished]
    )
    return stdout, ""


def special_mutation_self_test() -> int:
    validate_semantic_failure(78, SEMANTIC_STDOUT, "")
    semantic_mutations = (
        ("semantic_success", 0, SEMANTIC_STDOUT, ""),
        ("semantic_wrong_exit", 77, SEMANTIC_STDOUT, ""),
        ("semantic_wrong_finding", 78, SEMANTIC_STDOUT.replace("FINDING_078", "FINDING_000"), ""),
        (
            "semantic_wrong_observed",
            78,
            SEMANTIC_STDOUT.replace("observed=accepted", "observed=rejected"),
            "",
        ),
        (
            "semantic_wrong_desired",
            78,
            SEMANTIC_STDOUT.replace("semantic-category-mismatch", "generic"),
            "",
        ),
        ("semantic_extra_stdout", 78, SEMANTIC_STDOUT + "extra\n", ""),
        ("semantic_stderr", 78, SEMANTIC_STDOUT, "warning\n"),
        ("semantic_tool_error", 78, "error: could not compile\n", ""),
    )
    caught = 0
    for name, returncode, stdout, stderr in semantic_mutations:
        try:
            validate_semantic_failure(returncode, stdout, stderr)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError(f"mutation survived: {name}")

    canonical_stdout, canonical_stderr = canonical_revision_self_test_output()
    validate_revision_failure(101, canonical_stdout, canonical_stderr)
    records = [json.loads(line) for line in canonical_stdout.splitlines()]

    def packed(mutated: list[dict[str, object]]) -> str:
        return "".join(
            json.dumps(record, separators=(",", ":")) + "\n"
            for record in mutated
        )

    revision_mutations: list[tuple[str, int, str, str]] = []
    wrong_code = copy.deepcopy(records)
    wrong_code[-3]["message"]["code"]["code"] = "E0000"
    revision_mutations.append(
        ("revision_wrong_code", 101, packed(wrong_code), canonical_stderr)
    )
    wrong_message = copy.deepcopy(records)
    wrong_message[-3]["message"]["message"] = "wrong"
    revision_mutations.append(
        ("revision_wrong_message", 101, packed(wrong_message), canonical_stderr)
    )
    wrong_rendered = copy.deepcopy(records)
    wrong_rendered[-3]["message"]["rendered"] += "forged\n"
    revision_mutations.append(
        ("revision_wrong_rendered", 101, packed(wrong_rendered), canonical_stderr)
    )
    wrong_span = copy.deepcopy(records)
    wrong_span[-3]["message"]["spans"][0]["line_start"] = 5
    revision_mutations.append(
        ("revision_wrong_span", 101, packed(wrong_span), canonical_stderr)
    )
    wrong_package = copy.deepcopy(records)
    wrong_package[-3]["package_id"] = "foreign"
    revision_mutations.append(
        ("revision_wrong_package", 101, packed(wrong_package), canonical_stderr)
    )
    child = copy.deepcopy(records)
    child[-3]["message"]["children"] = [{"level": "help"}]
    revision_mutations.append(
        ("revision_child", 101, packed(child), canonical_stderr)
    )
    missing_note = copy.deepcopy(records)
    missing_note.pop(-2)
    revision_mutations.append(
        ("revision_missing_note", 101, packed(missing_note), canonical_stderr)
    )
    duplicate_error = copy.deepcopy(records)
    duplicate_error.insert(-2, copy.deepcopy(duplicate_error[-3]))
    revision_mutations.append(
        ("revision_duplicate_error", 101, packed(duplicate_error), canonical_stderr)
    )
    build_success = copy.deepcopy(records)
    build_success[-1]["success"] = True
    revision_mutations.append(
        ("revision_build_success", 101, packed(build_success), canonical_stderr)
    )
    foreign_record = copy.deepcopy(records)
    foreign_record.insert(0, {"reason": "future-record"})
    revision_mutations.append(
        ("revision_foreign_record", 101, packed(foreign_record), canonical_stderr)
    )
    foreign_token = copy.deepcopy(records)
    foreign_token[-3]["package_id"] = (
        "path+file:///foreign/remediation_v9_report_revision_probe"
        f"#{REVISION_PACKAGE}@{REVISION_VERSION}"
    )
    revision_mutations.append(
        (
            "revision_foreign_package_containing_token",
            101,
            packed(foreign_token),
            canonical_stderr,
        )
    )
    schema_less_artifact = copy.deepcopy(records)
    schema_less_artifact.insert(0, {"reason": "compiler-artifact"})
    revision_mutations.append(
        (
            "revision_schema_less_compiler_artifact",
            101,
            packed(schema_less_artifact),
            canonical_stderr,
        )
    )
    artifact_linker_failure = copy.deepcopy(records)
    artifact_linker_failure[0]["target"]["name"] = "foreign linker failure"
    revision_mutations.append(
        (
            "revision_foreign_artifact_linker_failure",
            101,
            packed(artifact_linker_failure),
            canonical_stderr,
        )
    )
    build_script_linker_failure = copy.deepcopy(records)
    build_script_linker_failure[1]["env"].append(["STATUS", "linker failure"])
    revision_mutations.append(
        (
            "revision_foreign_build_script_linker_failure",
            101,
            packed(build_script_linker_failure),
            canonical_stderr,
        )
    )
    revision_mutations.extend(
        [
            ("revision_success_exit", 0, canonical_stdout, canonical_stderr),
            ("revision_non_json", 101, "not-json\n", canonical_stderr),
            (
                "revision_extra_stderr",
                101,
                canonical_stdout,
                canonical_stderr + "warning\n",
            ),
            ("revision_tool_error", 101, canonical_stdout, "error: linker failed\n"),
        ]
    )
    for name, returncode, stdout, stderr in revision_mutations:
        try:
            validate_revision_failure(returncode, stdout, stderr)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError(f"mutation survived: {name}")
    fixed_stdout, fixed_stderr = canonical_revision_success_self_test_output()
    validate_revision_success(0, fixed_stdout, fixed_stderr)
    fixed_records = [json.loads(line) for line in fixed_stdout.splitlines()]
    fixed_mutations: list[tuple[str, int, str, str]] = [
        ("fixed_revision_wrong_exit", 101, fixed_stdout, fixed_stderr),
        ("fixed_revision_non_json", 0, "not-json\n", fixed_stderr),
        ("fixed_revision_stderr", 0, fixed_stdout, "error: linker failed\n"),
    ]
    missing_artifact = fixed_records[1:]
    fixed_mutations.append(("fixed_revision_missing_artifact", 0, packed(missing_artifact), ""))
    failed_finish = copy.deepcopy(fixed_records)
    failed_finish[-1]["success"] = False
    fixed_mutations.append(("fixed_revision_failed_finish", 0, packed(failed_finish), ""))
    wrong_target = copy.deepcopy(fixed_records)
    wrong_target[0]["target"]["name"] = "foreign"
    fixed_mutations.append(("fixed_revision_wrong_target", 0, packed(wrong_target), ""))
    duplicate_artifact = copy.deepcopy(fixed_records)
    duplicate_artifact.insert(1, copy.deepcopy(duplicate_artifact[0]))
    fixed_mutations.append(("fixed_revision_duplicate_artifact", 0, packed(duplicate_artifact), ""))
    for name, returncode, stdout, stderr in fixed_mutations:
        try:
            validate_revision_success(returncode, stdout, stderr)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError(f"mutation survived: {name}")
    return caught


def mutation_self_test() -> int:
    target = "public_engine_api"
    test_name = "finding_000_harness_self_test"
    diagnostic = "FINDING_000 reproduced: harness self-test"
    assertion_expression = "left == right"
    panic_source = "crates/nostr_automerge/tests/public_engine_api.rs"
    observed_left = "Actual"
    expected_right = "Expected"
    canonical_stdout, canonical_stderr = canonical_self_test_output(
        test_name,
        diagnostic,
        assertion_expression,
        panic_source,
        observed_left,
        expected_right,
    )
    arguments = (
        target,
        test_name,
        diagnostic,
        assertion_expression,
        panic_source,
        observed_left,
        expected_right,
    )
    validate_expected_failure(*arguments, 101, canonical_stdout, canonical_stderr)
    boundary = "\n<<<STDERR>>>\n"
    canonical = canonical_stdout + boundary + canonical_stderr
    compile_error = (
        boundary
        + f"error: could not compile `nostr_automerge`\n{diagnostic}\n"
    )
    mutations = (
        ("compile_error", 101, compile_error),
        ("success", 0, canonical),
        ("wrong_test", 101, canonical.replace(test_name, "finding_999_wrong_test")),
        ("missing_panic_header", 101, canonical.replace(f"---- {test_name} stdout ----\n", "")),
        ("wrong_summary", 101, canonical.replace("0 passed; 1 failed", "0 passed; 2 failed")),
        ("wrong_returncode", 2, canonical),
        ("appended_launcher_failure", 101, canonical + f"fatal: launcher failed: {diagnostic}\n"),
        (
            "extra_wrong_target",
            101,
            canonical.replace(
                "running 1 test",
                "Running tests/wrong_target.rs (/tmp/wrong-target)\n\nrunning 1 test",
            ),
        ),
        ("forged_assertion", 101, canonical.replace("assertion `left == right`", "assertion `value`")),
        ("forged_source", 101, canonical.replace(panic_source, "crates/wrong.rs")),
        ("extra_status", 101, canonical.replace("failures:\n", "test other ... ok\n\nfailures:\n", 1)),
        ("multiple_summaries", 101, canonical.replace("error: test failed", "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s\n\nerror: test failed", 1)),
        ("extra_error", 101, canonical.replace("error: test failed", "error: unrelated tool failure\nerror: test failed", 1)),
        ("diagnostic_suffix", 101, canonical.replace(diagnostic, diagnostic + " TAMPERED", 1)),
        ("extra_thread", 101, canonical.replace(f"thread '{test_name}'", "thread 'other' (9) panicked at crates/wrong.rs:1:1:\nthread '" + test_name + "'", 1)),
        ("split_panic", 101, canonical.replace(f"thread '{test_name}' (1234)", "thread 'other' (1234)", 1)),
        ("duplicate_diagnostic", 101, canonical.replace(f"  left: {observed_left}", diagnostic + f"\n  left: {observed_left}", 1)),
        ("annotation_forgery", 101, canonical.replace(f"assertion `{assertion_expression}` failed: {diagnostic}", f"assertion `wrong` failed: wrong\nannotation: assertion `{assertion_expression}` failed: {diagnostic}", 1)),
        ("extra_failure_name", 101, canonical.replace(f"    {test_name}\n\ntest result:", f"    {test_name}\n    other\n\ntest result:", 1)),
        ("foreign_stdout_header", 101, canonical.replace(f"---- {test_name} stdout ----", f"---- other stdout ----\n\n---- {test_name} stdout ----", 1)),
        ("status_in_panic", 101, canonical.replace(f"test {test_name} ... FAILED\n\nfailures:", f"failures:\n\n---- {test_name} stdout ----\n\ntest {test_name} ... FAILED", 1)),
        ("missing_first_failures", 101, canonical.replace("\nfailures:\n\n----", "\n----", 1)),
        ("panic_before_status", 101, canonical.replace(f"test {test_name} ... FAILED\n\nfailures:\n\n---- {test_name} stdout ----", f"---- {test_name} stdout ----\n\ntest {test_name} ... FAILED\n\nfailures:", 1)),
        ("error_in_panic", 101, canonical.replace(f"  left: {observed_left}", "error: test failed, to rerun pass `-p nostr_automerge --test public_engine_api`\n" + f"  left: {observed_left}", 1)),
        ("trailing_abort", 101, canonical + "process abort after test\n"),
        ("observed_detail_tamper", 101, canonical.replace(f"  left: {observed_left}", "  left: Different", 1)),
    )
    caught = 0
    for name, returncode, packed in mutations:
        stdout, stderr = packed.split(boundary, 1)
        try:
            validate_expected_failure(*arguments, returncode, stdout, stderr)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError(f"mutation survived: {name}")
    fixed_name = "finding_000_fixed_harness_self_test"
    fixed_stdout, fixed_stderr = canonical_success_self_test_output(fixed_name)
    validate_expected_success(
        target,
        fixed_name,
        0,
        fixed_stdout,
        fixed_stderr,
    )
    fixed_boundary = "\n<<<FIXED_STDERR>>>\n"
    fixed_canonical = fixed_stdout + fixed_boundary + fixed_stderr
    fixed_mutations = (
        ("fixed_wrong_exit", 101, fixed_canonical),
        (
            "fixed_ignored",
            0,
            fixed_canonical.replace(
                f"test {fixed_name} ... ok",
                f"test {fixed_name} ... ignored",
            ).replace(
                "1 passed; 0 failed; 0 ignored",
                "0 passed; 0 failed; 1 ignored",
            ),
        ),
        ("fixed_wrong_test", 0, fixed_canonical.replace(fixed_name, "wrong_test")),
        ("fixed_zero_tests", 0, fixed_canonical.replace("running 1 test", "running 0 tests")),
        ("fixed_two_passed", 0, fixed_canonical.replace("1 passed", "2 passed")),
        ("fixed_extra_stdout", 0, "unexpected\n" + fixed_canonical),
        (
            "fixed_wrong_target",
            0,
            fixed_canonical.replace("tests/public_engine_api.rs", "tests/wrong.rs"),
        ),
        ("fixed_tool_error", 0, fixed_canonical + "error: linker failed\n"),
    )
    for name, returncode, packed in fixed_mutations:
        stdout, stderr = packed.split(fixed_boundary, 1)
        try:
            validate_expected_success(target, fixed_name, returncode, stdout, stderr)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError(f"mutation survived: {name}")
    return caught + special_mutation_self_test()


def main() -> int:
    args = parse_args()
    mutations = mutation_self_test()
    print(f"PASS: reproduction harness self-test ({mutations} negative mutations)")
    if args.self_test_only:
        return 0
    environment = os.environ.copy()
    environment.update(
        {
            "CARGO_TERM_COLOR": "never",
            "NO_COLOR": "1",
            "RUST_BACKTRACE": "0",
            "RUST_LIB_BACKTRACE": "0",
        }
    )
    for target, test_name in FIXED_CASES:
        result = subprocess.run(
            command(target, test_name, ignored=False),
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
            env=environment,
        )
        try:
            validate_expected_success(
                target,
                test_name,
                result.returncode,
                result.stdout,
                result.stderr,
            )
        except ReproductionError as error:
            raise ReproductionError(
                f"{error}:\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}"
            ) from error
        print(f"PASS: fixed regression {test_name}")
    for case in OPEN_CASES:
        target, test_name, *_ = case
        result = subprocess.run(
            command(target, test_name, ignored=True),
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
            env=environment,
        )
        try:
            validate_expected_failure(
                *case,
                result.returncode,
                result.stdout,
                result.stderr,
            )
        except ReproductionError as error:
            raise ReproductionError(
                f"{error}:\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}"
            ) from error
        print(f"PASS: reproduced {test_name}")
    semantic = subprocess.run(
        semantic_command(),
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
        env=environment,
    )
    try:
        validate_semantic_failure(
            semantic.returncode,
            semantic.stdout,
            semantic.stderr,
        )
    except ReproductionError as error:
        raise ReproductionError(
            f"{error}:\nSTDOUT:\n{semantic.stdout}\nSTDERR:\n{semantic.stderr}"
        ) from error
    print("PASS: reproduced semantic requirement-proof weakness")
    revision = subprocess.run(
        revision_command(),
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
        env=environment,
    )
    try:
        validate_revision_success(
            revision.returncode,
            revision.stdout,
            revision.stderr,
        )
    except ReproductionError as error:
        raise ReproductionError(
            f"{error}:\nSTDOUT:\n{revision.stdout}\nSTDERR:\n{revision.stderr}"
        ) from error
    print("PASS: fixed typed report revision API")
    print(f"PASS: verified {len(FIXED_CASES) + 1} fixed remediation-v9 Rust cases")
    print(f"PASS: reproduced {len(OPEN_CASES) + 1} still-open remediation-v9 Rust cases")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
