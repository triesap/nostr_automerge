#!/usr/bin/env python3
"""Validate combined public and opaque independent v19 assurance."""

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
REPORT = ROOT / "reports/causal_projection_combined_assurance_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_combined_assurance_v19.schema.json"
BASE = "493d7f8b4e97dc548e350bbe591cab24e75e1cc6"
FIELDS = ["schema","status","rcld","base_candidate","imports","candidate_roles","counts","applicability","identities","finding_closure","result_classes","canonical_process_bytes","public_api_changed","protocol_changed","release_claimed","publication_claimed","remote_actions","result","result_identity_sha256"]
PATHS = {
    "operation_contract_sha256": ("a8f8651c26a0ac7822e1e5d61f3aafa3fec53d1f", "spec/causal_projection_contracts_v19.json"),
    "public_proofs_sha256": ("7c5a1603c4ee9f92d541a8ff6fd66071806518df", "reports/causal_projection_proofs_v19.json"),
    "public_mutations_sha256": ("001bea9cb5edc8fb83aa972901f749f4707d59e1", "reports/causal_projection_mutations_v19.json"),
    "public_catalogs_sha256": ("266e80b67e13ea377ea874bbd4b75ac4ca5898fa", "reports/causal_projection_catalogs_v19.json"),
    "public_final_inventory_sha256": ("b489ef551ace84791f3d513dd5add0a5729b53a7", "reports/causal_projection_final_inventory_v19.json"),
    "public_evidence_graph_sha256": ("ac24566ae4b96bc1ac17b0c87b7ae70cbb9f939f", "reports/causal_projection_evidence_graph_v19.json"),
    "public_qualification_sha256": ("0735f7e23524764c889f4ec0cc9942c9e795740a", "reports/causal_projection_public_qualification_v19.json"),
    "public_assurance_sha256": ("156194f6453bd4f31da4eda512cf51361ec7c978", "reports/causal_projection_public_assurance_v19.json"),
    "distribution_transition_sha256": ("53f0bf69809cfdf2c59393dc9c3f696940ff7554", "spec/distribution_v19_transition.json"),
    "opaque_import_sha256": ("656fcb03d6241496c775db721555753510ddf6e7", "reports/opaque_causal_projection_v19.json"),
    "finding_registry_sha256": (BASE, "spec/remediation_findings_v19.json"),
    "runtime_ledger_sha256": (BASE, "implementation/runtime_ledger_v19.json"),
}


class CombinedError(RuntimeError): pass
def require(value: bool, code: str) -> None:
    if not value: raise CombinedError(code)
def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
def committed_sha(candidate: str, path: str) -> str:
    run = subprocess.run(["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True)
    require(run.returncode == 0, "binding:" + path)
    return hashlib.sha256(run.stdout).hexdigest()
def load(path: Path) -> Any:
    return json.loads(path.read_text())


def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS, "shape")
    require(record["schema"] == "nostr_automerge.causal_projection_combined_assurance.v19.v1" and record["status"] == "verified" and record["rcld"] == 146 and record["base_candidate"] == BASE and record["result"] == "pass", "state")
    for name, (candidate, path) in PATHS.items(): require(record["imports"][name] == committed_sha(candidate, path), "import:" + name)
    counts = record["counts"]
    require(counts["public_operation_sites"] == counts["public_site_proofs"] == 68 and counts["public_mutations"] == 40 and counts["public_mutation_edges"] == 488, "counts:public")
    require(counts["independent_operation_sites"] == counts["independent_site_proofs"] == 142 and counts["independent_mutations"] == 19 and counts["independent_mutation_edges"] == 1711, "counts:independent")
    require(counts["combined_operation_sites"] == counts["combined_site_proofs"] == 210 and counts["combined_mutations"] == 59 and counts["combined_mutation_edges"] == 2199 and counts["mutation_survivors"] == 0, "counts:combined")
    require([row["id"] for row in record["finding_closure"]] == [f"FINDING_{n}" for n in range(130, 134)] and all(row["status"] == "closed" and len(row["evidence"]) == 3 for row in record["finding_closure"]), "findings")
    require(record["canonical_process_bytes"] == "identical" and record["public_api_changed"] is False and record["protocol_changed"] is False and record["release_claimed"] is False and record["publication_claimed"] is False and record["remote_actions"] == 0, "result")
    require(record["result_identity_sha256"] == hashlib.sha256(canonical({key: record[key] for key in FIELDS[:-1]})).hexdigest(), "identity")
    require(schema.get("additionalProperties") is False and schema.get("required") == FIELDS and list(schema.get("properties", {})) == FIELDS, "schema")


def main() -> int:
    record, schema = load(REPORT), load(SCHEMA); validate(record, schema)
    attacks: list[Callable[[dict[str, Any]], None]] = [
        lambda v: v["imports"].update(opaque_import_sha256="0" * 64), lambda v: v["counts"].update(public_mutations=39),
        lambda v: v["counts"].update(independent_mutations=18), lambda v: v["counts"].update(mutation_survivors=1),
        lambda v: v["finding_closure"].pop(), lambda v: v["finding_closure"][0].update(status="open"),
        lambda v: v.update(protocol_changed=True), lambda v: v.update(publication_claimed=True),
        lambda v: v.update(remote_actions=1), lambda v: v.update(result_identity_sha256="0" * 64), lambda v: v.update(extra=False),
    ]
    caught = 0
    for attack in attacks:
        changed = copy.deepcopy(record); attack(changed)
        try: validate(changed, schema)
        except CombinedError: caught += 1; continue
        raise CombinedError("attack:survived")
    print(f"PASS: causal projection combined assurance v19 sites=68+142 mutations=40+19 survivors=0 attacks={caught}")
    return 0


if __name__ == "__main__": raise SystemExit(main())
