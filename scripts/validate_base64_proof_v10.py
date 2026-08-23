#!/usr/bin/env python3
"""Validate and optionally execute the exact NCRDT-B64-001 proof lane."""

from __future__ import annotations

import argparse
import dataclasses
import subprocess
import sys
from collections.abc import Mapping, Sequence
from typing import Callable

sys.dont_write_bytecode = True

from validate_report_contract_v9 import (
    PASS_RESULT,
    ROOT,
    ReportProof,
    ReportSuiteError,
    anchor_is_executable,
    extract_rust_test,
    inventory_digest,
    load_sources,
    require,
)


PROOFS = (
    ReportProof(
        "NCRDT-B64-001:canonical_vectors",
        "rust_lib",
        "crates/nostr_automerge/src/wire/base64.rs",
        "wire::base64::tests::accepts_only_standard_padded_canonical_form",
        "for invalid in [",
    ),
    ReportProof(
        "NCRDT-B64-001:encoded_decoded_boundaries",
        "rust_lib",
        "crates/nostr_automerge/src/wire/base64.rs",
        "wire::base64::tests::enforces_encoded_and_decoded_boundaries_exactly",
        "let oversized_canonical_shape =",
    ),
    ReportProof(
        "NCRDT-B64-001:signed_event_vectors",
        "public_api",
        "crates/nostr_automerge/tests/base64_contract.rs",
        "signed_change_events_reject_every_noncanonical_base64_class",
        "for (offset, invalid_content) in [",
    ),
)
EXPECTED_CLAUSES = tuple(proof.clause for proof in PROOFS)
EXPECTED_TESTS = tuple(proof.test for proof in PROOFS)
APPROVED_INVENTORY_SHA256 = "99fbf64a034f31b7ce2f85998a1d24f8334d796c34cf6504460652d6fb6bdf87"


def test_command(proof: ReportProof) -> list[str]:
    command = [
        "cargo", "extbuild", "run", "--", "cargo", "test", "-p", "nostr_automerge"
    ]
    if proof.source.endswith("base64_contract.rs"):
        command.extend(("--test", "base64_contract"))
    else:
        command.append("--lib")
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


def run_suite(proofs: Sequence[ReportProof]) -> int:
    for proof in proofs:
        command = test_command(proof)
        result = subprocess.run(
            command, cwd=ROOT, check=False, capture_output=True, text=True
        )
        validate_test_transcript(
            proof, command, result.returncode, result.stdout, result.stderr
        )
    return len(proofs)


def validate_inventory(
    proofs: Sequence[ReportProof],
    sources: Mapping[str, str],
    *,
    expected_digest: str = APPROVED_INVENTORY_SHA256,
) -> None:
    require(tuple(proof.clause for proof in proofs) == EXPECTED_CLAUSES, "clauses")
    require(tuple(proof.test for proof in proofs) == EXPECTED_TESTS, "tests")
    require(len(set(EXPECTED_TESTS)) == len(EXPECTED_TESTS), "test_unique")
    require(inventory_digest(proofs) == expected_digest, "inventory_identity")
    require(set(sources) == {proof.source for proof in proofs}, "source_inventory")
    for proof in proofs:
        test = extract_rust_test(sources[proof.source], proof)
        require(anchor_is_executable(test, proof.anchor), f"behavior_anchor:{proof.clause}")


def rejected(work: Callable[[], object], diagnostic: str) -> int:
    try:
        work()
    except ReportSuiteError:
        return 1
    raise ReportSuiteError(f"mutation_survived:{diagnostic}")


def mutation_self_test(sources: Mapping[str, str]) -> int:
    extra = dataclasses.replace(PROOFS[-1], clause="NCRDT-B64-001:extra", test="extra")
    mutations = (
        PROOFS[:-1],
        (*PROOFS, extra),
        (*PROOFS, PROOFS[-1]),
        tuple(reversed(PROOFS)),
        (dataclasses.replace(PROOFS[0], test="stale_test"), *PROOFS[1:]),
        (dataclasses.replace(PROOFS[0], clause="NCRDT-OTHER-001"), *PROOFS[1:]),
    )
    caught = sum(
        rejected(lambda mutation=mutation: validate_inventory(mutation, sources), "inventory")
        for mutation in mutations
    )

    proof = PROOFS[0]
    source = sources[proof.source]
    comment = dict(sources)
    comment[proof.source] = source.replace(proof.anchor, f"/*{proof.anchor}*/", 1)
    caught += rejected(lambda: validate_inventory(PROOFS, comment), "commented_anchor")
    missing_source = dict(sources)
    missing_source.pop(proof.source)
    caught += rejected(lambda: validate_inventory(PROOFS, missing_source), "missing_source")
    ignored = dict(sources)
    name = proof.test.rsplit("::", 1)[-1]
    ignored[proof.source] = source.replace(
        f"fn {name}()", f'#[ignore = "not executable proof"]\n    fn {name}()', 1
    )
    caught += rejected(lambda: validate_inventory(PROOFS, ignored), "ignored_test")

    command = test_command(proof)
    transcript = (
        "running 1 test\n"
        f"test {proof.test} ... ok\n\n"
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; "
        "0 filtered out; finished in 0.01s\n"
    )
    validate_test_transcript(proof, command, 0, transcript, "")
    wrong_name = transcript.replace(proof.test, "wire::base64::tests::unrelated", 1)
    caught += rejected(
        lambda: validate_test_transcript(proof, command, 0, wrong_name, ""),
        "wrong_transcript_test",
    )
    wrong_command = [*command]
    wrong_command[-2] = "unrelated"
    caught += rejected(
        lambda: validate_test_transcript(proof, wrong_command, 0, transcript, ""),
        "wrong_command",
    )
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-suite", action="store_true")
    arguments = parser.parse_args()
    sources = load_sources(PROOFS)
    validate_inventory(PROOFS, sources)
    mutations = mutation_self_test(sources)
    executed = run_suite(PROOFS) if arguments.run_suite else 0
    print("PASS: exact canonical base64 proof lane")
    print("- requirement=NCRDT-B64-001")
    print(f"- proofs={len(PROOFS)}")
    print(f"- negative_mutations={mutations}")
    print(f"- inventory_sha256={APPROVED_INVENTORY_SHA256}")
    print(f"- executed={executed}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ReportSuiteError as error:
        raise SystemExit(f"FAIL: {error}") from error
