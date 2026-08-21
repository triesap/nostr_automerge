# ADR 0065: Checkpoint Control Precedence

## Status

Approved staged candidate for remediation v9.

## Authority transition

At `transition_installed`, this ADR is approved but is not effective current
protocol authority. The unchanged NIP and current companion remain controlling.
This decision becomes effective for the staged local implementation candidate
at `companion_authority_installed`. It then refines ADR 0055's descriptor
reference resolution and ADR 0015's checkpoint authorization order; it does not
supersede their compatible rules or override the NIP.

If the unchanged NIP conflicts with this candidate decision, the NIP continues
to control NIP-conformance claims. Candidate closure, release, and
NIP-conformance remain held until that text is reconciled through its own
change process.

## Context

A checkpoint descriptor cannot be trusted merely because its signed content is
well formed. Its referenced control determines the descriptor's coordinate,
canonical branch, and checkpoint role authorization. Building chunk sets or
loading snapshot state before that decision performs work for evidence that may
already be unusable and can blur invalid evidence into a recoverable state.

Historical carrier coverage and the changes accepted no later than a control
are also different facts. Current inputs may make the two ordered sets equal,
but one cannot be substituted for the other.

## Decision

Static descriptor and chunk validation finishes at ingestion. Target
evaluation then classifies the descriptor's referenced control as exactly one
of: missing, statefully pending, canonical and statefully valid, statefully
valid but noncanonical, wrong kind, wrong coordinate, statically invalid,
dynamically invalid, or unsupported.

Only a canonical, statefully valid control that grants the descriptor author
the `checkpoint` role permits downstream checkpoint work. A missing or
statefully pending control makes the descriptor pending. Every other control
classification, including a valid noncanonical control or role denial, makes
the draft-v1 descriptor invalid.

This control and role decision precedes all of the following:

- collecting or ordering chunks;
- computing carrier coverage;
- looking up changes accepted no later than the control;
- loading or inspecting a snapshot;
- checking heads, closure, history, counts, hashes, or proofs.

Dependent chunk outcomes follow the resolved descriptor outcome and their own
static bindings. Historical carrier coverage and accepted-at-control history
remain separate canonical ordered sets and are compared only where checkpoint
verification requires that comparison.

## Rationale

Authorization is a prerequisite for checkpoint verification, not one of its
late validation passes. Resolving it first makes known unusable evidence
conclusive, preserves pending only for evidence that can still arrive or
resolve, and prevents unauthorized input from initiating target-sized work.

## Consequences

- Noncanonical, wrong-kind, wrong-coordinate, static-invalid,
  dynamic-invalid, unsupported, and role-denied controls are invalid rather
  than pending.
- Missing and genuinely statefully pending controls remain recoverable.
- Invalid authorization performs no chunk, snapshot, history, or proof work.
- This decision changes no event kind, tag, content field, digest domain,
  protocol revision, or NIP text.
