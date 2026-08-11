# ADR 0037: Complete Evaluation Metering

Status: Approved

## Context

Control preparation and post-stop report work can still traverse untrusted
evidence without complete budget or cancellation coverage.

## Decision

Control preparation, ancestry, authorization comparisons, change grouping,
manifest resolution, event reporting, checkpoint refusals, digest inputs, and
post-stop work use the caller budget and cancellation policy.

## Consequences

An explicit work inventory controls implementation and tests. Optional work
stops immediately after interruption, and evidence-derived paths remain
panic-free.
