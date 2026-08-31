#!/usr/bin/env python3
"""Validate complete structural ownership of Rust causal-projection work."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
CATALOG = ROOT / "reports/causal_projection_proof_catalog_v15.json"
REPORT = ROOT / "reports/causal_projection_source_ownership_v15.json"
SCHEMA = ROOT / "tools/validation/causal_projection_source_ownership_v15.schema.json"
SOURCE_CANDIDATE = "af63c193f6588dc3768053fecdef16858a41a0e8"
SOURCE_SHA256 = "dd9f56235cf918ed91f4f4294aa56c1b4dba0c90b10278eb0c1a725520197727"
CATALOG_SHA256 = "9a4fa04c1c3be3934d3ef40d8573c16a955eca08092dd7e2f1ec8747580a7f96"
CATALOG_CANONICAL_SHA256 = "8cf0a202666d6f53a578e3cd3abf28fa8aea337a2092eb08331f5891dc98d956"
CATALOG_IDENTITY = "cd7bde0111e0180c5841318ff412e732087798a038f01fe19b280db2e698d91b"
TOP_FIELDS = [
    "schema",
    "status",
    "source_candidate",
    "source_path",
    "source_sha256",
    "proof_catalog_sha256",
    "proof_catalog_identity_sha256",
    "phase_count",
    "operation_count",
    "phases",
    "call_graph",
    "test_only_oracles",
    "prohibited_patterns",
    "result_identity_sha256",
    "result",
]
PHASE_FIELDS = [
    "id",
    "symbol",
    "operation_enum",
    "wrapper",
    "family_count",
    "source_site_count",
    "direct_charge_count",
    "direct_observation_count",
    "function_sha256",
    "proof_rows",
]
PHASES = (
    ("construction", "build_trusted_epoch_projection_observed", "ProjectionBuildOperation", "perform_projection_build_operation"),
    ("lookup", "candidate_metered_observed", "ProjectionLookupOperation", "direct_charge_observe"),
    ("candidate_consumer", "causal_next_decision_metered_observed", "CausalNextOperation", "direct_charge_observe"),
    ("frontier", "empty_frontier_decision_metered_observed", "FrontierComparisonOperation", "metered_frontier_operation"),
)
CALL_GRAPH = [
    {"caller":"initialize_actor_states_metered","callee":"build_trusted_epoch_projection","calls":1},
    {"caller":"build_trusted_epoch_projection","callee":"build_trusted_epoch_projection_observed","calls":1},
    {"caller":"candidate_metered","callee":"candidate_metered_observed","calls":1},
    {"caller":"causal_next_decision_metered","callee":"causal_next_decision_metered_observed","calls":1},
    {"caller":"empty_frontier_decision_metered","callee":"empty_frontier_decision_metered_observed","calls":1},
]
TEST_ONLY_ORACLES = [
    "reference_apply_empty_counter",
    "reference_apply_nonempty_counter",
    "reference_causal_next_op",
    "initialize_actor_states",
]
PROHIBITED = [
    "unwrapped_target_read",
    "raw_charge_in_wrapped_phase",
    "alternate_constructor",
    "alternate_consumer_bypass",
    "charge_after_operation",
    "observation_before_operation",
    "production_reference_oracle",
    "stale_proof_binding",
]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_report_contract_v9 import ReportSuiteError, rust_code_view  # noqa: E402


class OwnershipError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise OwnershipError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def sha_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def code_view(source: str) -> str:
    try:
        return rust_code_view(source)
    except ReportSuiteError as error:
        raise OwnershipError("source:lexical") from error


def function_body(source: str, name: str) -> str:
    code = code_view(source)
    declaration = re.compile(
        rf"(?m)^[ \t]*(?:pub\s*\(\s*crate\s*\)\s+)?fn\s+{re.escape(name)}\b"
        rf"(?:[ \t\r\n]*<[^{{;]+>)?[^{{;]*\{{"
    )
    matches = tuple(declaration.finditer(code))
    require(len(matches) == 1, "function:cardinality:" + name)
    opening = matches[0].end() - 1
    depth = 0
    for cursor in range(opening, len(code)):
        if code[cursor] == "{":
            depth += 1
        elif code[cursor] == "}":
            depth -= 1
            if depth == 0:
                return code[opening + 1:cursor]
    raise OwnershipError("function:unclosed:" + name)


def snake(value: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", value).lower()


def enum_variants(source: str, name: str) -> list[str]:
    code = code_view(source)
    match = re.search(rf"\benum\s+{re.escape(name)}\s*\{{(?P<body>[^}}]+)\}}", code)
    require(match is not None, "enum:" + name)
    return [part.strip().split("(", 1)[0] for part in match.group("body").split(",") if part.strip()]


def call_spans(body: str, name: str) -> list[tuple[int, int]]:
    spans: list[tuple[int, int]] = []
    pattern = re.compile(rf"\b{re.escape(name)}\s*\(")
    for match in pattern.finditer(body):
        opening = body.find("(", match.start())
        depth = 0
        for cursor in range(opening, len(body)):
            if body[cursor] == "(":
                depth += 1
            elif body[cursor] == ")":
                depth -= 1
                if depth == 0:
                    spans.append((match.start(), cursor + 1))
                    break
        else:
            raise OwnershipError("call:unclosed:" + name)
    return spans


def without_spans(body: str, spans: list[tuple[int, int]]) -> str:
    value = list(body)
    for start, end in spans:
        for cursor in range(start, end):
            if value[cursor] not in "\r\n":
                value[cursor] = " "
    return "".join(value)


def validate_helper_order(source: str, symbol: str, operation: str) -> None:
    body = function_body(source, symbol)
    charge = body.find("charge(counter)")
    target = body.find("target()") if symbol == "metered_frontier_operation" else body.find("perform()")
    observed = body.find("observed(operation)")
    require(0 <= charge < target < observed, "helper:order:" + symbol)
    require(body.count("charge(counter)") == body.count(operation + "()") == body.count("observed(operation)") == 1, "helper:cardinality:" + symbol)


def validate_direct_phase(body: str, enum: str, variants: list[str], label: str) -> None:
    references = list(re.finditer(rf"\b{re.escape(enum)}::([A-Za-z0-9_]+)", body))
    charges = list(re.finditer(r"\bcharge\s*\(\s*WorkCounter::", body))
    observations = list(re.finditer(rf"\bobserved\s*\(\s*{re.escape(enum)}::", body))
    require([match.group(1) for match in references] == variants, "direct:variants:" + label)
    require(len(charges) == len(observations) == len(variants), "direct:cardinality:" + label)
    previous_observation = -1
    for charge, observation in zip(charges, observations, strict=True):
        require(previous_observation < charge.start() < observation.start(), "direct:order:" + label)
        previous_observation = observation.start()


def validate_call_graph(source: str) -> None:
    for edge in CALL_GRAPH:
        body = function_body(source, edge["caller"])
        require(len(call_spans(body, edge["callee"])) == edge["calls"], "call_graph:" + edge["caller"])
    production = code_view(source).split("#[cfg(test)]\npub(crate) mod tests", 1)
    require(len(production) == 2, "call_graph:test_module")
    prefix = production[0]
    require(prefix.count("build_trusted_epoch_projection_observed(") == 1, "call_graph:alternate_constructor")
    require(prefix.count(".candidate_metered_observed(") == 1, "call_graph:candidate_bypass")
    require(prefix.count(".causal_next_decision_metered_observed(") == 1, "call_graph:consumer_bypass")
    require(prefix.count(".empty_frontier_decision_metered_observed(") == 1, "call_graph:frontier_bypass")
    require(prefix.count("TrustedEpochProjection {") == 1, "call_graph:projection_constructor")


def validate_oracles(source: str) -> None:
    code = code_view(source)
    for name in TEST_ONLY_ORACLES:
        pattern = rf"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*(?:pub\s*\(\s*crate\s*\)\s*)?fn\s+{re.escape(name)}\b"
        require(len(re.findall(pattern, code)) == 1, "oracle:test_only:" + name)


def phase_rows(source: str, catalog: dict) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for phase, symbol, operation_enum, wrapper in PHASES:
        body = function_body(source, symbol)
        variants = enum_variants(source, operation_enum)
        references = re.findall(rf"\b{re.escape(operation_enum)}::([A-Za-z0-9_]+)", body)
        proof_rows = [row["id"] for row in catalog["rows"] if row["phase"] == phase]
        require(proof_rows == [f"{phase}.{snake(value)}" for value in variants], "proof:phase:" + phase)
        if wrapper == "direct_charge_observe":
            validate_direct_phase(body, operation_enum, variants, phase)
            wrapper_calls = 0
            direct_charges = len(variants)
            observations = len(variants)
        else:
            spans = call_spans(body, wrapper)
            call_variants = []
            for start, end in spans:
                found = re.findall(rf"\b{re.escape(operation_enum)}::([A-Za-z0-9_]+)", body[start:end])
                require(len(found) == 1, "wrapper:operation:" + phase)
                call_variants.extend(found)
            require(call_variants == references, "wrapper:coverage:" + phase)
            outside = without_spans(body, spans)
            require("charge(WorkCounter::" not in outside, "wrapper:raw_charge:" + phase)
            if phase == "construction":
                forbidden = re.compile(r"\bsource\.(?:member_count|next_member|accepted_member|candidate|dependency_count|dependency)\s*\(")
                require(forbidden.search(outside) is None, "wrapper:unwrapped_target_read")
                require("TrustedEpochProjection {" not in outside, "wrapper:alternate_constructor")
            wrapper_calls = len(spans)
            direct_charges = 0
            observations = 0
        require(set(references) == set(variants), "source:reachable:" + phase)
        rows.append({
            "id":phase,
            "symbol":symbol,
            "operation_enum":operation_enum,
            "wrapper":wrapper,
            "family_count":len(variants),
            "source_site_count":len(references),
            "direct_charge_count":direct_charges,
            "direct_observation_count":observations,
            "function_sha256":sha_bytes(body.encode()),
            "proof_rows":proof_rows,
        })
        require(wrapper_calls == (len(references) if wrapper != "direct_charge_observe" else 0), "wrapper:count:" + phase)
    return rows


def structural_report(source: str, catalog: dict) -> dict[str, object]:
    validate_helper_order(source, "perform_projection_build_operation", "perform")
    validate_helper_order(source, "metered_frontier_operation", "target")
    validate_call_graph(source)
    validate_oracles(source)
    phases = phase_rows(source, catalog)
    value: dict[str, object] = {
        "schema":"nostr_automerge.causal_projection_source_ownership.v15.v1",
        "status":"pass",
        "source_candidate":SOURCE_CANDIDATE,
        "source_path":"crates/nostr_automerge/src/graph/actor_state.rs",
        "source_sha256":SOURCE_SHA256,
        "proof_catalog_sha256":CATALOG_SHA256,
        "proof_catalog_identity_sha256":CATALOG_IDENTITY,
        "phase_count":4,
        "operation_count":43,
        "phases":phases,
        "call_graph":CALL_GRAPH,
        "test_only_oracles":TEST_ONLY_ORACLES,
        "prohibited_patterns":PROHIBITED,
        "result_identity_sha256":"",
        "result":"pass",
    }
    identity = {key:item for key,item in value.items() if key != "result_identity_sha256"}
    value["result_identity_sha256"] = sha_bytes(canonical(identity))
    return value


def validate(report: object, schema: object, source: str, catalog: object) -> None:
    require(type(catalog) is dict, "catalog:shape")
    require(sha_bytes(canonical(catalog)) == CATALOG_CANONICAL_SHA256, "catalog:canonical_sha256")
    require(sha_bytes(CATALOG.read_bytes()) == CATALOG_SHA256, "catalog:sha256")
    require(catalog.get("result_identity_sha256") == CATALOG_IDENTITY, "catalog:identity")
    expected = structural_report(source, catalog)
    require(type(report) is dict and list(report) == TOP_FIELDS and report == expected, "report:value")
    require(sha_bytes(source.encode()) == SOURCE_SHA256, "source:sha256")
    resolved = subprocess.run(["git","rev-parse","--verify",SOURCE_CANDIDATE + "^{commit}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == SOURCE_CANDIDATE, "source:candidate")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "schema:closed")
    require(schema["$defs"]["phase"]["required"] == PHASE_FIELDS, "schema:phase")


def replace_in_function(source: str, symbol: str, before: str, after: str) -> str:
    body = function_body(source, symbol)
    require(body.count(before) > 0, "mutation:anchor:" + symbol)
    return source.replace(body, body.replace(before, after, 1), 1)


def self_test(report: dict, schema: dict, source: str, catalog: dict) -> int:
    source_cases = [
        ("unwrapped_read", replace_in_function(source,"build_trusted_epoch_projection_observed","let member_count =", "let _ = source.member_count();\n    let member_count =")),
        ("wrapper_bypass", replace_in_function(source,"build_trusted_epoch_projection_observed","perform_projection_build_operation(", "unmetered_projection_build_operation(")),
        ("alternate_constructor", source.replace("#[cfg(test)]\npub(crate) mod tests", "fn alternate_constructor() { let _ = TrustedEpochProjection { todo!() }; }\n#[cfg(test)]\npub(crate) mod tests",1)),
        ("alternate_entry", source.replace("#[cfg(test)]\npub(crate) mod tests", "fn alternate_entry() { build_trusted_epoch_projection_observed(); }\n#[cfg(test)]\npub(crate) mod tests",1)),
        ("helper_reorder", replace_in_function(source,"perform_projection_build_operation","charge(counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();", "let result = perform();\n    charge(counter).map_err(MeteredActorStateError::Work)?;")),
        ("lookup_charge", replace_in_function(source,"candidate_metered_observed","charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;", "")),
        ("consumer_observation", replace_in_function(source,"causal_next_decision_metered_observed","observed(CausalNextOperation::StoredCounterRead);", "")),
        ("frontier_wrapper", replace_in_function(source,"empty_frontier_decision_metered_observed","metered_frontier_operation(", "unmetered_frontier_operation(")),
        ("oracle_exposed", source.replace("#[cfg(test)]\nfn reference_apply_empty_counter", "fn reference_apply_empty_counter",1)),
    ]
    caught = 0
    for label, changed_source in source_cases:
        try:
            structural_report(changed_source, catalog)
        except OwnershipError:
            caught += 1
            continue
        raise OwnershipError("source_mutation_survived:" + label)
    benign = "// source.member_count(); build_trusted_epoch_projection_observed();\n" + source + '\nconst DECOY: &str = r#"TrustedEpochProjection { source.member_count(); }"#;\n'
    structural_report(benign, catalog)
    cases = [
        ("missing_phase","report",lambda value: value["phases"].pop()),
        ("phase_order","report",lambda value: value["phases"].reverse()),
        ("family_count","report",lambda value: value["phases"][0].update(family_count=19)),
        ("proof_row","report",lambda value: value["phases"][0]["proof_rows"].pop()),
        ("call_graph","report",lambda value: value["call_graph"].pop()),
        ("oracle","report",lambda value: value["test_only_oracles"].pop()),
        ("identity","report",lambda value: value.update(result_identity_sha256="0"*64)),
        ("catalog","catalog",lambda value: value["rows"].pop()),
        ("schema","schema",lambda value: value.update(additionalProperties=True)),
    ]
    for label, target, mutate in cases:
        changed_report=copy.deepcopy(report); changed_schema=copy.deepcopy(schema); changed_catalog=copy.deepcopy(catalog)
        if target == "report": mutate(changed_report)
        elif target == "catalog": mutate(changed_catalog)
        else: mutate(changed_schema)
        try:
            if target == "catalog":
                structural_report(source, changed_catalog)
                require(changed_catalog.get("result_identity_sha256") == CATALOG_IDENTITY, "catalog:mutation")
            else:
                expected = structural_report(source, changed_catalog)
                require(changed_report == expected, "report:mutation")
                require(changed_schema.get("additionalProperties") is False, "schema:mutation")
        except OwnershipError:
            caught += 1
            continue
        raise OwnershipError("mutation_survived:" + label)
    return caught


def main() -> int:
    parser=argparse.ArgumentParser(); parser.add_argument("--write-report",action="store_true"); args=parser.parse_args()
    source=SOURCE.read_text(); catalog=json.loads(CATALOG.read_text()); expected=structural_report(source,catalog)
    if args.write_report:
        REPORT.write_text(json.dumps(expected,ensure_ascii=True,indent=2)+"\n")
    report=json.loads(REPORT.read_text()); schema=json.loads(SCHEMA.read_text())
    validate(report,schema,source,catalog); mutations=self_test(report,schema,source,catalog)
    print(f"PASS: causal projection source ownership phases=4 operations=43 mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
