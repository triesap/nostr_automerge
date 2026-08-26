#!/usr/bin/env python3
"""Execute the two intentionally open resource follow-up reproductions."""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "spec/resource_operation_inventory_v10.json"


class ReproductionError(AssertionError):
    pass


def validate_transcript(
    *, test: str, diagnostic: str, returncode: int, stdout: str, stderr: str
) -> None:
    if returncode != 101:
        raise ReproductionError(f"{test}:returncode:{returncode}")
    if f"test {test} ... FAILED" not in stdout:
        raise ReproductionError(f"{test}:exact_test")
    if diagnostic not in stdout:
        raise ReproductionError(f"{test}:diagnostic")
    if "test result: FAILED. 0 passed; 1 failed; 0 ignored;" not in stdout:
        raise ReproductionError(f"{test}:summary")
    if f"failures:\n    {test}\n" not in stdout:
        raise ReproductionError(f"{test}:failure_list")
    if "error: test failed, to rerun pass `-p nostr_automerge --lib`" not in stderr:
        raise ReproductionError(f"{test}:rerun")


def mutation_self_test() -> int:
    test = "module::open_case"
    diagnostic = "OPEN reproduced"
    stdout = (
        f"\nrunning 1 test\ntest {test} ... FAILED\n\n"
        f"failures:\n\n---- {test} stdout ----\n{diagnostic}\n\n"
        f"failures:\n    {test}\n\n"
        "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out;\n"
    )
    stderr = "error: test failed, to rerun pass `-p nostr_automerge --lib`\n"
    validate_transcript(
        test=test,
        diagnostic=diagnostic,
        returncode=101,
        stdout=stdout,
        stderr=stderr,
    )
    mutations = (
        (0, stdout, stderr),
        (101, stdout.replace(test, "module::other_case", 1), stderr),
        (101, stdout.replace(diagnostic, "other"), stderr),
        (101, stdout.replace("0 passed; 1 failed", "1 passed; 0 failed"), stderr),
        (101, stdout.replace(f"failures:\n    {test}\n", "failures:\n"), stderr),
        (101, stdout, ""),
    )
    for index, (returncode, changed_stdout, changed_stderr) in enumerate(mutations):
        try:
            validate_transcript(
                test=test,
                diagnostic=diagnostic,
                returncode=returncode,
                stdout=changed_stdout,
                stderr=changed_stderr,
            )
        except ReproductionError:
            continue
        raise ReproductionError(f"mutation:{index}")
    return len(mutations)


def verify_open() -> int:
    data = json.loads(INVENTORY.read_text())
    reproductions = data.get("reproductions")
    if not isinstance(reproductions, list) or len(reproductions) != 2:
        raise ReproductionError("inventory:reproductions")
    for reproduction in reproductions:
        test = reproduction["test"]
        command = (
            "cargo", "extbuild", "run", "--", "cargo", "test",
            "-p", "nostr_automerge", "--lib", "--locked", "--",
            "--ignored", "--exact", test,
        )
        result = subprocess.run(
            command,
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        validate_transcript(
            test=test,
            diagnostic=reproduction["diagnostic"],
            returncode=result.returncode,
            stdout=result.stdout,
            stderr=result.stderr,
        )
    return len(reproductions)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify-open", action="store_true")
    parser.add_argument("--self-test-only", action="store_true")
    args = parser.parse_args()
    if args.verify_open == args.self_test_only:
        parser.error("select exactly one mode")
    mutations = mutation_self_test()
    reproduced = verify_open() if args.verify_open else 0
    print("PASS: resource follow-up reproduction harness")
    print(f"- open={reproduced}")
    print(f"- mutations={mutations}")


if __name__ == "__main__":
    main()
