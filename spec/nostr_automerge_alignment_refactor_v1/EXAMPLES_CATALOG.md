> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Canonical examples catalog

Program: `nostr_automerge_alignment_refactor_v1`
Status: **implementation specification; none of these 19 root scenarios is claimed implemented by this handoff**.
Reviewed baseline: `5085fd30f873396c8e882a8cd7c5aec241e73824`.

The required target catalog contains **19 scenarios: 14 pure-core examples, four external integration examples, and one public interoperability example**. Each is a release-readiness deliverable for the approved alignment scope. The five extensions at the end are deferred and excluded from that count. The current repository's two package-local quickstarts remain supported and packaged; they are not substitutes for this catalog.

The public crate remains deterministic, batch-oriented, pure Rust, and independent of networking, storage, async runtimes, key custody, platform bindings, and application schemas. Integration examples consume separately owned components. Public examples use only exported semantic APIs; they do not expose upstream Automerge types or reach into private adapters.

This catalog follows the repository authority order: NIP draft, V1 and focused contracts, requirements/approved ADRs/fixtures, then the approved alignment execution authority. It does not change sealed wire behavior. Historical v19 authority, source-bound scripts, attestations, and receipts retain their exact bytes and original candidate identities.

## Package and execution design

| Directory | Planned workspace package | Contents | Published |
|---|---|---|---|
| `examples/core/` | `nostr_automerge_examples_core` | Fourteen explicit Cargo example targets plus thin public-API helpers | No |
| `examples/integration/` | `nostr_automerge_examples_integration` | Four explicit Cargo example targets and external transport/storage glue | No |
| `examples/interop/` | `nostr_automerge_examples_interop` | One explicit Cargo example target and public current-vector orchestration | No |
| `crates/nostr_automerge/examples/` | Existing public package `nostr_automerge` | Repaired `basic_authoring` and `public_engine` quickstarts | Included in crate |

E02 introduces the three workspace packages with `publish = false` and `autoexamples = false`. Every implemented scenario receives an explicit `[[example]]` entry with `name`, `path`, and `test = true`. An empty target or success-only stub is never an implemented scenario. The compiler must continue checking the public-core dependency boundary.

Each scenario directory contains `main.rs` and `README.md`; additional fixture/data files are allowed only when needed. The README states the learning objective, prerequisites, exact commands, expected assertions and statuses, cleanup, limits, and the relevant requirements. A small shared `run` function performs the actual scenario and assertions; `main` and a `#[test]` wrapper call that logic. Integration examples can inject a deterministic test transport into the shared scenario while the dedicated local-relay run executes the real network/process path. No default unit test contacts a public relay or downloads a tool.

Cargo compiles examples during ordinary testing, and an example test target normally substitutes the libtest harness for `main`. Therefore every scenario needs both a real binary execution and a nonempty test wrapper. The catalog runner checks declared assertion IDs, process exit, and candidate-bound receipts; successful compilation and printed claims cannot qualify it. See the primary Cargo references at the end.

### Run lanes

| Lane | Actual work | Default network behavior |
|---|---|---|
| `core` | Execute all selected core binaries and their semantic tests | No network |
| `local_relay` | Run selected integration process scenarios against a pinned real relay, plus deterministic tests | Loopback only; prepared explicitly |
| `interop_public` | Execute the selected public Rust distribution twice and run the real comparator's negative controls | No independent implementation access needed |
| `interop_independent` | Execute the same distribution in a separately identified implementation and compare canonical reports | Explicit private operator runner configuration; public receipts are redacted |

`run_examples.py`, `local_relay.py`, and `interop.py` under `scripts/alignment_v1/` are **planned tools introduced by the stated E steps**. The commands below become valid only after those steps and their parent prerequisites land. Current commands and observed results are listed separately in `VERIFICATION.md`.

## Six-example learning path

Read and execute these in order:

