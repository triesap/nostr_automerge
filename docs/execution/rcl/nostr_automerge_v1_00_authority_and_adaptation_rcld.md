# nostr_automerge V1 RCLD 00: Authority And Adaptation

Status: active
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Repository: `triesap/nostr_automerge`
Base commit: `a67d446`
Governing plan: `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`
Current checkpoint: `step_002`

## Purpose

Establish complete, standalone, executable repository authority before any Rust
protocol implementation begins. Import the approved draft-v1 contracts with
verifiable provenance, record the repository-identity adaptation, harden the
fixture/report contracts, and install repository-local governance and agent
instructions.

## Scope Boundary

This child changes documentation, schemas, fixtures, validation scripts, and
repository policy only. It does not add Rust protocol behavior, dependencies,
networking, persistence, releases, or external repository mutations.

The repository identity is `triesap/nostr_automerge`. Source references to
`radrootslabs/nostr_automerge` are adapted only where they describe repository
ownership or package metadata. Normative wire bytes and consensus behavior are
copied without change.

## Definition Of Green

- Repository identity and every adaptation are explicit and reviewable.
- Imported authority is complete, repository-relative, and independently
  checksum-verifiable.
- The draft NIP, companion specification, requirements, ADRs, implementation
  sequence, schemas, fixtures, and process policies are repository-owned.
- Public content contains no private filesystem or coordination references.
- Fixture and report schemas are closed and have procedural semantic checks.
- Governance documents make current non-claims and contribution boundaries
  accurate.
- Baseline validation is deterministic and `git diff --check` is green.

## Dominant Verification Lane

Until the baseline validator exists:

```sh
python3 <checkpoint-specific-validator-or-test>
git diff --check
```

At `step_016`:

```sh
python3 scripts/validate_spec.py
python3 scripts/validate_spec.py
git diff --check
```

The two validator runs must be byte-identical.

## Checkpoint Map

### step_000 — Record repository adaptation

Scope:

- record source and target repository identity evidence;
- define the allowed metadata-only transformation;
- freeze normative wire and consensus fields;
- identify later step adjustments required by actual repository state.

Green:

- the deviation exists before adapted authority is imported;
- no signed string, protocol revision, kind, encoding, or algorithm changes;
- standalone-content scan and `git diff --check` are green.

Commit: `docs(plan): record repository identity adaptation`

### step_001 — Import approved baseline

Scope:

- import the authoritative draft NIP, companion spec, requirements, protocol
  revision, architecture decisions, implementation sequence, and source
  manifest into repository-owned paths;
- add public provenance and an explicit adaptation manifest;
- record canonical source-manifest and imported-file digests.

Green:

- all required files exist and have non-empty provenance;
- imported hashes match;
- repository metadata names `triesap/nostr_automerge`;
- normative wire content matches the approved source.

Commit: `docs(spec): import approved nostr_automerge_v1_spec baseline`

### step_002 — Add repository-local agent instructions

Scope:

- add root `AGENTS.md` with reading order, architecture, naming, safety,
  verification, deviation, and completion rules.

Green:

- required sections validate;
- instructions are standalone and consistent with imported authority.

Commit: `docs(repo): add agent implementation instructions`

### step_003 — Establish governance and security policy

Scope:

- correct README and CONTRIBUTING status/scope;
- add SECURITY and CODEOWNERS;
- preserve existing dual licenses;
- state that implementation, conformance, release, and production claims are
  not yet complete.

Green:

- required policy sections and links validate;
- no irrelevant UI/accessibility boilerplate remains;
- license metadata and files agree.

Commit: `docs(repo): establish governance and security policies`

### step_004 — Add normative NIP snapshot

Scope:

- anchor the exact repository NIP snapshot and SHA-256;
- document normative precedence and consensus change control.

Green:

- checksum recomputation passes;
- the snapshot has no unrecorded modification.

Commit: `docs(spec): add normative NIP draft snapshot`

### step_005 — Add companion specification set

Scope:

- add architecture, API, data, Automerge, control, checkpoint, conformance,
  security, versioning, wire, scope, product, and acceptance contracts.

Green:

- every required companion document exists and links resolve;
- repository adaptations are explicit and wire invariants remain unchanged.

Commit: `docs(spec): add companion protocol contracts`

### step_006 — Validate normative requirements registry

Scope:

- add the machine-readable requirements registry, schema, validator, and
  positive/negative validation fixtures.

Green:

- IDs are unique and source references resolve;
- missing fields, duplicates, and invalid categories fail deterministically;
- no requirement is falsely marked implemented.

Commit: `test(spec): validate normative requirements registry`

### step_007 — Define sealed draft protocol revision

Scope:

- add schema-validated draft revision metadata with sealed kinds, formats,
  limits status, checkpoint provenance, and normative domain strings.

Green:

- custom/missing kinds and changed actor domain fail;
- repository identity is correct;
- sealed draft status validates.

Commit: `docs(spec): define sealed draft protocol revision`

### step_008 — Record approved architecture decisions

Scope:

- add all approved ADRs plus an index mapping status and requirements;
- add the repository-identity adaptation ADR/deviation cross-reference.

Green:

- ADR numbering, status, links, and requirement mappings validate.

Commit: `docs(adr): record approved architecture decisions`

### step_009 — Record prior art and rejected alternatives

Scope:

- add repository-relevant Nostr/CRDT prior art and rejected alternatives with
  source links and resulting decisions.

Green:

- identifiers and links validate;
- the record is concise, current, and standalone.

Commit: `docs(research): record Nostr CRDT prior art`

### step_010 — Define language-neutral fixture schema

Scope:

- add a closed fixture metadata schema and representative positive/negative
  instances;
- validate safe relative paths, checksums, requirements, provenance, revision,
  and deterministic seeds.

Green:

- traversal, unknown properties, missing fields, bad hashes, bad revisions,
  and duplicate requirements fail.

Commit: `test(fixtures): define language-neutral fixture schema`

### step_011 — Define canonical report schema

Scope:

- add the closed report schema and procedural semantic validator;
- define canonical collection ordering, strict identifiers, typed assertions,
  integrity alerts, dispositions, digests, and local completion separation.

Green:

- unknown fields and outcomes fail;
- unordered or duplicate canonical collections fail;
- every alert/assertion variant validates exact fields and value shape;
- local completion is excluded from disposition digest input.

Commit: `test(conformance): define canonical report schema`

## Reconciliation Rules

- Keep only the checkpoint named by `Current checkpoint` active.
- After each green commit, record its commit ID below and advance the current
  checkpoint.
- If repository evidence changes scope, add the required deviation before the
  affected implementation.
- RCLD 01 may be materialized only after step 011 is green.

## Checkpoint Ledger

| Step | Status | Commit | Result |
| --- | --- | --- | --- |
| `step_000` | complete | this checkpoint | Repository adaptation recorded |
| `step_001` | complete | this checkpoint | Approved baseline imported and validated |
| `step_002` | active | — | Agent instructions |
| `step_003` | pending | — | Governance and security |
| `step_004` | pending | — | NIP snapshot |
| `step_005` | pending | — | Companion specifications |
| `step_006` | pending | — | Requirements registry |
| `step_007` | pending | — | Protocol revision |
| `step_008` | pending | — | ADR set |
| `step_009` | pending | — | Prior art |
| `step_010` | pending | — | Fixture schema |
| `step_011` | pending | — | Report schema |
