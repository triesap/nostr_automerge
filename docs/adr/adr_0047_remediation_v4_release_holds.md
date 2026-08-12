# ADR 0047: Remediation V4 Release Holds

Status: Approved

## Context

Local code completion does not establish sustained native fuzzing, independent
external review, or publication authority.

## Decision

Keep fuzzing, independent review, external NIP reconciliation, and every remote
publication action as separate explicit holds.

## Consequences

Ordinary implementation and evidence gates may complete without overclaiming
production readiness or authorizing any remote mutation.