1. `signed_round_trip`: make, sign, ingest, evaluate, and assert one edit.
2. `authoring_resume`: continue the same actor after another actor's accepted descendant.
3. `concurrent_offline_merge`: demonstrate useful two-device collaboration and continue editing after merge.
4. `dependency_recovery`: understand out-of-order delivery, pending evidence, and duplicates.
5. `relay_round_trip`: perform the full journey in two processes through a real ordinary relay.
6. `durable_outbox`: survive crashes and ambiguous acknowledgements without losing or resigning a committed edit.

This is the priority development and tutorial order, not a reduced release catalog. E43 executes the full six-scenario group from fresh state. The other thirteen close protocol, recovery, and interoperability gaps before the catalog is qualified.

## Complete directory index

| Exact directory | Lane | Step | Priority position |
|---|---|---|---|
| `examples/core/signed_round_trip/` | `core` | E05 | 1 |
| `examples/core/authoring_resume/` | `core` | E06 | 2 |
| `examples/core/concurrent_offline_merge/` | `core` | E07 | 3 |
| `examples/core/data_model/` | `core` | E09 | — |
| `examples/core/dependency_recovery/` | `core` | E08 | 4 |
| `examples/core/authorization_lifecycle/` | `core` | E10 | — |
| `examples/core/control_forks/` | `core` | E11 | — |
| `examples/core/device_equivocation/` | `core` | E12 | — |
| `examples/core/manifest_replacement/` | `core` | E13 | — |
| `examples/core/verified_checkpoints/` | `core` | E14 | — |
| `examples/core/budgets_and_cancellation/` | `core` | E15 | — |
| `examples/core/invalid_and_unsupported/` | `core` | E16 | — |
| `examples/core/document_isolation/` | `core` | E17 | — |
| `examples/core/frontier_fan_in/` | `core` | E18 | — |
| `examples/integration/relay_round_trip/` | `local_relay` | E21 | 5 |
| `examples/integration/durable_outbox/` | `local_relay` | E22 | 6 |
| `examples/integration/multi_relay_recovery/` | `local_relay` | E23 | — |
| `examples/integration/backup_restore/` | `local_relay` | E24 | — |
| `examples/interop/current_vectors/` | `interop_public` | E27 | — |

## Scenario contracts

All scenarios trace to `NA-EX-001` (complete catalog) and `NA-EX-002` (executed semantic assertions). The six priority scenarios also trace to `NA-EX-003`. The parent requirements matrix links their protocol assertions to the approved core or integration requirements; the example catalog does not invent new wire requirements.

### 01. `signed_round_trip`

**Exact directory:** `examples/core/signed_round_trip/`
**Target:** `nostr_automerge_examples_core` / `signed_round_trip`
**Implementation step / lane:** E05 / `core`
**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003.

**Purpose.** Provide the first copyable path from local edit to trusted materialized state.

**Preconditions.** Public drafts/signing helpers, strict ingress, accepted report, and typed materialized access. Separate disposable controller and device keys; bounded fixed values.

**Required assertions.**

- Assert exact title/value, canonical control, accepted change, head, complete status, and available manifest.
- Assert the carrier Event and semantic ChangeHash are separate report identities.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example signed_round_trip
```

### 02. `authoring_resume`

**Exact directory:** `examples/core/authoring_resume/`
**Target:** `nostr_automerge_examples_core` / `authoring_resume`
**Implementation step / lane:** E06 / `core`
**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003.

**Purpose.** Turn the reviewed A-to-B-to-A and counter failures into a durable consumer-level contract.

**Preconditions.** Report-bound authoring bridge plus fixed ancestor/sequence/causal-operation validation. A complete accepted history and persisted caller-owned actor state.

**Required assertions.**

- A-to-B-to-A succeeds with exact encoded and returned sequence/counter agreement.
- Forged or stale sequence, causal counter, actor identity, and excluded last-change state fail through the documented API.
- Failed authoring leaves document bytes, heads, and actor state unchanged.
- A missing local last-authored hash and malformed actor state cannot be replaced by trusting caller-provided counters; failures preserve caller state atomically.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example authoring_resume
```

### 03. `concurrent_offline_merge`

**Exact directory:** `examples/core/concurrent_offline_merge/`
**Target:** `nostr_automerge_examples_core` / `concurrent_offline_merge`
**Implementation step / lane:** E07 / `core`
**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003.

