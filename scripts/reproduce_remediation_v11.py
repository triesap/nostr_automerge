#!/usr/bin/env python3
"""Execute and validate the remediation-v11 finding reproductions."""

from __future__ import annotations

import argparse
import copy
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "spec/remediation_v11_reproductions.json"
FINDINGS = ("FINDING_096", "FINDING_097", "FINDING_098", "FINDING_099")
KINDS = ("rust_failure", "source_failure", "source_failure", "rust_failure")


class ReproductionError(AssertionError):
    pass


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ReproductionError(diagnostic)


def validate_inventory(value: object) -> list[dict[str, str]]:
    require(isinstance(value, dict) and tuple(value) == ("schema", "cases", "result"), "inventory:keys")
    require(value["schema"] == "nostr_automerge.remediation_v11_reproductions.v1" and value["result"] == "pass", "inventory:identity")
    rows = value["cases"]
    require(isinstance(rows, list) and len(rows) == 4, "inventory:count")
    required = ("finding", "kind", "path", "test", "diagnostic", "expected")
    for index, row in enumerate(rows):
        require(isinstance(row, dict) and tuple(row) == required, f"case:{index}:keys")
        require(row["finding"] == FINDINGS[index] and row["kind"] == KINDS[index], f"case:{index}:identity")
        require(row["expected"] in {"open_failure", "fixed_pass"}, f"case:{index}:expected")
        require(all(isinstance(row[key], str) and row[key] for key in required), f"case:{index}:values")
        require((ROOT / row["path"]).is_file(), f"case:{index}:path")
    require(len({row["test"] for row in rows}) == 4, "inventory:duplicate")
    return rows


def validate_failure_transcript(row: dict[str, str], result: subprocess.CompletedProcess[str]) -> None:
    require(result.returncode == 101, f"{row['finding']}:returncode")
    require(f"test {row['test']} ... FAILED" in result.stdout, f"{row['finding']}:test")
    require(row["diagnostic"] in result.stdout, f"{row['finding']}:diagnostic")
    require("test result: FAILED. 0 passed; 1 failed; 0 ignored;" in result.stdout, f"{row['finding']}:summary")
    require(f"failures:\n    {row['test']}\n" in result.stdout, f"{row['finding']}:failure_list")
    require("error: test failed, to rerun pass `-p nostr_automerge --lib`" in result.stderr, f"{row['finding']}:rerun")


def validate_pass_transcript(row: dict[str, str], result: subprocess.CompletedProcess[str]) -> None:
    require(result.returncode == 0, f"{row['finding']}:pass_returncode")
    require(f"test {row['test']} ... ok" in result.stdout, f"{row['finding']}:pass_test")
    require("test result: ok. 1 passed; 0 failed; 0 ignored;" in result.stdout, f"{row['finding']}:pass_summary")


def rust_case(row: dict[str, str]) -> bool:
    command = [
        "cargo", "extbuild", "run", "--", "cargo", "test", "-p",
        "nostr_automerge", "--lib", "--locked", "--",
    ]
    if row["expected"] == "open_failure":
        command.append("--ignored")
    command.extend(("--exact", row["test"]))
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
    if row["expected"] == "open_failure":
        validate_failure_transcript(row, result)
        return True
    validate_pass_transcript(row, result)
    return False


def source_reproduces(row: dict[str, str]) -> bool:
    source = (ROOT / row["path"]).read_text()
    if row["finding"] == "FINDING_097":
        anchors = (
            "let candidates = controls",
            "let raw_changes = controls",
            "let derived_heads = derive_heads(&accepted_changes, &controls);",
        )
        return all(anchor in source for anchor in anchors)
    if row["finding"] == "FINDING_098":
        contradiction = (
            "otherwise, a hash with only unsupported carriers is\n"
            "  `unsupported_revision`;"
        )
        api = (ROOT / "spec/API_CONTRACTS.md").read_text()
        return contradiction in source and "remains visible as an Event with `unsupported_revision`" in api and "does not create a semantic disposition" in api
    raise ReproductionError(f"{row['finding']}:source_kind")


def verify_state(rows: list[dict[str, str]]) -> tuple[int, int]:
    fixed = 0
    opened = 0
    for row in rows:
        reproduced = rust_case(row) if row["kind"] == "rust_failure" else source_reproduces(row)
        if row["expected"] == "open_failure":
            require(reproduced, f"{row['finding']}:not_reproduced")
            opened += 1
        else:
            require(not reproduced, f"{row['finding']}:still_reproduced")
            fixed += 1
    return fixed, opened


def mutation_self_test(value: object) -> int:
    mutations = []
    for mutate in (
        lambda item: item["cases"].pop(),
        lambda item: item["cases"].reverse(),
        lambda item: item["cases"][0].update(finding="FINDING_097"),
        lambda item: item["cases"][0].update(kind="source_failure"),
        lambda item: item["cases"][0].update(path="missing"),
        lambda item: item["cases"][1].update(test=item["cases"][0]["test"]),
        lambda item: item["cases"][0].update(expected="closed"),
        lambda item: item["cases"][0].update(diagnostic=""),
        lambda item: item.update(extra=False),
    ):
        candidate = copy.deepcopy(value)
        mutate(candidate)
        mutations.append(candidate)
    caught = 0
    for candidate in mutations:
        try:
            validate_inventory(candidate)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError("inventory mutation survived")

    row = validate_inventory(value)[0]
    stdout = (
        f"\nrunning 1 test\ntest {row['test']} ... FAILED\n\n"
        f"failures:\n\n{row['diagnostic']}\n\nfailures:\n    {row['test']}\n\n"
        "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out;\n"
    )
    stderr = "error: test failed, to rerun pass `-p nostr_automerge --lib`\n"
    base = subprocess.CompletedProcess([], 101, stdout, stderr)
    validate_failure_transcript(row, base)
    for changed in (
        subprocess.CompletedProcess([], 0, stdout, stderr),
        subprocess.CompletedProcess([], 101, stdout.replace(row["test"], "other", 1), stderr),
        subprocess.CompletedProcess([], 101, stdout.replace(row["diagnostic"], "other"), stderr),
        subprocess.CompletedProcess([], 101, stdout.replace("0 passed; 1 failed", "1 passed; 0 failed"), stderr),
        subprocess.CompletedProcess([], 101, stdout.replace(f"failures:\n    {row['test']}\n", "failures:\n"), stderr),
        subprocess.CompletedProcess([], 101, stdout, ""),
    ):
        try:
            validate_failure_transcript(row, changed)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError("transcript mutation survived")
    return caught


def main() -> None:
    parser = argparse.ArgumentParser()
    modes = parser.add_mutually_exclusive_group(required=True)
    modes.add_argument("--verify-state", action="store_true")
    modes.add_argument("--self-test-only", action="store_true")
    args = parser.parse_args()
    value = json.loads(INVENTORY.read_text())
    rows = validate_inventory(value)
    mutations = mutation_self_test(value)
    fixed, opened = verify_state(rows) if args.verify_state else (0, 0)
    print("PASS: remediation v11 reproduction harness")
    print(f"- fixed={fixed}")
    print(f"- open={opened}")
    print(f"- mutations={mutations}")


if __name__ == "__main__":
    main()
