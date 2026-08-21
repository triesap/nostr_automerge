#!/usr/bin/env python3
"""Reproduce remediation-v9 Rust defects without making default tests red."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CASES = (
    (
        "public_engine_api",
        "finding_073_checkpoint_authorization_precedes_history",
        "FINDING_073 reproduced: known-invalid checkpoint control is classified pending before authorization",
        "left == right",
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "Some(PendingControl)",
        "Some(Unauthorized)",
    ),
    (
        "public_engine_api",
        "finding_074_invalid_carrier_is_independent_of_excluded_hash",
        "FINDING_074 reproduced: known-invalid carrier inherits the excluded semantic-hash outcome",
        "left == right",
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "Some(Excluded)",
        "Some(Invalid)",
    ),
    (
        "lib",
        "engine::reference_evaluator::tests::finding_079_unsupported_carrier_does_not_create_semantic_hash_state",
        "FINDING_079 reproduced: an unverified unsupported carrier can create semantic ChangeHash state",
        "left != right",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "UnsupportedRevision",
        "UnsupportedRevision",
    ),
    (
        "public_engine_api",
        "finding_083_budget_stop_is_not_relabelled_by_cancellation_requery",
        "FINDING_083 reproduced: a typed budget stop is relabelled by a repeated cancellation observation",
        "left == right",
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "(Cancelled, 2)",
        "(BudgetExhausted, 1)",
    ),
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
    mode.add_argument("--expect-baseline-fail", action="store_true")
    mode.add_argument("--self-test-only", action="store_true")
    return parser.parse_args()


def command(target: str, test_name: str) -> tuple[str, ...]:
    selection = ("--lib",) if target == "lib" else ("--test", target)
    return (
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
        "--ignored",
        "--exact",
        test_name,
    )


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
    return caught


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
    for case in CASES:
        target, test_name, *_ = case
        result = subprocess.run(
            command(target, test_name),
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
    print(f"PASS: reproduced {len(CASES)} remediation-v9 Rust findings")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
