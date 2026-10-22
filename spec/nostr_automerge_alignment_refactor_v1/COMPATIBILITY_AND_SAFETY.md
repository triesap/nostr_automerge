> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Compatibility, safety and change control

## Revision contract

The reviewed draft is sealed. Its NIP states that changes to accepted evidence, actor derivation, control selection, frontier semantics, Automerge encoding or checkpoint verification require a new version, and a document changing its core format creates a successor. Read the exact current clause before coding. This handoff approves the desired repairs and availability behavior, not an implicit rewrite of that rule.

Classify each change in an ADR before implementation:

| Class | Examples | Required evidence |
| --- | --- | --- |
| Public API/local-state repair | Reject impossible restore counters; add safe report-to-authoring API | Encoded-semantics regressions, migration notes, no unintended accepted-wire change |
| Implementation repair to an existing rule | Reject a raw change chunk where full checkpoint save is already required | Exact normative citation, signed positive/negative vectors, old/new outcome matrix |
| New/clarified normative behavior | Ignore unrelated unauthorized extra chunk while retaining its rejected event outcome if current authority does not already require it | Explicit rule, revision/successor decision, updated requirement/fixture authority, independent differential evidence |

Do not invent revision/domain strings, event kinds, alternate acceptance modes or consensus-changing feature flags. Determine unused identifiers through the repository's existing scheme and the relevant authority. Local candidate design can proceed under an approved scope; external allocation/submission/publication remains a separate held action.

## Actor safety

Actor identity derives from the documented coordinate/device key rule. Per-actor sequence and causal operation counter are different quantities. Returned durable state must reflect actual encoded change semantics, not the caller's unchecked arithmetic. For a fresh actor on a nonempty document, the next operation counter follows accepted causal history. The last local change may be an accepted ancestor after remote work. A history that pruned or superseded that local state must fail with an explicit compatible recovery path, not silently reset sequence one under the same actor.

Zero-operation merges advance sequence and preserve the appropriate causal operation state. Retrying a publish uses the existing prepared/signed identity; generating a different change at the same sequence can create equivocation. Backup restore to a new independent writer requires the documented actor ownership/recovery decision; never quietly clone an active actor record into competing installations.

## Checkpoint safety

Full-save framing is a structural encoding requirement, not a demand for equal `save()` bytes across implementations. A generic loader accepting a raw change does not prove a valid checkpoint. Validate complete framing, bounded sizes and commitments, and preserve history/authorization validation. Correct hashes/signatures alone cannot make the wrong encoding valid. Snapshot-only restoration does not establish signed immutable history.

The observed hostile-extra-chunk behavior changes checkpoint availability but did not change accepted history in the reproducer. Treat it as a protocol availability weakness requiring the explicit selection policy, not as a demonstrated document-corruption exploit. If invalid candidates are isolated, their Event outcomes must remain visible and budgeted; a large malicious candidate set still needs bounded processing.

## Rust and resource rules

Use `#![forbid(unsafe_code)]` in owned crates; preserve workspace lints. No `unwrap`, `expect` or panic on untrusted input. Tests can use trusted-fixture assertions under existing lint policy. Check arithmetic/conversions and bounds before allocations where possible. Keep graph algorithms iterative. Add process-level CPU/RSS evidence for bulk upstream decode paths; choose measurable budgets in the dedicated qualification decision rather than inventing unexplained numeric guarantees here.

No wall-clock or arrival-order state decisions, unordered-map-derived canonical output, display-string canonicalization, signed-input repair, tolerant mode or global mutable state. Relay limits and timeouts affect transport work and availability only. Feature flags may select native adapters/capabilities but never change accepted protocol semantics for the same revision.

## Privacy and dependency rules

Default logs omit event contents, raw changes, private keys and full coordinates. Use synthetic keys/data in reproducible tests. Keep private TypeScript source, its package/repo location, operator paths and raw private logs out of the public Rust repository and final public artifacts. Public neutral runner contracts, independently supplied implementations and permitted opaque attestations are appropriate.

Audit new dependencies for license, MSRV, transitive native requirements, maintained APIs, runtime/platform support and package size. The current Rust core's Rust API does not automatically imply WASM compatibility: its secp256k1 dependency has native backing, and upstream randomness/platform behavior must be checked. Browser support remains conditional.

## API and stored-data migration

Document public alpha API changes with old/new usage and error semantics. Decide whether to deprecate or make a deliberate alpha breaking change; do not retain an unsafe compatibility path solely to avoid an API change. Local store schema versions are not wire protocol versions. A store migration may rebuild derived projections but must not rewrite signed immutable evidence or assume stale actor state is safe. Backups must declare format/version/integrity and which actor recovery claims are supported.

## Qualification and external holds

Historical v19 closed its local findings and contains opaque assurance; that does not close this new program's findings. Keep release readiness separate from implementation completion. The existing holds include external assurance, event-kind allocation, NIP submission, production qualification, publication, release, deployment and remote mutation. A locally complete candidate may be reported while specific external qualifications remain blocked, but it must not be called fully qualified/canonical interoperability until those criteria have evidence.
