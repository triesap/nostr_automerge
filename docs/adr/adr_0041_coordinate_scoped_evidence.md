# ADR 0041: Coordinate-Scoped Evidence

Status: Approved

## Context

Corpus-wide scans allow unrelated documents to affect one coordinate's work,
completion, dispositions, evidence, and digests.

## Decision

Every evaluation consumes one immutable coordinate-scoped evidence view. The
view separates reportable target evidence from explicitly referenced,
non-reportable lifecycle support.

## Consequences

All work accounting and canonical output use the view. Unrelated evidence is
inert, while signed predecessor continuity remains verifiable.
