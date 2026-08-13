# External NIP V5 Reconciliation Proposal

Status: local implementation delta; not submitted

The externally authored NIP remains read-only in this repository. Its next
authorized revision should absorb the following interoperability rules without
changing `NIP-XX`, provisional event kinds, coordinates, wire encodings,
signature rules, roles, revisions, profiles, or hash domains:

1. Define `next_op(C)` from the maximum operation counter in the exact accepted
   dependency closure, or one for an operation-empty closure, independently of
   actor-local sequence.
2. Define reportable target evidence, direct nonreportable lifecycle support,
   unattributable evidence, and unrelated-coordinate output and work isolation.
3. Define `ChangeHash` as semantic identity, carrier events as claims, accepted
   non-poisoning, accepted-base filtering, and the exact final claim precedence.
4. Define accepted-base, same-epoch, pruned, other-control, invalid,
   unsupported, prior-equivocation-excluded, and unknown dependency knowledge;
   only unknown or unresolved selected-control evidence is pending.
5. Define manifest prevalidation attribution from exactly one distinct valid
   `d` value before strict validation and no fallback from the selected event.
6. Define the shared referenced-control states and the manifest, claim, and
   checkpoint mappings, including invalid draft-v1 dependants of unsupported
   control evidence.
7. Define dynamic manifest, descriptor, and chunk event dispositions and their
   participation in the dispositions digest.
8. Define local completion as non-authoritative, cancellation before target
   lookup, typed reserved finalization, and constant pre-reservation fallback.

After separately supplied prose is accepted locally, record its exact hash,
reconcile every registry source, rerun both implementations, and regenerate all
authority-bound evidence. This proposal grants no submission or allocation
authority.
