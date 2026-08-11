# ADR 0027: conflict-aware state projection

## Decision

`MaterializedPathElement` keeps `Key` and `Index` and adds `Branch`. A branch
element contains the parent object identity, the operation identity that chose
the conflicting value, and the child object identity. The element is inserted
after the conflicted property and before every descendant key or index. This
context is retained even when the selected child has no descendants.

The identities use Automerge's stable external identifier strings. Ordering is
the derived tuple order of path element kind and each identity string. The
projector sorts all property conflicts by operation identity before scheduling
child projection, so stack order cannot affect canonical output.

Every projected mark contains its branch-aware text path, name, exact scalar
value, UTF-16 half-open range, and one of the four expansion values `none`,
`before`, `after`, or `both`.

Neutral assertion paths use the same key, index, and branch-element model. An
assertion succeeds only when its complete path resolves to exactly one entry;
zero or multiple matches are errors. Ordinary paths therefore cannot silently
select among conflicting composite branches.

## Consequences

Nested conflicting object descendants cannot collapse onto one ordinary path,
and ambiguous assertions fail rather than select the first match. Independent
implementations can reproduce the representation without importing Rust or
Automerge types. Projection remains iterative and charges each object,
property, conflict, path element, text unit, mark, and canonical sort unit.
