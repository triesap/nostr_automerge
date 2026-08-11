# ADR 0035: Selected Manifest Dynamic Validation

Status: Approved

## Context

Static replacement selection does not establish whether a manifest's referenced
control is present, same-coordinate, dynamically valid, or canonical.

## Decision

NIP-01 replacement occurs first. The selected manifest reference is then
resolved against same-coordinate stateful control outcomes without fallback.

## Consequences

The result distinguishes missing, wrong-coordinate, invalid,
valid-noncanonical, and canonical references. The manifest remains advisory and
never becomes protocol authority.
