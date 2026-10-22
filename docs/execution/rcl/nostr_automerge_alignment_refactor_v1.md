# Nostr Automerge alignment refactor RCL program

Approved intent: `nostr_automerge_alignment_refactor_v1`. Status: active bootstrap;
no runtime repair or new example is complete merely because its plan is adopted.

## Authority and baseline

The NIP and existing focused protocol contracts retain wire authority. The
[adopted contracts](../../../spec/nostr_automerge_alignment_refactor_v1/PRODUCT_SPEC.md),
[authority](../../../spec/nostr_automerge_alignment_refactor_v1/authority.json),
[ordered steps](../../../implementation/nostr_automerge_alignment_refactor_v1/COMMIT_SEQUENCE.md)
and [ledger](../../../implementation/nostr_automerge_alignment_refactor_v1/ledger.json)
govern this approved post-v19 implementation. The ledger is the mechanically
checked cursor; this document does not duplicate live completion status.

Reviewed public anchor: `5085fd30f873396c8e882a8cd7c5aec241e73824`.
Qualified compiler-maintenance baseline:
`e2faf8fea65f23479a70009c5d9531b881838c0e`. Its complete 24-file delta was
reviewed, including the active inventory refresh. Current development Rust and
MSRV are 1.99.0; old compiler observations remain historical evidence. The
[baseline](../nostr_automerge_alignment_refactor_v1/baseline.md) distinguishes
fresh execution from original review observations.

Historical v19 RCLDs 141–146 are complete with active/next null. Preserve their
authority, ledger, T/E/A lineage, signed fixtures, report bytes and holds.
Historical receipt validation is not a rerun of its original campaigns.
The public repository builds and verifies without private parent material,
private source, operator paths, credentials or a private implementation.

## Ordered work

Execute NA-001 through NA-166 in original ordinal order. Preserve every original
ID, alias, prerequisite, requirement, scope, test, command and expected result.
RCLDs organize these entries; they are not permission to squash steps. Keep one
active step, review complete diffs, repair red work and commit only verified green
slices. Local commit authorization does not grant push or publication.

| RCLD | Original entries | Scope |
| --- | --- | --- |
| 147 | NA-001–NA-008 | Baseline, authority, ledger, historical/current validation, compatibility and threat model |
| 148 | NA-009–NA-015 | Portable execution, example infrastructure, current selector and comparator |
| 149 | NA-016–NA-027 | History-bound actors, counters, authoring atomicity and control construction |
| 150 | NA-028–NA-034 | Trusted accepted-state bridge and typed missing-reference recovery |
| 151 | NA-035–NA-052 | Semantic editing, UTF-16, atomic batches and bounded work |
| 152 | NA-053–NA-056 | First four public core examples |
| 153 | NA-057–NA-061 | Bounded iterative causal fan-in |
| 154 | NA-062–NA-065 | Full-save checkpoint framing |
| 155 | NA-066–NA-073 | Versioned checkpoint isolation, reporting and complete-route qualification |
| 156 | NA-074–NA-076 | Bounded inventory, public checkpoint preparation and new distribution |
| 157 | NA-077–NA-094 | Remaining core/interop examples and core acceptance |
| 158 | NA-095–NA-108 | Native companion, immutable evidence and durable storage |
| 159 | NA-109–NA-116 | Actor fencing, prepared signing and atomic outbox |
| 160 | NA-117–NA-128 | Real local relay, persisted-byte retry and acquisition |
| 161 | NA-129–NA-136 | Restart, reorganization, process crashes, backup and restore |
| 162 | NA-137–NA-144 | Four native examples and native acceptance |
| 163 | NA-145–NA-156 | Current assurance, extracted packages and native/compiler matrix |
| 164 | NA-157–NA-160 | Complete gate graph, docs and all executed examples |
| 165 | NA-161–NA-166 | Clean qualification, sustained campaigns and truthful readiness |

## Recorded bootstrap and supplemental mappings

The [bootstrap deviation](../../../implementation/nostr_automerge_alignment_refactor_v1/deviations/bootstrap.md)
records the proven conflict between separate documentation commits and current
cleanliness/frozen-baseline gates. NA-001–NA-005 retain their acceptance but share
one coherent source commit S. Only NA-001 is active initially. A following
observation binds actual S and separately identifies each original step's passing
acceptance before completing all five and advancing to NA-006. No source or
receipt claims its own future commit. A source repair creates a new candidate.

Only the specification-baseline wrapper and clean-attestation check initially
move to immutable-owner validation. Keep the other 55 selected unique Python
leaves, current requirement coverage and Rust gates current. Validate live frozen
artifacts before a historical pass. Later source-sensitive route changes require
explicit current replacements and recorded evidence before proceeding.

The compiler mapping changes current NA-149/support obligations to Rust 1.99.0;
it does not relabel historical 1.97.1/1.92.0 observations. The supplemental bounded
public checkpoint-preparation slice belongs to RCLD 156 before NA-076/NA-082 and
retains the requirement mapping in the deviation. Neither mapping changes signed
wire behavior or grants a new publication feature.

## Per-step green and review

Use each entry's exact tests and commands, with current formatting, locked
check/tests, strict Clippy, docs and xtask as required. Add regressions with
behavior and run relevant doctests, conformance, resource, package and native
lanes. Execute example bodies and nonempty semantic tests; compilation or a
zero-test selection does not establish behavior. Operator output routing may
wrap portable argv without becoming a public build dependency.

Independently inspect results and obtain required fresh adversarial review
before local green commits. Record actual source commit, changed files,
requirements, command/toolchain/results, meaningful assertions/counts, deviations,
unverified work and next-step safety. Keep implemented, verified and independently
accepted states distinct. Missing tools, timeout, absent output and mismatch are
different outcomes. Historical proof validation cannot substitute for current
execution or an independent implementation.

## Boundaries and recovery

Keep the core pure, deterministic and batch-oriented; integration, persistence,
networking, signing/key custody and application schemas stay outside it. Preserve
strict signed ingress, immutable original evidence, iterative bounded work and
atomic failure behavior. A checkpoint semantic change requires an explicit
revision/successor decision and complete coupled report/fixture/resource proofs
before activation. Do not silently change the sealed revision or legacy default.

No private TypeScript source/history import is part of this program. An external
runner supplies independent evidence only after actual same-candidate execution.
Browser/WASM, NIP-77, editor, private transport and external-signer protocol work
remain conditional. Native Linux, independent review/interop and sustained
campaigns must actually run for corresponding qualification claims. Event-kind
allocation, NIP submission, external assurance, production qualification,
publication, release, deployment and remote mutation remain held separately.

On failure repair/split or block the slice; never commit red work or weaken
acceptance. Preserve unrelated changes, branches and the exact restart cursor;
do not reset, clean or auto-stash. Resume the first incomplete eligible ledger
entry after reconciling source commits and observations. A green commit, phase
boundary or context size is not a reason to stop an approved full-range batch.