**Purpose.** Prove the ordinary complete user journey rather than only first-write ingestion.

**Preconditions.** Safe authoring continuation and the approved operation API. Two separately identified devices start from the same accepted history.

**Required assertions.**

- Both independent edits are present; concurrent conflicts are preserved where intentionally created.
- Accepted sets, heads, history/disposition digests, and typed state agree for equivalent complete evidence.
- A post-merge edit from each device is accepted with correct actor counters.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example concurrent_offline_merge
```

### 04. `data_model`

**Exact directory:** `examples/core/data_model/`
**Target:** `nostr_automerge_examples_core` / `data_model`
**Implementation step / lane:** E09 / `core`
**Requirements:** NA-EX-001, NA-EX-002.

**Purpose.** Make the authoring API demonstrate edits to existing objects.

**Preconditions.** The complete approved authoring operation surface and typed materialized object/conflict/text access. Supported scalar and mark semantics must be documented; the example cannot imply every upstream Automerge feature is supported.

**Required assertions.**

- Assert exact nested values, list ordering, counter total, conflict alternatives, UTF-16 length and mark intervals.
- A text edit preserves the original text object identity rather than replacing it.
- Authoring and strict reingestion agree.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example data_model
```

### 05. `dependency_recovery`

**Exact directory:** `examples/core/dependency_recovery/`
**Target:** `nostr_automerge_examples_core` / `dependency_recovery`
**Implementation step / lane:** E08 / `core`
**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003.

**Purpose.** Teach pending evidence and idempotence early in the learning path.

**Preconditions.** Signed event preparation, strict ingestion, and reports exposing pending/invalid/accepted dispositions plus missing-dependency identities.

**Required assertions.**

- Pending promotes to accepted only after real evidence arrives.
- Known-invalid dependencies remain invalid; missing and invalid are not conflated.
- Duplicate carriers do not apply a change twice or change materialized state.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example dependency_recovery
```

### 06. `authorization_lifecycle`

**Exact directory:** `examples/core/authorization_lifecycle/`
**Target:** `nostr_automerge_examples_core` / `authorization_lifecycle`
**Implementation step / lane:** E10 / `core`
**Requirements:** NA-EX-001, NA-EX-002.

**Purpose.** Show safe grants, revocation, fresh-device replacement, freeze, and successor handling.

**Preconditions.** Parent work has settled and implemented the causal control/lifecycle contract, including retained-writer frontier, permanent revocation, role rules, terminal controls, and successor continuity. Fixtures use that approved contract.

**Required assertions.**

- Retained-writer frontier violations are rejected.
- A removed key cannot be reintroduced or regain elevated roles.
- Terminal controls authorize no later edit or valid child; successor links remain separately validated.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example authorization_lifecycle
```

### 07. `control_forks`

**Exact directory:** `examples/core/control_forks/`
**Target:** `nostr_automerge_examples_core` / `control_forks`
**Implementation step / lane:** E11 / `core`
**Requirements:** NA-EX-001, NA-EX-002.

**Purpose.** Expose governance failures without allowing arrival time to choose history.

**Preconditions.** Stateful control validation and lifecycle example. The public reevaluation/alert API can compare reports without selecting by arrival time.

**Required assertions.**

- Only statefully valid siblings participate in lowest-ID selection.
- Noncanonical branch state is evaluated against its own parent.
- Reorganization alert and affected accepted/excluded history match the actual before/after reports.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example control_forks
```

### 08. `device_equivocation`

**Exact directory:** `examples/core/device_equivocation/`
**Target:** `nostr_automerge_examples_core` / `device_equivocation`
**Implementation step / lane:** E12 / `core`
**Requirements:** NA-EX-001, NA-EX-002.

**Purpose.** Make the device-key uniqueness rule executable and understandable.

**Preconditions.** Correct actor sequence handling and approved equivocation/nonpoisoning semantics. Two conflicting signed events represent an intentionally cloned test device.

**Required assertions.**

- True equivocation and descendants receive specified quarantine/excluded outcomes and integrity alerts.
- No timestamp or arbitrary hash winner replaces the required quarantine.
- Invalid duplicate claims remain visible without poisoning sufficient accepted evidence.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example device_equivocation
```

