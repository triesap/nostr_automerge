#!/usr/bin/env python3
"""Validate the v18 completion record before clean-descendant attestation."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_completion_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_completion_v18.schema.json"
CANDIDATE = "7150c33febcd0227484af4d95b2decf1c83ef6f8"
BASE = "8673ff8546b9e9d57218c15a4b81890d82137184"
CANONICAL = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
SERIALIZED = "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"
FIELDS = [
    "schema", "status", "rcld", "candidate", "imports", "sequence", "counts",
    "findings", "verification", "self_review", "unverified_items", "deviations",
    "repository_status", "next_checkpoint", "holds", "release_claimed",
    "publication_claimed", "remote_actions", "result", "result_identity_sha256",
]
IMPORTS = {
    "plan_sha256": "68be685d23c90641f58a3f3fa50c7b836b2febd2bf156fb9bb2c45958079b596",
    "authority_sha256": "c45b241c6a300ab3bb3a6120ba5fac3b53cb4a3589347ed4e1edda8b17923caa",
    "finding_registry_sha256": "e7d39002597ae53c0cd1cd6a4247bd53903514682ccc709bdc11771faf51c76b",
    "runtime_ledger_sha256": "1b8ad265fd9db38f809bec0b98869df44db6e1fc988a3bcbd65c6bb0e6b78e4f",
    "combined_assurance_sha256": "44a6bdb0add32ff92dfd43d89339f5c6860ef4bc7e7e3497eb568aa36eed422a",
    "finding_closure_sha256": "deb203c98556a3cc139652becd6d8d93291db47824625fc93f32fccccdc9d4b9",
    "opaque_import_sha256": "7c65ac14d8fa4a1ef72c5a26ddac50c0b6629d0839be2ca6e459348143f6d8b6",
    "public_qualification_sha256": "943622deb233fe3f597f71b159b58f23349bd5c4b3f156c7306c7cf8282f3bcc",
    "distribution_transition_sha256": "1408b71c6e7ee31a99e6e0436c4ed290467675a67f517bc0be082b10149a5153",
    "evidence_graph_sha256": "48a82ead9b1baf911638651191e2592df3f6ce259077ffc77642c39d8636a9e5",
}
PATHS = {
    "plan_sha256": "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v18.md",
    "authority_sha256": "spec/remediation_v18_authority.json",
    "finding_registry_sha256": "spec/remediation_findings_v18.json",
    "runtime_ledger_sha256": "implementation/runtime_ledger_v18.json",
    "combined_assurance_sha256": "reports/causal_projection_combined_assurance_v18.json",
    "finding_closure_sha256": "reports/causal_projection_finding_closure_v18.json",
    "opaque_import_sha256": "reports/opaque_causal_projection_v18.json",
    "public_qualification_sha256": "reports/causal_projection_public_qualification_v18.json",
    "distribution_transition_sha256": "spec/distribution_v18_transition.json",
    "evidence_graph_sha256": "reports/causal_projection_evidence_graph_v18.json",
}
COUNTS = {
    "public_operation_sites": 68, "public_site_proofs": 68,
    "independent_operation_sites": 142, "independent_site_proofs": 142,
    "public_mutations": 21, "independent_mutations": 19, "combined_mutations": 40,
    "mutation_survivors": 0, "scenarios": 204, "signed_events": 771,
    "delivery_orders": 8, "processes_per_implementation": 2,
    "transition_affected": 0,
}
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
]


class CompletionError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise CompletionError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)
    return json.loads(path.read_text(), object_pairs_hook=closed)


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS, "shape")
    require(
        record["schema"] == "nostr_automerge.causal_projection_completion.v18.v1"
        and record["status"] == "code_complete_publication_held"
        and record["rcld"] == 140 and record["candidate"] == CANDIDATE
        and record["result"] == "pass",
        "state",
    )
    require(record["imports"] == IMPORTS, "imports")
    for name, path in PATHS.items():
        require(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == IMPORTS[name], "import:" + name)
    sequence = record["sequence"]
    require(sequence["rclds"] == list(range(134, 141)) and sequence["unfinished_rclds"] == [], "sequence:rclds")
    require(sequence["public_checkpoints"] == 19 and sequence["independent_checkpoints"] == 17, "sequence:counts")
    require(sequence["public_first_candidate"] == "7156f309ef0ffa5d0b73ba050a81ebf3046acf0d", "sequence:first")
    require(sequence["public_terminal_base_candidate"] == CANDIDATE, "sequence:terminal")
    require(sequence["independent_first_candidate"] == "ab98f134278ec62c6bafa9fb5f4019a990c97e11", "sequence:independent_first")
    require(sequence["independent_assurance_candidate"] == "5ecb65582555ac27c89bbed5f7d551b69b68b04a", "sequence:independent")
    require(sequence["candidate_lifecycle"] == "acyclic_later_attestation", "sequence:lifecycle")
    process = subprocess.run(
        [chr(103) + "it", "rev-list", "--count", f"{BASE}..{CANDIDATE}"],
        cwd=ROOT, capture_output=True, text=True, check=False,
    )
    require(process.returncode == 0 and process.stdout.strip() == "19", "sequence:public_count")
    require(record["counts"] == COUNTS, "counts")
    require(
        record["findings"] == {
            "closed": [f"FINDING_{number}" for number in range(123, 130)],
            "held": ["FINDING_080"], "open": [],
        },
        "findings",
    )
    verification = record["verification"]
    require(
        verification == {
            "qualification_status": "pass", "qualification_gate_jobs": 9,
            "qualification_processes": 2, "post_terminal_standard_runs_required": 2,
            "post_terminal_conformance_runs_required": 2, "post_terminal_runs_completed": 0,
            "canonical_process_bytes": "identical",
            "deliberate_expectation_mismatch": "rejected",
            "canonical_output_sha256": CANONICAL, "serialized_run_sha256": SERIALIZED,
        },
        "verification",
    )
    require(
        record["self_review"] == "pass"
        and record["unverified_items"] == ["post_terminal_clean_descendant_double_gate"]
        and record["deviations"] == []
        and record["repository_status"] == "clean_candidate_attestation_required_later"
        and record["next_checkpoint"] == "clean_candidate_attestation",
        "handoff",
    )
    require(
        record["holds"] == HOLDS and record["release_claimed"] is False
        and record["publication_claimed"] is False and record["remote_actions"] == 0,
        "holds",
    )
    require(
        schema.get("additionalProperties") is False and schema.get("required") == FIELDS
        and list(schema.get("properties", {})) == FIELDS,
        "schema",
    )
    require(
        record["result_identity_sha256"]
        == hashlib.sha256(canonical({key: record[key] for key in FIELDS[:-1]})).hexdigest(),
        "identity",
    )


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks: list[Callable[[dict[str, Any]], None]] = [
        lambda value: value["imports"].update(authority_sha256="0" * 64),
        lambda value: value["sequence"]["rclds"].pop(),
        lambda value: value["sequence"].update(unfinished_rclds=[140]),
        lambda value: value["counts"].update(mutation_survivors=1),
        lambda value: value["findings"]["closed"].pop(),
        lambda value: value["findings"]["held"].clear(),
        lambda value: value["verification"].update(post_terminal_runs_completed=2),
        lambda value: value.update(unverified_items=[]),
        lambda value: value.update(publication_claimed=True),
        lambda value: value.update(remote_actions=1),
        lambda value: value.update(result_identity_sha256="0" * 64),
        lambda value: value.update(extra=False),
    ]
    caught = 0
    for mutate in attacks:
        changed = copy.deepcopy(record)
        mutate(changed)
        try:
            validate(changed, schema)
        except CompletionError:
            caught += 1
            continue
        raise CompletionError("attack:survived")
    return caught


def main() -> int:
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(f"PASS: causal projection completion v18 rclds=7 findings=7 held=1 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
