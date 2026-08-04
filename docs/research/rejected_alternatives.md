# Rejected protocol alternatives

These alternatives remain rejected for draft v1. Reconsidering one requires a
new ADR and the consensus change-control process.

## Generic CRDT envelope

Rejected because a carrier envelope without exact engine semantics cannot make
independent implementations converge. Draft v1 defines one sealed Automerge
profile.

## Relay-defined sequence or conflict winner

Rejected because relay order, arrival order, and relay-local sequence state are
not portable across acquisition channels. Clients derive canonical state from
signed evidence and causal dependencies.

## New relay messages or event range

Rejected because core validity does not require CRDT-aware relays. The protocol
uses ordinary immutable/addressable events and existing query mechanisms.

## Timestamp or last-writer-wins authorization

Rejected because device clocks and relay arrival are not trustworthy causal
boundaries. Complete controller ACLs and base frontiers define epochs.

## Shared online signer

Rejected because it creates an availability and trust dependency. Authorized
device keys sign immutable changes independently and offline.

## Generic caller-supplied kinds or limits

Rejected because configurable validity rules would let incompatible clients
claim the same revision. Kinds, limits, encodings, and selection algorithms are
sealed.

## Second logical clock

Rejected because Automerge already carries actor sequences, operation counters,
and dependency hashes. A Lamport, vector, or MMR clock could disagree with the
actual change graph.

## Replaceable change history

Rejected because replacement can discard a concurrent branch before another
client receives it. Changes and controls are immutable evidence.

## Controller-endorsed missing-history recovery

Rejected for draft v1 because an advisory replaceable manifest cannot safely
replace historical carrier signatures. Checkpoints require verified history.

## Normative Automerge save-byte digest

Rejected until independent Rust and JavaScript implementations prove
byte-identical saves. Draft conformance uses history and disposition digests
plus typed materialized-state assertions.

## Incremental evaluator as initial oracle

Rejected because incremental mutable state is harder to reason about and test
for delivery-order invariance. Complete deterministic batch replay is the
initial reference oracle.
