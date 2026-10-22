#!/usr/bin/env python3
"""Validate alignment ordering and source-before-observation evidence (stdlib only)."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

PROGRAM = "nostr_automerge_alignment_refactor_v1"
SPEC = f"spec/{PROGRAM}"
IMPL = f"implementation/{PROGRAM}"
HOLDS = ["external_assurance", "event_kind_allocation", "nip_submission",
         "production_qualification", "publication", "release", "deployment", "remote_mutation"]
BOOTSTRAP = [f"NA-{i:03}" for i in range(1, 6)]
# Approved adopted contracts, including the recorded NA-149 Rust 1.99.0
# deviation. Future semantic changes require an evidenced controlled deviation
# and explicit review of these anchors; formatting changes do not change them.
CONTRACT_DIGESTS = {
    "steps": "7baa54a964eb8948b8bfa4b80bb7491fc8bcb3933ac279e51fe4dcf4e278a1d2",
    "requirements": "fff9b31ba45fe36eb0eca14e236e321e5bc8f955c3ae0eaf83551dc10e95e24e",
}
AUTHORITY_KEYS = {"schema", "program", "reviewed_commit", "baseline_commit", "governing_plan",
                  "requirements", "steps", "ledger", "validation_registry", "holds", "bootstrap_deviation"}
LEDGER_KEYS = {"schema", "program", "active_step", "next_step", "holds", "bootstrap", "steps"}
STEP_KEYS = {"id", "alias", "ordinal", "requirements", "depends_on", "status", "report"}
REPORT_KEYS = {"source_commit", "observation_base", "files", "commands", "acceptance", "self_review",
               "review", "deviations", "unverified", "next_step_safety"}
COMMAND_KEYS = {"command", "kind", "status", "exit_code", "result", "evidence", "toolchain", "counts"}


def _unique_object(pairs: list[tuple[str, object]]) -> dict:
    result: dict = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _load(text: str) -> object:
    return json.loads(text, object_pairs_hook=_unique_object)


def _digest(value: object) -> str:
    canonical = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def _git(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", "-C", str(root), *args], text=True, capture_output=True, check=False)


def _nonempty(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _path(root: Path, value: object) -> Path:
    if not isinstance(value, str) or not value or Path(value).is_absolute() or ".." in Path(value).parts:
        raise ValueError("path must be a nonempty repository-relative path")
    path = root / value
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError("path escapes repository")
    return path


def _keys(value: object, expected: set[str], label: str, errors: list[str]) -> bool:
    if not isinstance(value, dict) or set(value) != expected:
        errors.append(f"{label}: expected closed keys {sorted(expected)}")
        return False
    return True


def validate_plan(root: Path, *, authority_path: Path | None = None,
                  ledger_path: Path | None = None, steps_path: Path | None = None,
                  requirements_path: Path | None = None) -> list[str]:
    """Return diagnostics; alternate inputs support isolated validator negative controls."""
    errors: list[str] = []
    root = Path(root).resolve()
    try:
        authority = _load((authority_path or root / SPEC / "authority.json").read_text())
        if not _keys(authority, AUTHORITY_KEYS, "authority", errors):
            return errors
        ledger_file = ledger_path or _path(root, authority["ledger"])
        ledger = _load(ledger_file.read_text())
        plan = _load((steps_path or _path(root, authority["steps"])).read_text())
        requirements = _load((requirements_path or _path(root, authority["requirements"])).read_text())
        for label, contract in (("steps", plan), ("requirements", requirements)):
            if _digest(contract) != CONTRACT_DIGESTS[label]:
                errors.append(f"{label}: approved semantic contract digest mismatch; controlled deviation required")
        if not _keys(ledger, LEDGER_KEYS, "ledger", errors):
            return errors
        if authority["schema"] != f"{PROGRAM}.authority.1" or authority["program"] != PROGRAM:
            errors.append("authority: unsupported identity")
        if ledger["schema"] != f"{PROGRAM}.ledger.1" or ledger["program"] != PROGRAM:
            errors.append("ledger: unsupported identity")
        expected_paths = {"governing_plan": f"docs/execution/rcl/{PROGRAM}.md",
                          "requirements": f"{SPEC}/requirements.json", "steps": f"{IMPL}/steps.json",
                          "ledger": f"{IMPL}/ledger.json", "validation_registry": f"{SPEC}/validation_registry.json",
                          "bootstrap_deviation": f"{IMPL}/deviations/bootstrap.md"}
        for key, expected in expected_paths.items():
            if authority[key] != expected:
                errors.append(f"authority: wrong {key} owner")
            if not _path(root, authority[key]).is_file():
                errors.append(f"authority: missing {key}")
        for label in ("reviewed_commit", "baseline_commit"):
            sha = authority[label]
            if not isinstance(sha, str) or not re.fullmatch(r"[0-9a-f]{40}", sha) or _git(root, "cat-file", "-e", f"{sha}^{{commit}}").returncode:
                errors.append(f"authority: unavailable {label}")
        if _git(root, "merge-base", "--is-ancestor", authority["reviewed_commit"], authority["baseline_commit"]).returncode:
            errors.append("authority: baseline does not descend from reviewed commit")
        historical = _load(_git(root, "show", f"{authority['reviewed_commit']}:spec/remediation_v19_authority.json").stdout)
        if authority["holds"] != historical["holds"] or authority["holds"] != HOLDS or ledger["holds"] != authority["holds"]:
            errors.append("holds: preserved v19 qualification holds required")
        bootstrap = ledger["bootstrap"]
        if _keys(bootstrap, {"deviation", "steps", "source_commit", "receipt_commit"}, "bootstrap", errors):
            if bootstrap["deviation"] != authority["bootstrap_deviation"] or bootstrap["steps"] != BOOTSTRAP:
                errors.append("bootstrap: wrong deviation ownership or step mapping")
            if bootstrap["receipt_commit"] is not None:
                receipt = bootstrap["receipt_commit"]
                if not isinstance(receipt, str) or not re.fullmatch(r"[0-9a-f]{40}", receipt) or _git(root, "cat-file", "-e", f"{receipt}^{{commit}}").returncode:
                    errors.append("bootstrap: receipt must be an existing observed commit")
                elif receipt == bootstrap["source_commit"]:
                    errors.append("bootstrap: circular source/receipt binding")
        expected_steps = plan["steps"]
        ids = [f"NA-{i:03}" for i in range(1, 167)]
        if plan["step_count"] != 166 or [s["id"] for s in expected_steps] != ids:
            errors.append("plan: exact 166 ordered identities required")
        requirement_ids = [r["id"] for r in requirements["requirements"]]
        if len(requirement_ids) != 32 or len(set(requirement_ids)) != 32:
            errors.append("requirements: exact 32 unique families required")
        if not isinstance(ledger["steps"], list) or len(ledger["steps"]) != len(expected_steps):
            errors.append("ledger: exact plan coverage required")
            return errors
        if len({s["alias"] for s in expected_steps}) != len(expected_steps):
            errors.append("plan: duplicate aliases")
        statuses: dict[str, str] = {}
        active: list[str] = []
        for ordinal, (step, original) in enumerate(zip(ledger["steps"], expected_steps), 1):
            label = original["id"]
            if not _keys(step, STEP_KEYS, label, errors):
                continue
            for key in ("id", "alias", "ordinal", "requirements", "depends_on"):
                if step[key] != original[key]:
                    errors.append(f"{label}: original {key} changed")
            if type(step["ordinal"]) is not int or type(original["ordinal"]) is not int or original["ordinal"] != ordinal or not set(original["requirements"]).issubset(requirement_ids):
                errors.append(f"{label}: invalid plan ordinal or requirement")
            if any(p not in ids[:ordinal - 1] for p in original["depends_on"]):
                errors.append(f"{label}: prerequisite is not an earlier original step")
            status = step["status"]
            if status not in ("planned", "active", "complete", "blocked"):
                errors.append(f"{label}: invalid status")
                continue
            statuses[label] = status
            if status == "active":
                active.append(label)
            if status == "complete":
                for predecessor in ids[:ordinal - 1]:
                    if statuses.get(predecessor) != "complete":
                        errors.append(f"{label}: skipped prerequisite/prefix {predecessor}")
                        break
                _validate_report(root, authority, step, original, errors)
            elif step["report"] is not None:
                errors.append(f"{label}: incomplete step must not carry completion report")
        incomplete = [s["id"] for s in ledger["steps"] if s["status"] != "complete"]
        expected_cursor = incomplete[0] if incomplete else None
        if len(active) > 1 or ledger["active_step"] != (active[0] if len(active) == 1 else None):
            errors.append("ledger: one active cursor must agree with records")
        if active and active[0] != expected_cursor:
            errors.append("ledger: active step skips incomplete prerequisite")
        if ledger["next_step"] != expected_cursor:
            errors.append("ledger: next_step must identify first incomplete step")
        completed_bootstrap = [statuses.get(s) == "complete" for s in BOOTSTRAP]
        if any(completed_bootstrap) and not all(completed_bootstrap):
            errors.append("bootstrap: all five original steps require atomic observation")
        if all(completed_bootstrap):
            sources = {s["report"]["source_commit"] for s in ledger["steps"][:5] if isinstance(s["report"], dict)}
            if len(sources) != 1 or bootstrap["source_commit"] not in sources:
                errors.append("bootstrap: completion requires one actual shared source commit")
            if any(authority["bootstrap_deviation"] not in s["report"]["deviations"] for s in ledger["steps"][:5] if isinstance(s["report"], dict)):
                errors.append("bootstrap: per-step controlled deviation required")
            receipt = bootstrap["receipt_commit"]
            source = bootstrap["source_commit"]
            if isinstance(receipt, str) and isinstance(source, str):
                if _git(root, "merge-base", "--is-ancestor", source, receipt).returncode or _git(root, "merge-base", "--is-ancestor", receipt, "HEAD").returncode:
                    errors.append("bootstrap: receipt must follow source in current history")
                try:
                    observed = _load(_git(root, "show", f"{receipt}:{authority['ledger']}").stdout)
                    observed_steps = observed["steps"][:5]
                    if len(observed_steps) != 5 or any(s["status"] != "complete" or s["report"]["source_commit"] != source for s in observed_steps):
                        errors.append("bootstrap: receipt commit does not contain five source-bound observations")
                except (ValueError, KeyError, TypeError):
                    errors.append("bootstrap: receipt commit has no valid observation ledger")
        elif bootstrap["source_commit"] is not None or bootstrap["receipt_commit"] is not None:
            errors.append("bootstrap: initial state must not invent source/receipt commits")
    except (OSError, ValueError, TypeError, KeyError, AttributeError) as exc:
        errors.append(f"invalid plan input: {exc}")
    return errors


def _validate_report(root: Path, authority: dict, step: dict, original: dict, errors: list[str]) -> None:
    label = step["id"]
    report = step["report"]
    if not _keys(report, REPORT_KEYS, f"{label} report", errors):
        return
    valid_commits = True
    for key in ("source_commit", "observation_base"):
        sha = report[key]
        if not isinstance(sha, str) or not re.fullmatch(r"[0-9a-f]{40}", sha) or _git(root, "cat-file", "-e", f"{sha}^{{commit}}").returncode:
            errors.append(f"{label}: unavailable/future {key}")
            valid_commits = False
    if valid_commits:
        source = report["source_commit"]
        if _git(root, "merge-base", "--is-ancestor", authority["baseline_commit"], source).returncode or _git(root, "merge-base", "--is-ancestor", source, report["observation_base"]).returncode or _git(root, "merge-base", "--is-ancestor", report["observation_base"], "HEAD").returncode:
            errors.append(f"{label}: source/observation ancestry mismatch")
        source_ledger = _git(root, "show", f"{source}:{authority['ledger']}")
        try:
            prior = next(s for s in _load(source_ledger.stdout)["steps"] if s["id"] == label)
            if prior["status"] == "complete" or prior["report"] is not None:
                errors.append(f"{label}: circular source already contains completion claim")
        except (ValueError, KeyError, TypeError, StopIteration):
            errors.append(f"{label}: source lacks committed pre-observation ledger")
    if not isinstance(report["files"], list) or not report["files"]:
        errors.append(f"{label}: changed files required")
    else:
        for file in report["files"]:
            _path(root, file)
            if valid_commits and _git(root, "cat-file", "-e", f"{report['source_commit']}:{file}").returncode:
                errors.append(f"{label}: file absent from source candidate: {file}")
    commands = report["commands"]
    if not isinstance(commands, list) or not commands:
        errors.append(f"{label}: missing commands")
        commands = []
    actual = []
    for command in commands:
        if not _keys(command, COMMAND_KEYS, f"{label} command", errors):
            continue
        actual.append(command["command"])
        if command["kind"] not in ("executed", "historical_receipt_validation", "external_attestation"):
            errors.append(f"{label}: unknown evidence kind")
        if command["status"] != "passed" or type(command["exit_code"]) is not int or command["exit_code"] != 0:
            errors.append(f"{label}: incomplete/failed command cannot complete step")
        if any(not _nonempty(command[key]) for key in ("command", "result", "evidence", "toolchain")):
            errors.append(f"{label}: command provenance missing")
        counts = command["counts"]
        if _keys(counts, {"tests", "scenarios"}, f"{label} counts", errors):
            for count in counts.values():
                if count is not None and (type(count) is not int or count < 0):
                    errors.append(f"{label}: invalid execution count")
            invocation = command["command"]
            if isinstance(invocation, str):
                test_command = bool(re.search(r"\bcargo(?: \+[^ ]+)? test\b|\bpython\S* -m unittest\b", invocation))
                if test_command and (type(counts["tests"]) is not int or counts["tests"] <= 0):
                    errors.append(f"{label}: zero/missing executed test count")
                if "run_distribution" in invocation and (type(counts["scenarios"]) is not int or counts["scenarios"] <= 0):
                    errors.append(f"{label}: zero/missing executed scenario count")
        if command["command"] in original["commands"] and command["kind"] != "executed":
            errors.append(f"{label}: current required command needs actual execution")
    if any(command not in actual for command in original["commands"]):
        errors.append(f"{label}: missing required command")
    acceptance = report["acceptance"]
    if not isinstance(acceptance, list) or [a.get("criterion") for a in acceptance if isinstance(a, dict)] != original["tests"]:
        errors.append(f"{label}: exact per-step acceptance criteria required")
    else:
        for assertion in acceptance:
            if _keys(assertion, {"criterion", "status", "evidence"}, f"{label} acceptance", errors):
                if assertion["status"] != "passed" or not _nonempty(assertion["evidence"]):
                    errors.append(f"{label}: unverified acceptance")
    for key in ("self_review", "review"):
        expected = {"status", "evidence", "kind"} if key == "review" else {"status", "evidence"}
        review = report[key]
        if _keys(review, expected, f"{label} {key}", errors):
            if review["status"] != "passed" or not _nonempty(review["evidence"]):
                errors.append(f"{label}: {key} must have passed with evidence")
            if key == "review" and review["kind"] != "fresh_adversarial":
                errors.append(f"{label}: fresh adversarial review required")
    if not isinstance(report["unverified"], list) or not isinstance(report["deviations"], list):
        errors.append(f"{label}: unverified/deviations must be explicit lists")
    for path in report["deviations"]:
        if not _path(root, path).is_file():
            errors.append(f"{label}: missing deviation evidence")
    if report["next_step_safety"] not in ("safe", "safe with documented pre-existing issue"):
        errors.append(f"{label}: unsafe next-step decision")
    if report["unverified"] and report["next_step_safety"] == "safe":
        errors.append(f"{label}: unverified work needs explicit qualified safety")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    args = parser.parse_args()
    errors = validate_plan(args.root)
    if errors:
        print("\n".join(errors))
        return 1
    print("alignment plan: PASS (166 steps; completion claims require committed source observations)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
