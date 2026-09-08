# Nostr Automerge v19 assurance-closure RCL program

Status: active at RCLD 141

## Authority and scope

This append-only program continues the completed v18 history from public
candidate `4cc1bb5060b7477214eb22cf7b086735e4a70b7a`. It closes local assurance
Findings 130 through 133 while preserving the proven runtime order, protocol,
public API, signed fixtures, canonical output, and all external holds.

The reviewed 53-step proposal is acceptance input, not a mandatory commit
topology. RCLD 141 through RCLD 146 consolidate that input into coherent green
checkpoints. Every consolidation, split, reorder, corrective commit, or
superseding checkpoint is recorded in append-only execution mapping.

The strongest authorized outcome is `code_complete_publication_held`. No
remote action, publication, release, deployment, NIP submission, event-kind
allocation, production qualification, or external-assurance claim is allowed.

## Frozen invariants

- The requirements registry contains 156 rows.
- The selected distribution contains 204 scenarios and 771 signed Events.
- Qualification uses eight delivery orders and two processes per
  implementation.
- Canonical output remains
  `e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415`.
- Serialized-run identity remains
  `000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344`.
- Public and independently owned compatibility implementations remain
  separate. Only opaque identities, counts, abstract classes, result classes,
  and hashes may cross the public boundary.
- V18 records are immutable history.

## Independent proof contract

Every exact production-path proof independently emits and validates:

```text
ChargeAttempt
ChargeAccepted
TargetDispatched
TargetReturned
CompletionObserved
PublicationCompleted, when applicable
```

`TargetDispatched` and `TargetReturned` come only from test-only target
instrumentation. `CompletionObserved` comes only from the completion observer.
A failed charge emits exactly one attempt and no later event. N and N+1 emit
exactly one attempt, acceptance, dispatch, return, and completion for the
requested site. Cancellation and unexpected errors retain exact identity.

The probe is crate-private, absent from non-test compiled code and public API,
panic-safe, and isolated between parallel tests. Counts, ordering, and pass
labels are reconstructed from captured traces rather than copied fields.

## Mutation and oracle contract

The public campaign contains at least 40 genuine mutants: the four physical
helpers by seven mandatory mutation classes, seven direct site-local target
hoists, and five non-overlapping provenance mutations. Every mutant executes in
an isolated worktree, compiles as declared, fails for its intended
behavior-derived property, records replayable compile and property commands,
restores exactly, and leaves zero survivors.

The independently owned implementation qualifies the same abstract obligations
using its native helper structure, with at least 19 genuine mutants. Public
evidence may retain only its opaque counts, identities, result classes, and
hashes.

Property results use the closed vocabulary in
`spec/causal_projection_contracts_v19.json`. Marker-selected, source-hash-only,
inert, aliased, or relabelled mutations do not qualify. Structural validation
tolerates neutral comments and harmless refactors; identity validation binds
committed candidates separately.

## Candidate lifecycle

Evidence moves strictly forward:

```text
source candidate
-> clean execution base
-> raw proof and mutation artifacts
-> later catalogs that bind artifact commits
-> final inventory and evidence graph
-> public qualification
-> opaque independent assurance
-> combined closure and terminal commit T
-> strict descendant execution base E
-> clean attestation commit A
```

No artifact names its own containing commit. Full final gates run twice from
clean `E`; the attestation committed at `A` names `T` and `E`, and validation at
clean `A` proves strict ancestry and immutable terminal artifacts.

## Ordered RCLDs

### RCLD 141: authority, reproduction, and traceability

Append v19 authority and contracts, reproduce Findings 130 through 133, map all
43 approved v18 steps and every relevant actual checkpoint, and record all
deviations. This stage changes no production semantics.

### RCLD 142: independent Rust proof events

Add test-only target instrumentation, replace aliased event accounting, derive
one exact production-path proof per live Rust site, reject malformed traces, and
prove that release and public surfaces are unchanged.

### RCLD 143: runtime mutation qualification

Replace marker-driven and inert checks with runtime-derived property oracles,
execute the complete public mutation campaign, retain replayable evidence, and
prove zero survivors and exact restoration.

### RCLD 144: public evidence and qualification

Create later artifact catalogs, final source-derived inventory, bidirectional
evidence graph, zero-change distribution transition, and two clean public
qualification receipts.

### RCLD 145: independent compatibility requalification

The independently owned implementation executes exact production-path proofs,
its native helper matrix, direct and provenance mutations, final evidence, and
two-process compatibility qualification. Only a leak-free opaque record is
made available to this repository.

### RCLD 146: opaque join, terminal decision, and attestation

Validate and import the opaque record, build combined assurance, close Findings
130 through 133, retain Finding 080, commit terminal `T`, establish strict
descendant `E`, run all gates twice, and commit and validate attestation `A`.

## Per-checkpoint discipline

Only one RCLD and checkpoint is active at a time. Each checkpoint inspects the
owning repository, changes one coherent scope, runs its narrowest credible
verification, records any deviation before changing the plan, commits only
green work, and reports changed files, commands, results, self-review, residual
risk, and next-step safety. Historical evidence is never rewritten.
