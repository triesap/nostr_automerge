#!/usr/bin/env python3
"""Candidate-aware bootstrap validation; historical artifacts are never rewritten."""
from __future__ import annotations

import ast
import hashlib
from functools import lru_cache
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from contextlib import contextmanager

ROOT = Path(__file__).resolve().parents[2]
ANCHOR = "e2faf8fea65f23479a70009c5d9531b881838c0e"
REGISTRY = "spec/nostr_automerge_alignment_refactor_v1/validation_registry.json"
HISTORICAL = {"scripts/validate_spec.py", "scripts/run_causal_projection_clean_attestation_v19.py"}
FROZEN_PREFIXES = ("scripts/", "reports/", "spec/", "fixtures/", "tools/validation/")


def require(condition: bool, reason: str) -> None:
    if not condition:
        raise ValueError(reason)


def git(root: Path, *args: str) -> bytes:
    return subprocess.run(["git", *args], cwd=root, capture_output=True, check=True).stdout


@lru_cache(maxsize=4)
def anchor_blobs(root: Path, anchor: str) -> dict[str, bytes]:
    entries = [entry.split(b"\t", 1) for entry in git(root, "ls-tree", "-r", "-z", anchor).split(b"\0") if entry]
    blobs = [(os.fsdecode(path), metadata.split()[2]) for metadata, path in entries if metadata.split()[1] == b"blob"]
    result = subprocess.run(["git", "cat-file", "--batch"], cwd=root, input=b"".join(oid + b"\n" for _path, oid in blobs), capture_output=True, check=True).stdout
    offset = 0
    contents = {}
    for path, _oid in blobs:
        end = result.index(b"\n", offset)
        header = result[offset:end].split()
        require(len(header) == 3 and header[1] == b"blob", "missing historical blob: " + path)
        length = int(header[2])
        offset = end + 1
        contents[path] = result[offset:offset + length]
        offset += length + 1
    return contents


def committed(root: Path, path: str) -> bytes:
    return anchor_blobs(root, ANCHOR)[path]


