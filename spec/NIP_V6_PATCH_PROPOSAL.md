# External NIP V6 Reconciliation Proposal

Status: local portable editorial delta; not submitted

This document mirrors implementation-owned companion authority for later use
by the external NIP author. It grants no submission, allocation, publication,
or adoption authority and does not change the checked-in NIP snapshot.

## Causal operation counters

Define the next operation counter for a candidate from the maximum operation
counter in its exact accepted dependency closure, or one when that closure has
no operations. Require the candidate `start_op` to equal that value with
checked arithmetic. Keep actor-local sequence distinct: sequence starts at one
and increments exactly, while an empty change consumes sequence without
advancing the causal operation counter.

## Coordinate-scoped evaluation

Define reportable target evidence, direct nonreportable lifecycle support,
unattributable evidence, and unrelated-coordinate evidence. Require every
target output and target-local work counter to depend only on attributable
target evidence and explicit lifecycle support. Require cancellation and
capacity checks before target lookup or target-proportional allocation.

## Semantic ChangeHash claims

Define `ChangeHash` as semantic identity and signed change carriers as
independent claims. Group all attributable claims by hash, retain each claim's
reasoned state, accept one dynamically valid claim as sufficient, prevent
invalid or unresolved claims from poisoning it, and filter hashes already in
the selected accepted base from new epoch admission.

## Dependent change authorization

Make a draft-v1 change invalid when it references known unusable or unsupported
control evidence; do not propagate `unsupported_revision` to the dependant.
Require device, ActorId, and write-role authorization before branch disposition.
An unauthorized noncanonical claim and every terminal-control claim are invalid.

## Final claim precedence

Specify the ordered hash reduction table: accepted closure, canonical-ancestor
pruning, genuinely unresolved claims, otherwise-valid noncanonical claims,
all-unsupported carriers, then conclusive invalidity. State explicitly that
accepted and pruned lineage cannot be poisoned and pending outranks
noncanonical or invalid duplicate claims.

## Complete dependency knowledge

Define all eight prior states: accepted base, same-epoch candidate, pruned
canonical ancestor, other control, invalid, unsupported,
prior-equivocation-excluded, and unknown. Only absent or unresolved evidence is
pending; every known-impossible state invalidates the dependant transitively.

## Control parent and frontier references

Define exhaustive parent states and map only missing or pending evidence to a
pending child. Make every known-unusable parent invalid. Define accepted,
pending, missing, invalid, excluded, unsupported, and other-control frontier
states with the same absent-versus-known distinction. Propagate pending and
invalid ancestry, and validate noncanonical branches relative to their own
ancestry before exclusion.

## Descriptor and chunk references

Require every target chunk to resolve its referenced descriptor. Missing or
pending descriptor evidence is pending; wrong-kind, wrong-coordinate, invalid,
unsupported, or dynamically unusable evidence is invalid. Require complete
binding and set verification before acceptance, consistent event dispositions
for a verified set, and promotion of a retained orphan after descriptor
delivery.
