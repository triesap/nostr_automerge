#!/usr/bin/env python3
"""Reject non-opaque material from the v9 reproduction and runtime records."""

from __future__ import annotations

import ast
import copy
import io
import re
import subprocess
import sys
import tokenize
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

from validate_runtime_ledger_v9 import (
    ABSOLUTE_PATH_TEXT,
    CASE_TEXT,
    COMMAND_TEXT,
    COMMIT_SUBJECT_TEXT,
    LOG_TEXT,
    PACKAGE_SUFFIX_TEXT,
    RELATIVE_PATH_TEXT,
    URI_TEXT,
    LedgerError,
    load_object,
    validate_no_leak,
)


ROOT = Path(__file__).resolve().parents[1]
JSON_RECORDS = (
    "reports/opaque_reproduction_v9.json",
    "implementation/runtime_ledger_v9.json",
    "tools/validation/opaque_reproduction_v9.schema.json",
    "tools/validation/runtime_ledger_v9.schema.json",
)
TEXT_RECORDS = ("docs/execution/remediation_v9/ledger.md",)
PYTHON_SURFACES = (
    "scripts/validate_runtime_ledger_v9.py",
    "scripts/validate_private_reproduction_boundary_v9.py",
    "scripts/validate_spec.py",
)
OTHER_SURFACES = (
    "tools/nostr_automerge_xtask/src/validate.rs",
    "reports/spec_baseline.txt",
)
LEGITIMATE_PUBLIC_ROUTES = frozenset(
    {
        "../..",
        "crates/nostr_automerge/src/checkpoint/authorize.rs",
        "crates/nostr_automerge/src/engine/checkpoint_result.rs",
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "crates/nostr_automerge/src/control/reference_state.rs",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "deviations/step_001.md",
        "docs/adr",
        "docs/adr/README.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md",
        "docs/execution/remediation_v9/ledger.md",
        "docs/provenance",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_invalid_control.expected.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_invalid_control.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_invalid_control.input.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_unsupported_control.expected.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_unsupported_control.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_unsupported_control.input.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_coordinate_control.expected.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_coordinate_control.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_coordinate_control.input.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_kind_control.expected.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_kind_control.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_kind_control.input.json",
        "fixtures/README.md",
        "fixtures/examples",
        "fixtures/schema",
        "implementation/runtime_ledger_v9.json",
        "reports/opaque_reproduction_v9.json",
        "reports/spec_baseline.txt",
        "scripts/validate_architecture.py",
        "scripts/validate_authority_transition_v10.py",
        "scripts/validate_diagnostics.py",
        "scripts/validate_fixtures.py",
        "scripts/validate_private_reproduction_boundary_v9.py",
        "scripts/validate_protocol_revision.py",
        "scripts/validate_runtime_ledger_v9.py",
        "scripts/validate_spec.py",
        "spec/authority_transition_v10.json",
        "spec/requirements.json",
        "tools/nostr_automerge_xtask/src/validate.rs",
        "tools/validation/opaque_reproduction_v9.schema.json",
        "tools/validation/authority_transition_v10.schema.json",
        "tools/validation/runtime_ledger_v9.schema.json",
    }
)
RUST_STRING = re.compile(r'"([^"\\]*(?:\\.[^"\\]*)*)"')
UNIVERSAL_SOURCE_PATTERNS = (
    URI_TEXT,
    ABSOLUTE_PATH_TEXT,
    LOG_TEXT,
    PACKAGE_SUFFIX_TEXT,
    COMMAND_TEXT,
    CASE_TEXT,
    COMMIT_SUBJECT_TEXT,
)


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise LedgerError(diagnostic)


def validate_records(records: list[dict[str, Any]], text: list[str]) -> None:
    for index, record in enumerate(records):
        validate_no_leak(record, f"json_record:{index}")
    for index, value in enumerate(text):
        validate_no_leak(value, f"text_record:{index}")


def validate_source_literal(
    value: str, diagnostic: str, *, allow_command_token: bool = False
) -> None:
    for index, pattern in enumerate(UNIVERSAL_SOURCE_PATTERNS):
        matched = pattern.search(value) is not None
        require(
            not matched or (pattern is COMMAND_TEXT and allow_command_token),
            f"{diagnostic}:pattern:{index}",
        )
    if RELATIVE_PATH_TEXT.search(value) is not None:
        require(is_public_route(value), f"{diagnostic}:relative_route")


def is_public_route(value: str) -> bool:
    return value in LEGITIMATE_PUBLIC_ROUTES


def python_comments(source: str, relative: str) -> list[str]:
    try:
        tokens = tokenize.generate_tokens(io.StringIO(source).readline)
        return [
            token.string.removeprefix("#").strip()
            for token in tokens
            if token.type == tokenize.COMMENT
            and not (token.start == (1, 0) and token.string.startswith("#!"))
        ]
    except (IndentationError, tokenize.TokenError) as error:
        raise LedgerError(f"source_comments:{relative}") from error


