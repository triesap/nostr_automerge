# Step 583 Green-Slice Deviation

Status: approved workflow adaptation
Affected checkpoints: `step_583` through `step_595`

The dynamic carrier disposition regressions require the reducer and report
invariants to compile, while the active `pfc` workflow forbids committing a
red repository snapshot. The manifest reducer, checkpoint status mapping,
static-mapping removal, canonical record merge, digest coverage, cross-status
invariants, existing report-schema compatibility, tests, permutations, and
mutations are therefore committed as one green TDD slice.

No requirement is removed or weakened. Every statically valid manifest,
descriptor, and chunk begins non-accepted and receives its final event
disposition only from replacement or verification outcomes. The existing
language-neutral event namespace already represents these outcomes without a
schema extension, so the schema and writer remain byte-compatible while their
expected disposition values are regenerated.
