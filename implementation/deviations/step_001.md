# Deviation: step_001 repository identity adaptation

Status: approved
Recorded: 2026-08-04
Base commit: `a67d446`
Affected step: `step_001`

## Repository Evidence

The implementation repository is `triesap/nostr_automerge`. Its configured
origin and Cargo package metadata identify:

```text
git@github.com:triesap/nostr_automerge.git
https://github.com/triesap/nostr_automerge
```

The approved source package instead names `radrootslabs/nostr_automerge` in
repository-ownership metadata. Importing that value unchanged would make the
standalone repository's authority, links, release metadata, and validation
factually incorrect.

## Replacement Action

During `step_001` and later repository-governance checkpoints:

- adapt repository ownership and URLs to `triesap/nostr_automerge`;
- record every transformed file and field in the import adaptation manifest;
- retain the approved source artifact name, version, generation date, and
  checksums in public provenance;
- keep all public paths repository-relative;
- retain `LICENSE-MIT` and `LICENSE-APACHE` filenames;
- reconcile Cargo resolver, MSRV, and toolchain metadata with the approved
  repository defaults before workspace implementation.

## Frozen Protocol Fields

This deviation must not change:

- `nostr-crdt/automerge/actor/v1` or another signed domain string;
- protocol revision, format, text encoding, provisional kinds, or limits;
- ActorId derivation;
- canonical JSON, base64, framing, ChangeHash, or digest algorithms;
- authorization, base-frontier, control selection, equivocation, disposition,
  or checkpoint semantics;
- fixture inputs or expected protocol outcomes except repository-only
  provenance fields.

## Affected Requirements

- `NCRDT-REPO-001`
- `NCRDT-CORE-001`
- `NCRDT-PROFILE-001`
- `NCRDT-FEATURES-001`

## Tests And Verification

- import-manifest test for every adapted path and field;
- forbidden stale repository-identity scan outside provenance and this
  explicit mapping;
- normative wire-constant comparison against the approved source;
- repository URL and package metadata assertions;
- `git diff --check`.

## Reviewer Attention

Reviewers must distinguish repository metadata from signed or consensus-bound
protocol bytes. Any transformation outside the explicit adaptation manifest is
an error and blocks the next checkpoint.