### 09. `manifest_replacement`

**Exact directory:** `examples/core/manifest_replacement/`
**Target:** `nostr_automerge_examples_core` / `manifest_replacement`
**Implementation step / lane:** E13 / `core`
**Requirements:** NA-EX-001, NA-EX-002.

**Purpose.** Separate advisory discovery from history authority.

**Preconditions.** Public manifest/control preparation and replaceable selection diagnostics. Metadata and hints are available separately from authoritative history.

**Required assertions.**

- Replacement selection occurs before semantic validation and never falls back to an older manifest after selected invalidity.
- Valid hints cannot grant a write role, select a control branch, or authorize a checkpoint.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example manifest_replacement
```

### 10. `verified_checkpoints`

**Exact directory:** `examples/core/verified_checkpoints/`
**Target:** `nostr_automerge_examples_core` / `verified_checkpoints`
**Implementation step / lane:** E14 / `core`
**Requirements:** NA-EX-001, NA-EX-002.

**Purpose.** Cover the checkpoint trust boundary using the repaired public engine.

**Preconditions.** Public checkpoint authoring plus repaired full-save framing and sufficient-carrier verification. The example retains complete historical signed carriers for verified replay.

**Required assertions.**

- Full save plus valid complete signed history verifies and matches replay.
- A raw Change Chunk does not qualify as a full-save checkpoint.
- Invalid extra carriers cannot poison a sufficient valid checkpoint set under the approved repaired rule.
- Missing historical signatures never become verified provenance.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example verified_checkpoints
```

### 11. `budgets_and_cancellation`

**Exact directory:** `examples/core/budgets_and_cancellation/`
**Target:** `nostr_automerge_examples_core` / `budgets_and_cancellation`
**Implementation step / lane:** E15 / `core`
**Requirements:** NA-EX-001, NA-EX-002.

**Purpose.** Teach consumers to distinguish local incomplete execution from protocol invalidity.

**Preconditions.** Approved deterministic work accounting and stopped-report contract. Cancellation is explicitly injected, without wall-clock timing choosing the semantic result.

**Required assertions.**

- Stopped reports have the prescribed empty/no-progress canonical surface and original stop cause.
- Increasing a local budget cannot alter a complete protocol result.
- Cancelled work does not fabricate partial accepted state or classify valid input as invalid.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example budgets_and_cancellation
```

### 12. `invalid_and_unsupported`

**Exact directory:** `examples/core/invalid_and_unsupported/`
**Target:** `nostr_automerge_examples_core` / `invalid_and_unsupported`
**Implementation step / lane:** E16 / `core`
**Requirements:** NA-EX-001, NA-EX-002.

**Purpose.** Provide representative negative examples without embedding a second validator.

**Preconditions.** Public raw-event ingestion, typed error/disposition results, and bounded valid fixtures that can be deliberately corrupted without bypassing signature verification.

**Required assertions.**

- Assert exact typed diagnostics/dispositions for each case.
- Malformed or unauthorized carriers do not poison a sufficient valid carrier.
- No normalization or tolerant fallback repairs signed input.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example invalid_and_unsupported
```

### 13. `document_isolation`

**Exact directory:** `examples/core/document_isolation/`
**Target:** `nostr_automerge_examples_core` / `document_isolation`
**Implementation step / lane:** E17 / `core`
**Requirements:** NA-EX-001, NA-EX-002.

**Purpose.** Show that a multi-document corpus cannot affect one target through unrelated evidence.

**Preconditions.** Target-scoped evidence and budget accounting. Any predecessor/successor support exceptions must match the approved normative rules.

**Required assertions.**

