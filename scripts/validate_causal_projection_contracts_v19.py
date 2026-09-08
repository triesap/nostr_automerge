#!/usr/bin/env python3
"""Validate the closed v19 proof, mutation, and property contracts."""

from __future__ import annotations

import copy
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "spec/causal_projection_contracts_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_contracts_v19.schema.json"


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


def validate(contract: dict[str, Any], schema: dict[str, Any]) -> None:
    fields = list(schema["required"])
    require(schema["additionalProperties"] is False, "schema:closed")
    require(list(contract) == fields, "contract:fields")
    require(contract["schema"] == "nostr_automerge.causal_projection_contracts.v19.v1", "contract:schema")
    require(contract["proof_events"] == ["ChargeAttempt", "ChargeAccepted", "TargetDispatched", "TargetReturned", "CompletionObserved", "PublicationCompleted"], "contract:events")
    require(contract["successful_order"] == contract["proof_events"][:5], "contract:success_order")
    require(contract["failed_charge_order"] == ["ChargeAttempt"], "contract:failed_order")
    require(len(contract["helpers"]) == 4 and len(set(contract["helpers"])) == 4, "contract:helpers")
    require(len(contract["helper_mutations"]) == 7 and len(set(contract["helper_mutations"])) == 7, "contract:helper_mutations")
    require(len(contract["direct_sites"]) == 7 and len(set(contract["direct_sites"])) == 7, "contract:direct_sites")
    require(len(contract["provenance_mutations"]) == 5 and len(set(contract["provenance_mutations"])) == 5, "contract:provenance")
    require(len(contract["property_codes"]) == 13 and len(set(contract["property_codes"])) == 13, "contract:properties")
    require(contract["public_minimum_mutations"] == 4 * 7 + 7 + 5 == 40, "contract:public_count")
    require(contract["independent_minimum_mutations"] == 7 + 7 + 5 == 19, "contract:independent_count")
    require(contract["source_derived_counts"] is True, "contract:source_derived")
    require(contract["marker_selected_properties"] is False, "contract:markers")
    require(contract["identity_only_mutation_kills"] is False, "contract:identity_kills")
    require(contract["result"] == "pass", "contract:result")


def self_test(values: list[dict[str, Any]]) -> int:
    cases = [
        lambda c, _s: c["proof_events"].remove("TargetReturned"),
        lambda c, _s: c.update(failed_charge_order=["ChargeAttempt", "TargetDispatched"]),
        lambda c, _s: c["helpers"].pop(),
        lambda c, _s: c["helper_mutations"].pop(),
        lambda c, _s: c.update(public_minimum_mutations=39),
        lambda c, _s: c.update(independent_minimum_mutations=28),
        lambda c, _s: c.update(marker_selected_properties=True),
        lambda c, _s: c.update(identity_only_mutation_kills=True),
        lambda _c, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        changed = copy.deepcopy(values)
        mutate(*changed)
        try:
            validate(*changed)
        except ContractError:
            caught += 1
            continue
        raise ContractError("self_test:survivor")
    return caught


def main() -> int:
    values = [load(CONTRACT), load(SCHEMA)]
    validate(*values)
    print(f"PASS v19 causal projection contracts: public=40 independent=19 attacks={self_test(values)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
