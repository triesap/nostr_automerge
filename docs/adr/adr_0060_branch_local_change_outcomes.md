# ADR 0060: Branch-Local Change Outcomes

## Status

Approved for remediation v8.

## Decision

Retain each valid control branch's per-hash epoch outcomes and use the
referenced branch result as an authoritative input to final carrier-claim
reduction.

## Rationale

A valid control branch does not imply that every change carried under that
branch is valid. Canonical selection and branch-local change validity are
separate decisions.

## Consequences

- Rust and TypeScript implement the neutral rule independently.
- Signed fixtures and exact requirement evidence cover the rule.
- No wire format, event kind, public API, or hash domain changes.
- Deviations require an ADR amendment before implementation.
