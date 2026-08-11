# Step 542 Green-Slice Deviation

Status: approved workflow adaptation
Affected checkpoints: `step_542` through `step_553`

The first poisoning checkpoints specify regression tests that fail against the
reviewed implementation, while the active `pfc` workflow forbids committing a
red repository snapshot. The regression tests and the smallest authoritative
epoch-result refactor are therefore committed as one green TDD slice.

No requirement is removed or weakened. The combined slice proves bad
`start_op`, missing-predecessor, base-omission, and accepted-base sequence-reuse
cases; false-alert suppression; exact accepted state and alert propagation;
removal of the second broad quarantine; and exclusion of rejected candidates
from accepted maps. Subsequent conformance, permutation, mutation, and phase
closure checkpoints remain separately reviewable.
