# RCLD 34 Combined Green Slice

Steps `step_597` through `step_611` are implemented as one reviewable TDD
slice because control preparation, ancestry indexing, optional post-processing,
checkpoint interruption, and their exact budget boundaries share one caller
budget and cannot be made independently green without temporarily leaving
unmetered paths.

The slice preserves the existing typed counters and protocol outputs while
adding proportional ownership for every newly inventoried traversal. A stop
before authoritative evaluation returns a constant-size report. A later stop
returns a compact report containing already-conclusive control, change,
manifest, and completed checkpoint outcomes, without scanning retained evidence
or expanding refusals for remaining descriptors.

The canonical report schema does not change: compact interrupted reports use
the existing completion and failure fields and existing optional collections.
The source panic audit found no evidence-derived `unwrap`, `expect`, `panic!`,
or `unreachable!` in the affected production modules. Phase closure and full
evidence refresh remain assigned to `step_612`.
