#!/usr/bin/env python3
"""Validate the frozen v18 sealed-boundary and evidence contracts."""

from __future__ import annotations

import copy
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "spec/causal_projection_contracts_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_contracts_v18.schema.json"
DIRECT_SITES = [
    "ActorStateRead", "PredecessorCandidateRead", "ActorIdentityDecision",
    "SequenceRelationDecision", "StoredCounterRead", "ExpectedStartComparison",
    "CheckedAdvance",
]
PROPERTIES = [
    "TYPED_BUDGET_EXHAUSTED_IDENTITY", "TYPED_CANCELLED_IDENTITY",
    "UNEXPECTED_WORK_ERROR_IDENTITY", "CHARGE_AFTER_OPERATION",
    "OPERATION_OBSERVATION_BEFORE_TARGET", "SITE_TARGET_BEFORE_CHARGE",
    "TARGET_AFTER_STOP", "OBSERVATION_AFTER_STOP", "PUBLICATION_AFTER_STOP",
    "SITE_ID_MISMATCH", "COUNTER_MISMATCH", "ALTERNATE_CONSUMER_BYPASS",
]


class ContractError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise ContractError(code)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)
    return json.loads(path.read_text(), object_pairs_hook=closed)


def validate(value: Any, schema: Any) -> None:
    require(list(value) == schema["required"], "contract:shape")
    require(schema["additionalProperties"] is False, "schema:closed")
    require(value["schema"] == "nostr_automerge.causal_projection_contracts.v18.v1", "contract:schema")
    require(value["status"] == "frozen" and value["result"] == "pass", "contract:state")

    descriptor = value["descriptor"]
    require(descriptor["fields"] == ["site_id", "phase", "family", "counter", "abstract_owner_class", "applicability"], "descriptor:fields")
    require("projection_construction" in descriptor["phase_vocabulary"], "descriptor:phase")
    require(descriptor["applicability"] == "required" and descriptor["counts"] == "source_derived", "descriptor:vocabulary")

    sealed = value["sealed_operation"]
    require(sealed["order"] == ["descriptor", "charge", "target", "completion_observation", "return"], "sealed:order")
    require(sealed["sole_preapproval_callback"] == "descriptor_aware_charge", "sealed:preapproval")
    require(sealed["charge_attempt_telemetry"] == "inside_charge_invocation", "sealed:attempt")
    require(all(sealed[field] == 0 for field in ("target_after_failed_charge", "completion_observation_after_failed_charge", "publication_after_failed_charge")), "sealed:post_stop")
    require(sealed["target_execution_count_after_success"] == sealed["completion_observation_count_after_success"] == 1, "sealed:success_counts")
    require(sealed["unexpected_error_identity"] == "exact", "sealed:error")

    proof = value["proof_record"]
    required_proof_fields = {"requested_site", "observed_completed_site", "n_minus_one_result", "n_result", "n_plus_one_result", "cancelled_result", "unexpected_error_identity", "target_count_at_n_minus_one", "completion_observation_count_at_n_minus_one", "publication_count_at_n_minus_one", "charge_attempt_count_at_n_minus_one", "trace_artifact", "trace_sha256", "source_candidate", "execution_base_candidate"}
    require(required_proof_fields <= set(proof["fields"]), "proof:fields")
    require(proof["all_facts"] == "structured_trace_derived", "proof:derivation")
    require(proof["count_scope"] == "requested_site_or_post_failed_charge_suffix", "proof:scope")
    require(proof["production_path_required"] is True and proof["helper_probe_role"] == "supplemental", "proof:path")

    mutation = value["mutation"]
    require(mutation["mandatory_direct_sites"] == DIRECT_SITES, "mutation:sites")
    require(mutation["direct_kind"] == "site_local_target_hoist_and_cache" and mutation["shared_helper_unchanged"] is True, "mutation:local")
    require(mutation["site_local_property"] == "SITE_TARGET_BEFORE_CHARGE", "mutation:property")
    require(mutation["execution"] == "isolated_root_subprocess" and mutation["structural_identity_split"] is True, "mutation:execution")
    require(mutation["zero_survivors"] is True, "mutation:survivors")
    require(value["property_codes"] == PROPERTIES, "properties:exact")

    transcript = value["transcript"]
    require(transcript["commands"] == ["compile_command", "property_command", "restoration_command"], "transcript:commands")
    require({"compile_exit_status", "property_exit_status", "expected_property_code", "actual_property_code"} <= set(transcript["outcomes"]), "transcript:outcomes")
    require(transcript["execution_envelope"] == ["argv", "cwd", "environment"], "transcript:envelope")
    require(transcript["artifact_commit_binding"] == "later_catalog", "transcript:lifecycle")

    roles = value["candidate_roles"]
    require("artifact_commit" not in roles["raw_artifact_fields"], "roles:self")
    require(roles["catalog_fields"] == ["proof_artifact_commit", "mutation_artifact_commit"], "roles:catalog")
    require(roles["self_reference_allowed"] is False and roles["ancestry"] == "strict_forward", "roles:acyclic")
    require(value["distribution"] == {"selection": "transition_record_pointer", "zero_change": "reuse_existing_manifest", "synthetic_rebinding_allowed": False}, "distribution:pointer")
    require(value["independent"] == {"owner_history": "separate", "public_detail": "opaque_only", "join_after_owner_commit": True}, "independent:boundary")
    require(value["remote_actions"] == 0, "remote")


def self_test(value: Any, schema: Any) -> int:
    cases = [
        lambda v, _s: v["sealed_operation"].update(sole_preapproval_callback="completion_observer"),
        lambda v, _s: v["proof_record"].update(all_facts="fixed_labels"),
        lambda v, _s: v["proof_record"].update(count_scope="whole_trace"),
        lambda v, _s: v["mutation"].update(shared_helper_unchanged=False),
        lambda v, _s: v["mutation"].update(site_local_property="CHARGE_AFTER_OPERATION"),
        lambda v, _s: v["transcript"].update(artifact_commit_binding="self"),
        lambda v, _s: v["candidate_roles"].update(self_reference_allowed=True),
        lambda _v, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        values = [copy.deepcopy(value), copy.deepcopy(schema)]
        mutate(*values)
        try:
            validate(*values)
        except ContractError:
            caught += 1
            continue
        raise ContractError("mutation:survived")
    return caught


def main() -> int:
    value, schema = load(CONTRACT), load(SCHEMA)
    validate(value, schema)
    print(f"PASS: causal projection contracts v18 direct_sites={len(DIRECT_SITES)} properties={len(PROPERTIES)} mutations={self_test(value, schema)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
