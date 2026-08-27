#!/usr/bin/env python3
"""Validate the closed remediation-v11 adversarial qualification record."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import sys
from typing import Any, Callable

from validate_remediation_v11_proof_catalog import enabled_test

sys.dont_write_bytecode = True

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/remediation_v11_adversarial_qualification.json"
SCHEMA_PATH = "tools/validation/remediation_v11_adversarial_qualification.schema.json"
REPORT_SHA256 = "a1e4a3657214ac21f750173abb2d737f3ecbcc974fa081c38217c8a05c7487c3"
SCHEMA_SHA256 = "749faaaca7390d78b41d7e850c4b20eb753259871c06177b2bdcfaa11a5130eb"
RESULT_IDENTITY = "26b4ac781413a8bc73af69564b9add0a555790b2967fa93498dafd1caf8b0e1e"
FIELDS = (
    "schema", "status", "checkpoint", "revision", "candidates", "imports",
    "public_lanes", "opaque_private_lanes", "qualification", "holds", "result",
    "result_identity_sha256",
)
PUBLIC = "9e075bba3636efcd6bc6925e134398ad18db202a"
PRIVATE = "5d833e0235efe64f970b9c6a5a7c4e748a031b52"
IMPORTS = (
    ("proof_catalog", "reports/remediation_v11_proof_catalog.json", "0127e7e475e1548d183ea8ab2488ebc5ae89475be2f003c6d7da9f3e3bdef2c0"),
    ("resource", "reports/target_work_accounting_v11.json", "e15dca3958e9c9cf98da585c5a60135e4b3c9d8b59ddec9c0e3ef068615948ae"),
    ("ownership", "reports/persistent_ownership_v11.json", "10235d1eac0b09a2b22ba70959a47a06478a08f595b31c9f843bb9fb41dcc67f"),
    ("parity", "reports/opaque_distribution_parity_v12.json", "900f90b55b16f75f1e86bb066767449989b2fea67504002de9a62baf8008a145"),
)
LANES = (
    ("persistent_depth", (
        "reference::branch_state::tests::finding_096_deep_persistent_lookup_is_internally_metered",
        "reference::branch_state::tests::deep_persistent_boundaries_are_exact_and_cancellable",
    )),
    ("every_prefix_cancellation", (
        "reference::evaluate::tests::initial_evaluator_maps_charge_before_every_item",
        "engine::reference_evaluator::tests::member_and_dependency_preparation_interleaves_every_owned_operation",
        "evidence::corpus_builder::tests::selected_manifest_metering_owns_every_read_clone_comparison_and_replacement",
        "graph::equivocation::tests::quarantine_traversal_has_exact_prefix_and_cancellation_boundaries",
    )),
    ("no_post_stop", (
        "engine::reference_evaluator::tests::every_interrupted_prefix_uses_only_fallback_and_never_refunds",
        "engine::reference_evaluator::tests::v12_post_branch_fixture_stops_at_exact_internal_boundary",
    )),
    ("deep_scaling_and_overflow", (
        "graph::scaling::expanded_control_actor_conflict_and_projection_models_are_bounded",
        "engine::reference_evaluator::tests::complete_report_plan_is_exact_named_and_overflow_checked",
    )),
    ("bounded_teardown", (
        "reference::branch_state::tests::deep_unique_delta_teardown_is_bounded_stack",
        "reference::branch_state::tests::constrained_stack_wide_delta_fork_preserves_shared_parent_teardown",
        "control::ancestry::tests::deep_unique_control_ancestry_teardown_is_bounded_stack",
        "control::ancestry::tests::constrained_stack_wide_ancestry_fork_preserves_shared_parent_teardown",
    )),
    ("public_boundary", (
        "cancellation_is_safe_at_every_evaluator_boundary",
        "complete_report_resource_boundaries_are_exact_and_deterministic",
    )),
)
SOURCE_BY_PREFIX = {
    "reference::branch_state::": "crates/nostr_automerge/src/reference/branch_state.rs",
    "reference::evaluate::": "crates/nostr_automerge/src/reference/evaluate.rs",
    "engine::reference_evaluator::": "crates/nostr_automerge/src/engine/reference_evaluator.rs",
    "evidence::corpus_builder::": "crates/nostr_automerge/src/evidence/corpus_builder.rs",
    "graph::equivocation::": "crates/nostr_automerge/src/graph/equivocation.rs",
    "graph::scaling::": "crates/nostr_automerge/src/graph/scaling.rs",
    "control::ancestry::": "crates/nostr_automerge/src/control/ancestry.rs",
}
PUBLIC_API_SOURCE = "crates/nostr_automerge/tests/public_engine_api.rs"
PRIVATE_LANES = {
    "self_test_mutations": 276, "fixed_reproductions": 21, "open_reproductions": 2,
    "distribution_scenarios": 198, "delivery_permutations": 8, "processes": 2,
    "canonical_output_sha256": "ac1d326a2fe6fbc3ba495ecd7635250efd72179ac50985392757c1784cf59372",
    "result": "pass",
}
QUALIFICATION = {
    "public_test_count": 16, "public_lane_count": 6,
    "selected_mutation_survivors": 0, "stack_regressions": 0,
    "provenance_regressions": 0, "resource_regressions": 0,
    "private_source_disclosed": False, "result": "pass",
}
HOLDS = (
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
)


class QualificationError(RuntimeError):
    pass


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise QualificationError(diagnostic)


def require_record(value: Any, keys: tuple[str, ...], label: str) -> dict[str, Any]:
    require(type(value) is dict and tuple(value) == keys, f"{label}:keys")
    return value


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def identity(value: dict[str, Any]) -> str:
    projection = {key: value[key] for key in FIELDS[:-1]}
    encoded = json.dumps(projection, ensure_ascii=True, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def source_for(test: str) -> str:
    for prefix, source in SOURCE_BY_PREFIX.items():
        if test.startswith(prefix):
            return source
    require("::" not in test, f"test:unmapped:{test}")
    return PUBLIC_API_SOURCE


def validate_schema(value: Any) -> None:
    record = require_record(value, ("title", "type", "required", "properties", "additionalProperties"), "schema")
    require(record["type"] == "object" and record["additionalProperties"] is False, "schema:closed")
    require(tuple(record["required"]) == FIELDS and tuple(record["properties"]) == FIELDS, "schema:fields")
    for field in ("candidates", "opaque_private_lanes", "qualification"):
        nested = record["properties"][field]
        require(nested.get("additionalProperties") is False, f"schema:{field}:closed")
        require(tuple(nested.get("required", ())) == tuple(nested.get("properties", {})), f"schema:{field}:fields")
    for field in ("imports", "public_lanes"):
        item = record["properties"][field]["items"]
        require(item.get("additionalProperties") is False, f"schema:{field}:closed")
        require(tuple(item.get("required", ())) == tuple(item.get("properties", {})), f"schema:{field}:fields")


def validate_report(value: Any, sources: dict[str, str]) -> None:
    record = require_record(value, FIELDS, "report")
    require(
        (record["schema"], record["status"], record["checkpoint"], record["revision"], record["result"])
        == ("nostr_automerge.remediation_v11_adversarial_qualification.v1", "pass", "step_1360", "draft_2026_08", "pass"),
        "report:identity",
    )
    require(record["candidates"] == {"public": PUBLIC, "opaque_private": PRIVATE}, "report:candidates")
    expected_imports = [{"category": category, "path": path, "sha256": sha} for category, path, sha in IMPORTS]
    require(record["imports"] == expected_imports, "report:imports")
    for _category, path, sha in IMPORTS:
        require(digest(path) == sha, f"binding:{path}")
    expected_lanes = [{"lane": lane, "tests": list(tests), "result": "pass"} for lane, tests in LANES]
    require(record["public_lanes"] == expected_lanes, "report:public_lanes")
    tests = [test for _lane, lane_tests in LANES for test in lane_tests]
    require(len(tests) == len(set(tests)) == 16, "report:test_inventory")
    for test in tests:
        source = source_for(test)
        require(source in sources, f"source:{source}")
        enabled_test(sources[source], test)
    require(record["opaque_private_lanes"] == PRIVATE_LANES and tuple(record["opaque_private_lanes"]) == tuple(PRIVATE_LANES), "report:private")
    require(record["qualification"] == QUALIFICATION and tuple(record["qualification"]) == tuple(QUALIFICATION), "report:qualification")
    require(tuple(record["holds"]) == HOLDS, "report:holds")
    require(record["result_identity_sha256"] == RESULT_IDENTITY == identity(record), "report:result_identity")


def rejected(work: Callable[[], None], diagnostic: str) -> int:
    try:
        work()
    except (QualificationError, ProofCatalogError):
        return 1
    raise QualificationError(f"mutation_survived:{diagnostic}")


# Imported only for the exact enabled-test error type without weakening provenance.
from validate_remediation_v11_proof_catalog import ProofCatalogError  # noqa: E402


def mutation_self_test(value: dict[str, Any], schema: dict[str, Any], sources: dict[str, str]) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    mutators = (
        ("candidate", lambda row: row["candidates"].__setitem__("public", "0" * 40)),
        ("private", lambda row: row["candidates"].__setitem__("opaque_private", "0" * 40)),
        ("import_missing", lambda row: row["imports"].pop()),
        ("import_reorder", lambda row: row["imports"].reverse()),
        ("import_hash", lambda row: row["imports"][0].__setitem__("sha256", "0" * 64)),
        ("lane_missing", lambda row: row["public_lanes"].pop()),
        ("lane_reorder", lambda row: row["public_lanes"].reverse()),
        ("test_missing", lambda row: row["public_lanes"][0]["tests"].pop()),
        ("test_duplicate", lambda row: row["public_lanes"][0]["tests"].__setitem__(1, row["public_lanes"][0]["tests"][0])),
        ("test_stale", lambda row: row["public_lanes"][0]["tests"].__setitem__(0, "stale_test")),
        ("lane_failed", lambda row: row["public_lanes"][0].__setitem__("result", "failed")),
        ("private_mutations", lambda row: row["opaque_private_lanes"].__setitem__("self_test_mutations", 275)),
        ("private_fixed", lambda row: row["opaque_private_lanes"].__setitem__("fixed_reproductions", 20)),
        ("private_open", lambda row: row["opaque_private_lanes"].__setitem__("open_reproductions", 3)),
        ("private_scenarios", lambda row: row["opaque_private_lanes"].__setitem__("distribution_scenarios", 197)),
        ("private_orders", lambda row: row["opaque_private_lanes"].__setitem__("delivery_permutations", 7)),
        ("private_processes", lambda row: row["opaque_private_lanes"].__setitem__("processes", 1)),
        ("private_hash", lambda row: row["opaque_private_lanes"].__setitem__("canonical_output_sha256", "0" * 64)),
        ("survivor", lambda row: row["qualification"].__setitem__("selected_mutation_survivors", 1)),
        ("stack", lambda row: row["qualification"].__setitem__("stack_regressions", 1)),
        ("provenance", lambda row: row["qualification"].__setitem__("provenance_regressions", 1)),
        ("resource", lambda row: row["qualification"].__setitem__("resource_regressions", 1)),
        ("leak", lambda row: row["qualification"].__setitem__("private_source_disclosed", True)),
        ("hold", lambda row: row["holds"].pop()),
        ("identity", lambda row: row.__setitem__("result_identity_sha256", "0" * 64)),
        ("extra", lambda row: row.__setitem__("extra", False)),
    )
    for name, mutate in mutators:
        candidate = copy.deepcopy(value); mutate(candidate); mutations.append((name, candidate))
    caught = sum(rejected(lambda candidate=candidate: validate_report(candidate, sources), name) for name, candidate in mutations)
    opened = copy.deepcopy(schema); opened["additionalProperties"] = True
    caught += rejected(lambda: validate_schema(opened), "schema_open")
    nested = copy.deepcopy(schema); nested["properties"]["qualification"]["additionalProperties"] = True
    caught += rejected(lambda: validate_schema(nested), "schema_nested")
    missing = copy.deepcopy(schema); missing["required"].pop()
    caught += rejected(lambda: validate_schema(missing), "schema_missing")
    return caught


def main() -> int:
    report = json.loads((ROOT / REPORT_PATH).read_text(encoding="utf-8"))
    schema = json.loads((ROOT / SCHEMA_PATH).read_text(encoding="utf-8"))
    require(digest(REPORT_PATH) == REPORT_SHA256, "binding:report")
    require(digest(SCHEMA_PATH) == SCHEMA_SHA256, "binding:schema")
    tests = [test for _lane, lane_tests in LANES for test in lane_tests]
    sources = {source_for(test): (ROOT / source_for(test)).read_text(encoding="utf-8") for test in tests}
    validate_schema(schema)
    validate_report(report, sources)
    mutations = mutation_self_test(report, schema, sources)
    print(f"PASS: remediation v11 adversarial qualification public_tests=16 mutations={mutations} private=21_fixed+2_open")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, QualificationError, ProofCatalogError) as error:
        raise SystemExit(f"FAIL: {error}") from error