- Target values, controls, heads, digests, completion, and target work counters are unchanged.
- Only explicitly permitted predecessor/successor support is read as nonreportable support.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example document_isolation
```

### 14. `frontier_fan_in`

**Exact directory:** `examples/core/frontier_fan_in/`
**Target:** `nostr_automerge_examples_core` / `frontier_fan_in`
**Implementation step / lane:** E18 / `core`
**Requirements:** NA-EX-001, NA-EX-002.

**Purpose.** Show empty merge changes without changing application state.

**Preconditions.** Safe empty-merge authoring and the approved multi-step bounded fan-in API, including successful frontiers larger than 256 heads. All dependencies exist in accepted history; the maximum 256 dependencies per authored change remains enforced.

**Required assertions.**

- Heads reduce to a control-admissible frontier while typed materialized state stays identical.
- Each empty merge consumes one actor sequence and no operation counter.
- A frontier with more than 256 accepted heads consolidates successfully across multiple authored empty merges; each authored change respects the 256-dependency limit.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --example frontier_fan_in
```

### 15. `relay_round_trip`

**Exact directory:** `examples/integration/relay_round_trip/`
**Target:** `nostr_automerge_examples_integration` / `relay_round_trip`
**Implementation step / lane:** E21 / `local_relay`
**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003, NA-VER-001.

**Purpose.** Complete the fifth priority learning scenario over real Nostr transport.

**Preconditions.** The external transport/client component and E20 pinned real relay harness. Two separate client processes, loopback-only endpoints, fresh temporary stores, and independent disposable device identities.

**Required assertions.**

- Client A durably submits genesis control, manifest and dependent authored change through the correct carrier-only and actor-transition paths before publishing.
- Client B begins with an empty evidence store and joins from only coordinate/configured loopback relay; assert zero preseeded signed history/reports/state, real acquisition of all required carriers and exact evaluated convergence.

- Both processes converge to exact values, heads, and digests.
- Late backfill plus live delivery neither loses nor double-applies an edit.
- OK does not establish permanent retention, and EOSE does not establish global completeness.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane local_relay --example relay_round_trip
```

### 16. `durable_outbox`

**Exact directory:** `examples/integration/durable_outbox/`
**Target:** `nostr_automerge_examples_integration` / `durable_outbox`
**Implementation step / lane:** E22 / `local_relay`
**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003, NA-VER-001.

**Purpose.** Complete the sixth priority learning scenario with atomic durable state and retry.

**Preconditions.** External durable storage/outbox component, a safe actor-state commit protocol, and real relay/client harness. The store supports the explicitly documented transactional and native durability contract.

**Required assertions.**

- No committed local edit is lost and no actor sequence is reused.
- Retry preserves signed bytes/EventId; an ambiguous acknowledgement creates no new signature or timestamp.
- Restarted evaluation and a second replica agree, with bounded retries and queue sizes.
- Inspect stored exact bytes and actor state at each crash point; an eventual matching visible value alone is insufficient evidence of safe persistence.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane local_relay --example durable_outbox
```

### 17. `multi_relay_recovery`

**Exact directory:** `examples/integration/multi_relay_recovery/`
**Target:** `nostr_automerge_examples_integration` / `multi_relay_recovery`
**Implementation step / lane:** E23 / `local_relay`
**Requirements:** NA-EX-001, NA-EX-002, NA-VER-001.

**Purpose.** Verify client reliability independently of one relay retaining complete history.

**Preconditions.** Relay client, durable evidence/outbox, bounded missing-evidence planner, reconnect behavior, and two separately stored local relay processes with a fault proxy.

**Required assertions.**

- Second-relay recovery produces the same complete accepted history as direct replay.
- Timestamp overlap does not miss equal-created_at events; duplicates remain idempotent.
- Failure stays explicitly incomplete when every source omits required evidence.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane local_relay --example multi_relay_recovery
```

### 18. `backup_restore`

**Exact directory:** `examples/integration/backup_restore/`
**Target:** `nostr_automerge_examples_integration` / `backup_restore`
**Implementation step / lane:** E24 / `local_relay`
**Requirements:** NA-EX-001, NA-EX-002, NA-VER-001.

**Purpose.** Make independent recovery possible when relay retention ends.

**Preconditions.** Versioned external backup format, strict import, stored exact signed evidence, and complete source report. A fresh restore directory is required; mandatory restore runs with every relay disconnected.

**Required assertions.**

- Restored controls, accepted hashes, heads, digests, and typed state equal the original complete report.
- A snapshot without historical carriers does not gain verified provenance.
- Derived caches can be discarded and rebuilt without changing evidence.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane local_relay --example backup_restore
```

