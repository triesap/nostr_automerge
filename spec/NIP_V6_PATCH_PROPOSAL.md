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
