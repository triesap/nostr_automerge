# ADR 0059: Remediation-v7 companion authority

Status: Approved

## Context

Branch-local control validity, coordinate-qualified dependent membership,
linear parent-state propagation, explicit finalization settlement, and signed
distribution-v8 conformance are implementation-critical rules. The externally
authored NIP snapshot is outside this implementation task and must remain
byte-identical.

## Decision

The repository-owned companion specification defines the complete local rules.
An unsubmitted portable v7 proposal mirrors the delta for the external author.
The proposal grants no authority to submit, publish, allocate, adopt, or release
anything. `NCRDT-NIP-002` remains explicitly deferred, while local Rust and
TypeScript implementation and conformance evidence may be complete.

## Consequences

Implementers can reproduce the local protocol behavior without private context,
and future NIP reconciliation has a bounded portable input. The unchanged NIP
hash and the external hold prevent local implementation success from being
misrepresented as external protocol adoption.
