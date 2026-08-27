#!/usr/bin/env python3
"""Validate the closed cross-implementation distribution-v12 parity record."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/opaque_distribution_parity_v12.json"
SCHEMA_PATH = "tools/validation/opaque_distribution_parity_v12.schema.json"
RUST_REPORT_PATH = "reports/rust_conformance_v12.json"
MANIFEST_PATH = "fixtures/distribution/manifest_v12.json"
REPORT_SHA256 = "900f90b55b16f75f1e86bb066767449989b2fea67504002de9a62baf8008a145"
SCHEMA_SHA256 = "c25be8ac0e7a1a383286bbcb74aa34b98d6f151a8ee4d3a9faaeed7e602196b9"
RUST_REPORT_SHA256 = "781d52f5095d18d408a6eac6f26e79372a809778ce24d6270931baf547b1da48"
MANIFEST_SHA256 = "29d1304aae027d33ff66b39b2cc499cca0e40fb24e5d4f5d749e33bf7dafd7c0"
RESULT_IDENTITY = "5f54f900673660f8e41220e1da2ccab265975292ac9dab8e8d31371ae34ea9d7"
CANONICAL_OUTPUT = "ac1d326a2fe6fbc3ba495ecd7635250efd72179ac50985392757c1784cf59372"
PUBLIC_STEPS = (
    ("step_1352", "892939f83901109b2acc85e7346168d123b32fff"),
    ("step_1353", "561e99287479b7831fb7e9912b1442880f1dcc51"),
    ("step_1354", "983e3ae7cbdd59ba0c0a8aa7bb86e4a6e2da04b6"),
    ("step_1355", "69a9e10050c8674a462a712f0c8215351f4657a7"),
    ("step_1356", "de716296d88b9908e350ec2eb7bc9406573a2a5d"),
)
PRIVATE_STEP = ("step_1357", "5d833e0235efe64f970b9c6a5a7c4e748a031b52")
APPENDED_FIXTURES = (
    "deep_delta_root_lookup_exact_budget",
    "deep_delta_absent_lookup_exact_budget",
    "deep_delta_extend_exact_budget",
    "post_branch_stop_has_no_target_work",
    "unsupported_change_event_has_no_semantic_hash",
)
FIELDS = (
    "schema", "status", "checkpoint", "public_predecessor", "manifest_sha256",
    "candidate_chain", "rust_evidence", "opaque_private_evidence", "comparison",
    "rcld", "publication_status", "result_identity_sha256",
)
CHAIN_FIELDS = ("step", "candidate", "owner_class", "result")
RUST_FIELDS = ("candidate", "result_identity_sha256", "serialized_run_sha256", "result")
PRIVATE_FIELDS = (
    "candidate", "predecessor", "result_identity_sha256", "execution_identity_sha256",
    "signed_input_projection_sha256", "result",
)
COMPARISON_FIELDS = (
    "scenario_count", "signed_input_count", "signed_event_count", "delivery_permutations",
    "process_count_per_implementation", "canonical_output_sha256", "byte_mismatch_count",
    "deliberate_expectation_mismatch", "aggregation", "result",
)
RCLD_FIELDS = ("id", "first_step", "last_step", "status")


class ParityError(RuntimeError):
    pass


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ParityError(diagnostic)


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def identity(value: dict[str, Any]) -> str:
    return hashlib.sha256(canonical({key: value[key] for key in FIELDS[:-1]})).hexdigest()


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(type(value) is dict, f"object:{relative}")
    return value


def expected_chain() -> list[dict[str, str]]:
    return [
        {"step": step, "candidate": candidate, "owner_class": "public", "result": "pass"}
        for step, candidate in PUBLIC_STEPS
    ] + [
        {
            "step": PRIVATE_STEP[0],
            "candidate": PRIVATE_STEP[1],
            "owner_class": "opaque_private",
            "result": "pass",
        }
    ]


def validate_report(value: Any) -> None:
    require(type(value) is dict and tuple(value) == FIELDS, "report:keys")
    require(
        value["schema"] == "nostr_automerge.opaque_distribution_parity.v12.v1"
        and value["status"] == "pass"
        and value["checkpoint"] == "step_1358"
        and value["public_predecessor"] == PUBLIC_STEPS[-1][1]
        and value["manifest_sha256"] == MANIFEST_SHA256,
        "report:identity",
    )
    chain = value["candidate_chain"]
    require(type(chain) is list and chain == expected_chain(), "report:chain")
    require(all(type(row) is dict and tuple(row) == CHAIN_FIELDS for row in chain), "report:chain_keys")
    rust = value["rust_evidence"]
    require(type(rust) is dict and tuple(rust) == RUST_FIELDS, "report:rust_keys")
    require(
        rust
        == {
            "candidate": PUBLIC_STEPS[-1][1],
            "result_identity_sha256": "e9ab4602f209a03a9366ec9ac2953fcd4d41b9aaab55f94ca6b1e3d5a3158967",
            "serialized_run_sha256": "27e2febf15d800a81a9b87066ec9a4989d861fa8b8938b73c7a4fc3e87881932",
            "result": "pass",
        },
        "report:rust",
    )
    private = value["opaque_private_evidence"]
    require(type(private) is dict and tuple(private) == PRIVATE_FIELDS, "report:private_keys")
    require(
        private
        == {
            "candidate": PRIVATE_STEP[1],
            "predecessor": "8250cfae174ab619808cfde1a076299ec6b60923",
            "result_identity_sha256": "16d10e8a508cb7b2448dd16f371d3260c02a0ac1b5e994485aa26ae6aff24da6",
            "execution_identity_sha256": "05645677994e5f245443d9742ec6f908546d124d979b39b12b7bee8e6de7d7a6",
            "signed_input_projection_sha256": "a326bbe748b4dfe0e8e75afcd3c2ef02896d15ad54038b33ef1e3ad889af611d",
            "result": "pass",
        },
        "report:private",
    )
    comparison = value["comparison"]
    require(type(comparison) is dict and tuple(comparison) == COMPARISON_FIELDS, "report:comparison_keys")
    require(
        comparison
        == {
            "scenario_count": 198,
            "signed_input_count": 5,
            "signed_event_count": 55,
            "delivery_permutations": 8,
            "process_count_per_implementation": 2,
            "canonical_output_sha256": CANONICAL_OUTPUT,
            "byte_mismatch_count": 0,
            "deliberate_expectation_mismatch": "rejected_by_both",
            "aggregation": "ordered_fixture_id_and_canonical_report_bytes_length_prefixed_sha256",
            "result": "pass",
        },
        "report:comparison",
    )
    rcld = value["rcld"]
    require(type(rcld) is dict and tuple(rcld) == RCLD_FIELDS, "report:rcld_keys")
    require(
        rcld == {"id": 107, "first_step": "step_1352", "last_step": "step_1358", "status": "complete"},
        "report:rcld",
    )
    require(value["publication_status"] == "held", "report:hold")
    require(value["result_identity_sha256"] == RESULT_IDENTITY == identity(value), "report:result_identity")


def validate_schema(schema: Any) -> None:
    require(type(schema) is dict and digest(SCHEMA_PATH) == SCHEMA_SHA256, "schema:sha256")
    require(schema.get("type") == "object" and schema.get("additionalProperties") is False, "schema:closed")
    require(schema.get("required") == list(FIELDS), "schema:required")
    require(tuple(schema.get("properties", {})) == FIELDS, "schema:properties")
    for field, keys in (
        ("rust_evidence", RUST_FIELDS),
        ("opaque_private_evidence", PRIVATE_FIELDS),
        ("comparison", COMPARISON_FIELDS),
        ("rcld", RCLD_FIELDS),
    ):
        nested = schema["properties"][field]
        require(nested.get("required") == list(keys), f"schema:{field}:required")
        require(tuple(nested.get("properties", {})) == keys, f"schema:{field}:properties")
        require(nested.get("additionalProperties") is False, f"schema:{field}:closed")
    item = schema["properties"]["candidate_chain"]["items"]
    require(
        item.get("required") == list(CHAIN_FIELDS)
        and tuple(item.get("properties", {})) == CHAIN_FIELDS
        and item.get("additionalProperties") is False,
        "schema:chain",
    )


def validate_bindings(value: dict[str, Any], rust: dict[str, Any], manifest: dict[str, Any]) -> None:
    require(digest(REPORT_PATH) == REPORT_SHA256, "binding:report")
    require(digest(RUST_REPORT_PATH) == RUST_REPORT_SHA256, "binding:rust_report")
    require(digest(MANIFEST_PATH) == MANIFEST_SHA256, "binding:manifest")
    require(
        rust.get("result_identity_sha256") == value["rust_evidence"]["result_identity_sha256"]
        and rust.get("serialized_run_sha256") == value["rust_evidence"]["serialized_run_sha256"]
        and rust.get("canonical_output_sha256") == value["comparison"]["canonical_output_sha256"]
        and rust.get("scenario_count") == value["comparison"]["scenario_count"]
        and rust.get("process_count") == value["comparison"]["process_count_per_implementation"]
        and rust.get("delivery_order_count") == value["comparison"]["delivery_permutations"]
        and rust.get("deliberate_expectation_mismatch") == "rejected",
        "binding:rust",
    )
    require(
        manifest.get("distribution_schema") == "nostr_automerge.fixture_distribution.v12"
        and manifest.get("transition_stage") == "distribution_complete"
        and manifest.get("complete") is True
        and manifest.get("fixture_count") == 198
        and tuple(manifest.get("appended_v12_fixtures", ())) == APPENDED_FIXTURES,
        "binding:manifest_shape",
    )
    fixtures = manifest.get("fixtures")
    require(type(fixtures) is list and len(fixtures) == 198, "binding:fixture_count")
    fixture_ids = [row.get("fixture_id") for row in fixtures if type(row) is dict]
    require(
        len(fixture_ids) == len(fixtures)
        and fixture_ids == sorted(fixture_ids)
        and len(fixture_ids) == len(set(fixture_ids)),
        "binding:fixture_order",
    )
    by_id = {row["fixture_id"]: row for row in fixtures}
    require(all(fixture_id in by_id for fixture_id in APPENDED_FIXTURES), "binding:fixture_inventory")
    appended = [by_id[fixture_id] for fixture_id in APPENDED_FIXTURES]
    event_count = 0
    for row in appended:
        paths = row.get("input_paths")
        require(type(paths) is list and len(paths) == 1, "binding:input_path")
        scenario = json.loads((ROOT / paths[0]).read_text(encoding="utf-8"))
        require(type(scenario) is dict and scenario.get("fixture_id") == row["fixture_id"], "binding:input")
        events = scenario.get("raw_events")
        require(type(events) is list and len(events) > 0, "binding:events")
        event_count += len(events)
    require(event_count == value["comparison"]["signed_event_count"], "binding:event_count")
    for index in range(1, len(PUBLIC_STEPS)):
        parent = subprocess.run(
            ("git", "rev-parse", f"{PUBLIC_STEPS[index][1]}^"),
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        require(parent == PUBLIC_STEPS[index - 1][1], f"binding:public_parent:{index}")
    ancestor = subprocess.run(
        ("git", "merge-base", "--is-ancestor", PUBLIC_STEPS[-1][1], "HEAD"),
        cwd=ROOT,
        check=False,
        capture_output=True,
    )
    require(ancestor.returncode == 0, "binding:public_predecessor")


def expect_rejected(work: Callable[[], None], diagnostic: str) -> int:
    try:
        work()
    except ParityError:
        return 1
    raise ParityError(f"mutation_survived:{diagnostic}")


def mutation_self_test(value: dict[str, Any], schema: dict[str, Any], rust: dict[str, Any], manifest: dict[str, Any]) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    for field, replacement in (
        ("status", "held"),
        ("checkpoint", "step_1357"),
        ("public_predecessor", "0" * 40),
        ("manifest_sha256", "0" * 64),
        ("publication_status", "published"),
        ("result_identity_sha256", "0" * 64),
    ):
        candidate = copy.deepcopy(value)
        candidate[field] = replacement
        mutations.append((field, candidate))
    for index, field, replacement in (
        (0, "candidate", "0" * 40),
        (5, "owner_class", "public"),
        (5, "result", "failed"),
    ):
        candidate = copy.deepcopy(value)
        candidate["candidate_chain"][index][field] = replacement
        mutations.append((f"chain:{index}:{field}", candidate))
    for name, path, replacement in (
        ("rust_candidate", ("rust_evidence", "candidate"), "0" * 40),
        ("rust_result", ("rust_evidence", "result_identity_sha256"), "0" * 64),
        ("private_candidate", ("opaque_private_evidence", "candidate"), "0" * 40),
        ("private_predecessor", ("opaque_private_evidence", "predecessor"), "0" * 40),
        ("private_result", ("opaque_private_evidence", "result_identity_sha256"), "0" * 64),
        ("private_execution", ("opaque_private_evidence", "execution_identity_sha256"), "0" * 64),
        ("private_inputs", ("opaque_private_evidence", "signed_input_projection_sha256"), "0" * 64),
        ("canonical", ("comparison", "canonical_output_sha256"), "0" * 64),
        ("mismatch", ("comparison", "byte_mismatch_count"), 1),
        ("count", ("comparison", "scenario_count"), 197),
        ("rcld", ("rcld", "status"), "open"),
    ):
        candidate = copy.deepcopy(value)
        candidate[path[0]][path[1]] = replacement
        mutations.append((name, candidate))
    missing = copy.deepcopy(value)
    missing.pop("comparison")
    mutations.append(("missing", missing))
    extra = copy.deepcopy(value)
    extra["extra"] = False
    mutations.append(("extra", extra))
    reordered = {"status": value["status"], **value}
    mutations.append(("reordered", reordered))
    chain_missing = copy.deepcopy(value)
    chain_missing["candidate_chain"].pop()
    mutations.append(("chain_missing", chain_missing))
    chain_reordered = copy.deepcopy(value)
    chain_reordered["candidate_chain"].reverse()
    mutations.append(("chain_reordered", chain_reordered))
    coordinated = copy.deepcopy(value)
    coordinated["opaque_private_evidence"]["candidate"] = "1" * 40
    coordinated["opaque_private_evidence"]["result_identity_sha256"] = "2" * 64
    coordinated["candidate_chain"][-1]["candidate"] = "1" * 40
    coordinated["result_identity_sha256"] = identity(coordinated)
    mutations.append(("coordinated_private", coordinated))
    caught = sum(
        expect_rejected(lambda candidate=candidate: validate_report(candidate), name)
        for name, candidate in mutations
    )
    for name, mutate in (
        ("schema_open", lambda item: item.update(additionalProperties=True)),
        ("schema_missing", lambda item: item["required"].pop()),
        ("schema_private_open", lambda item: item["properties"]["opaque_private_evidence"].update(additionalProperties=True)),
        ("schema_chain_open", lambda item: item["properties"]["candidate_chain"]["items"].update(additionalProperties=True)),
    ):
        candidate = copy.deepcopy(schema)
        mutate(candidate)
        caught += expect_rejected(lambda candidate=candidate: validate_schema(candidate), name)
    rust_mutation = copy.deepcopy(rust)
    rust_mutation["canonical_output_sha256"] = "0" * 64
    caught += expect_rejected(
        lambda: validate_bindings(value, rust_mutation, manifest), "binding_rust"
    )
    manifest_mutation = copy.deepcopy(manifest)
    manifest_mutation["fixture_count"] = 197
    caught += expect_rejected(
        lambda: validate_bindings(value, rust, manifest_mutation), "binding_manifest"
    )
    return caught


def main() -> None:
    value = load(REPORT_PATH)
    schema = load(SCHEMA_PATH)
    rust = load(RUST_REPORT_PATH)
    manifest = load(MANIFEST_PATH)
    validate_report(value)
    validate_schema(schema)
    validate_bindings(value, rust, manifest)
    mutations = mutation_self_test(value, schema, rust, manifest)
    print(
        "PASS: opaque distribution v12 parity "
        f"(198 scenarios, 8 deliveries, 2 processes per implementation, {mutations} mutations)"
    )


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.SubprocessError, ParityError) as error:
        raise SystemExit(f"FAIL: {error}") from error
