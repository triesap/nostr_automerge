# ADR 0064: Local NIP Reconciliation

## Status

Approved for remediation v8.

## Decision

Treat the reconciled repository-local NIP draft as the self-contained proposal
authority while retaining all draft and publication boundaries.

## Rationale

An independent implementer must not need hidden companion-only rules to
converge on branch, claim, scope, disposition, and resource behavior.

## Consequences

- The local draft may be edited only by the approved reconciliation sequence.
- `NIP-XX`, provisional kinds, wire names, and hash domains remain unchanged.
- No submission, allocation, publication, or upstream action is implied.
- Requirements and evidence bind the reconciled local draft by exact hash.
