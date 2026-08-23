#!/usr/bin/env python3
"""Validate the opaque TypeScript v10 result and public byte-parity proof."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/opaque_conformance_v10.json"
SCHEMA_PATH = "tools/validation/opaque_conformance_v10.schema.json"
RUST_REPORT_PATH = "reports/rust_conformance_v10.json"
MANIFEST_PATH = "fixtures/distribution/manifest_v10.json"
REPORT_SHA256 = "8c1b72d4a4aab763ab226fb7eb8bcb7ff30c176404b9b8568da4a68df666804c"
SCHEMA_SHA256 = "d9556405c62bf065c67e066ca31b9cce10d82054c181aa83a16b65f33322e50b"
RESULT_IDENTITY = "4430b84445e0d30b78922d9ef2df2528d74447eb8a064c8c097fc431e92a3acd"
CANONICAL_OUTPUT = "c9f28deb32dfedce674a6871b0eb949f38b5a5f977a67ca993f7ed639df1e112"
FIELDS = (
    "schema",
    "status",
    "checkpoint",
    "public_predecessor",
    "private_candidate",
    "private_predecessor",
    "manifest_sha256",
    "private_evidence",
    "rust_evidence",
    "comparison",
    "publication_status",
    "result_identity_sha256",
)
PRIVATE_FIELDS = (
    "report_sha256",
    "schema_sha256",
    "runner_sha256",
    "implementation_projection_sha256",
    "dependency_lock_sha256",
    "execution_sha256",
    "result_identity_sha256",
    "result",
)
RUST_FIELDS = ("candidate", "report_identity_sha256", "distribution_run_sha256", "result")
COMPARISON_FIELDS = (
    "scenario_count",
    "compared_report_count",
    "delivery_permutations",
    "process_count_per_implementation",
    "canonical_output_sha256",
    "byte_mismatch_count",
    "aggregation",
    "malformed_report",
    "noncanonical_report",
    "structurally_valid_semantic_mismatch",
    "result",
)
EXPECTED = {
    "schema": "nostr_automerge.opaque_conformance.v10.v1",
    "status": "pass",
    "checkpoint": "step_1273",
    "public_predecessor": "6e7084ae32b9d20e55e76b5496c126bd52974f0d",
    "private_candidate": "36db673b8e5b62df69a5ee321b2e13c040fc8237",
    "private_predecessor": "fb585804db1f869014f4d10f57847c081c3635a4",
    "manifest_sha256": "86ec32f34dd99ef0c1e5ea3531360a1f78bf07d62818375096e0bdf0f209b8e5",
    "private_evidence": {
        "report_sha256": "0db4d1e7f7b93134057ebe4ed8e370b60fe3dffd14de37c99197ff5971e90a4f",
        "schema_sha256": "f02cb24a8f6bebfa44dcd9f4c345d1393cae5db32550cbd881cdabc869495581",
        "runner_sha256": "902d3f072fcabd5bea55407e7a736def3c43417c57530b3f015065acc04b480f",
        "implementation_projection_sha256": "c54a845f0e53b6217bc5f32b8d240be402938442099e3afe3e1f1fc24213e4cc",
        "dependency_lock_sha256": "d881757529b805b8ae4da935127730fe901b8b13a71382023be161016cd35a7d",
        "execution_sha256": "4a663141aa5d122fd388e8c08e115d8ceb58efe8e36408abc0339f9aeba4a958",
        "result_identity_sha256": "3226b1ae0c6534c928e0bcf61e4b82f68d1447060f9e42aa85275d1178ff43c4",
        "result": "pass",
    },
    "rust_evidence": {
        "candidate": "20b786c5c3ff143786aaaca56ad19bd26739b67b",
        "report_identity_sha256": "7be69317ca8f007f8b0b74f1bc355558981ba55a75bc3eb8b2b609b3590184c7",
        "distribution_run_sha256": "377b0fe6ae2916b829b3ada84f7adf760d874123ce8be14130999a076c8578c6",
        "result": "pass",
    },
    "comparison": {
        "scenario_count": 192,
        "compared_report_count": 192,
        "delivery_permutations": 8,
        "process_count_per_implementation": 2,
        "canonical_output_sha256": CANONICAL_OUTPUT,
        "byte_mismatch_count": 0,
        "aggregation": "ordered_fixture_id_and_canonical_report_bytes_length_prefixed_sha256",
        "malformed_report": "rejected_by_parser",
        "noncanonical_report": "rejected_by_parser",
        "structurally_valid_semantic_mismatch": "rejected_by_comparison",
        "result": "pass",
    },
    "publication_status": "held",
    "result_identity_sha256": RESULT_IDENTITY,
}
SOURCE_HASHES = {
    "tools/nostr_automerge_conformance/src/expected.rs": "c6d36c048972c8301c33672a80872badd909572abdbe5aac081f9f771344bc12",
    "tools/nostr_automerge_conformance/src/report_json.rs": "ff0245b2ecd83b3dcf36889002cca2d789305bfb24a07729a0a8636af1ee70ea",
    "tools/nostr_automerge_conformance/src/runner.rs": "222c195338ea139e5c9887c19e2ba16f5a63d6939dd4354d28a9c3e44431f733",
}


class EvidenceError(ValueError):
    """One opaque parity invariant failed."""


def digest_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def digest_file(relative: str) -> str:
    return digest_bytes((ROOT / relative).read_bytes())


def identity(value: dict[str, Any]) -> str:
    projection = {key: value[key] for key in FIELDS[:-1]}
    payload = json.dumps(projection, sort_keys=True, separators=(",", ":")).encode()
    return digest_bytes(payload)


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise EvidenceError(diagnostic)


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"object:{relative}")
    return value


def validate_schema(schema: dict[str, Any]) -> None:
    require(
        tuple(schema) == ("title", "type", "required", "properties", "additionalProperties"),
        "schema:keys",
    )
    require(schema.get("type") == "object", "schema:type")
    require(schema.get("required") == list(FIELDS), "schema:required")
    require(tuple(schema.get("properties", {})) == FIELDS, "schema:properties")
    require(schema.get("additionalProperties") is False, "schema:closed")
    for field, keys in (
        ("private_evidence", PRIVATE_FIELDS),
        ("rust_evidence", RUST_FIELDS),
        ("comparison", COMPARISON_FIELDS),
    ):
        nested = schema["properties"][field]
        require(nested.get("required") == list(keys), f"schema:{field}:required")
        require(tuple(nested.get("properties", {})) == keys, f"schema:{field}:properties")
        require(nested.get("additionalProperties") is False, f"schema:{field}:closed")


def validate(value: dict[str, Any], schema: dict[str, Any]) -> None:
    require(tuple(value) == FIELDS, "report:keys")
    require(value == EXPECTED, "report:binding")
    require(tuple(value["private_evidence"]) == PRIVATE_FIELDS, "report:private_keys")
    require(tuple(value["rust_evidence"]) == RUST_FIELDS, "report:rust_keys")
    require(tuple(value["comparison"]) == COMPARISON_FIELDS, "report:comparison_keys")
    require(identity(value) == RESULT_IDENTITY, "report:identity")
    validate_schema(schema)
    require(digest_file(REPORT_PATH) == REPORT_SHA256, "report:file")
    require(digest_file(SCHEMA_PATH) == SCHEMA_SHA256, "schema:file")
    require(digest_file(MANIFEST_PATH) == value["manifest_sha256"], "manifest:file")
    rust = load(RUST_REPORT_PATH)
    require(rust.get("candidate") == value["rust_evidence"]["candidate"], "rust:candidate")
    require(
        rust.get("result_identity_sha256")
        == value["rust_evidence"]["report_identity_sha256"],
        "rust:identity",
    )
    require(
        rust.get("distribution_run_sha256")
        == value["rust_evidence"]["distribution_run_sha256"],
        "rust:distribution",
    )
    require(
        rust.get("canonical_output_sha256") == value["comparison"]["canonical_output_sha256"],
        "rust:canonical",
    )
    for relative, expected in SOURCE_HASHES.items():
        require(digest_file(relative) == expected, f"source:{relative}")
    ancestry = subprocess.run(
        ("git", "merge-base", "--is-ancestor", value["public_predecessor"], "HEAD"),
        cwd=ROOT,
        check=False,
        capture_output=True,
    )
    require(ancestry.returncode == 0, "public_predecessor")


def expect_rejected(work: Any, diagnostic: str) -> int:
    try:
        work()
    except EvidenceError:
        return 1
    raise EvidenceError(f"mutation_survived:{diagnostic}")


def mutation_self_test(value: dict[str, Any], schema: dict[str, Any]) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    for field, replacement in (
        ("status", "held"),
        ("checkpoint", "step_1272"),
        ("public_predecessor", "0" * 40),
        ("private_candidate", "1" * 40),
        ("manifest_sha256", "0" * 64),
        ("publication_status", "published"),
        ("result_identity_sha256", "0" * 64),
    ):
        mutated = copy.deepcopy(value)
        mutated[field] = replacement
        mutations.append((field, mutated))
    for field, replacement in (
        ("report_sha256", "0" * 64),
        ("execution_sha256", "0" * 64),
        ("result", "failed"),
    ):
        mutated = copy.deepcopy(value)
        mutated["private_evidence"][field] = replacement
        mutations.append((f"private:{field}", mutated))
    for field, replacement in (
        ("scenario_count", 191),
        ("compared_report_count", 191),
        ("canonical_output_sha256", "0" * 64),
        ("byte_mismatch_count", 1),
        ("malformed_report", "accepted"),
        ("structurally_valid_semantic_mismatch", "accepted"),
    ):
        mutated = copy.deepcopy(value)
        mutated["comparison"][field] = replacement
        mutations.append((f"comparison:{field}", mutated))
    missing = copy.deepcopy(value)
    missing.pop("status")
    mutations.append(("missing", missing))
    extra = copy.deepcopy(value)
    extra["extra"] = False
    mutations.append(("extra", extra))
    reordered = {"status": value["status"], **value}
    mutations.append(("reordered", reordered))
    coordinated = copy.deepcopy(value)
    coordinated["comparison"]["canonical_output_sha256"] = "f" * 64
    coordinated["result_identity_sha256"] = identity(coordinated)
    mutations.append(("coordinated", coordinated))
    caught = sum(
        expect_rejected(lambda item=item: validate(item, schema), name)
        for name, item in mutations
    )
    schema_mutations: list[tuple[str, dict[str, Any]]] = []
    opened = copy.deepcopy(schema)
    opened["additionalProperties"] = True
    schema_mutations.append(("schema_open", opened))
    nested_open = copy.deepcopy(schema)
    nested_open["properties"]["comparison"]["additionalProperties"] = True
    schema_mutations.append(("schema_nested_open", nested_open))
    required = copy.deepcopy(schema)
    required["required"].pop()
    schema_mutations.append(("schema_required", required))
    caught += sum(
        expect_rejected(lambda item=item: validate(value, item), name)
        for name, item in schema_mutations
    )
    require(caught == 23, "mutation_count")
    return caught


def update_length_prefixed(digest: Any, value: bytes) -> None:
    digest.update(len(value).to_bytes(8, "big"))
    digest.update(value)


def validate_expected_projection(run: dict[str, Any]) -> None:
    manifest = load(MANIFEST_PATH)
    fixtures = manifest.get("fixtures")
    reports = run.get("reports")
    require(isinstance(fixtures, list) and len(fixtures) == 192, "run:fixtures")
    require(isinstance(reports, list) and len(reports) == 192, "run:reports")
    aggregate = hashlib.sha256()
    previous = None
    for fixture, report in zip(fixtures, reports, strict=True):
        fixture_id = fixture.get("fixture_id")
        expected_path = fixture.get("expected_path")
        require(isinstance(fixture_id, str) and fixture_id > (previous or ""), "run:order")
        require(isinstance(expected_path, str), "run:expected_path")
        expected = (ROOT / expected_path).read_bytes()
        require(
            report == {"fixture_id": fixture_id, "report_sha256": digest_bytes(expected)},
            f"run:report:{fixture_id}",
        )
        update_length_prefixed(aggregate, fixture_id.encode())
        update_length_prefixed(aggregate, expected)
        previous = fixture_id
    require(aggregate.hexdigest() == CANONICAL_OUTPUT, "run:aggregate")


def run_suite() -> None:
    command = (
        "cargo",
        "run",
        "--quiet",
        "-p",
        "nostr_automerge_conformance",
        "--locked",
        "--",
        "run_distribution",
        MANIFEST_PATH,
    )
    first = subprocess.run(command, cwd=ROOT, check=True, capture_output=True).stdout
    second = subprocess.run(command, cwd=ROOT, check=True, capture_output=True).stdout
    require(first == second, "run:process_identity")
    require(digest_bytes(first) == EXPECTED["rust_evidence"]["distribution_run_sha256"], "run:sha")
    run = json.loads(first)
    require(run.get("status") == "pass", "run:status")
    require(run.get("fixture_count") == 192, "run:count")
    require(run.get("delivery_permutations") == 8, "run:permutations")
    require(run.get("canonical_output_sha256") == CANONICAL_OUTPUT, "run:canonical")
    validate_expected_projection(run)
    test_run = subprocess.run(
        (
            "cargo",
            "test",
            "-p",
            "nostr_automerge_conformance",
            "--locked",
            "report_parity_rejects_malformed_and_structurally_valid_mismatch",
        ),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    transcript = f"{test_run.stdout}{test_run.stderr}"
    require(
        "test runner::tests::report_parity_rejects_malformed_and_structurally_valid_mismatch ... ok"
        in transcript,
        "run:mismatch_test",
    )
    require("test result: ok. 1 passed; 0 failed" in transcript, "run:mismatch_result")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run", action="store_true")
    arguments = parser.parse_args()
    value = load(REPORT_PATH)
    schema = load(SCHEMA_PATH)
    validate(value, schema)
    mutations = mutation_self_test(value, schema)
    if arguments.run:
        run_suite()
    print("PASS: TypeScript signed-v10 opaque byte parity")
    print("- compared_reports=192")
    print("- delivery_permutations=8")
    print("- processes_per_implementation=2")
    print(f"- negative_mutations={mutations}")
    print(f"- executed={int(arguments.run)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