### 19. `current_vectors`

**Exact directory:** `examples/interop/current_vectors/`
**Target:** `nostr_automerge_examples_interop` / `current_vectors`
**Implementation step / lane:** E27 / `interop_public`
**Requirements:** NA-EX-001, NA-EX-002, NA-VER-002.

**Purpose.** Let contributors reproduce the current Rust distribution and understand independent comparison.

**Preconditions.** Approved current distribution selector, complete public fixture contract, production comparator and real negative controls. An independent operator runner is needed only for the separately labelled independent lane.

**Required assertions.**

- Selected scenario IDs/counts and canonical reports are exact; every expected report was executed, not copied.
- Two Rust processes produce byte-identical complete results.
- A mock or reused Rust runner cannot be labelled the independent implementation.
- Absence of independent runner access is explicitly held; public self-consistency cannot be relabelled cross-implementation compatibility.

**Run after introduction.**

```sh
python3 scripts/alignment_v1/run_examples.py --lane interop_public --example current_vectors
```

## Shared harness, catalog, and receipt contract

E01–E04 establish `examples/catalog.json`, a closed schema, bounded public-API helpers, and an executable runner. A row contains a stable scenario ID, exact directory, Cargo package/target, lane, learning-path position when relevant, prerequisites, command arrays, semantic assertion IDs, requirements, timeout, and implementation status. Unknown fields, duplicate IDs, directory traversal, missing targets, unexpected scenario counts, and deferred scenarios counted as required all fail validation.

During development, `validate_examples.py --allow-planned` permits documented unimplemented rows; it never emits successful execution for them. At qualification, `--require-complete` requires all selected IDs implemented. A lane run cannot select zero examples. At E44, `run_examples.py --aggregate --require-complete` validates fresh same-candidate lane receipts. The final aggregate requires exactly the 19 required IDs once each, with four real integration process runs and one public interop run. Running an example twice does not replace a missing example.

Assertions use manually specified expected values, identities, counts, states, or documented invariants. The helper may extract typed values and compare reports; it must not generate expected results using the production rule under test. Each reported assertion ID is emitted only after its comparison executes successfully. Missing assertions, malformed output, a nonzero exit, unrelated crash, timeout, or process-launch failure cannot produce a pass receipt. At least one controlled corruption test must show that expected-value assertions and the runner actually fail.

The receipt records source candidate, toolchain, package/target, selected scenario, lane, executed command arrays, process exits, semantic assertion IDs, fresh output hashes, bounded sanitized diagnostics, and cleanup status. Environment measurements and canonical protocol values remain separate. Private keys, raw document contents, full private coordinates, operator runner configuration, and private implementation logs do not enter normal tracked evidence. Test keys are fixed and visibly disposable; they are never promoted as production credentials.

Only qualification projections are committed under a new alignment report namespace. Raw outputs go to the configured `NOSTR_AUTOMERGE_OUTPUT_ROOT`, with an ignored default. A receipt references a source commit that already exists; its containing evidence commit is recorded later and cannot be part of its own input identity.

## Real relay execution contract

I02 selects the ordinary relay, I23 pins and owns the single manifest/lifecycle, and E20 reuses that implementation for example wrappers and additional probes from primary upstream source after checking compatibility. `tools/native_integration/relay_manifest.json` records exact revision or artifact digest, license, host applicability, startup argument array, local configuration, and prepare/verification method. This handoff intentionally supplies no fabricated relay version or guessed command. The future implementation commit makes that selection concrete and executes it.

The harness binds loopback, allocates ephemeral ports, isolates relay data and client storage, uses bounded readiness polling, and cleans all children and temporary data after success, failure, and cancellation. It checks signed event publication and exact-ID readback, relevant kinds/tag filters, `OK` and `EOSE` handling, and event-size rejection through the real process. NIP-11 metadata is advisory input; the probe establishes the behavior needed by the examples. A fault proxy may disconnect, withhold, reorder, or duplicate frames while preserving signed event bytes. It must not replace the ordinary relay in the success lane.