def python_literals(relative: str) -> tuple[list[str], list[str], list[str]]:
    try:
        source = (ROOT / relative).read_text(encoding="utf-8")
        tree = ast.parse(source, filename=relative)
    except (OSError, UnicodeDecodeError, SyntaxError) as error:
        raise LedgerError(f"source_surface:{relative}") from error
    literals = [
        node.value
        for node in ast.walk(tree)
        if isinstance(node, ast.Constant) and isinstance(node.value, str)
    ]
    coordinated: list[str] = []

    def static_string(node: ast.AST) -> str | None:
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            return node.value
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
            left = static_string(node.left)
            right = static_string(node.right)
            if left is not None and right is not None:
                return left + right
        return None

    for node in ast.walk(tree):
        if isinstance(node, ast.BinOp):
            value = static_string(node)
            if value is not None:
                coordinated.append(value)
        if not isinstance(node, (ast.List, ast.Set, ast.Tuple)):
            continue
        values = [
            child.value
            for child in node.elts
            if isinstance(child, ast.Constant) and isinstance(child.value, str)
        ]
        residual = [value for value in values if not is_public_route(value)]
        if len(residual) > 1:
            coordinated.append("".join(residual))
    return literals, coordinated, python_comments(source, relative)


def rust_comments(source: str) -> list[str]:
    comments: list[str] = []
    index = 0
    length = len(source)

    def cleaned(value: str) -> str:
        if value.startswith(("!", "/", "*")):
            value = value[1:]
        return value.strip()

    while index < length:
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            if end < 0:
                end = length
            comments.append(cleaned(source[index + 2 : end]))
            index = end
            continue
        if source.startswith("/*", index):
            depth = 1
            cursor = index + 2
            body: list[str] = []
            while cursor < length and depth:
                if source.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                else:
                    body.append(source[cursor])
                    cursor += 1
            require(depth == 0, "source_comments:rust_unclosed")
            comments.append(cleaned("".join(body)))
            index = cursor
            continue

        raw_prefix = None
        if source.startswith("br", index):
            raw_prefix = index + 2
        elif source.startswith("r", index):
            raw_prefix = index + 1
        if raw_prefix is not None:
            cursor = raw_prefix
            while cursor < length and source[cursor] == "#":
                cursor += 1
            if cursor < length and source[cursor] == '"':
                hashes = source[raw_prefix:cursor]
                terminator = '"' + hashes
                end = source.find(terminator, cursor + 1)
                require(end >= 0, "source_comments:rust_raw_string")
                index = end + len(terminator)
                continue

        if source[index] == '"':
            cursor = index + 1
            while cursor < length:
                if source[cursor] == "\\":
                    cursor += 2
                    continue
                if source[cursor] == '"':
                    cursor += 1
                    break
                cursor += 1
            index = cursor
            continue
        if source[index] == "'":
            cursor = index + 1
            if cursor < length and source[cursor] == "\\":
                cursor += 2
            else:
                cursor += 1
            if cursor < length and source[cursor] == "'":
                index = cursor + 1
                continue
        index += 1
    return comments


def validate_source_surfaces() -> None:
    audited = 0
    for relative in PYTHON_SURFACES:
        literals, coordinated, comments = python_literals(relative)
        for index, value in enumerate(literals):
            validate_source_literal(
                value,
                f"source:{relative}:{index}",
                allow_command_token=(
                    (value == "git" and relative == "scripts/validate_runtime_ledger_v9.py")
                    or (
                        value in {"git", "python3"}
                        and relative
                        == "scripts/validate_private_reproduction_boundary_v9.py"
                    )
                ),
            )
        for index, value in enumerate(coordinated):
            validate_source_literal(value, f"source:{relative}:coordinated:{index}")
        for index, value in enumerate(comments):
            validate_source_literal(value, f"source:{relative}:comment:{index}")
        audited += 1
    for relative in OTHER_SURFACES:
        try:
            source = (ROOT / relative).read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as error:
            raise LedgerError(f"source_surface:{relative}") from error
        literals = RUST_STRING.findall(source) if relative.endswith(".rs") else [source]
        for index, value in enumerate(literals):
            validate_source_literal(
                value,
                f"source:{relative}:{index}",
                allow_command_token=(
                    relative == "tools/nostr_automerge_xtask/src/validate.rs"
                    and value == "python3"
                ),
            )
        if relative.endswith(".rs"):
            for line_number, line in enumerate(source.splitlines(), start=1):
                line_literals = [
                    value
                    for value in RUST_STRING.findall(line)
                    if not is_public_route(value)
                ]
                if len(line_literals) > 1:
                    validate_source_literal(
                        "".join(line_literals),
                        f"source:{relative}:coordinated:{line_number}",
                    )
            for index, value in enumerate(rust_comments(source)):
                validate_source_literal(value, f"source:{relative}:comment:{index}")
        audited += 1
    require(audited == len(PYTHON_SURFACES) + len(OTHER_SURFACES), "source:inventory")