def digest(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def load(path: Path) -> dict:
    def closed(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate JSON key: " + key)
            result[key] = value
        return result
    return json.loads(path.read_text(), object_pairs_hook=closed)


def original_inventory(root: Path) -> tuple[list[dict], list[str], list[dict]]:
    rust = committed(root, "tools/nostr_automerge_xtask/src/validate.rs").decode()
    pairs = re.findall(r'\(\s*"([^"\n]+)",\s*"([^"\n]+)"\s*,?\s*\)', rust.split("#[derive")[0])
    excluded = set()
    for name in ("followup", "v12", "v13", "v18"):
        body = rust.split("fn " + name + "_historical_validator")[1].split("\n}")[0]
        excluded.update(re.findall(r'"([^"]+)"', body))
    rust_active = {path for name, path in pairs if name not in excluded and name != "remediation_v18" and not ("causal_projection" in name and "_v19" not in name)}
    spec_source = committed(root, "scripts/validate_spec.py").decode()
    values = {}
    for node in ast.parse(spec_source).body:
        if isinstance(node, ast.Assign) and isinstance(node.targets[0], ast.Name):
            name = node.targets[0].id
            if name == "VALIDATORS" or name.endswith("HISTORICAL_VALIDATORS"):
                if name == "V19_HISTORICAL_VALIDATORS":
                    values[name] = ast.literal_eval(node.value.left) | {item for item in values["VALIDATORS"] if "causal_projection" in item and "_v19" not in item}
                else:
                    values[name] = ast.literal_eval(node.value)
    removed = set().union(*(value for name, value in values.items() if name != "VALIDATORS"))
    spec_active = {"scripts/" + name for name in values["VALIDATORS"] if name not in removed}
    rows = []
    for entrypoint, declarations, active in (("xtask", pairs, rust_active), ("complete_specification", [(name.removesuffix(".py"), "scripts/" + name) for name in values["VALIDATORS"]], spec_active)):
        for name, path in declarations:
            selected = path in active
            source = committed(root, path)
            literals = re.findall(r'[\"\']([^\"\'\n]+)[\"\']', source.decode())
            inputs = sorted({item for item in literals if item in anchor_blobs(root, ANCHOR)})
            rows.append({"id": entrypoint + ":" + name, "entrypoint": entrypoint, "path": path, "validator_sha256": digest(source), "literal_input_bindings": [{"path": item, "anchor_sha256": digest(committed(root, item))} for item in inputs], "declared_revision_literals": sorted(set(re.findall(r'(?<![0-9a-f])[0-9a-f]{40}(?![0-9a-f])', source.decode()))), "selected": selected, "route": ("historical" if path in HISTORICAL else "current") if selected else "inactive_historical", "evidence_kind": "receipt_validation" if path in HISTORICAL or not selected else "current_invariant", "argv": ["python3", path]})
    selected = sorted(rust_active | spec_active)
    require(len(pairs) == 171 and len(values["VALIDATORS"]) == 207 and len(selected) == 57, "anchor inventory changed")
    local = committed(root, "scripts/local_gate.py").decode()
    functions = {node.name: ast.get_source_segment(local, node) for node in ast.parse(local).body if isinstance(node, ast.FunctionDef)}
    jobs = []
    for node in ast.parse(local).body:
        if isinstance(node, ast.Assign) and isinstance(node.targets[0], ast.Name) and node.targets[0].id == "JOBS":
            jobs = [{"id": key.value, "argv": ["python3", "scripts/local_gate.py", key.value], "function_source": functions[value.id], "execution": "separate_explicit_lane"} for key, value in zip(node.value.keys, node.value.values)]
    return rows, selected, jobs


def expected_registry(root: Path) -> dict:
    rows, selected, jobs = original_inventory(root)
    return {"schema": "nostr_automerge.alignment_validation_registry.v1", "historical_validation_anchor": ANCHOR, "declarations": rows, "selected_python_paths": selected, "current_python_paths": [path for path in selected if path not in HISTORICAL], "historical_command": ["python3", "scripts/validate_spec.py"], "historical_covered_paths": sorted(HISTORICAL), "current_builtin_obligations": ["requirement_coverage", "alignment_plan", "candidate_input_fingerprint", "frozen_artifact_integrity", "output_root_policy"], "local_jobs": jobs, "frozen_prefixes": list(FROZEN_PREFIXES), "mutable_existing_paths": ["AGENTS.md", "implementation/COMMIT_SEQUENCE.md", "tools/nostr_automerge_xtask/src/main.rs", "tools/nostr_automerge_xtask/src/validate.rs"], "future_routing": "requires_explicit_current_replacement_and_reviewed_registry_revision"}


def validate_registry(root: Path, value: dict) -> list[str]:
    require(value == expected_registry(root), "registry omission, unauthorized route, command, or classification")
    return value["current_python_paths"]


def validate_candidate_bindings(root: Path) -> None:
    authority = load(root / "spec/nostr_automerge_alignment_refactor_v1/authority.json")
    plan = load(root / "implementation/nostr_automerge_alignment_refactor_v1/steps.json")
    require(authority.get("baseline_commit") == ANCHOR, "alignment baseline differs from validation anchor")
    require(authority.get("reviewed_commit") == plan.get("baseline"), "alignment reviewed candidate differs from adopted plan")


def frozen_artifacts(root: Path) -> None:
    entries = git(root, "ls-tree", "-r", "-z", ANCHOR).split(b"\0")
    for entry in entries:
        if not entry:
            continue
        metadata, encoded_path = entry.split(b"\t", 1)
        path = os.fsdecode(encoded_path)
        if path.startswith(FROZEN_PREFIXES):
            mode, kind, oid = metadata.split()
            require(kind == b"blob" and mode != b"120000", "unsupported frozen artifact: " + path)
            actual = root / path
            require(actual.is_file() and actual.resolve() == actual, "missing or aliased frozen artifact: " + path)
            require(actual.read_bytes() == committed(root, path), "altered frozen artifact: " + path)


def output_root(root: Path, raw: str | None) -> Path:
    candidate = Path(raw) if raw is not None else root / ".local/evidence"
    require(".." not in candidate.parts, "output root traversal")
    if not candidate.is_absolute():
        candidate = root / candidate
    require(candidate == candidate.resolve(), "output root symlink or path alias")
    source = root.resolve()
    require(candidate != source and candidate not in source.parents, "output root is source or ancestor")
    if candidate.is_relative_to(source):
        require(candidate == source / ".local/evidence", "output root overlaps source")
        result = subprocess.run(["git", "check-ignore", "--quiet", "--", ".local/evidence/probe"], cwd=root)
        require(result.returncode == 0, "standalone output is not ignored")
    return candidate


def input_fingerprint(root: Path) -> dict:
    paths = sorted(set(git(root, "ls-files", "-z", "--cached", "--others", "--exclude-standard").split(b"\0")) - {b""})
    aggregate = hashlib.sha256()
    for encoded in paths:
        path = root / os.fsdecode(encoded)
        require(not path.is_symlink(), "symlink current input: " + os.fsdecode(encoded))
        require(path.is_file(), "missing current input: " + os.fsdecode(encoded))
        content = path.read_bytes()
        for item in (encoded, content):
            aggregate.update(len(item).to_bytes(8, "big"))
            aggregate.update(item)
    return {"head": git(root, "rev-parse", "HEAD").decode().strip(), "input_count": len(paths), "sha256": aggregate.hexdigest(), "evidence_kind": "current_input_observation"}


def validate_observation(root: Path, observation: dict) -> None:
    require(observation == input_fingerprint(root), "stale current candidate or input observation")


@contextmanager
def historical_checkout(root: Path):
    # Outside source; no retained recovery clone, no historical build artifacts.
    with tempfile.TemporaryDirectory(prefix="nostr_automerge_history_") as temporary:
        checkout = (Path(temporary) / "candidate").resolve()
        require(not checkout.is_relative_to(root.resolve()), "historical checkout must be physically outside source")
        subprocess.run(["git", "worktree", "add", "--quiet", "--detach", str(checkout), ANCHOR], cwd=root, check=True)
        try:
            require(git(checkout, "rev-parse", "HEAD").decode().strip() == ANCHOR, "wrong historical checkout")
            require(git(checkout, "status", "--porcelain=v1", "--untracked-files=all") == b"", "dirty historical checkout")
            yield checkout
            require(git(checkout, "status", "--porcelain=v1", "--untracked-files=all") == b"", "historical validator modified checkout")
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(checkout)], cwd=root, check=True)


def run_python(root: Path, path: str, output: Path) -> None:
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", NOSTR_AUTOMERGE_OUTPUT_ROOT=str(output))
    subprocess.run(["python3", path], cwd=root, env=environment, check=True)


def validate(root: Path) -> dict:
    root = root.resolve()
    output = output_root(root, os.environ.get("NOSTR_AUTOMERGE_OUTPUT_ROOT"))
    current = validate_registry(root, load(root / REGISTRY))
    validate_candidate_bindings(root)
    frozen_artifacts(root)
    before = input_fingerprint(root)
    run_python(root, "scripts/alignment_v1/validate_plan.py", output)
    with historical_checkout(root) as historical:
        run_python(historical, "scripts/validate_spec.py", output)
    for path in current:
        run_python(root, path, output)
    validate_observation(root, before)
    return {"schema": "nostr_automerge.alignment_validation_observation.v1", "current_candidate": before, "current_python_count": len(current), "historical_validation_anchor": ANCHOR, "historical_execution_kind": "receipt_validation", "runtime_campaigns_executed": False, "result": "pass"}


def main() -> None:
    print(json.dumps(validate(ROOT), sort_keys=True))


if __name__ == "__main__":
    main()