The relay path carries signed Nostr evidence and rebuilds accepted Automerge history through the module. It does not claim that Automerge's ordinary stateful peer sync stream can be sent unchanged over unordered relay event delivery. Upstream Automerge documents its sync stream assumptions separately; the repository's sealed event profile controls this module.

Example statuses distinguish durable local commit, relay acknowledgement, current dependency acquisition, and evaluated materialized state. `OK` alone cannot establish long-term retention; `EOSE` alone cannot establish the existence of every relevant event. Missing required relay software or unsupported behavior blocks the integration lane and cannot be hidden by substituting an in-memory fake.

## Package quickstarts

E19 repairs both existing package-local examples. `public_engine` currently places its scenario only in `main`, and the reviewed all-target test log reports zero tests for that example. Both quickstarts must execute their shared scenario from `main` and a test wrapper, sign and ingest the authored change where applicable, and assert concrete materialized values. Their fixtures and dependencies must exist in the published archive. E30–E32 add package-local test assets and licenses and verify extracted execution plus an isolated external consumer.

## Five conditional extensions

These are explicit future scopes, excluded from the required 19, and must not delay or be inferred from the baseline native module implementation. Add one only after its prerequisites, owner, support matrix, and security/testing plan are accepted.

| Future exact directory | Condition for promotion | Purpose and acceptance outline |
|---|---|---|
| `examples/extensions/nip77_reconciliation/` | A supported relay and independently reviewed reconciliation adapter | Discover missing event IDs through NIP-77, fetch actual events with ordinary `REQ`/`EVENT`, and prove identical accepted results to bounded ordinary recovery. NIP-77 itself transfers IDs, not historical provenance. |
| `examples/extensions/external_signer/` | An approved signer/key-custody boundary and real integration access | Handle asynchronous approval, denial, timeout, changed unsigned drafts, and signer identity binding; verify exact signed bytes and durable outbox behavior. |
| `examples/extensions/browser/` | Explicit browser/WASM/JavaScript binding scope and native/browser support commitments | Demonstrate browser persistence and cross-tab/device continuation under an actual browser test runner. A Rust `wasm-release` profile alone is not readiness evidence. |
| `examples/extensions/private_transport/` | An approved confidentiality design and key-lifecycle threat model | Demonstrate encrypt/decrypt/authenticate/recover while keeping the core evidence semantics explicit; test metadata leakage and revoked-key handling within the approved design. |
| `examples/applications/collaborative_editor/` | Stable text/object operations and approved UI adapter scope | Demonstrate cursor/selection mapping, Unicode, concurrent editing, reconnect, and undo semantics as a separate application. Its appearance cannot replace assertions on canonical document state. |

## Release-readiness acceptance

The catalog is complete only when every directory and target is implemented, all 19 binaries run with their declared semantic assertions, real local-relay behavior is exercised, extracted package quickstarts run, and receipts bind to the current committed source. E44 produces the exact-coverage catalog receipt; E46 runs clean qualification twice. A candidate can have a complete public catalog while independent interoperability, external review, protocol standardization, or publication remain held. The readiness matrix records those dimensions separately and never removes a hold because examples passed.

## Primary implementation references

- [Cargo targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html): explicit example targets and `test = true`.
- [Cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html): example harness behavior and separate documentation-test selection.
- [Cargo package](https://doc.rust-lang.org/cargo/commands/cargo-package.html): archive normalization and build verification; extracted tests require their own gate.
- [NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md): signed events and relay messages used by external integrations.
- [NIP-11](https://github.com/nostr-protocol/nips/blob/master/11.md): optional relay capability metadata.
- [NIP-77](https://github.com/nostr-protocol/nips/blob/master/77.md): conditional event-ID reconciliation extension.
- [Automerge sync documentation](https://docs.rs/automerge/latest/automerge/sync/index.html): reliable ordered peer-stream assumptions; background context only, not authority to change this repository's pinned Automerge version or sealed protocol.

These live upstream references were checked during handoff preparation. The approved repository specification and exact dependency pin remain the implementation authority. Changes to upstream documentation do not silently revise the wire profile.
