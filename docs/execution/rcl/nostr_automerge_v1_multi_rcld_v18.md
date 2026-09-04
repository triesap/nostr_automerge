# Nostr Automerge v18 sealed-boundary RCL program

Status: code complete — `code_complete_publication_held`; clean descendant attestation follows the terminal checkpoint

## Authority and scope

This append-only program continues the completed v17 history from public
candidate `8673ff8546b9e9d57218c15a4b81890d82137184`. It closes the local runtime
and assurance defects recorded as Findings 123 through 129 while preserving
the protocol, public API, signed fixtures, and external holds.

The program owns RCLD 134 through RCLD 140. The reviewed six-stage proposal is
split into seven reviewable stages so the independently owned compatibility
evidence is committed in its own history before the public repository imports
an opaque record. This is a sequencing refinement, not a scope expansion.

The strongest authorized outcome is `code_complete_publication_held`. No
remote action, publication, release, deployment, NIP submission, event-kind
allocation, production qualification, or external-assurance claim is allowed.

## Frozen invariants

- The requirements registry contains 156 rows.
- The selected distribution contains 204 scenarios and 771 signed Events.
- The ample-work canonical output SHA-256 is
  `e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415`.
- Rust and independently owned compatibility implementations remain separate.
- Public evidence may contain only opaque compatibility identities and result
  classes; it must not contain non-public source or operational details.
- V17 records are immutable history.

## Required boundary

Every owned operation follows this sealed order:

```text
resolve descriptor
invoke descriptor-aware charge
return the exact error if charge fails
execute target exactly once
invoke completion observer exactly once
return the target result
```

The charge invocation is the only callback allowed before approval. Attempt
telemetry belongs to that invocation and is not a completion observation.

## Candidate lifecycle

Evidence is generated acyclically:

```text
runtime source
-> source inventory
-> raw proof and mutation artifacts
-> later proof and mutation catalogs that bind producing commits
-> final inventory
-> evidence graph
-> public assurance and conformance
-> opaque compatibility import
-> combined closure and terminal decision
-> strict descendant clean attestation
```

No artifact names its own containing commit as its producing commit.

## Ordered RCLDs

### RCLD 134: authority, reproductions, and contracts

Adopt v18 authority; freeze the baseline; reproduce the known defects from the
frozen candidate; define exact trace, mutation, transcript, descriptor, count,
and candidate-role contracts; and install executable validators. The stage is
green when the authority validators pass and the repository specification gate
recognizes v18.

### RCLD 135: public Rust sealed boundary

Move attempted-descriptor telemetry into descriptor-aware charge adapters,
remove every pre-charge completion-observer call, normalize descriptor
vocabulary, and add focused ordering, exact-error, and zero-post-stop tests.
The stage is green when focused Rust tests and structural validation pass.

### RCLD 136: source inventory and trace-derived proofs

Derive the complete active site inventory from production source and generate
one exact proof per site from structured traces. Every result and count must be
derived for the requested site or the suffix after a failed charge. Helper-only
probes are supplemental and cannot substitute for production-path evidence.

### RCLD 137: isolated structural mutations and replayable transcripts

Run structural property checks as subprocesses against isolated roots. Execute
all seven true call-site target-hoist mutations, supplemental helper mutations,
and provenance mutations. Record separate compile and property commands,
status codes, output hashes, patches, restoration results, and zero survivors.

### RCLD 138: public qualification and committed evidence catalogs

Bind committed source, proof, and mutation artifacts in later catalogs; create
the final inventory and bidirectional graph; measure the budget transition; and
run the complete public assurance, conformance, package, policy, supply-chain,
resource, robustness, documentation, and release-evidence gates.

### RCLD 139: independent compatibility evidence

In the independently owned history, add descriptor-aware attempt telemetry,
trace-derived production-path proofs, all seven call-site mutations, replayable
transcripts, and two-process parity over the selected 204-scenario distribution.
Commit and validate this stage before any public import.

### RCLD 140: opaque join, terminal decision, and clean descendant

Import only the approved opaque compatibility record. Validate leak resistance,
combined assurance, finding closure, candidate roles, ancestry, and frozen
invariants. Commit the terminal artifacts, then create and verify a strict
descendant clean-candidate attestation. Run the required public gates twice from
clean committed states.

## Per-stage discipline

Only one RCLD is active at a time. Each stage must inspect its complete diff,
run the narrowest credible focused checks plus its specified repository gate,
and commit only when green. A failure blocks the next stage unless proven to be
pre-existing and outside the active scope. Every stage records changed files,
commands, results, deviations, residual risks, and next-stage safety.

All seven RCLDs reached their local acceptance criteria in dependency order.
Findings 123 through 129 are locally closed. `FINDING_080`, publication,
release, deployment, and every other external hold remain held, with no remote
action performed. The terminal decision is complete and must be followed by
the separately committed clean-descendant attestation required above.