def validate_tracked_boundary() -> None:
    result = subprocess.run(
        ("git", "ls-files"),
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    require(result.returncode == 0, "tracked_boundary:git")
    require(result.stderr == "", "tracked_boundary:diagnostic")
    blocked = []
    for relative in result.stdout.splitlines():
        parts = relative.casefold().split("/")
        if parts and (parts[0] == ".act" or "workflows" in parts):
            blocked.append(relative)
    require(not blocked, "tracked_boundary:material")


def mutation_self_test(records: list[dict[str, Any]], text: list[str]) -> int:
    key_names = (
        "sourcePath",
        "test-path",
        "file_path",
        "packagePath",
        "case-name",
        "commandLine",
        "log_path",
        "urlValue",
        "workflow-artifact",
        "artifactSource",
        "rootPath",
        "submodule-path",
        "implementationDetail",
    )
    value_markers = (
        chr(47) + "alpha" + chr(47) + "beta",
        "alpha" + chr(47) + "beta.json",
        "ssh" + chr(58) + chr(47) * 2 + "host",
        "custom" + chr(58) + chr(47) * 2 + "endpoint",
        "output" + chr(46) + "log",
        "engine" + chr(95) + "typescript",
        chr(99) + "argo" + chr(32) + "test",
        "f" + str(85).zfill(3) + chr(95) + "checkpoint",
        "fix" + chr(40) + "scope" + chr(41) + chr(58) + chr(32) + "hidden",
    )
    mutations: list[tuple[str, list[dict[str, Any]], list[str]]] = []
    for key in key_names:
        candidates = copy.deepcopy(records)
        candidates[0][key] = "hidden"
        mutations.append((f"key:{key}", candidates, text))
    for index, marker in enumerate(value_markers):
        candidates = copy.deepcopy(records)
        candidates[0]["status"] = marker
        mutations.append((f"value:{index}", candidates, text))
    split_values = (
        ["alpha", chr(47) + "beta"],
        ["ssh", chr(58) + chr(47) * 2 + "host"],
        ["engine", chr(95) + "typescript"],
    )
    for index, values in enumerate(split_values):
        candidates = copy.deepcopy(records)
        candidates[0]["toolchain_classes"] = values
        mutations.append((f"coordinated:{index}", candidates, text))
    split_key_values = (
        ("alpha", chr(47) + "beta"),
        ("ssh", chr(58) + chr(47) * 2 + "host"),
        ("engine", chr(95) + "typescript"),
    )
    for index, (key, value) in enumerate(split_key_values):
        candidates = copy.deepcopy(records)
        candidates[0][key] = value
        mutations.append((f"coordinated_key_value:{index}", candidates, text))

    caught = 0
    for name, candidates, candidate_text in mutations:
        try:
            validate_records(candidates, candidate_text)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"boundary_mutation_survived:{name}")
    return caught


def source_mutation_self_test() -> int:
    separator = chr(47)
    underscore = chr(95)
    reviewer_routes = (
        "docs" + separator + "alpha" + separator + "beta",
        "scripts" + separator + "engine" + underscore + "typescript",
        "reports" + separator + "output" + chr(46) + "log",
        "docs"
        + separator
        + "f"
        + str(85).zfill(3)
        + underscore
        + "private"
        + underscore
        + "case"
        + chr(46)
        + "md",
    )
    comment_markers = (
        "alpha" + separator + "beta",
        "engine" + underscore + "typescript",
        "f" + str(85).zfill(3) + underscore + "private",
        "fix" + chr(40) + "scope" + chr(41) + chr(58) + chr(32) + "hidden",
        chr(99) + "argo" + chr(32) + "test",
    )
    mutations: list[tuple[str, str]] = [
        (f"reviewer_route:{index}", value)
        for index, value in enumerate(reviewer_routes)
    ]
    for index, marker in enumerate(comment_markers):
        comments = python_comments("# " + marker + "\n", "mutation")
        require(len(comments) == 1, f"source_mutation:python_shape:{index}")
        mutations.append((f"python_comment:{index}", comments[0]))
    rust_sources = (
        "// " + comment_markers[0] + "\n",
        "/* " + comment_markers[1] + " */",
        "/// " + comment_markers[2] + "\n",
        "//! " + comment_markers[3] + "\n",
        "/** " + comment_markers[4] + " */",
    )
    for index, source in enumerate(rust_sources):
        comments = rust_comments(source)
        require(len(comments) == 1, f"source_mutation:rust_shape:{index}")
        mutations.append((f"rust_comment:{index}", comments[0]))

    caught = 0
    for name, value in mutations:
        try:
            validate_source_literal(value, f"source_mutation:{name}")
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"source_mutation_survived:{name}")
    return caught


def main() -> int:
    records = [load_object(relative) for relative in JSON_RECORDS]
    try:
        text = [(ROOT / relative).read_text(encoding="utf-8") for relative in TEXT_RECORDS]
    except (OSError, UnicodeDecodeError) as error:
        raise LedgerError("text_record") from error
    validate_records(records, text)
    validate_tracked_boundary()
    validate_source_surfaces()
    mutations = mutation_self_test(records, text)
    source_mutations = source_mutation_self_test()
    print("PASS: opaque reproduction boundary v9")
    print(f"- json_records={len(records)}")
    print(f"- text_records={len(text)}")
    print(f"- negative_mutations={mutations}")
    print(f"- source_negative_mutations={source_mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
